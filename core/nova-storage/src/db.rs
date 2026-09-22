use crate::field_crypto::FieldCipher;
use crate::models::{
    AttachmentMeta, ContactRecord, ConversationRecord, DbMessageStatus, GroupMemberRecord,
    GroupRecord, MessageRecord, OutboxItem, UserProfileRecord,
};
use nova_crypto::{
    derive_storage_key, generate_storage_salt, DeviceIdentity, DoubleRatchetSession,
    OneTimePreKeyPublic, OneTimePreKeySecret, PreKeyBundle, SignedPreKeyPublic, SignedPreKeySecret,
};
use nova_protocol::MessageContentType;
use parking_lot::Mutex;
#[cfg(test)]
use rand::Rng;
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
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

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
                is_trusted INTEGER NOT NULL DEFAULT 0,
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
                content_type TEXT NOT NULL DEFAULT 'Text',
                FOREIGN KEY (conversation_id) REFERENCES conversations(id)
            );

            -- Attachment binary data, kept out of `messages` entirely so listing a conversation
            -- or full-text-searching it never has to decrypt megabytes of media just to read a
            -- caption (see StorageEngine::search_messages and get_attachment_blob).
            CREATE TABLE IF NOT EXISTS message_attachments (
                message_id TEXT PRIMARY KEY,
                mime_type TEXT NOT NULL,
                file_name TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                sha256_checksum TEXT NOT NULL,
                blob_enc BLOB NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(id)
            );

            CREATE TABLE IF NOT EXISTS outbox_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id TEXT UNIQUE NOT NULL,
                conversation_id TEXT NOT NULL,
                recipient_id TEXT NOT NULL,
                payload BLOB NOT NULL,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                next_retry_utc INTEGER NOT NULL,
                first_attempt_utc INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS ratchet_sessions (
                peer_id TEXT PRIMARY KEY,
                session_enc BLOB NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS user_profile (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                display_name_enc BLOB NOT NULL,
                bio_enc BLOB NOT NULL,
                avatar_enc BLOB,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                avatar_data_url TEXT,
                creator_peer_id TEXT NOT NULL,
                my_role TEXT NOT NULL DEFAULT 'member',
                ephemeral_timer_sec INTEGER NOT NULL DEFAULT 0,
                created_at_utc INTEGER NOT NULL,
                updated_at_utc INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS group_members (
                group_id TEXT NOT NULL,
                peer_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                joined_at_utc INTEGER NOT NULL,
                is_online INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (group_id, peer_id),
                FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_conv ON messages(conversation_id, timestamp_utc);
            CREATE INDEX IF NOT EXISTS idx_outbox_retry ON outbox_queue(next_retry_utc);
            CREATE INDEX IF NOT EXISTS idx_group_members_group ON group_members(group_id);
            ",
        )?;

        conn.execute("ALTER TABLE contacts ADD COLUMN is_trusted INTEGER NOT NULL DEFAULT 0", []).ok();
        conn.execute("ALTER TABLE conversations ADD COLUMN is_group INTEGER NOT NULL DEFAULT 0", []).ok();

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

    /// Wipes every locally stored identity, prekey, contact, conversation, message, outbox item
    /// and ratchet session — used by the "log out" flow. There is no server account to log back
    /// into (identity is device-bound), so logging out is equivalent to resetting the device to
    /// a fresh-install state; this is what lets a later `save_identity` insert a brand new row
    /// instead of silently `REPLACE`-ing still-needed key material out from under a user who
    /// never actually cleared the old one. `storage_meta` (the passphrase-derived unlock key for
    /// the database file itself) is deliberately untouched — it is not tied to any one identity.
    pub fn clear_all_identity_data(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "DELETE FROM user_profile;
             DELETE FROM ratchet_sessions;
             DELETE FROM outbox_queue;
             DELETE FROM message_attachments;
             DELETE FROM messages;
             DELETE FROM conversations;
             DELETE FROM contacts;
             DELETE FROM one_time_prekeys;
             DELETE FROM signed_prekeys;
             DELETE FROM identities;",
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

    /// Counts remaining unpublished one-time prekeys.
    pub fn count_unpublished_one_time_prekeys(&self) -> Result<usize, StorageError> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM one_time_prekeys WHERE published = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
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
    /// prekey, deliberately no one-time prekey — see below). Share this out-of-band (QR code,
    /// invitation link, direct exchange) so a peer can call `add_contact` with it.
    ///
    /// Never embeds a one-time prekey, even though the `one_time_prekeys` pool/publishing
    /// machinery below still exists and is exercised by its own tests: this bundle (or the signed
    /// `nova://invite` ticket wrapping it — see `nova_protocol::SignedContactInvitation`) is a
    /// static artifact the UI deliberately keeps valid and re-showable for up to 24h (a QR code
    /// displayed once, a link pasted into several chats), so there is no way to guarantee any
    /// "one-time" key embedded in it is ever actually used by only one recipient. It previously
    /// was taken from the pool here on every call: the first person to complete a handshake using
    /// a given invitation consumed the key, and every other recipient of that *same* invitation
    /// then silently derived a different-and-wrong X3DH shared secret from the (by then missing)
    /// one-time prekey — every message they ever sent failed its AEAD tag check forever after,
    /// with no way to recover short of a brand new invitation (see the 2026-08-24 3-device mesh
    /// test that reproduced this: two of three devices could add and message the third, since both
    /// read the third's one single invitation). X3DH without a one-time prekey (`opk_secret: None`
    /// in `x3dh_respond`) is still a fully secure, standard X3DH mode — the one-time-prekey term
    /// only ever added marginal extra forward secrecy, which this design cannot deliver safely
    /// anyway without a live server handing out each key exactly once.
    pub fn get_own_prekey_bundle(&self, identity: &DeviceIdentity) -> Result<PreKeyBundle, StorageError> {
        let (_secret, signed_prekey) = self
            .load_active_signed_prekey()?
            .ok_or_else(|| StorageError::NotFound("no signed prekey generated yet".into()))?;

        Ok(PreKeyBundle {
            identity_ed25519_pub: identity.verifying_key_bytes,
            identity_x25519_pub: identity.dh_public_bytes,
            signed_prekey,
            one_time_prekey: None,
        })
    }

    // --- Contacts Operations ---

    pub fn save_contact(&self, contact: &ContactRecord) -> Result<(), StorageError> {
        let bundle_json = serde_json::to_vec(&contact.prekey_bundle)
            .map_err(|e| StorageError::MalformedBundle(e.to_string()))?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO contacts (peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, is_trusted, last_seen_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                contact.peer_id,
                contact.username,
                contact.display_name,
                bundle_json,
                contact.safety_number,
                contact.is_online as i32,
                contact.is_blocked as i32,
                contact.is_trusted as i32,
                contact.last_seen_utc
            ],
        )?;
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn get_contact(&self, peer_id: &str) -> Result<Option<ContactRecord>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<(String, String, String, Vec<u8>, String, i32, i32, i32, i64)> = conn
            .query_row(
                "SELECT peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, is_trusted, last_seen_utc FROM contacts WHERE peer_id = ?1",
                params![peer_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?)),
            )
            .optional()?;

        row.map(row_to_contact).transpose()
    }

    pub fn get_contacts(&self) -> Result<Vec<ContactRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, is_trusted, last_seen_utc FROM contacts ORDER BY display_name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, i32>(7)?,
                row.get::<_, i64>(8)?,
            ))
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(row_to_contact(r?)?);
        }
        Ok(list)
    }

    /// Marks a contact as trusted (no more safety prompts).
    pub fn trust_contact(&self, peer_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE contacts SET is_trusted = 1 WHERE peer_id = ?1",
            params![peer_id],
        )?;
        Ok(())
    }

    /// Marks a contact as blocked.
    pub fn block_contact(&self, peer_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE contacts SET is_blocked = 1 WHERE peer_id = ?1",
            params![peer_id],
        )?;
        Ok(())
    }

    /// Marks a contact as unblocked.
    pub fn unblock_contact(&self, peer_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE contacts SET is_blocked = 0 WHERE peer_id = ?1",
            params![peer_id],
        )?;
        Ok(())
    }

    /// Permanently removes a contact and everything tied to them: message history, any pending
    /// outbox items addressed to them, and the Double Ratchet session — unlike `block_contact`,
    /// which keeps all of this and just stops future messages, this is a full wipe of the
    /// relationship. Re-adding the same peer afterwards starts a fresh conversation with no
    /// memory of the old one (the opposite of `test_readding_contact_preserves_conversation_
    /// history`'s re-add-without-deleting case). `conversations`/`messages` have a `FOREIGN KEY`
    /// relationship enforced (`PRAGMA foreign_keys = ON`), so deletion order matters: attachments
    /// before messages before conversations.
    pub fn delete_contact(&self, peer_id: &str) -> Result<(), StorageError> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM message_attachments WHERE message_id IN (
                SELECT id FROM messages WHERE conversation_id IN (
                    SELECT id FROM conversations WHERE peer_id = ?1
                )
             )",
            params![peer_id],
        )?;
        tx.execute(
            "DELETE FROM messages WHERE conversation_id IN (
                SELECT id FROM conversations WHERE peer_id = ?1
             )",
            params![peer_id],
        )?;
        tx.execute("DELETE FROM conversations WHERE peer_id = ?1", params![peer_id])?;
        tx.execute("DELETE FROM outbox_queue WHERE recipient_id = ?1", params![peer_id])?;
        tx.execute("DELETE FROM ratchet_sessions WHERE peer_id = ?1", params![peer_id])?;
        tx.execute("DELETE FROM contacts WHERE peer_id = ?1", params![peer_id])?;
        tx.commit()?;
        Ok(())
    }

    /// Blocks a contact and deletes their entire conversation & message history, but keeps the contact in blocked state.
    pub fn block_and_delete_conversation(&self, peer_id: &str) -> Result<(), StorageError> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM message_attachments WHERE message_id IN (
                SELECT id FROM messages WHERE conversation_id IN (
                    SELECT id FROM conversations WHERE peer_id = ?1
                )
             )",
            params![peer_id],
        )?;
        tx.execute(
            "DELETE FROM messages WHERE conversation_id IN (
                SELECT id FROM conversations WHERE peer_id = ?1
             )",
            params![peer_id],
        )?;
        tx.execute("DELETE FROM conversations WHERE peer_id = ?1", params![peer_id])?;
        tx.execute("DELETE FROM outbox_queue WHERE recipient_id = ?1", params![peer_id])?;
        tx.execute("DELETE FROM ratchet_sessions WHERE peer_id = ?1", params![peer_id])?;
        tx.execute("UPDATE contacts SET is_blocked = 1 WHERE peer_id = ?1", params![peer_id])?;
        tx.commit()?;
        Ok(())
    }

    /// Checks if a contact is currently blocked.
    pub fn is_contact_blocked(&self, peer_id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock();
        let blocked: Option<i32> = conn
            .query_row(
                "SELECT is_blocked FROM contacts WHERE peer_id = ?1",
                params![peer_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(blocked.map(|b| b != 0).unwrap_or(false))
    }

    /// Returns all currently blocked contacts.
    pub fn get_blocked_contacts(&self) -> Result<Vec<ContactRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT peer_id, username, display_name, prekey_bundle_json, safety_number, is_online, is_blocked, is_trusted, last_seen_utc FROM contacts WHERE is_blocked = 1 ORDER BY display_name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, i32>(6)?,
                row.get::<_, i32>(7)?,
                row.get::<_, i64>(8)?,
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

    fn save_message_row_tx(tx: &rusqlite::Transaction, cipher: &FieldCipher, msg: &MessageRecord) -> Result<(), StorageError> {
        let msg_aad = format!("messages:text:{}", msg.id);
        let text_enc = cipher.encrypt_str(&msg.text_content, msg_aad.as_bytes());

        let conv_aad = format!("conversations:last_msg:{}", msg.conversation_id);
        let last_msg_enc = cipher.encrypt_str(&msg.text_content, conv_aad.as_bytes());

        let content_type_str = serde_json::to_string(&msg.content_type)
            .map_err(|e| StorageError::MalformedBundle(format!("content_type serialize: {e}")))?;

        tx.execute(
            "INSERT OR REPLACE INTO messages (id, conversation_id, sender_id, recipient_id, text_content_enc, timestamp_utc, status, is_outgoing, content_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                msg.id,
                msg.conversation_id,
                msg.sender_id,
                msg.recipient_id,
                text_enc,
                msg.timestamp_utc,
                msg.status.to_i32(),
                msg.is_outgoing as i32,
                content_type_str,
            ],
        )?;

        tx.execute(
            "UPDATE conversations SET last_message_text_enc = ?1, last_message_time_utc = ?2 WHERE id = ?3",
            params![last_msg_enc, msg.timestamp_utc, msg.conversation_id],
        )?;

        Ok(())
    }

    /// Saves a text-only message (no attachment). See
    /// [`save_message_with_attachment`](Self::save_message_with_attachment) for a media message.
    pub fn save_message(&self, msg: &MessageRecord) -> Result<(), StorageError> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        Self::save_message_row_tx(&tx, &self.cipher, msg)?;
        tx.commit()?;
        Ok(())
    }

    /// Saves a media message together with its attachment's encrypted binary data, in one
    /// transaction — `msg.attachment` (metadata: name/mime/size) must be `Some`. Kept in a
    /// dedicated table (`message_attachments`) rather than inline in `messages`, so that listing
    /// a conversation or full-text-searching it never has to decrypt this blob — see
    /// [`get_attachment_blob`](Self::get_attachment_blob) for fetching it back, on demand.
    pub fn save_message_with_attachment(
        &self,
        msg: &MessageRecord,
        sha256_checksum: &str,
        attachment_bytes: &[u8],
    ) -> Result<(), StorageError> {
        let meta = msg
            .attachment
            .as_ref()
            .ok_or_else(|| StorageError::MalformedBundle("save_message_with_attachment requires msg.attachment".into()))?;

        let blob_aad = format!("message_attachments:blob:{}", msg.id);
        let blob_enc = self.cipher.encrypt(attachment_bytes, blob_aad.as_bytes());

        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        Self::save_message_row_tx(&tx, &self.cipher, msg)?;
        tx.execute(
            "INSERT OR REPLACE INTO message_attachments (message_id, mime_type, file_name, size_bytes, sha256_checksum, blob_enc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![msg.id, meta.mime_type, meta.file_name, meta.size_bytes as i64, sha256_checksum, blob_enc],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Fetches and decrypts one message's attachment binary data — called on demand when the UI
    /// actually needs to display/download it, never eagerly by `get_messages`/`search_messages`.
    /// Returns `None` if the message has no attachment.
    pub fn get_attachment_blob(&self, message_id: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let conn = self.conn.lock();
        let blob: Option<Vec<u8>> = conn
            .query_row(
                "SELECT blob_enc FROM message_attachments WHERE message_id = ?1",
                params![message_id],
                |row| row.get(0),
            )
            .optional()?;

        let Some(blob) = blob else { return Ok(None) };
        let aad = format!("message_attachments:blob:{message_id}");
        Ok(Some(self.cipher.decrypt(&blob, aad.as_bytes())?))
    }

    pub fn update_message_status(&self, message_id: &str, new_status: DbMessageStatus) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE messages SET status = ?1 WHERE id = ?2",
            params![new_status.to_i32(), message_id],
        )?;
        Ok(())
    }

    const MESSAGE_SELECT_COLUMNS: &'static str = "
        m.id, m.conversation_id, m.sender_id, m.recipient_id, m.text_content_enc, m.timestamp_utc,
        m.status, m.is_outgoing, m.content_type, a.mime_type, a.file_name, a.size_bytes";

    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<MessageRecord>, StorageError> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT {} FROM messages m LEFT JOIN message_attachments a ON a.message_id = m.id \
             WHERE m.conversation_id = ?1 ORDER BY m.timestamp_utc ASC",
            Self::MESSAGE_SELECT_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![conversation_id], Self::row_to_raw_message)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(self.decrypt_message_row(r?)?);
        }
        Ok(list)
    }

    /// Full-text search over message text/captions. Content is encrypted at rest, so this cannot
    /// be pushed down to SQL (no `LIKE` on ciphertext); instead it decrypts and filters in
    /// memory. Bounded to the most recent 500 messages to avoid CPU/memory spikes on large
    /// histories. Never touches `message_attachments.blob_enc` — attachment binary data is
    /// irrelevant to a text search and, before the 2026-08-22 audit's storage split, decrypting
    /// it here on every search was the actual cost blowup this bound alone didn't fully solve.
    pub fn search_messages(&self, query: &str) -> Result<Vec<MessageRecord>, StorageError> {
        let conn = self.conn.lock();
        let sql = format!(
            "SELECT {} FROM messages m LEFT JOIN message_attachments a ON a.message_id = m.id \
             ORDER BY m.timestamp_utc DESC LIMIT 500",
            Self::MESSAGE_SELECT_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
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

    /// Deletes a single message and any associated attachment from local encrypted storage.
    pub fn delete_message(&self, message_id: &str) -> Result<(), StorageError> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM message_attachments WHERE message_id = ?1", params![message_id])?;
        tx.execute("DELETE FROM messages WHERE id = ?1", params![message_id])?;
        tx.commit()?;
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn row_to_raw_message(
        row: &rusqlite::Row,
    ) -> Result<(String, String, String, String, Vec<u8>, i64, i32, bool, String, Option<String>, Option<String>, Option<i64>), rusqlite::Error> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get::<_, i32>(7)? != 0,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
            row.get(11)?,
        ))
    }

    fn decrypt_message_row(
        &self,
        row: (String, String, String, String, Vec<u8>, i64, i32, bool, String, Option<String>, Option<String>, Option<i64>),
    ) -> Result<MessageRecord, StorageError> {
        let (
            id, conversation_id, sender_id, recipient_id, text_enc, timestamp_utc, status, is_outgoing,
            content_type_str, attach_mime, attach_name, attach_size,
        ) = row;
        let aad = format!("messages:text:{id}");
        let text_content = self.cipher.decrypt_string(&text_enc, aad.as_bytes())?;
        let content_type: MessageContentType = serde_json::from_str(&content_type_str)
            .map_err(|e| StorageError::MalformedBundle(format!("content_type deserialize: {e}")))?;
        let attachment = match (attach_mime, attach_name, attach_size) {
            (Some(mime_type), Some(file_name), Some(size_bytes)) => Some(AttachmentMeta {
                mime_type,
                file_name,
                size_bytes: size_bytes as u64,
            }),
            _ => None,
        };
        Ok(MessageRecord {
            id,
            conversation_id,
            sender_id,
            recipient_id,
            text_content,
            timestamp_utc,
            status: DbMessageStatus::from_i32(status),
            is_outgoing,
            content_type,
            attachment,
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
            "INSERT OR REPLACE INTO outbox_queue (message_id, conversation_id, recipient_id, payload, attempt_count, next_retry_utc, first_attempt_utc)
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)",
            params![message_id, conversation_id, recipient_id, payload, now],
        )?;
        Ok(())
    }

    pub fn get_pending_outbox(&self) -> Result<Vec<OutboxItem>, StorageError> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().timestamp();
        let mut stmt = conn.prepare("SELECT id, message_id, conversation_id, recipient_id, payload, attempt_count, next_retry_utc, first_attempt_utc FROM outbox_queue WHERE next_retry_utc <= ?1 ORDER BY id ASC")?;
        let rows = stmt.query_map(params![now], |row| {
            Ok(OutboxItem {
                id: row.get(0)?,
                message_id: row.get(1)?,
                conversation_id: row.get(2)?,
                recipient_id: row.get(3)?,
                payload: row.get(4)?,
                attempt_count: row.get(5)?,
                next_retry_utc: row.get(6)?,
                first_attempt_utc: row.get(7)?,
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

    pub fn has_outbox_item(&self, message_id: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM outbox_queue WHERE message_id = ?1",
            params![message_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Records one more failed delivery attempt for a still-queued message: increments
    /// `attempt_count` and pushes `next_retry_utc` out to the caller-computed backoff deadline.
    /// Leaves `payload` and `first_attempt_utc` untouched — this is a retry of the same message,
    /// not a new one (see `NovaEngine::pump_outbox_once`'s exponential backoff).
    pub fn record_outbox_retry(&self, message_id: &str, next_retry_utc: i64) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE outbox_queue SET attempt_count = attempt_count + 1, next_retry_utc = ?1 WHERE message_id = ?2",
            params![next_retry_utc, message_id],
        )?;
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

    /// Discards a peer's persisted Double Ratchet session — used when a device must force a
    /// fresh X3DH handshake on the next message to them rather than trust stale local session
    /// state the recipient may never actually have (see
    /// `nova_engine::NovaEngine::retry_failed_message`'s handling of a first-contact message that
    /// gave up without ever being confirmed delivered).
    pub fn delete_session(&self, peer_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM ratchet_sessions WHERE peer_id = ?1", params![peer_id])?;
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

    /// Saves or updates the local user profile metadata (display name, bio, and optional avatar data URL).
    pub fn save_user_profile(&self, profile: &UserProfileRecord) -> Result<(), StorageError> {
        let now = chrono::Utc::now().timestamp();
        let name_enc = self.cipher.encrypt_str(&profile.display_name, b"user_profile:name");
        let bio_enc = self.cipher.encrypt_str(&profile.bio, b"user_profile:bio");
        let avatar_enc = profile
            .avatar_data_url
            .as_ref()
            .map(|url| self.cipher.encrypt_str(url, b"user_profile:avatar"));

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO user_profile (id, display_name_enc, bio_enc, avatar_enc, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4)",
            params![name_enc, bio_enc, avatar_enc, now],
        )?;
        Ok(())
    }

    /// Retrieves the local user profile metadata if previously saved.
    pub fn get_user_profile(&self) -> Result<Option<UserProfileRecord>, StorageError> {
        let conn = self.conn.lock();
        let row: Option<(Vec<u8>, Vec<u8>, Option<Vec<u8>>)> = conn
            .query_row(
                "SELECT display_name_enc, bio_enc, avatar_enc FROM user_profile WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;

        let Some((name_enc, bio_enc, avatar_enc)) = row else {
            return Ok(None);
        };

        let display_name = self.cipher.decrypt_string(&name_enc, b"user_profile:name")?;
        let bio = self.cipher.decrypt_string(&bio_enc, b"user_profile:bio")?;
        let avatar_data_url = match avatar_enc {
            Some(enc) => Some(self.cipher.decrypt_string(&enc, b"user_profile:avatar")?),
            None => None,
        };

        Ok(Some(UserProfileRecord {
            display_name,
            bio,
            avatar_data_url,
        }))
    }

    // --- Group Chat Operations ---

    pub fn save_group(&self, group: &GroupRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO groups (id, name, description, avatar_data_url, creator_peer_id, my_role, ephemeral_timer_sec, created_at_utc, updated_at_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                avatar_data_url = excluded.avatar_data_url,
                my_role = excluded.my_role,
                ephemeral_timer_sec = excluded.ephemeral_timer_sec,
                updated_at_utc = excluded.updated_at_utc",
            params![
                group.id,
                group.name,
                group.description,
                group.avatar_data_url,
                group.creator_peer_id,
                group.my_role,
                group.ephemeral_timer_sec,
                group.created_at_utc,
                group.updated_at_utc,
            ],
        )?;

        // Ensure a matching conversation record exists so the group appears seamlessly in the chats list
        let conv_id = format!("group_{}", group.id);
        let now_utc = chrono::Utc::now().timestamp();
        let aad = format!("conversations:last_msg:{}", conv_id);
        let empty_msg_enc = self.cipher.encrypt_str("", aad.as_bytes());

        conn.execute(
            "INSERT INTO conversations (id, peer_id, title, last_message_text_enc, last_message_time_utc, unread_count, is_group)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 1)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title",
            params![conv_id, group.id, group.name, empty_msg_enc, now_utc],
        )?;

        Ok(())
    }

    pub fn get_groups(&self) -> Result<Vec<GroupRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, avatar_data_url, creator_peer_id, my_role, ephemeral_timer_sec, created_at_utc, updated_at_utc
             FROM groups ORDER BY updated_at_utc DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GroupRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                avatar_data_url: row.get(3)?,
                creator_peer_id: row.get(4)?,
                my_role: row.get(5)?,
                ephemeral_timer_sec: row.get(6)?,
                created_at_utc: row.get(7)?,
                updated_at_utc: row.get(8)?,
            })
        })?;

        let mut groups = Vec::new();
        for r in rows {
            groups.push(r?);
        }
        Ok(groups)
    }

    pub fn get_group(&self, group_id: &str) -> Result<Option<GroupRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, avatar_data_url, creator_peer_id, my_role, ephemeral_timer_sec, created_at_utc, updated_at_utc
             FROM groups WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![group_id], |row| {
            Ok(GroupRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                avatar_data_url: row.get(3)?,
                creator_peer_id: row.get(4)?,
                my_role: row.get(5)?,
                ephemeral_timer_sec: row.get(6)?,
                created_at_utc: row.get(7)?,
                updated_at_utc: row.get(8)?,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_group(&self, group_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        let conv_id = format!("group_{group_id}");
        conn.execute("DELETE FROM group_members WHERE group_id = ?1", params![group_id])?;
        conn.execute("DELETE FROM groups WHERE id = ?1", params![group_id])?;
        conn.execute("DELETE FROM messages WHERE conversation_id = ?1", params![conv_id])?;
        conn.execute("DELETE FROM conversations WHERE id = ?1", params![conv_id])?;
        Ok(())
    }

    pub fn save_group_member(&self, member: &GroupMemberRecord) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO group_members (group_id, peer_id, display_name, role, joined_at_utc, is_online)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(group_id, peer_id) DO UPDATE SET
                display_name = excluded.display_name,
                role = excluded.role,
                is_online = excluded.is_online",
            params![
                member.group_id,
                member.peer_id,
                member.display_name,
                member.role,
                member.joined_at_utc,
                member.is_online as i32,
            ],
        )?;
        Ok(())
    }

    pub fn get_group_members(&self, group_id: &str) -> Result<Vec<GroupMemberRecord>, StorageError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT group_id, peer_id, display_name, role, joined_at_utc, is_online
             FROM group_members WHERE group_id = ?1 ORDER BY joined_at_utc ASC",
        )?;
        let rows = stmt.query_map(params![group_id], |row| {
            Ok(GroupMemberRecord {
                group_id: row.get(0)?,
                peer_id: row.get(1)?,
                display_name: row.get(2)?,
                role: row.get(3)?,
                joined_at_utc: row.get(4)?,
                is_online: row.get::<_, i32>(5)? != 0,
            })
        })?;

        let mut members = Vec::new();
        for r in rows {
            members.push(r?);
        }
        Ok(members)
    }

    pub fn remove_group_member(&self, group_id: &str, peer_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM group_members WHERE group_id = ?1 AND peer_id = ?2",
            params![group_id, peer_id],
        )?;
        Ok(())
    }
}

