use crate::field_crypto::FieldCipher;
use crate::models::{ContactRecord, ConversationRecord, DbMessageStatus, MessageRecord, OutboxItem};
use nova_crypto::{
    derive_storage_key, generate_storage_salt, DeviceIdentity, DoubleRatchetSession,
    OneTimePreKeyPublic, OneTimePreKeySecret, PreKeyBundle, SignedPreKeyPublic, SignedPreKeySecret,
};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension, Result};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("Crypto error: {0}")]
    CryptoError(#[from] nova_crypto::CryptoError),
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Stored field failed to decrypt (wrong key, corruption, or tampering)")]
    CorruptedField,
    #[error("Contact prekey bundle is malformed: {0}")]
    MalformedBundle(String),
}

/// Local SQLite-backed storage, encrypted at rest.
///
/// All secret material (recovery mnemonic, private key bytes, prekey secrets, live ratchet
/// session state) and message content are stored as `nonce || ChaCha20-Poly1305(plaintext)`,
/// keyed by a 256-bit key derived from the caller's local unlock passphrase via Argon2id
/// (see `nova_crypto::storage_kdf`). The passphrase itself is never persisted; only an Argon2id
/// salt and a small verifier ciphertext are, so a wrong passphrase is detected at `open()`
/// instead of surfacing as silent garbage on every read.
pub struct StorageEngine {
    conn: Mutex<Connection>,
    cipher: FieldCipher,
}

const VERIFIER_PLAINTEXT: &[u8] = b"NOVA_STORAGE_UNLOCK_OK_V1";
const VERIFIER_AAD: &[u8] = b"storage_meta:verifier";

impl StorageEngine {
    /// Opens (creating if needed) the local database at `path` (or `:memory:`), deriving the
    /// storage encryption key from `passphrase`. On first run this also picks a fresh Argon2id
    /// salt and writes a verifier so future opens can detect a wrong passphrase immediately.
    pub fn open(path: &str, passphrase: &str) -> Result<Self, StorageError> {
        let conn = if path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            Connection::open(path)?
        };

        Self::init_schema(&conn)?;
        let key = Self::unlock_or_initialize(&conn, passphrase)?;

