# Déploiement Automatique Backend : Vercel + Supabase (PostgreSQL) + Render

Ce document explique le fonctionnement du déploiement automatisé du backend NOVA / MySend avec Supabase et Vercel/Render.

---

## 🚀 1. Déploiement 1-Clic sur Vercel avec Intégration Supabase

Cliquez sur l'URL directe ci-dessous pour cloner et déployer instantanément le backend Edge API sur Vercel avec provisionnement automatique de la base de données Supabase :

### [🔗 Déployer sur Vercel avec Supabase](https://vercel.com/new/clone?repository-url=https://github.com/MR8700/MySend&integration-ids=oac_VqOgBHqhEoFTPzGkPd7L0iH6&project-name=mysend-backend)

```text
URL directe :
https://vercel.com/new/clone?repository-url=https://github.com/MR8700/MySend&integration-ids=oac_VqOgBHqhEoFTPzGkPd7L0iH6&project-name=mysend-backend
```

> **Note technique :**
> L'identifiant `oac_VqOgBHqhEoFTPzGkPd7L0iH6` est l'identifiant officiel de l'intégration **Supabase sur Vercel**. 
> Lorsque vous cliquez sur ce lien, Vercel provisionne ou associe votre compte Supabase et injecte automatiquement les variables d'environnement nécessaires (`POSTGRES_URL`, `DATABASE_URL`, `POSTGRES_PRISMA_URL`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`).

---

## ⚡ 2. Architecture Hybride Vercel + Render + Supabase

Pour garantir une **rapidité maximale**, l'architecture est distribuée en 3 piliers complémentaires :

| Composant | Hébergeur | Rôle |
|---|---|---|
| **Edge Serverless API** | **Vercel** | Requêtes HTTP mondiales ultra-rapides (< 20ms) via Edge Network : recherche d'annuaire (`/api/directory`), présence (`/api/presence`), signalement (`/api/report`), retours (`/api/feedback`). |
| **Signalement & Blind Relay** | **Render** | Connexions WebSocket bidirectionnelles persistantes (`wss://...`) pour le handshaking P2P E2EE et la distribution instantanée des messages relayés hors-ligne. Déployable via `render.yaml`. |
| **Base de Données Globale** | **Supabase (PostgreSQL)** | Stockage centralisé ultra-rapide des profils publics (bundles de pré-clés Signal Protocol), statuts de présence, files d'attente chiffrées de bout en bout et modération. |

---

## 🛠️ 3. Indexation & Création de Schéma 100% Automatiques

Vous n'avez **aucun script SQL manuel à exécuter** :
1. Dès que le backend démarre (que ce soit sur Vercel via `api/_db.js` ou sur Render via `server/src/db.rs`), il exécute automatiquement la migration et la création des index :
   - `directory_profiles` avec index `idx_dir_username` et `idx_dir_display_name` pour recherche instantanée insensible à la casse.
   - `presence_entries` avec index `idx_presence_last_seen`.
   - `relay_queue` avec index `idx_relay_target` et `idx_relay_received`.
   - `user_reports`, `user_feedbacks`, et `banned_users`.
2. Le fichier SQL autonome reste consultable et réutilisable dans `server/migrations/001_supabase_schema.sql`.

---

## 📦 4. Déploiement sur Render (WebSocket Service)

1. Connectez votre dépôt GitHub sur [Render Dashboard](https://dashboard.render.com).
2. Sélectionnez **New Web Service** ou appliquez le fichier `render.yaml` :
   - Runtime : **Docker** (utilise `server/Dockerfile`).
   - Variable d'environnement requise :
     - `DATABASE_URL` : Copiez la chaîne de connexion Supabase (`postgresql://postgres:[PASSWORD]@db.[PROJECT_REF].supabase.co:5432/postgres` ou la variable `POSTGRES_URL` générée par Vercel).
3. Le serveur compile, démarre, initialise le schéma Supabase et écoute les WebSockets.