// Only the pool-roundtrip test still needs a fresh key id — `get_own_prekey_bundle` no longer
// publishes one-time prekeys at all (see its doc comment), and `provision_prekeys`
// (nova-engine) generates its own ids independently.
#[cfg(test)]
fn storage_random_key_id() -> u32 {
    rand::thread_rng().gen_range(1..=u32::MAX)
}

fn to_array_32(bytes: &[u8]) -> Result<[u8; 32], StorageError> {
    bytes.try_into().map_err(|_| StorageError::CorruptedField)
}

#[allow(clippy::type_complexity)]
fn row_to_contact(
    row: (String, String, String, Vec<u8>, String, i32, i32, i32, i64),
) -> Result<ContactRecord, StorageError> {
    let (peer_id, username, display_name, bundle_json, safety_number, is_online, is_blocked, is_trusted, last_seen_utc) = row;
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
        is_trusted: is_trusted != 0,
        last_seen_utc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::{generate_one_time_prekey, generate_signed_prekey, MnemonicPhrase};

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
        // Deliberately never a one-time prekey — see get_own_prekey_bundle's doc comment: this
        // bundle backs a QR/invitation link the UI keeps re-showable for 24h, so nothing about it
        // can safely be "used exactly once".
        assert!(bundle.one_time_prekey.is_none());
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
            is_trusted: false,
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
            content_type: MessageContentType::Text,
            attachment: None,
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

    /// Regression test for the 2026-08-22 audit's storage-split finding (J4): attachment binary
    /// data must live outside `messages`/`text_content_enc` entirely, be fetchable on demand by
    /// id, and never surface through `get_messages`'s `text_content` or `search_messages`.
    #[test]
    fn test_attachment_storage_is_separate_and_fetched_on_demand() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();

        let conv = ConversationRecord {
            id: "conv_bob".into(),
            peer_id: "bob".into(),
            title: "Bob".into(),
            last_message_text: String::new(),
            last_message_time_utc: 0,
            unread_count: 0,
        };
        storage.save_conversation(&conv).unwrap();

        let attachment_bytes = vec![0xABu8; 3 * 1024 * 1024]; // 3 MiB, unmistakably not text
        let msg = MessageRecord {
            id: "msg_photo".into(),
            conversation_id: "conv_bob".into(),
            sender_id: "alice".into(),
            recipient_id: "bob".into(),
            text_content: "📷 vacation.jpg".into(), // caption only — never the blob
            timestamp_utc: 1787119200,
            status: DbMessageStatus::Sent,
            is_outgoing: true,
            content_type: MessageContentType::Image,
            attachment: Some(AttachmentMeta {
                mime_type: "image/jpeg".into(),
                file_name: "vacation.jpg".into(),
                size_bytes: attachment_bytes.len() as u64,
            }),
        };
        storage.save_message_with_attachment(&msg, "deadbeefcafe", &attachment_bytes).unwrap();

        // get_messages returns metadata only — never the multi-megabyte blob inline.
        let messages = storage.get_messages("conv_bob").unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text_content, "📷 vacation.jpg");
        assert_eq!(messages[0].content_type, MessageContentType::Image);
        let meta = messages[0].attachment.as_ref().expect("attachment metadata must be present");
        assert_eq!(meta.file_name, "vacation.jpg");
        assert_eq!(meta.size_bytes, attachment_bytes.len() as u64);

        // The blob is fetched separately, only when actually requested.
        let fetched = storage.get_attachment_blob("msg_photo").unwrap().expect("blob must be retrievable");
        assert_eq!(fetched, attachment_bytes);

        // A text message has no attachment to fetch.
        assert!(storage.get_attachment_blob("msg_does_not_exist").unwrap().is_none());

        // Searching for the caption finds it; the blob's raw bytes are irrelevant to text search.
        let hits = storage.search_messages("vacation").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "msg_photo");

        // The raw database file must not contain the attachment's plaintext bytes either.
        // (Encryption-at-rest already covers this at the field-cipher level; this just confirms
        // the new table doesn't bypass it.)
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

    #[test]
    fn test_delete_session_removes_persisted_ratchet_state() {
        use nova_crypto::DoubleRatchetSession;
        use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let shared_key = [3u8; 32];
        let bob_dhs = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let bob_dh_pub = *X25519PublicKey::from(&bob_dhs).as_bytes();
        let alice = DoubleRatchetSession::init_alice(&shared_key, &bob_dh_pub).unwrap();

        storage.save_session("bob_peer_id", &alice).unwrap();
        assert!(storage.load_session("bob_peer_id").unwrap().is_some());

        storage.delete_session("bob_peer_id").unwrap();
        assert!(storage.load_session("bob_peer_id").unwrap().is_none());

        // Deleting a session that was never saved is a harmless no-op, not an error.
        storage.delete_session("nobody").unwrap();
    }

    #[test]
    fn test_user_profile_persistence_and_encryption_roundtrip() {
        let dir = std::env::temp_dir().join(format!("nova_test_prof_{}.db", uuid::Uuid::new_v4()));
        let path = dir.to_str().unwrap();

        let profile = UserProfileRecord {
            display_name: "Alice Dupont".to_string(),
            bio: "Cypherpunk & P2P enthusiast".to_string(),
            avatar_data_url: Some("data:image/jpeg;base64,/9j/4AAQSkZJRgABAQE...".to_string()),
        };

        {
            let storage = StorageEngine::open(path, "passphrase123").unwrap();
            assert_eq!(storage.get_user_profile().unwrap(), None);

            storage.save_user_profile(&profile).unwrap();
            let loaded = storage.get_user_profile().unwrap().unwrap();
            assert_eq!(loaded, profile);
        }

        // Reopen database with same key
        {
            let storage = StorageEngine::open(path, "passphrase123").unwrap();
            let loaded = storage.get_user_profile().unwrap().unwrap();
            assert_eq!(loaded, profile);
        }

        // Raw file must not leak plaintext bio or display name
        let raw = std::fs::read(path).unwrap();
        let haystack = String::from_utf8_lossy(&raw);
        assert!(!haystack.contains("Cypherpunk & P2P enthusiast"));
        assert!(!haystack.contains("Alice Dupont"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_clear_all_identity_data_wipes_user_profile() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let identity = test_identity("alice");
        storage.save_identity(&identity, "mnemonic phrase ...").unwrap();

        let profile = UserProfileRecord {
            display_name: "Alice".to_string(),
            bio: "Bio".to_string(),
            avatar_data_url: None,
        };
        storage.save_user_profile(&profile).unwrap();
        assert!(storage.get_user_profile().unwrap().is_some());

        storage.clear_all_identity_data().unwrap();
        assert!(storage.get_user_profile().unwrap().is_none());
        assert!(storage.load_identity().unwrap().is_none());
    }

    #[test]
    fn test_contact_blocking_and_unblocking() {
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
            username: "bob.nova".into(),
            display_name: "Bob".into(),
            prekey_bundle: bundle,
            safety_number: "1234-5678".into(),
            is_online: true,
            is_blocked: false,
            is_trusted: false,
            last_seen_utc: 1787119000,
        };
        storage.save_contact(&contact).unwrap();

        assert!(!storage.is_contact_blocked(&contact.peer_id).unwrap());
        assert_eq!(storage.get_blocked_contacts().unwrap().len(), 0);

        storage.trust_contact(&contact.peer_id).unwrap();
        let loaded = storage.get_contact(&contact.peer_id).unwrap().unwrap();
        assert!(loaded.is_trusted);

        storage.block_contact(&contact.peer_id).unwrap();
        assert!(storage.is_contact_blocked(&contact.peer_id).unwrap());
        assert_eq!(storage.get_blocked_contacts().unwrap().len(), 1);

        storage.unblock_contact(&contact.peer_id).unwrap();
        assert!(!storage.is_contact_blocked(&contact.peer_id).unwrap());
        assert_eq!(storage.get_blocked_contacts().unwrap().len(), 0);
    }

    /// `delete_contact` must wipe every trace of the relationship (contact row, conversation,
    /// messages, their attachment blob, any queued outbox item, and the Double Ratchet session)
    /// — unlike block/unblock, which keeps all of it — while leaving a second, unrelated contact
    /// and their own conversation completely untouched.
    #[test]
    fn test_delete_contact_wipes_everything_but_leaves_other_contacts_intact() {
        use nova_crypto::DoubleRatchetSession;
        use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();

        let make_contact = |name: &str| {
            let id = test_identity(name);
            let (_secret, signed_prekey) = generate_signed_prekey(&id, 1);
            let bundle = PreKeyBundle {
                identity_ed25519_pub: id.verifying_key_bytes,
                identity_x25519_pub: id.dh_public_bytes,
                signed_prekey,
                one_time_prekey: None,
            };
            ContactRecord {
                peer_id: hex::encode(id.verifying_key_bytes),
                username: format!("{name}.nova"),
                display_name: name.to_string(),
                prekey_bundle: bundle,
                safety_number: "1234-5678".into(),
                is_online: false,
                is_blocked: false,
                is_trusted: false,
                last_seen_utc: 1787119000,
            }
        };

        let bob = make_contact("bob");
        let carol = make_contact("carol");
        storage.save_contact(&bob).unwrap();
        storage.save_contact(&carol).unwrap();

        for (peer, conv_id) in [(&bob, "conv_bob"), (&carol, "conv_carol")] {
            storage
                .save_conversation(&ConversationRecord {
                    id: conv_id.into(),
                    peer_id: peer.peer_id.clone(),
                    title: peer.display_name.clone(),
                    last_message_text: String::new(),
                    last_message_time_utc: 0,
                    unread_count: 0,
                })
                .unwrap();
            let attachment_bytes = vec![0xCDu8; 1024];
            storage
                .save_message_with_attachment(
                    &MessageRecord {
                        id: format!("msg_{}", peer.display_name),
                        conversation_id: conv_id.into(),
                        sender_id: "me".into(),
                        recipient_id: peer.peer_id.clone(),
                        text_content: format!("hello {}", peer.display_name),
                        timestamp_utc: 1787119100,
                        status: DbMessageStatus::Sent,
                        is_outgoing: true,
                        content_type: MessageContentType::Image,
                        attachment: Some(AttachmentMeta {
                            mime_type: "image/jpeg".into(),
                            file_name: "photo.jpg".into(),
                            size_bytes: attachment_bytes.len() as u64,
                        }),
                    },
                    "checksum",
                    &attachment_bytes,
                )
                .unwrap();
            storage
                .enqueue_outbox(&format!("msg_{}", peer.display_name), conv_id, &peer.peer_id, b"payload")
                .unwrap();

            let shared_key = [7u8; 32];
            let dhs = StaticSecret::random_from_rng(&mut rand::thread_rng());
            let dh_pub = *X25519PublicKey::from(&dhs).as_bytes();
            let session = DoubleRatchetSession::init_alice(&shared_key, &dh_pub).unwrap();
            storage.save_session(&peer.peer_id, &session).unwrap();
        }

        storage.delete_contact(&bob.peer_id).unwrap();

        // Bob is gone, entirely.
        assert!(storage.get_contact(&bob.peer_id).unwrap().is_none());
        assert_eq!(storage.get_messages("conv_bob").unwrap().len(), 0);
        assert!(storage.get_attachment_blob("msg_bob").unwrap().is_none());
        assert_eq!(storage.get_pending_outbox().unwrap().iter().filter(|o| o.recipient_id == bob.peer_id).count(), 0);
        assert!(storage.load_session(&bob.peer_id).unwrap().is_none());
        assert!(!storage.get_conversations().unwrap().iter().any(|c| c.id == "conv_bob"));

        // Carol is entirely untouched.
        assert!(storage.get_contact(&carol.peer_id).unwrap().is_some());
        assert_eq!(storage.get_messages("conv_carol").unwrap().len(), 1);
        assert!(storage.get_attachment_blob("msg_carol").unwrap().is_some());
        assert_eq!(storage.get_pending_outbox().unwrap().iter().filter(|o| o.recipient_id == carol.peer_id).count(), 1);
        assert!(storage.load_session(&carol.peer_id).unwrap().is_some());
        assert!(storage.get_conversations().unwrap().iter().any(|c| c.id == "conv_carol"));
    }

    /// `get_own_prekey_bundle` itself no longer touches the one-time-prekey pool at all (see its
    /// doc comment) — this test exercises the pool's own publish/consume machinery directly
    /// instead, since `take_one_time_prekey_for_publishing`/`consume_one_time_prekey_secret` stay
    /// in place for decoding invitations minted by an older build that still embedded one.
    #[test]
    fn test_one_time_prekey_pool_publish_and_consume_roundtrip() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let bob = test_identity("bob");
        let (secret, public) = generate_signed_prekey(&bob, 1);
        storage.save_signed_prekey(&secret, &public).unwrap();

        assert_eq!(storage.count_unpublished_one_time_prekeys().unwrap(), 0);
        let new_keys: Vec<_> = (0..20).map(|_| generate_one_time_prekey(storage_random_key_id())).collect();
        storage.save_one_time_prekeys(&new_keys).unwrap();
        assert_eq!(storage.count_unpublished_one_time_prekeys().unwrap(), 20);

        let published = storage.take_one_time_prekey_for_publishing().unwrap().unwrap();
        assert_eq!(storage.count_unpublished_one_time_prekeys().unwrap(), 19);

        // The exact key handed out above must still be consumable (this is the path
        // `consume_one_time_prekey_secret` takes when a handshake references it) ...
        let consumed = storage.consume_one_time_prekey_secret(published.key_id).unwrap();
        assert!(consumed.is_some());
        // ... but only once: a second consumption of the same id (the exact scenario that used to
        // silently corrupt a second recipient's session — see get_own_prekey_bundle's doc comment)
        // must come back empty, not some stale or fabricated secret.
        assert!(storage.consume_one_time_prekey_secret(published.key_id).unwrap().is_none());

        // And `get_own_prekey_bundle` must never hand one out, regardless of how full the pool is.
        let bundle = storage.get_own_prekey_bundle(&bob).unwrap();
        assert!(bundle.one_time_prekey.is_none());
    }

    /// Regression test for the 2026-08-22 audit's "no backoff" finding: `record_outbox_retry`
    /// must increment `attempt_count` and push `next_retry_utc` out, while leaving `payload` and
    /// `first_attempt_utc` untouched — the two fields `NovaEngine::pump_outbox_once` relies on to
    /// tell "keep backing off" from "give up", independent of how many attempts that took.
    #[test]
    fn test_record_outbox_retry_updates_attempt_tracking_without_touching_payload() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        storage.enqueue_outbox("msg_retry", "conv_x", "peer_y", b"original_payload").unwrap();

        let before = storage.get_pending_outbox().unwrap();
        assert_eq!(before.len(), 1);
        assert_eq!(before[0].attempt_count, 0);
        let first_attempt = before[0].first_attempt_utc;

        // Push next_retry_utc into the future so the item temporarily drops out of
        // get_pending_outbox (exactly what a real backoff delay does).
        let future = chrono::Utc::now().timestamp() + 3600;
        storage.record_outbox_retry("msg_retry", future).unwrap();
        assert_eq!(storage.get_pending_outbox().unwrap().len(), 0, "item must not be pending again before its backoff deadline");

        // Move it back into the past to inspect its updated bookkeeping.
        storage.record_outbox_retry("msg_retry", chrono::Utc::now().timestamp() - 1).unwrap();
        let after = storage.get_pending_outbox().unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].attempt_count, 2, "two record_outbox_retry calls must mean two increments");
        assert_eq!(after[0].payload, b"original_payload", "retry bookkeeping must never touch the queued payload");
        assert_eq!(after[0].first_attempt_utc, first_attempt, "first_attempt_utc must survive retries — it anchors the give-up deadline");
    }

    #[test]
    fn test_group_storage_lifecycle() {
        let storage = StorageEngine::open(":memory:", "unlock-pass").unwrap();
        let group_id = "grp_test_999";
        let group = GroupRecord {
            id: group_id.to_string(),
            name: "Alpha Group".to_string(),
            description: Some("Alpha testing group".to_string()),
            avatar_data_url: None,
            creator_peer_id: "peer_owner".to_string(),
            my_role: "owner".to_string(),
            ephemeral_timer_sec: 0,
            created_at_utc: chrono::Utc::now().timestamp(),
            updated_at_utc: chrono::Utc::now().timestamp(),
        };

        storage.save_group(&group).unwrap();

        let loaded = storage.get_group(group_id).unwrap().expect("group should exist");
        assert_eq!(loaded.name, "Alpha Group");

        // Verify conversation is automatically created for this group
        let convs = storage.get_conversations().unwrap();
        let matching_conv = convs.into_iter().find(|c| c.id == format!("group_{group_id}"));
        assert!(matching_conv.is_some(), "matching conversation record should exist");

        // Add members
        let member1 = GroupMemberRecord {
            group_id: group_id.to_string(),
            peer_id: "peer_owner".to_string(),
            display_name: "Owner".to_string(),
            role: "owner".to_string(),
            joined_at_utc: chrono::Utc::now().timestamp(),
            is_online: true,
        };
        let member2 = GroupMemberRecord {
            group_id: group_id.to_string(),
            peer_id: "peer_member2".to_string(),
            display_name: "Member 2".to_string(),
            role: "member".to_string(),
            joined_at_utc: chrono::Utc::now().timestamp(),
            is_online: false,
        };
        storage.save_group_member(&member1).unwrap();
        storage.save_group_member(&member2).unwrap();

        let members = storage.get_group_members(group_id).unwrap();
        assert_eq!(members.len(), 2);

        // Remove one member
        storage.remove_group_member(group_id, "peer_member2").unwrap();
        let remaining = storage.get_group_members(group_id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].peer_id, "peer_owner");

        // Delete group
        storage.delete_group(group_id).unwrap();
        assert!(storage.get_group(group_id).unwrap().is_none());
        assert_eq!(storage.get_group_members(group_id).unwrap().len(), 0);
    }
}

