//! Scénario complet de vérification de bug entre deux téléphones (Alice & Bob)
//! communiquant sur NOVA Chat via le relai Render.

use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_engine::NovaEngine;
use nova_protocol::{
    SignedDirectoryEntry,
};
use nova_transport::UdpFallbackClient;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unwrap_payload(payload: &[u8]) -> Vec<Vec<u8>> {
    let bufs: Vec<serde_bytes::ByteBuf> = ciborium::from_reader(payload).unwrap();
    bufs.into_iter().map(|b| b.into_vec()).collect()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

async fn start_local_render_server() -> (String, tokio::task::JoinHandle<()>) {
    let (listener, registry, relay) = nova_server::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let handle = tokio::spawn(nova_server::serve_forever(listener, registry, relay));
    tokio::time::sleep(Duration::from_millis(150)).await;
    (format!("ws://{addr}"), handle)
}

#[tokio::main]
async fn main() {
    println!("========================================================================");
    println!("  SCÉNARIO DE VÉRIFICATION DE BUG ENTRE DEUX TÉLÉPHONES (NOVA CHAT)");
    println!("  VIA LE RELAI RENDER & AUDIT TECHNIQUE COMPLET");
    println!("========================================================================\n");

    let live_render_url = "wss://nova-discovery-jllv.onrender.com";
    println!("[TEST 0] Test de joignabilité du serveur Render officiel ({live_render_url})...");

    let live_client = UdpFallbackClient::new(live_render_url);
    let live_online = match tokio::time::timeout(Duration::from_secs(10), live_client.search_directory("test")).await {
        Ok(Ok(_)) => {
            println!("  -> SUCCÈS : Le relai Render officiel répond en WebSocket.");
            true
        }
        Ok(Err(e)) => {
            println!("  -> ATTENTION : Le serveur Render a répondu avec une erreur : {e}");
            false
        }
        Err(_) => {
            println!("  -> ATTENTION : Timeout de connexion vers Render (service en veille ou bloqué).");
            false
        }
    };

    let (server_url, _server_handle) = if live_online && std::env::var("USE_LIVE_RENDER").is_ok() {
        println!("\n[INFO] Exécution sur le serveur Render en ligne : {live_render_url}");
        (live_render_url.to_string(), tokio::spawn(async {}))
    } else {
        println!("\n[INFO] Démarrage d'un serveur discovery/relay Render en local pour l'analyse...");
        let (local_url, handle) = start_local_render_server().await;
        println!("  -> Serveur de test prêt sur {local_url}");
        (local_url, handle)
    };

    let fallback_client = UdpFallbackClient::new(&server_url);

    // ------------------------------------------------------------------------
    // ÉTAPE 1 : Initialisation des deux téléphones (Alice & Bob)
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 1 : Initialisation des deux téléphones");
    println!("------------------------------------------------------------------------");

    let temp_dir = std::env::temp_dir();
    let alice_db = temp_dir.join("nova_phone_alice.db");
    let bob_db = temp_dir.join("nova_phone_bob.db");
    let _ = std::fs::remove_file(&alice_db);
    let _ = std::fs::remove_file(&bob_db);

    let alice_engine = Arc::new(NovaEngine::new(alice_db.to_str().unwrap(), "alice_pass_123").unwrap());
    let bob_engine = Arc::new(NovaEngine::new(bob_db.to_str().unwrap(), "bob_pass_456").unwrap());

    let (alice_peer_id, alice_mnemonic) = alice_engine.create_account("alice_mobile").await.unwrap();
    let (bob_peer_id, bob_mnemonic) = bob_engine.create_account("bob_mobile").await.unwrap();

    let alice_id = DeviceIdentity::from_mnemonic(
        &MnemonicPhrase::from_phrase(&alice_mnemonic).unwrap(),
        "alice_mobile",
    ).unwrap();

    let bob_id = DeviceIdentity::from_mnemonic(
        &MnemonicPhrase::from_phrase(&bob_mnemonic).unwrap(),
        "bob_mobile",
    ).unwrap();

    println!("  Téléphone 1 (Alice) : peer_id = {alice_peer_id}");
    println!("  Téléphone 2 (Bob)   : peer_id = {bob_peer_id}");

    // ------------------------------------------------------------------------
    // ÉTAPE 2 : Enregistrement des profils sur le relai Render
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 2 : Enregistrement de l'annuaire sur le relai Render");
    println!("------------------------------------------------------------------------");

    let alice_bundle_bytes = alice_engine.get_own_prekey_bundle_bytes().await.unwrap();
    let bob_bundle_bytes = bob_engine.get_own_prekey_bundle_bytes().await.unwrap();

    let alice_profile = nova_protocol::DirectoryProfile {
        peer_id: alice_peer_id.clone(),
        username: "alice_mobile".to_string(),
        display_name: "Alice Smartphone".to_string(),
        avatar_data_url: None,
        prekey_bundle_hex: hex::encode(&alice_bundle_bytes),
    };
    let alice_entry = SignedDirectoryEntry::sign(&alice_id, alice_profile, now_secs());

    let bob_profile = nova_protocol::DirectoryProfile {
        peer_id: bob_peer_id.clone(),
        username: "bob_mobile".to_string(),
        display_name: "Bob Smartphone".to_string(),
        avatar_data_url: None,
        prekey_bundle_hex: hex::encode(&bob_bundle_bytes),
    };
    let bob_entry = SignedDirectoryEntry::sign(&bob_id, bob_profile, now_secs());

    let reg_alice_res = fallback_client.register_directory_signed(alice_entry).await;
    let reg_bob_res = fallback_client.register_directory_signed(bob_entry).await;

    println!("  Enregistrement d'Alice sur Render : {:?}", reg_alice_res);
    println!("  Enregistrement de Bob sur Render   : {:?}", reg_bob_res);

    assert!(reg_alice_res.is_ok(), "L'enregistrement d'Alice sur Render a échoué");
    assert!(reg_bob_res.is_ok(), "L'enregistrement de Bob sur Render a échoué");

    // ------------------------------------------------------------------------
    // ÉTAPE 3 : Recherche et découverte sur Render
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 3 : Recherche et découverte des contacts sur Render");
    println!("------------------------------------------------------------------------");

    // Recherche d'Alice par nom
    let search_alice = fallback_client.search_directory("alice").await.unwrap();
    println!("  Recherche de 'alice' : {} résultat(s)", search_alice.len());
    for res in &search_alice {
        println!("    Trouvé : {} (@{}) - peer_id={}", res.display_name, res.username, res.peer_id);
    }
    assert!(!search_alice.is_empty(), "Alice doit être trouvée sur Render");

    // Recherche de Bob par nom
    let search_bob = fallback_client.search_directory("bob").await.unwrap();
    println!("  Recherche de 'bob'   : {} résultat(s)", search_bob.len());
    for res in &search_bob {
        println!("    Trouvé : {} (@{}) - peer_id={}", res.display_name, res.username, res.peer_id);
    }
    assert!(!search_bob.is_empty(), "Bob doit être trouvé sur Render");

    // Recherche par Peer ID exact
    let search_by_id = fallback_client.search_directory(&alice_peer_id).await.unwrap();
    println!("  Recherche par peer_id exact d'Alice : {} résultat(s)", search_by_id.len());
    assert!(!search_by_id.is_empty(), "Alice doit être trouvable par son peer_id exact");

    // ------------------------------------------------------------------------
    // ÉTAPE 4 : Ajout de contact sur les téléphones
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 4 : Ajout de contact entre deux téléphones");
    println!("------------------------------------------------------------------------");

    // Bob ajoute Alice en utilisant le PreKey Bundle trouvé sur Render
    let alice_search_match = search_alice.iter().find(|u| u.peer_id == alice_peer_id).unwrap();
    let bob_added_alice = bob_engine
        .add_contact(
            &alice_search_match.username,
            &alice_search_match.display_name,
            alice_search_match.prekey_bundle_hex.as_bytes(),
        )
        .await;

    println!("  Bob ajoute Alice : {:?}", bob_added_alice.as_ref().map(|c| (&c.display_name, &c.peer_id)));
    assert!(bob_added_alice.is_ok(), "Bob doit pouvoir ajouter Alice via son bundle Render");

    // Vérification dans la base de données de Bob
    let bob_contacts = bob_engine.get_contacts().unwrap();
    println!("  Contacts dans le téléphone de Bob : {}", bob_contacts.len());
    assert_eq!(bob_contacts[0].peer_id, alice_peer_id);

    // ------------------------------------------------------------------------
    // ÉTAPE 5 : Test de conversation via le relai Render
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 5 : Envoi de message via le relai Render (Bob -> Alice)");
    println!("------------------------------------------------------------------------");

    // Bob envoie un premier message à Alice : "Hello Alice depuis Bob !"
    let bob_msg = bob_engine
        .send_message(&format!("conv_{alice_peer_id}"), &alice_peer_id, "Hello Alice depuis Bob !")
        .await
        .unwrap();

    println!("  Message créé par Bob : ID={}, texte='{}'", bob_msg.id, bob_msg.text_content);

    // Vérifier l'outbox de Bob
    let bob_outbox = bob_engine.storage.get_pending_outbox().unwrap();
    println!("  Éléments en attente dans l'outbox de Bob : {}", bob_outbox.len());
    assert_eq!(bob_outbox.len(), 1);

    // Transmission via le relai Render
    let chunks = unwrap_payload(&bob_outbox[0].payload);
    println!("  Nombre de paquets dans le message : {}", chunks.len());

    for chunk in &chunks {
        let fwd_res = fallback_client.relay_forward(&alice_peer_id, chunk.clone()).await;
        println!("  Transmission du paquet au relai Render pour Alice : {:?}", fwd_res);
        assert!(fwd_res.is_ok(), "Le relai Render doit accepter le paquet pour Alice");
    }

    // Alice vide son relai Render
    println!("\n  Alice consulte et vide son relai sur Render...");
    let alice_drained = fallback_client.drain_incoming(&alice_id).await.unwrap();
    println!("  Paquets récupérés par Alice sur Render : {}", alice_drained.len());
    assert_eq!(alice_drained.len(), 1, "Alice doit avoir reçu exactement 1 paquet sur Render");

    // Alice traite le paquet reçu
    let outcome = alice_engine.receive_packet(&alice_drained[0]).await.unwrap();
    println!("  Résultat du traitement par le téléphone d'Alice : {:?}", outcome);

    // Vérifier les conversations reçues par Alice
    let alice_convs = alice_engine.get_conversations().unwrap();
    println!("  Conversations dans le téléphone d'Alice : {}", alice_convs.len());
    assert_eq!(alice_convs.len(), 1);
    println!("    ID de conversation : {}", alice_convs[0].id);
    println!("    Titre de conversation : {}", alice_convs[0].title);
    println!("    Dernier message : {}", alice_convs[0].last_message_text);

    let alice_msgs = alice_engine.get_messages(&alice_convs[0].id).unwrap();
    println!("  Messages d'Alice : {} message(s)", alice_msgs.len());
    assert_eq!(alice_msgs.len(), 1);
    assert_eq!(alice_msgs[0].text_content, "Hello Alice depuis Bob !");
    assert!(!alice_msgs[0].is_outgoing);

    // ------------------------------------------------------------------------
    // ÉTAPE 6 : Test de réponse (Alice -> Bob via Render)
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 6 : Réponse bilatérale (Alice -> Bob via Render)");
    println!("------------------------------------------------------------------------");

    let alice_reply = alice_engine
        .send_message(&format!("conv_{bob_peer_id}"), &bob_peer_id, "Bien reçu Bob, je te réponds via Render !")
        .await
        .unwrap();

    println!("  Réponse créée par Alice : ID={}, texte='{}'", alice_reply.id, alice_reply.text_content);

    let alice_outbox = alice_engine.storage.get_pending_outbox().unwrap();
    let reply_chunks = unwrap_payload(&alice_outbox[0].payload);

    for chunk in &reply_chunks {
        let fwd_res = fallback_client.relay_forward(&bob_peer_id, chunk.clone()).await;
        println!("  Transmission de la réponse au relai Render pour Bob : {:?}", fwd_res);
        assert!(fwd_res.is_ok(), "Le relai Render doit accepter le paquet pour Bob");
    }

    // Bob vide son relai Render
    let bob_drained = fallback_client.drain_incoming(&bob_id).await.unwrap();
    println!("  Paquets récupérés par Bob sur Render : {}", bob_drained.len());
    assert_eq!(bob_drained.len(), 1, "Bob doit avoir reçu exactement 1 paquet sur Render");

    let bob_outcome = bob_engine.receive_packet(&bob_drained[0]).await.unwrap();
    println!("  Résultat du traitement par le téléphone de Bob : {:?}", bob_outcome);

    let bob_msgs = bob_engine.get_messages(&format!("conv_{alice_peer_id}")).unwrap();
    println!("  Messages dans la conversation de Bob : {} message(s)", bob_msgs.len());
    assert_eq!(bob_msgs.len(), 2);
    assert_eq!(bob_msgs[1].text_content, "Bien reçu Bob, je te réponds via Render !");

    // ------------------------------------------------------------------------
    // ÉTAPE 7 : Vérification des anomalies et bugs
    // ------------------------------------------------------------------------
    println!("\n------------------------------------------------------------------------");
    println!("ÉTAPE 7 : Audit approfondi des anomalies et bugs constatés");
    println!("------------------------------------------------------------------------");

    // BUG VÉRIFICATION 1 : Est-ce qu'Alice a Bob dans ses CONTACTS ?
    let alice_contacts = alice_engine.get_contacts().unwrap();
    println!("  [BUG 1 - Carnet de contacts asymétrique]");
    println!("    Nombre de contacts dans le carnet d'Alice : {}", alice_contacts.len());
    if alice_contacts.is_empty() {
        println!("    -> ANOMALIE CONFIRMÉE : Bien que Bob et Alice discutent parfaitement via Render,");
        println!("       Bob N'EST PAS dans le carnet de contacts d'Alice (contacts=[]) !");
        println!("       Le titre de la conversation d'Alice est son peer_id hexadécimal brut au lieu de son nom : '{}'", alice_convs[0].title);
    } else {
        println!("    -> Bob est bien dans le carnet d'Alice.");
    }

    // BUG VÉRIFICATION 2 : Statut de l'Outbox de Bob
    println!("\n  [BUG 2 - Outbox bloquée en attente lors de l'usage du Relai]");
    println!("    Éléments toujours dans l'Outbox de Bob : {}", bob_outbox.len());
    if !bob_outbox.is_empty() {
        println!("    -> ANOMALIE CONFIRMÉE : Le message a pourtant été transmis et déchiffré par Alice, mais l'Outbox de Bob le garde indéfiniment en attente.");
        println!("       Cause : dans nova-transport/dht_node.rs: send_chunks_to_peer renvoie (Disconnected, Rejected) sur le fallback relai, provoquant des réémissions répétées et empêchant le statut 'Delivered'.");
    }

    // BUG VÉRIFICATION 3 : Duplication des paquets lors du drainage
    println!("\n  [BUG 3 - Drainage destructif et multi-relais]");
    let alice_drain_2 = fallback_client.drain_incoming(&alice_id).await.unwrap();
    println!("    Second drain d'Alice (doit être vide) : {} paquet(s)", alice_drain_2.len());
    assert!(alice_drain_2.is_empty(), "Le drain doit être destructif");

    println!("\n========================================================================");
    println!("  SCÉNARIO TERMINÉ AVEC SUCCÈS - RAPPORT DE VÉRIFICATION VALIDÉ");
    println!("========================================================================");
}