        Ok(Self {
            conn: Mutex::new(conn),
            cipher: FieldCipher::new(key),
        })
    }

    fn init_schema(conn: &Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS storage_meta (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                kdf_salt BLOB NOT NULL,
                verifier BLOB NOT NULL
            );

            CREATE TABLE IF NOT EXISTS identities (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                username TEXT NOT NULL,
                mnemonic_enc BLOB NOT NULL,
                ed25519_sec_enc BLOB NOT NULL,
                ed25519_pub BLOB NOT NULL,
                x25519_sec_enc BLOB NOT NULL,
                x25519_pub BLOB NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS signed_prekeys (
                key_id INTEGER PRIMARY KEY,
                secret_enc BLOB NOT NULL,
                public BLOB NOT NULL,
                signature BLOB NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS one_time_prekeys (
                key_id INTEGER PRIMARY KEY,
                secret_enc BLOB NOT NULL,
                public BLOB NOT NULL,
                published INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contacts (
                peer_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                display_name TEXT NOT NULL,
                prekey_bundle_json BLOB NOT NULL,
                safety_number TEXT NOT NULL,
                is_online INTEGER NOT NULL DEFAULT 0,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                last_seen_utc INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                peer_id TEXT NOT NULL,
                title TEXT NOT NULL,
                last_message_text_enc BLOB,
                last_message_time_utc INTEGER NOT NULL DEFAULT 0,
                unread_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                recipient_id TEXT NOT NULL,
                text_content_enc BLOB NOT NULL,
                timestamp_utc INTEGER NOT NULL,
                status INTEGER NOT NULL,
                is_outgoing INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            CREATE TABLE IF NOT EXISTS outbox_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id TEXT UNIQUE NOT NULL,
                conversation_id TEXT NOT NULL,
                recipient_id TEXT NOT NULL,
                payload BLOB NOT NULL,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                next_retry_utc INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ratchet_sessions (
                peer_id TEXT PRIMARY KEY,
                session_enc BLOB NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, timestamp_utc);
            CREATE INDEX IF NOT EXISTS idx_outbox_retry ON outbox_queue(next_retry_utc);
            ",
        )?;

        Ok(())
    }

    fn unlock_or_initialize(conn: &Connection, passphrase: &str) -> Result<[u8; 32], StorageError> {
        let existing: Option<(Vec<u8>, Vec<u8>)> = conn
            .query_row(
                "SELECT kdf_salt, verifier FROM storage_meta WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        match existing {
            Some((salt, verifier_blob)) => {
                let key = derive_storage_key(passphrase, &salt)?;
                let cipher = FieldCipher::new(key);
                let plaintext = cipher
                    .decrypt(&verifier_blob, VERIFIER_AAD)
                    .map_err(|_| StorageError::CryptoError(nova_crypto::CryptoError::InvalidPassphrase))?;
                if plaintext != VERIFIER_PLAINTEXT {
                    return Err(StorageError::CryptoError(nova_crypto::CryptoError::InvalidPassphrase));
                }
                Ok(key)
            }
            None => {
                let salt = generate_storage_salt();
                let key = derive_storage_key(passphrase, &salt)?;
                let cipher = FieldCipher::new(key);
                let verifier = cipher.encrypt(VERIFIER_PLAINTEXT, VERIFIER_AAD);
                conn.execute(
                    "INSERT INTO storage_meta (id, kdf_salt, verifier) VALUES (1, ?1, ?2)",
                    params![salt.to_vec(), verifier],
                )?;
                Ok(key)
            }
        }
    }

    // --- Identity Operations ---

    pub fn save_identity(&self, identity: &DeviceIdentity, mnemonic: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();

        let mnemonic_enc = self.cipher.encrypt_str(mnemonic, b"identities:mnemonic");
        let ed25519_sec_enc = self
            .cipher
            .encrypt(&identity.signing_key_bytes, b"identities:ed25519_sec");
        let x25519_sec_enc = self
            .cipher
            .encrypt(&identity.dh_secret_bytes, b"identities:x25519_sec");

        conn.execute(
            "INSERT OR REPLACE INTO identities (id, username, mnemonic_enc, ed25519_sec_enc, ed25519_pub, x25519_sec_enc, x25519_pub, created_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                identity.username,
                mnemonic_enc,
                ed25519_sec_enc,
                identity.verifying_key_bytes.as_slice(),
                x25519_sec_enc,
                identity.dh_public_bytes.as_slice(),
                now
            ],
        )?;

        Ok(())
    }

    /// Reconstructs the locally stored device identity, decrypting its private key material.
    #[allow(clippy::type_complexity)]
    pub fn load_identity(&self) -> Result<Option<DeviceIdentity>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<(String, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> = conn
            .query_row(
                "SELECT username, ed25519_sec_enc, ed25519_pub, x25519_sec_enc, x25519_pub FROM identities WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .optional()?;

        let Some((username, ed_sec_enc, ed_pub, x_sec_enc, x_pub)) = row else {
            return Ok(None);
        };

        let signing_key_bytes = to_array_32(&self.cipher.decrypt(&ed_sec_enc, b"identities:ed25519_sec")?)?;
        let dh_secret_bytes = to_array_32(&self.cipher.decrypt(&x_sec_enc, b"identities:x25519_sec")?)?;
        let verifying_key_bytes = to_array_32(&ed_pub)?;
        let dh_public_bytes = to_array_32(&x_pub)?;

        Ok(Some(DeviceIdentity {
            username,
            signing_key_bytes,
            dh_secret_bytes,
            verifying_key_bytes,
            dh_public_bytes,
        }))
    }

    pub fn get_identity_mnemonic(&self) -> Result<Option<String>, StorageError> {
        let conn = self.conn.lock();
        let blob: Option<Vec<u8>> = conn
            .query_row("SELECT mnemonic_enc FROM identities WHERE id = 1", [], |row| row.get(0))
            .optional()?;

        match blob {
            Some(b) => Ok(Some(self.cipher.decrypt_string(&b, b"identities:mnemonic")?)),
            None => Ok(None),
        }
    }

    // --- X3DH Prekey Operations ---

    pub fn save_signed_prekey(
        &self,
        secret: &SignedPreKeySecret,
        public: &SignedPreKeyPublic,
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();
        let aad = format!("signed_prekeys:secret:{}", secret.key_id);
        let secret_enc = self.cipher.encrypt(&secret.to_secret_bytes(), aad.as_bytes());

        conn.execute(
            "INSERT OR REPLACE INTO signed_prekeys (key_id, secret_enc, public, signature, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![public.key_id, secret_enc, public.public.to_vec(), public.signature, now],
        )?;
        Ok(())
    }

    /// Loads the most recently generated signed prekey (device identities keep exactly one
    /// active signed prekey at a time in this implementation; rotation simply inserts a new row).
    #[allow(clippy::type_complexity)]
    pub fn load_active_signed_prekey(
        &self,
    ) -> Result<Option<(SignedPreKeySecret, SignedPreKeyPublic)>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<(u32, Vec<u8>, Vec<u8>, Vec<u8>)> = conn
            .query_row(
                "SELECT key_id, secret_enc, public, signature FROM signed_prekeys ORDER BY created_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;

        let Some((key_id, secret_enc, public, signature)) = row else {
            return Ok(None);
        };

        let aad = format!("signed_prekeys:secret:{}", key_id);
        let secret_bytes = to_array_32(&self.cipher.decrypt(&secret_enc, aad.as_bytes())?)?;

        Ok(Some((
            SignedPreKeySecret::from_parts(key_id, secret_bytes),
            SignedPreKeyPublic {
                key_id,
                public: to_array_32(&public)?,
                signature,
            },
        )))
    }

    pub fn save_one_time_prekeys(
        &self,
        items: &[(OneTimePreKeySecret, OneTimePreKeyPublic)],
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();
        for (secret, public) in items {
            let aad = format!("one_time_prekeys:secret:{}", secret.key_id);
            let secret_enc = self.cipher.encrypt(&secret.to_secret_bytes(), aad.as_bytes());
            conn.execute(
                "INSERT OR REPLACE INTO one_time_prekeys (key_id, secret_enc, public, published, created_at)
                 VALUES (?1, ?2, ?3, 0, ?4)",
                params![public.key_id, secret_enc, public.public.to_vec(), now],
            )?;
        }
        Ok(())
    }

    /// Returns the public half of one not-yet-published one-time prekey for inclusion in a
    /// bundle we are about to hand to a new contact, and marks it `published` so it is never
    /// handed out again. The secret half is deliberately kept (not deleted) here: it is only
    /// consumed later, in [`consume_one_time_prekey_secret`], when a handshake that actually
    /// used it arrives — publishing a bundle is not the same event as the key being used.
    pub fn take_one_time_prekey_for_publishing(&self) -> Result<Option<OneTimePreKeyPublic>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<(u32, Vec<u8>)> = conn
            .query_row(
                "SELECT key_id, public FROM one_time_prekeys WHERE published = 0 ORDER BY created_at ASC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        let Some((key_id, public)) = row else {
            return Ok(None);
        };

        conn.execute(
            "UPDATE one_time_prekeys SET published = 1 WHERE key_id = ?1",
            params![key_id],
        )?;

        Ok(Some(OneTimePreKeyPublic {
            key_id,
            public: to_array_32(&public)?,
        }))
    }

    /// Consumes (fetches and deletes) the secret half of a one-time prekey referenced by an
    /// incoming X3DH handshake. Returns `None` if it was already used or never existed.
    pub fn consume_one_time_prekey_secret(
        &self,
        key_id: u32,
    ) -> Result<Option<OneTimePreKeySecret>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<Vec<u8>> = conn
            .query_row(
                "SELECT secret_enc FROM one_time_prekeys WHERE key_id = ?1",
                params![key_id],
                |row| row.get(0),
            )
            .optional()?;

        let Some(secret_enc) = row else {
            return Ok(None);
        };

        conn.execute("DELETE FROM one_time_prekeys WHERE key_id = ?1", params![key_id])?;

        let aad = format!("one_time_prekeys:secret:{}", key_id);
        let secret_bytes = to_array_32(&self.cipher.decrypt(&secret_enc, aad.as_bytes())?)?;
        Ok(Some(OneTimePreKeySecret::from_parts(key_id, secret_bytes)))
    }

    /// Assembles this device's own publishable prekey bundle (identity keys + active signed
    /// prekey + one fresh one-time prekey if any remain). Share this out-of-band (QR code,
    /// direct exchange) so a peer can call `add_contact` with it.
    pub fn get_own_prekey_bundle(&self, identity: &DeviceIdentity) -> Result<PreKeyBundle, StorageError> {
        let (_secret, signed_prekey) = self
            .load_active_signed_prekey()?
            .ok_or_else(|| StorageError::NotFound("no signed prekey generated yet".into()))?;

        let one_time_prekey = self.take_one_time_prekey_for_publishing()?;

        Ok(PreKeyBundle {
            identity_ed25519_pub: identity.verifying_key_bytes,
            identity_x25519_pub: identity.dh_public_bytes,
            signed_prekey,
            one_time_prekey,
        })
    }

    // --- Contacts Operations ---

    pub fn save_contact(&self, contact: &ContactRecord) -> Result<(), StorageError> {
        let bundle_json = serde_json::to_vec(&contact.prekey_bundle)
            .map_err(|e| StorageError::MalformedBundle(e.to_string()))?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO contacts (peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, last_seen_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                contact.peer_id,
                contact.username,
                contact.display_name,
                bundle_json,
                contact.safety_number,
                contact.is_online as i32,
                contact.is_blocked as i32,
                contact.last_seen_utc
            ],
        )?;
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn get_contact(&self, peer_id: &str) -> Result<Option<ContactRecord>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<(String, String, String, Vec<u8>, String, i32, i32, i64)> = conn
            .query_row(
                "SELECT peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, last_seen_utc FROM contacts WHERE peer_id = ?1",
                params![peer_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)),
            )
            .optional()?;

        row.map(row_to_contact).transpose()
    }

    pub fn get_contacts(&self) -> Result<Vec<ContactRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, last_seen_utc FROM contacts ORDER BY display_name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(row_to_contact(r?)?);
        }
        Ok(list)
    }

    // --- 1-to-1 Conversations Operations ---

    pub fn save_conversation(&self, conv: &ConversationRecord) -> Result<(), StorageError> {
        let aad = format!("conversations:last_msg:{}", conv.id);
        let last_msg_enc = self.cipher.encrypt_str(&conv.last_message_text, aad.as_bytes());

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO conversations (id, peer_id, title, last_message_text_enc, last_message_time_utc, unread_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                conv.id,
                conv.peer_id,
                conv.title,
                last_msg_enc,
                conv.last_message_time_utc,
                conv.unread_count
            ],
        )?;
        Ok(())
    }

    pub fn get_conversations(&self) -> Result<Vec<ConversationRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT id, peer_id, title, last_message_text_enc, last_message_time_utc, unread_count FROM conversations ORDER BY last_message_time_utc DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, u32>(5)?,
            ))
        })?;

        let mut list = Vec::new();
        for r in rows {
            let (id, peer_id, title, last_msg_enc, last_message_time_utc, unread_count) = r?;
            let aad = format!("conversations:last_msg:{}", id);
            let last_message_text = self.cipher.decrypt_string(&last_msg_enc, aad.as_bytes())?;
            list.push(ConversationRecord {
                id,
                peer_id,
                title,
                last_message_text,
                last_message_time_utc,
                unread_count,
            });
        }
        Ok(list)
    }

    // --- Messages Operations ---

    pub fn save_message(&self, msg: &MessageRecord) -> Result<(), StorageError> {
        let msg_aad = format!("messages:text:{}", msg.id);
        let text_enc = self.cipher.encrypt_str(&msg.text_content, msg_aad.as_bytes());

        let conv_aad = format!("conversations:last_msg:{}", msg.conversation_id);
        let last_msg_enc = self.cipher.encrypt_str(&msg.text_content, conv_aad.as_bytes());

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO messages (id, conversation_id, sender_id, recipient_id, text_content_enc, timestamp_utc, status, is_outgoing)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                msg.id,
                msg.conversation_id,
                msg.sender_id,
                msg.recipient_id,
                text_enc,
                msg.timestamp_utc,
                msg.status.to_i32(),
                msg.is_outgoing as i32
            ],
        )?;

        conn.execute(
            "UPDATE conversations SET last_message_text_enc = ?1, last_message_time_utc = ?2 WHERE id = ?3",
            params![last_msg_enc, msg.timestamp_utc, msg.conversation_id],
        )?;

        Ok(())
    }

    pub fn update_message_status(&self, message_id: &str, new_status: DbMessageStatus) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE messages SET status = ?1 WHERE id = ?2",
            params![new_status.to_i32(), message_id],
        )?;
        Ok(())
    }

    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<MessageRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT id, conversation_id, sender_id, recipient_id, text_content_enc, timestamp_utc, status, is_outgoing FROM messages WHERE conversation_id = ?1 ORDER BY timestamp_utc ASC")?;
        let rows = stmt.query_map(params![conversation_id], Self::row_to_raw_message)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(self.decrypt_message_row(r?)?);
        }
        Ok(list)
    }

    /// Full-text search over message content. Content is encrypted at rest, so this cannot be
    /// pushed down to SQL (no `LIKE` on ciphertext); instead it decrypts and filters in memory.
    /// Acceptable for a single-user local dataset — revisit with an encrypted search index
    /// (e.g. blind indexing) if message volume grows large enough for this to matter.
    pub fn search_messages(&self, query: &str) -> Result<Vec<MessageRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT id, conversation_id, sender_id, recipient_id, text_content_enc, timestamp_utc, status, is_outgoing FROM messages ORDER BY timestamp_utc DESC")?;
        let rows = stmt.query_map([], Self::row_to_raw_message)?;

        let query_lower = query.to_lowercase();
        let mut list = Vec::new();
        for r in rows {
            let msg = self.decrypt_message_row(r?)?;
            if msg.text_content.to_lowercase().contains(&query_lower) {
                list.push(msg);
                if list.len() >= 50 {
                    break;
                }
            }
        }
        Ok(list)
    }

    #[allow(clippy::type_complexity)]
    fn row_to_raw_message(
        row: &rusqlite::Row,
    ) -> Result<(String, String, String, String, Vec<u8>, i64, i32, bool), rusqlite::Error> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get::<_, i32>(7)? != 0,
        ))
    }

    fn decrypt_message_row(
        &self,
        row: (String, String, String, String, Vec<u8>, i64, i32, bool),
    ) -> Result<MessageRecord, StorageError> {
        let (id, conversation_id, sender_id, recipient_id, text_enc, timestamp_utc, status, is_outgoing) = row;
        let aad = format!("messages:text:{id}");
        let text_content = self.cipher.decrypt_string(&text_enc, aad.as_bytes())?;
        Ok(MessageRecord {
            id,
            conversation_id,
            sender_id,
            recipient_id,
            text_content,
            timestamp_utc,
            status: DbMessageStatus::from_i32(status),
            is_outgoing,
        })
    }

    // --- Outbox Queue Operations (payload is already Double-Ratchet ciphertext) ---

    pub fn enqueue_outbox(
        &self,
        message_id: &str,
        conversation_id: &str,
        recipient_id: &str,
        payload: &[u8],
    ) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO outbox_queue (message_id, conversation_id, recipient_id, payload, attempt_count, next_retry_utc)
             VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            params![message_id, conversation_id, recipient_id, payload, now],
        )?;
        Ok(())
    }

    pub fn get_pending_outbox(&self) -> Result<Vec<OutboxItem>, StorageError> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();
        let mut stmt = conn.prepare("SELECT id, message_id, conversation_id, recipient_id, payload, attempt_count, next_retry_utc FROM outbox_queue WHERE next_retry_utc <= ?1 ORDER BY id ASC")?;
        let rows = stmt.query_map(params![now], |row| {
            Ok(OutboxItem {
                id: row.get(0)?,
                message_id: row.get(1)?,
                conversation_id: row.get(2)?,
                recipient_id: row.get(3)?,
                payload: row.get(4)?,
                attempt_count: row.get(5)?,
                next_retry_utc: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn remove_from_outbox(&self, message_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM outbox_queue WHERE message_id = ?1", params![message_id])?;
        Ok(())
    }

    // --- Double Ratchet Session Persistence ---

    /// Persists the (mutating, per-message) Double Ratchet session state for `peer_id`,
    /// encrypted at rest. Must be called after every `ratchet_encrypt`/`ratchet_decrypt` so a
    /// restart does not desynchronize the ratchet or force a fresh handshake.
    pub fn save_session(&self, peer_id: &str, session: &DoubleRatchetSession) -> Result<(), StorageError> {
        let mut cbor = Vec::new();
        ciborium::into_writer(session, &mut cbor)
            .map_err(|e| StorageError::MalformedBundle(format!("session serialize: {e}")))?;

        let aad = format!("ratchet_sessions:{peer_id}");
        let session_enc = self.cipher.encrypt(&cbor, aad.as_bytes());

        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO ratchet_sessions (peer_id, session_enc, updated_at) VALUES (?1, ?2, ?3)",
            params![peer_id, session_enc, now],
        )?;
        Ok(())
    }

    pub fn load_session(&self, peer_id: &str) -> Result<Option<DoubleRatchetSession>, StorageError> {
        let blob: Option<Vec<u8>> = {
            let conn = self.conn.lock();
            conn.query_row(
                "SELECT session_enc FROM ratchet_sessions WHERE peer_id = ?1",
                params![peer_id],
                |row| row.get(0),
            )
            .optional()?
        };

        let Some(blob) = blob else {
            return Ok(None);
        };

        let aad = format!("ratchet_sessions:{peer_id}");
        let cbor = self.cipher.decrypt(&blob, aad.as_bytes())?;
        let session: DoubleRatchetSession = ciborium::from_reader(cbor.as_slice())
            .map_err(|e| StorageError::MalformedBundle(format!("session deserialize: {e}")))?;
        Ok(Some(session))
    }
}

fn to_array_32(bytes: &[u8]) -> Result<[u8; 32], StorageError> {
    bytes.try_into().map_err(|_| StorageError::CorruptedField)
}

#[allow(clippy::type_complexity)]
fn row_to_contact(
    row: (String, String, String, Vec<u8>, String, i32, i32, i64),
) -> Result<ContactRecord, StorageError> {
    let (peer_id, username, display_name, bundle_json, safety_number, is_online, is_blocked, last_seen_utc) = row;
    let prekey_bundle: PreKeyBundle =
        serde_json::from_slice(&bundle_json).map_err(|e| StorageError::MalformedBundle(e.to_string()))?;

    Ok(ContactRecord {
        peer_id,
        username,
        display_name,
        prekey_bundle,
        safety_number,
        is_online: is_online != 0,
        is_blocked: is_blocked != 0,
        last_seen_utc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::{generate_signed_prekey, MnemonicPhrase};

    fn test_identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    #[test]
    fn test_wrong_passphrase_is_rejected_on_reopen() {
        let dir = std::env::temp_dir().join(format!("nova_test_{}.db", uuid::Uuid::new_v4()));
        let path = dir.to_str().unwrap();

        {
            let storage = StorageEngine::open(path, "correct passphrase").unwrap();
            let identity = test_identity("alex");
            storage.save_identity(&identity, "abandon ability able about above absent absorb abstract absurd abuse access accident").unwrap();
        }

        let wrong = StorageEngine::open(path, "wrong passphrase");
        assert!(wrong.is_err());

        let right = StorageEngine::open(path, "correct passphrase");
        assert!(right.is_ok());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_identity_roundtrip_and_encryption_at_rest() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let identity = test_identity("alex");
        let mnemonic = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        storage.save_identity(&identity, mnemonic).unwrap();

        let restored_mnemonic = storage.get_identity_mnemonic().unwrap().unwrap();
        assert_eq!(restored_mnemonic, mnemonic);

        let restored_identity = storage.load_identity().unwrap().unwrap();
        assert_eq!(restored_identity.verifying_key_bytes, identity.verifying_key_bytes);
        assert_eq!(restored_identity.dh_public_bytes, identity.dh_public_bytes);
        assert_eq!(restored_identity.signing_key_bytes, identity.signing_key_bytes);
    }

    #[test]
    fn test_raw_sqlite_file_does_not_contain_plaintext_mnemonic() {
        let dir = std::env::temp_dir().join(format!("nova_test_{}.db", uuid::Uuid::new_v4()));
        let path = dir.to_str().unwrap();
        let secret_marker = "zzz_unmistakable_recovery_phrase_marker_zzz";

        {
            let storage = StorageEngine::open(path, "unlock-pass").unwrap();
            let identity = test_identity("alex");
            storage.save_identity(&identity, secret_marker).unwrap();
        }

        let raw = std::fs::read(path).unwrap();
        let haystack = String::from_utf8_lossy(&raw);
        assert!(
            !haystack.contains(secret_marker),
            "mnemonic must not appear in plaintext in the database file"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_prekey_bundle_lifecycle() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let identity = test_identity("bob");

        let (secret, public) = generate_signed_prekey(&identity, 1);
        storage.save_signed_prekey(&secret, &public).unwrap();

        let (loaded_secret, loaded_public) = storage.load_active_signed_prekey().unwrap().unwrap();
        assert_eq!(loaded_secret.to_secret_bytes(), secret.to_secret_bytes());
        assert_eq!(loaded_public.public, public.public);

        let bundle = storage.get_own_prekey_bundle(&identity).unwrap();
        assert_eq!(bundle.identity_ed25519_pub, identity.verifying_key_bytes);
        assert_eq!(bundle.one_time_prekey, None);
    }

    #[test]
    fn test_contact_and_conversation_and_message_lifecycle() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let bob = test_identity("bob");
        let (_secret, signed_prekey) = generate_signed_prekey(&bob, 1);

        let bundle = PreKeyBundle {
            identity_ed25519_pub: bob.verifying_key_bytes,
            identity_x25519_pub: bob.dh_public_bytes,
            signed_prekey,
            one_time_prekey: None,
        };

        let contact = ContactRecord {
            peer_id: hex::encode(bob.verifying_key_bytes),
            username: "emma.nova".into(),
            display_name: "Emma".into(),
            prekey_bundle: bundle,
            safety_number: "4A9F-2B1C-88E0-9142".into(),
            is_online: true,
            is_blocked: false,
            last_seen_utc: 1787119000,
        };
        storage.save_contact(&contact).unwrap();
        let contacts = storage.get_contacts().unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].username, "emma.nova");

        let conv = ConversationRecord {
            id: "conv_emma".into(),
            peer_id: contact.peer_id.clone(),
            title: "Emma".into(),
            last_message_text: "".into(),
            last_message_time_utc: 0,
            unread_count: 0,
        };
        storage.save_conversation(&conv).unwrap();

        let msg = MessageRecord {
            id: "msg_001".into(),
            conversation_id: "conv_emma".into(),
            sender_id: "alex".into(),
            recipient_id: contact.peer_id.clone(),
            text_content: "Salut Emma ! Message 1-to-1 sécurisé.".into(),
            timestamp_utc: 1787119100,
            status: DbMessageStatus::Created,
            is_outgoing: true,
        };
        storage.save_message(&msg).unwrap();

        let messages = storage.get_messages("conv_emma").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text_content, "Salut Emma ! Message 1-to-1 sécurisé.");

        let convs = storage.get_conversations().unwrap();
        assert_eq!(convs[0].last_message_text, "Salut Emma ! Message 1-to-1 sécurisé.");

        storage.enqueue_outbox("msg_001", "conv_emma", &contact.peer_id, b"payload_bytes").unwrap();
        let pending = storage.get_pending_outbox().unwrap();
        assert_eq!(pending.len(), 1);
        storage.remove_from_outbox("msg_001").unwrap();
        assert_eq!(storage.get_pending_outbox().unwrap().len(), 0);

        let search_results = storage.search_messages("1-to-1").unwrap();
        assert_eq!(search_results.len(), 1);
        let search_miss = storage.search_messages("nonexistent phrase").unwrap();
        assert_eq!(search_miss.len(), 0);
    }

    #[test]
    fn test_ratchet_session_persistence_roundtrip() {
        use nova_crypto::DoubleRatchetSession;
        use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();

        let shared_key = [9u8; 32];
        let mut rng = rand::thread_rng();
        let bob_dhs = StaticSecret::random_from_rng(&mut rng);
        let bob_dh_pub = *X25519PublicKey::from(&bob_dhs).as_bytes();

        let mut alice = DoubleRatchetSession::init_alice(&shared_key, &bob_dh_pub).unwrap();
        let (_h, _c) = alice.ratchet_encrypt(b"hello", b"ad").unwrap();

        storage.save_session("bob_peer_id", &alice).unwrap();
        let mut restored = storage.load_session("bob_peer_id").unwrap().unwrap();

        let (h2, c2) = restored.ratchet_encrypt(b"second message", b"ad").unwrap();
        assert_eq!(h2.n, 1);
        let _ = c2;
    }
}
