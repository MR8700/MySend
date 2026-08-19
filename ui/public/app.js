// --- NOVA CHAT SOUVERAIN — PURE 1-TO-1 DIRECT P2P (15 ÉCRANS MATURES) ---

// Professional Vector SVG Icons (Lucide / Feather style)
const icons = {
    chat: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>`,
    users: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>`,
    user: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>`,
    settings: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>`,
    search: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>`,
    plus: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>`,
    arrowLeft: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"></line><polyline points="12 19 5 12 12 5"></polyline></svg>`,
    paperclip: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path></svg>`,
    mic: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="23"></line><line x1="8" y1="23" x2="16" y2="23"></line></svg>`,
    smile: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><path d="M8 14s1.5 2 4 2 4-2 4-2"></path><line x1="9" y1="9" x2="9.01" y2="9"></line><line x1="15" y1="9" x2="15.01" y2="9"></line></svg>`,
    send: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="22" y1="2" x2="11" y2="13"></line><polygon points="22 2 15 22 11 13 2 9 22 2"></polygon></svg>`,
    phone: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z"></path></svg>`,
    video: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="23 7 16 12 23 17 23 7"></polygon><rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect></svg>`,
    shield: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>`,
    lock: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>`,
    qr: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><rect x="7" y="7" width="3" height="3"></rect><rect x="14" y="7" width="3" height="3"></rect><rect x="7" y="14" width="3" height="3"></rect></svg>`,
    activity: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline></svg>`,
    bell: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path><path d="M13.73 21a2 2 0 0 1-3.46 0"></path></svg>`,
    image: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>`,
    file: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line></svg>`,
    checkCheck: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="18 6 7 17 2 12"></polyline><polyline points="22 10 13 19 11 17"></polyline></svg>`,
    trash: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>`,
    mapPin: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path><circle cx="12" cy="10" r="3"></circle></svg>`,
};

// --- SECURITY: HTML ESCAPING ---
// Message text, contact names/handles, and filenames all ultimately originate from a remote
// peer (or, for filenames, from data the user's own OS hands back to the page). None of it may
// ever be interpolated into innerHTML or into an inline event-handler attribute unescaped: a
// contact who chose a display name like `<img src=x onerror=...>`, or a message containing the
// same, would otherwise get their payload executed in the recipient's client. Every render
// path below MUST route peer-controlled text through this function first.
function escapeHtml(value) {
    return String(value ?? '').replace(/[&<>"']/g, (ch) => ({
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        '"': '&quot;',
        "'": '&#39;',
    }[ch]));
}

// Global Reactive State (Strictly 1-to-1 Device Sovereignty)
const state = {
    currentScreen: 'conversations',
    currentUser: {
        name: 'Alexandre V.',
        username: 'alex',
        handle: 'alex.nova',
        bio: 'Souveraineté numérique • Pair-à-pair direct',
        status: 'En ligne via QUIC P2P',
        publicKey: 'A7F3 92BC E451 988F ... 4D2A',
        mnemonic: 'crane jump river fabric blanket onion size stable window street faith morning',
        // Demo-only stand-in for a real local unlock check (device passcode/biometric). A
        // production build calls the native OS authentication API here instead — never a
        // client-side string compare — but the *gate itself* (no reveal without it) is real.
        localPin: '2468',
    },
    activeContact: {
        id: 'emma_1',
        name: 'Emma',
        handle: 'emma.nova',
        publicKey: 'A1B2 C3D4 E5F6 7890 ... 9A0B',
        safetyNumber: '4A9F-2B1C-88E0-9142',
        isOnline: true,
        p2pMode: 'Direct (38 ms)',
        latency: '38 ms',
    },
    // Pure 1-to-1 Sovereign Conversations
    conversations: [
        { id: 'conv_emma', name: 'Emma', handle: 'emma.nova', lastMsg: 'Salut ! Comment ça va ?', time: '09:40', unread: 2, online: true, mode: 'Direct' },
        { id: 'conv_lucas', name: 'Lucas', handle: 'lucas.nova', lastMsg: 'On se voit ce soir.', time: 'Hier', unread: 0, online: true, mode: 'Direct' },
        { id: 'conv_chloe', name: 'Chloé', handle: 'chloe.nova', lastMsg: 'Merci beaucoup !', time: 'Mar', unread: 0, online: false, mode: 'Offline' },
        { id: 'conv_thomas', name: 'Thomas', handle: 'thomas.nova', lastMsg: '📷 Photo transmise en P2P', time: 'Lun', unread: 0, online: true, mode: 'Relayed' },
        { id: 'conv_marie', name: 'Marie', handle: 'marie.nova', lastMsg: 'Parfait, merci Alex !', time: '14/08', unread: 0, online: true, mode: 'Direct' },
        { id: 'conv_antoine', name: 'Antoine', handle: 'antoine.nova', lastMsg: 'À bientôt', time: '10/08', unread: 0, online: false, mode: 'Offline' },
    ],
    messages: [
        { id: 'm1', type: 'text', text: 'Salut Alex ! Tu es bien connecté en direct ?', time: '09:37', isOutgoing: false },
        { id: 'm2', type: 'text', text: 'Salut Emma ! Oui, liaison P2P directe via QUIC.', time: '09:38', isOutgoing: true, status: 'read' },
        { id: 'm3', type: 'text', text: 'Parfait, aucun serveur ne stocke nos échanges 😊', time: '09:39', isOutgoing: false },
        { id: 'm4', type: 'text', text: 'Exactement, chiffrement Double Ratchet actif de bout en bout.', time: '09:40', isOutgoing: true, status: 'read' },
    ],
    contacts: [
        { name: 'Emma', handle: 'emma.nova', online: true, p2pMode: 'Direct (38 ms)', key: 'A1B2 C3D4 E5F6 7890 ... 9A0B' },
        { name: 'Lucas', handle: 'lucas.nova', online: true, p2pMode: 'Direct (45 ms)', key: 'F4E3 D2C1 B0A9 8765 ... 1234' },
        { name: 'Chloé', handle: 'chloe.nova', online: false, p2pMode: 'Hors ligne', key: '9876 5432 10FE DCBA ... ABCD' },
        { name: 'Thomas', handle: 'thomas.nova', online: true, p2pMode: 'Relayé (120 ms)', key: '5566 7788 99AA BBCC ... DDEE' },
        { name: 'Marie', handle: 'marie.nova', online: true, p2pMode: 'Direct (29 ms)', key: '1122 3344 5566 7788 ... 9900' },
        { name: 'Antoine', handle: 'antoine.nova', online: false, p2pMode: 'Hors ligne', key: 'AABB CCDD EEFF 0011 ... 2233' },
    ],
    isTyping: false,
    emojis: [
        '😀', '😃', '😄', '😁', '😆', '😅', '😂', '🤣', '😊', '😇',
        '🙂', '😉', '😍', '🥰', '😘', '😋', '😎', '🥳', '🤩', '😏',
        '🤔', '🤫', '🤗', '🤐', '😮', '😴', '😌', '🤓', '🥺', '😭',
        '👍', '👎', '👏', '🙌', '🤝', '👊', '✌️', '🤞', '💪', '🙏',
        '❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🔥', '✨',
        '🎉', '🚀', '⭐', '💡', '🔒', '🛡️', '⚡', '💯', '🎯', '📍'
    ]
};

// --- SCREEN RENDERERS (15 CONSOLIDATED 1-TO-1 SCREENS) ---
const screens = {
    // 1. Écran de bienvenue
    onboarding: () => `
        <div class="screen-view" style="justify-content: space-between; padding: 40px 24px; text-align: center; background: radial-gradient(circle at 50% 30%, #171A24 0%, #080A10 70%);">
            <div style="margin-top: 30px;">
                <div class="rail-logo" style="width: 72px; height: 72px; margin: 0 auto 20px; box-shadow: 0 8px 30px var(--accent-purple-glow);">
                    ${icons.shield}
                </div>
                <div style="font-size: 13px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 2px; font-weight: 600;">Souveraineté Numérique</div>
                <h1 style="font-size: 32px; font-weight: 800; color: white; margin: 8px 0 12px; letter-spacing: -0.5px;">NOVA Chat</h1>
                <p style="font-size: 14px; color: var(--text-muted); line-height: 1.5; max-width: 300px; margin: 0 auto;">Messagerie P2P sécurisée 1-to-1 sans serveur de transport ni stockage cloud.</p>
            </div>

            <div style="width: 210px; height: 210px; margin: 20px auto; border-radius: 50%; border: 1px dashed rgba(139, 92, 246, 0.4); display: flex; align-items: center; justify-content: center; position: relative;">
                <div style="width: 150px; height: 150px; border-radius: 50%; background: radial-gradient(circle, rgba(139,92,246,0.25) 0%, transparent 70%);"></div>
                <div style="position: absolute; font-size: 12px; color: var(--accent-purple-light); font-weight: 600; display: flex; align-items: center; gap: 6px;">
                    <div class="p2p-badge-pulse"></div> Liaison 1 ↔ 1 Directe
                </div>
            </div>

            <div>
                <button class="btn-primary" onclick="navigateTo('create_account')">Créer une identité</button>
                <button class="btn-secondary" style="margin-top: 12px; width: 100%;" onclick="navigateTo('create_account')">Restaurer un compte existant</button>
            </div>
        </div>
    `,

    // 2. Création de compte
    create_account: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('onboarding')">${icons.arrowLeft}</button>
                <div class="header-title">Créer un compte</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px; flex: 1; display: flex; flex-direction: column; justify-content: space-between;">
                <div>
                    <div style="width: 64px; height: 64px; border-radius: 50%; background: rgba(139, 92, 246, 0.12); border: 1px solid var(--accent-purple); display: flex; align-items: center; justify-content: center; margin: 0 auto 20px; color: var(--accent-purple-light);">
                        ${icons.lock}
                    </div>

                    <label style="font-size: 13px; color: var(--text-muted); font-weight: 500;">Nom d'affichage</label>
                    <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 12px 16px; margin: 8px 0 20px; border: 1px solid var(--border-subtle);">
                        <input type="text" id="account-name-input" value="${state.currentUser.name}" style="background: none; border: none; color: white; font-size: 15px; width: 100%; outline: none;">
                    </div>

                    <label style="font-size: 13px; color: var(--text-muted); font-weight: 500;">Phrase de récupération secrète (12 mots BIP-39)</label>
                    <div style="background-color: var(--bg-surface-2); border-radius: var(--radius-md); padding: 16px; margin: 8px 0 12px; border: 1px dashed rgba(139, 92, 246, 0.4); font-family: monospace; font-size: 14px; line-height: 1.6; color: var(--accent-purple-light);">
                        ${state.currentUser.mnemonic}
                    </div>
                    <p style="font-size: 12px; color: var(--text-muted); line-height: 1.4;">Vos clés privées restent sur cet appareil. Cette phrase permet de régénérer vos paires de clés Ed25519/X25519.</p>
                </div>

                <button class="btn-primary" onclick="navigateTo('conversations')">Valider et Rejoindre le Réseau</button>
            </div>
        </div>
    `,

    // 3. Liste des conversations (100% 1-to-1)
    conversations: () => `
        <div class="screen-view">
            <header class="app-header">
                <div class="header-title">Conversations</div>
                <div class="header-actions">
                    <button class="icon-btn" onclick="navigateTo('global_search')" title="Recherche">${icons.search}</button>
                    <button class="icon-btn" onclick="navigateTo('add_contact')" title="Ajouter un contact">${icons.plus}</button>
                </div>
            </header>

            <div class="search-bar-wrap">
                <div class="search-input-box" onclick="navigateTo('global_search')">
                    ${icons.search}
                    <input type="text" placeholder="Rechercher une conversation..." readonly>
                </div>
            </div>

            <div class="scroll-list">
                ${state.conversations.map(c => `
                    <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" onclick="openChatWithEl(this)">
                        <div class="avatar">
                            ${escapeHtml(c.name.charAt(0))}
                            <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                        </div>
                        <div class="item-content">
                            <div class="item-header">
                                <span class="item-name" style="${c.unread > 0 ? 'font-weight: 700; color: white;' : ''}">${escapeHtml(c.name)}</span>
                                <span class="item-time" style="${c.unread > 0 ? 'color: var(--accent-purple-light); font-weight: 600;' : ''}">${escapeHtml(c.time)}</span>
                            </div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 2px;">
                                <span class="item-sub" style="${c.unread > 0 ? 'color: var(--text-main); font-weight: 600;' : ''}">${escapeHtml(c.lastMsg)}</span>
                                ${c.unread > 0 ? `<span class="badge-unread">${escapeHtml(String(c.unread))}</span>` : `<span class="status-tick-read" style="margin-left: 6px;">${icons.checkCheck}</span>`}
                            </div>
                        </div>
                    </div>
                `).join('')}
            </div>
        </div>
    `,

    // 4. Conversation individuelle (1-to-1)
    chat: () => `
        <div class="screen-view">
            <!-- Hidden native file pickers for real device file access -->
            <input type="file" id="media-file-input" accept="image/*,video/*" style="display: none;" onchange="handleMediaFileSelect(event)">
            <input type="file" id="doc-file-input" accept=".pdf,.doc,.docx,.xls,.xlsx,.txt,.zip,.json" style="display: none;" onchange="handleDocFileSelect(event)">

            <header class="app-header">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <button class="icon-btn" onclick="navigateTo('conversations')">${icons.arrowLeft}</button>
                    <div class="avatar" style="width: 38px; height: 38px; font-size: 14px; cursor: pointer;" onclick="navigateTo('contact_profile')">
                        ${state.activeContact.name.charAt(0)}
                        <div class="status-dot status-online"></div>
                    </div>
                    <div onclick="navigateTo('contact_profile')" style="cursor: pointer;">
                        <div style="font-size: 15px; font-weight: 700; color: white;">${state.activeContact.name}</div>
                        <div style="font-size: 11px; color: var(--status-success); font-weight: 500; display: flex; align-items: center; gap: 4px;">
                            <span style="font-size: 8px;">●</span> En ligne (<span id="chat-header-latency">${state.activeContact.latencyMs || 14} ms</span>)
                        </div>
                    </div>
                </div>
                <div class="header-actions">
                    <button class="icon-btn" onclick="alert('Appel vocal sécurisé P2P vers ' + state.activeContact.name)" title="Appel vocal">${icons.phone}</button>
                    <button class="icon-btn" onclick="alert('Appel vidéo chiffré vers ' + state.activeContact.name)" title="Appel vidéo">${icons.video}</button>
                    <button class="icon-btn" onclick="navigateTo('contact_profile')" title="Infos du contact">${icons.user}</button>
                </div>
            </header>

            <div class="chat-body" id="chat-body">
                <div style="text-align: center; margin: 10px 0;">
                    <span style="background: rgba(139, 92, 246, 0.1); border: 1px solid rgba(139, 92, 246, 0.2); border-radius: var(--radius-full); padding: 4px 12px; font-size: 11px; color: var(--accent-purple-light); display: inline-flex; align-items: center; gap: 6px;">
                        ${icons.lock} Chiffrement Double Ratchet 1-to-1 Actif
                    </span>
                </div>

                ${state.messages.map(m => buildMessageHtml(m)).join('')}
            </div>

            <!-- Drawer for Multimedia Attachments -->
            <div class="attachment-drawer" id="attachment-drawer">
                <button class="drawer-option" onclick="triggerDeviceMediaPicker()">
                    ${icons.image}
                    <span>Photo & Vidéo HD</span>
                </button>
                <button class="drawer-option" onclick="triggerDeviceDocPicker()">
                    ${icons.file}
                    <span>Document Sécurisé</span>
                </button>
                <button class="drawer-option" onclick="startVoiceRecording(); closePanels();">
                    ${icons.mic}
                    <span>Note Vocale</span>
                </button>
                <button class="drawer-option" onclick="openLocationModal(); closePanels();">
                    ${icons.mapPin}
                    <span>Position Géographique</span>
                </button>
            </div>

            <!-- Clean HD Emoji Picker Drawer -->
            <div class="emoji-picker-panel" id="emoji-picker-panel">
                <div class="emoji-picker-header">
                    <span style="font-size: 12px; font-weight: 700; color: var(--text-muted); letter-spacing: 0.5px;">ÉMOJIS</span>
                    <button class="icon-btn" onclick="closePanels()" style="width: 24px; height: 24px; font-size: 12px;" title="Fermer">✕</button>
                </div>
                <div class="emoji-picker-grid">
                    ${state.emojis.map(e => `<button class="emoji-btn" onclick="insertEmoji('${e}')" title="${e}">${e}</button>`).join('')}
                </div>
            </div>

            <!-- Secure Chat Input Bar & WhatsApp Voice Recorder -->
            <div class="chat-input-bar">
                <div class="chat-input-main-row" id="normal-input-row">
                    <button class="icon-btn" id="attachment-toggle-btn" onclick="toggleAttachmentDrawer(event)" title="Pièces jointes">${icons.paperclip}</button>
                    <button class="icon-btn" id="emoji-toggle-btn" onclick="toggleEmojiPicker(event)" title="Émojis">${icons.smile}</button>
                    <div class="chat-input-container">
                        <input type="text" id="chat-input" placeholder="Votre message" onkeydown="if(event.key==='Enter') sendMessage()" autofocus>
                    </div>
                    <button class="send-btn" id="send-btn" onclick="sendMessage()" title="Envoyer le message">${icons.send}</button>
                    <button class="mic-btn" id="mic-record-btn" onclick="startVoiceRecording()" title="Enregistrer une note vocale">${icons.mic}</button>
                </div>

                <!-- WhatsApp-like Voice Recording Row with Pause, Resume, Play Preview, Trash & Send -->
                <div class="voice-recording-bar" id="voice-recording-bar">
                    <div class="rec-status-col">
                        <div class="rec-indicator" id="rec-indicator"></div>
                        <span class="rec-timer" id="rec-timer">00:00</span>
                    </div>

                    <div class="rec-live-waveform" id="rec-live-waveform">
                        <div class="rec-wave-bar" style="height: 10px;"></div>
                        <div class="rec-wave-bar" style="height: 16px;"></div>
                        <div class="rec-wave-bar" style="height: 8px;"></div>
                        <div class="rec-wave-bar" style="height: 20px;"></div>
                        <div class="rec-wave-bar" style="height: 14px;"></div>
                        <div class="rec-wave-bar" style="height: 18px;"></div>
                        <div class="rec-wave-bar" style="height: 12px;"></div>
                        <div class="rec-wave-bar" style="height: 6px;"></div>
                        <div class="rec-wave-bar" style="height: 15px;"></div>
                        <div class="rec-wave-bar" style="height: 9px;"></div>
                    </div>

                    <!-- Cancel / Delete button -->
                    <button class="rec-icon-btn" onclick="cancelVoiceRecording()" title="Supprimer l'enregistrement">
                        ${icons.trash}
                    </button>

                    <!-- Pause / Resume button -->
                    <button class="rec-icon-btn" id="rec-pause-btn" onclick="togglePauseVoiceRecording()" title="Mettre en pause / Reprendre">
                        <span id="rec-pause-icon">⏸</span>
                    </button>

                    <!-- Preview / Listen button (active when paused) -->
                    <button class="rec-icon-btn" id="rec-preview-btn" onclick="toggleVoicePreview()" title="Écouter l'enregistrement" style="display: none; color: var(--accent-purple-light);">
                        <span id="rec-preview-icon">▶</span>
                    </button>

                    <!-- Send button -->
                    <button class="rec-send-btn" onclick="stopAndSendVoiceRecording()" title="Envoyer la note vocale">
                        ${icons.send}
                    </button>
                </div>
            </div>

            <!-- Location Sharing Confirmation Modal with Map Preview -->
            <div class="location-modal-overlay" id="location-modal">
                <div class="location-modal-card">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                        <div style="font-size: 16px; font-weight: 700; color: white; display: flex; align-items: center; gap: 8px;">
                            ${icons.mapPin} Position Géographique
                        </div>
                        <button class="icon-btn" onclick="closeLocationModal()" style="width: 28px; height: 28px;">✕</button>
                    </div>
                    <p style="font-size: 13px; color: var(--text-muted);">
                        Voulez-vous partager votre position actuelle avec <strong>${state.activeContact.name}</strong> ? Les coordonnées GPS seront chiffrées de bout en bout via Double Ratchet.
                    </p>

                    <div class="map-radar-preview">
                        <div class="map-grid-lines"></div>
                        <div class="map-pin-pulse">${icons.mapPin}</div>
                        <div style="position: absolute; bottom: 8px; font-size: 11px; font-weight: 600; color: var(--accent-purple-light); z-index: 2;" id="loc-coords-preview">
                            48.8566° N, 2.3522° E (Paris)
                        </div>
                    </div>

                    <div style="font-size: 11px; color: var(--text-dim); text-align: center; margin-bottom: 18px;">
                        Précision estimée : ~5 mètres • Aucun tiers serveur
                    </div>

                    <div style="display: flex; gap: 10px;">
                        <button class="btn-secondary" style="flex: 1;" onclick="closeLocationModal()">Annuler</button>
                        <button class="btn-primary" style="flex: 1;" onclick="confirmAndSendLocation()">Partager la Position</button>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 5. Profil du contact
    contact_profile: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('chat')">${icons.arrowLeft}</button>
                <div class="header-title">Profil du contact</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px 20px; overflow-y: auto;">
                <div style="text-align: center; margin-bottom: 24px;">
                    <div class="avatar" style="width: 84px; height: 84px; font-size: 32px; margin: 0 auto 12px;">
                        ${state.activeContact.name.charAt(0)}
                    </div>
                    <h2 style="font-size: 20px; font-weight: 700; color: white;">${state.activeContact.name}</h2>
                    <div style="font-size: 13px; color: var(--status-success); margin-top: 4px; display: flex; align-items: center; justify-content: center; gap: 6px;">
                        <span class="p2p-badge-pulse" style="width:8px;height:8px;"></span> En ligne • Connexion Directe
                    </div>
                </div>

                <div style="display: flex; justify-content: space-around; margin-bottom: 24px;">
                    <button class="btn-secondary" style="flex-direction: column; padding: 12px; font-size: 11px;" onclick="navigateTo('chat')">
                        ${icons.chat}
                        <span style="margin-top:4px;">Message</span>
                    </button>
                    <button class="btn-secondary" style="flex-direction: column; padding: 12px; font-size: 11px;" onclick="navigateTo('connection_diagnostics')">
                        ${icons.activity}
                        <span style="margin-top:4px;">Diagnostic</span>
                    </button>
                    <button class="btn-secondary" style="flex-direction: column; padding: 12px; font-size: 11px;" onclick="navigateTo('shared_media')">
                        ${icons.image}
                        <span style="margin-top:4px;">Médias</span>
                    </button>
                </div>

                <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 16px; border: 1px solid var(--border-subtle);">
                    <div style="font-size: 12px; color: var(--text-muted);">Identifiant souverain</div>
                    <div style="font-size: 15px; font-weight: 600; margin-top: 4px; color: white;">@${state.activeContact.handle}</div>
                    
                    <div style="height: 1px; background: var(--border-subtle); margin: 12px 0;"></div>

                    <div style="font-size: 12px; color: var(--text-muted);">Clé publique Ed25519</div>
                    <div style="font-size: 13px; font-family: monospace; color: var(--accent-purple-light); margin-top: 4px;">${state.activeContact.publicKey}</div>

                    <div style="height: 1px; background: var(--border-subtle); margin: 12px 0;"></div>

                    <div style="font-size: 12px; color: var(--text-muted);">Empreinte de sécurité (Safety Number)</div>
                    <div style="font-size: 14px; font-family: monospace; font-weight: 700; color: white; margin-top: 4px;">${state.activeContact.safetyNumber}</div>
                </div>

                <button class="btn-secondary" style="width: 100%; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.2);" onclick="alert('Contact bloqué'); navigateTo('conversations')">Bloquer ce contact</button>
            </div>
        </div>
    `,

    // 6. Liste des contacts
    contacts: () => `
        <div class="screen-view">
            <header class="app-header">
                <div class="header-title">Contacts</div>
                <div class="header-actions">
                    <button class="icon-btn" onclick="navigateTo('add_contact')" title="Ajouter un contact">${icons.plus}</button>
                </div>
            </header>

            <div class="search-bar-wrap">
                <div class="search-input-box">
                    ${icons.search}
                    <input type="text" placeholder="Rechercher un contact..." oninput="filterContactsList(this.value)">
                </div>
            </div>

            <div class="scroll-list" id="contacts-list-container">
                ${state.contacts.map(c => `
                    <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" onclick="openChatWithEl(this)">
                        <div class="avatar">
                            ${escapeHtml(c.name.charAt(0))}
                            <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                        </div>
                        <div class="item-content">
                            <div class="item-name">${escapeHtml(c.name)}</div>
                            <div class="item-sub">@${escapeHtml(c.handle)} • ${escapeHtml(c.p2pMode)}</div>
                        </div>
                    </div>
                `).join('')}
            </div>
        </div>
    `,

    // 7. Ajouter un contact
    add_contact: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('contacts')">${icons.arrowLeft}</button>
                <div class="header-title">Ajouter un contact</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px; overflow-y: auto;">
                <label style="font-size: 13px; color: var(--text-muted);">Recherche par identifiant pair</label>
                <div style="display: flex; gap: 10px; margin: 8px 0 24px;">
                    <div class="search-input-box" style="flex: 1;">
                        <span style="color: var(--text-muted);">@</span>
                        <input type="text" id="add-handle-input" placeholder="nom.nova">
                    </div>
                    <button class="btn-primary" style="width: auto; padding: 0 18px;" onclick="addNewContact()">Ajouter</button>
                </div>

                <div style="height: 1px; background: var(--border-subtle); margin-bottom: 24px;"></div>

                <button class="btn-secondary" style="width: 100%; margin-bottom: 12px; padding: 16px;" onclick="navigateTo('identity_qrcode')">
                    ${icons.qr}
                    <span>Scanner le QR code d'un pair</span>
                </button>

                <button class="btn-secondary" style="width: 100%; padding: 16px;" onclick="navigateTo('identity_qrcode')">
                    ${icons.share}
                    <span>Afficher mon QR code public</span>
                </button>
            </div>
        </div>
    `,

    // 8. Médias partagés (1-to-1)
    shared_media: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('chat')">${icons.arrowLeft}</button>
                <div class="header-title">Médias partagés</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="display: flex; gap: 8px; padding: 12px 20px; border-bottom: 1px solid var(--border-subtle);">
                <button class="btn-primary" style="padding: 6px 14px; font-size: 12px; width: auto;">Photos</button>
                <button class="btn-secondary" style="padding: 6px 14px; font-size: 12px; width: auto;">Fichiers</button>
                <button class="btn-secondary" style="padding: 6px 14px; font-size: 12px; width: auto;">Liens</button>
            </div>

            <div style="padding: 16px; overflow-y: auto;">
                <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 8px;">Transmis directement en P2P</div>
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-bottom: 16px;">
                    <div style="aspect-ratio: 1; background: #2A1F4D; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; color: var(--accent-purple-light);">${icons.image}</div>
                    <div style="aspect-ratio: 1; background: #1B3B4B; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; color: #38BDF8;">${icons.image}</div>
                    <div style="aspect-ratio: 1; background: #3B2A1B; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; color: #F59E0B;">${icons.image}</div>
                </div>

                <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 8px;">Fichiers sécurisés</div>
                <div style="background: var(--bg-surface); padding: 12px; border-radius: var(--radius-md); display: flex; align-items: center; gap: 12px; border: 1px solid var(--border-subtle);">
                    ${icons.file}
                    <div>
                        <div style="font-size: 14px; font-weight: 600; color: white;">rapport_architecture_nova.pdf</div>
                        <div style="font-size: 11px; color: var(--text-muted);">2.0 MB • Chiffré E2EE</div>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 9. Réglages (Gestion de profil complet, Options & Accès aux Écrans)
    settings: () => `
        <div class="screen-view">
            <header class="app-header">
                <div class="header-title">Réglages</div>
            </header>

            <div style="padding: 16px; overflow-y: auto; padding-bottom: 90px;">
                <!-- Full Profile Card -->
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 18px; border: 1px solid var(--border-subtle); margin-bottom: 20px;">
                    <div style="display: flex; align-items: center; gap: 14px; margin-bottom: 12px;">
                        <div class="avatar" style="width: 56px; height: 56px; font-size: 22px; background: var(--accent-purple); color: white;">
                            ${state.currentUser.name.charAt(0)}
                        </div>
                        <div style="flex: 1;">
                            <div style="font-size: 17px; font-weight: 700; color: white;">${state.currentUser.name}</div>
                            <div style="font-size: 13px; color: var(--accent-purple-light);">@${state.currentUser.handle}</div>
                            <div style="font-size: 11px; color: var(--text-muted); margin-top: 2px;">${state.currentUser.status}</div>
                        </div>
                        <button class="icon-btn" onclick="navigateTo('identity_qrcode')" title="Mon QR Code">${icons.qr}</button>
                    </div>
                    <div style="font-size: 12px; color: var(--text-muted); line-height: 1.4; border-top: 1px solid var(--border-subtle); padding-top: 10px;">
                        ${state.currentUser.bio}
                    </div>
                </div>

                <!-- Settings Menus -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">OPTIONS PRINCIPALES</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); overflow: hidden; margin-bottom: 24px; border: 1px solid var(--border-subtle);">
                    <div class="item-card" onclick="navigateTo('global_search')">
                        ${icons.search}
                        <div class="item-content"><div class="item-name">Recherche globale</div></div>
                        ${icons.chevronRight}
                    </div>
                    <div class="item-card" onclick="navigateTo('identity_security')">
                        ${icons.shield}
                        <div class="item-content"><div class="item-name">Identité & Sécurité</div></div>
                        ${icons.chevronRight}
                    </div>
                    <div class="item-card" onclick="navigateTo('connected_devices')">
                        ${icons.user}
                        <div class="item-content"><div class="item-name">Appareils connectés</div></div>
                        ${icons.chevronRight}
                    </div>
                    <div class="item-card" onclick="navigateTo('connection_diagnostics')">
                        ${icons.activity}
                        <div class="item-content"><div class="item-name">État de connexion & Diagnostics</div></div>
                        ${icons.chevronRight}
                    </div>
                    <div class="item-card" onclick="navigateTo('notifications')">
                        ${icons.bell}
                        <div class="item-content"><div class="item-name">Notifications</div></div>
                        ${icons.chevronRight}
                    </div>
                </div>

                <!-- Complete 15 Screens Direct Access Grid -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">NAVIGATION DIRECTE DES 15 ÉCRANS</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 14px; border: 1px solid var(--border-subtle); margin-bottom: 24px;">
                    <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px;">
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('onboarding')">1. Bienvenue</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('create_account')">2. Création compte</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('conversations')">3. Conversations</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('chat')">4. Chat 1-to-1</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('contact_profile')">5. Profil contact</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('contacts')">6. Liste contacts</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('add_contact')">7. Ajouter contact</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('shared_media')">8. Médias partagés</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('settings')">9. Réglages</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('identity_security')">10. Sécurité</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('connected_devices')">11. Appareils</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('identity_qrcode')">12. QR Code</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('connection_diagnostics')">13. Diagnostics</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start;" onclick="navigateTo('notifications')">14. Notifications</button>
                        <button class="btn-secondary" style="font-size: 12px; padding: 10px 8px; justify-content: flex-start; grid-column: span 2;" onclick="navigateTo('global_search')">15. Recherche globale</button>
                    </div>
                </div>

                <button class="btn-secondary" style="width: 100%; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.2);" onclick="navigateTo('onboarding')">Se déconnecter de cet appareil</button>
            </div>
        </div>
    `,

    // 10. Identité & Sécurité
    identity_security: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('settings')">${icons.arrowLeft}</button>
                <div class="header-title">Identité & Sécurité</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 20px; overflow-y: auto;">
                <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 16px; border: 1px solid var(--border-subtle);">
                    <div style="font-size: 12px; color: var(--text-muted);">Clé publique maîtresse Ed25519</div>
                    <div style="font-size: 13px; font-family: monospace; color: var(--accent-purple-light); margin-top: 6px;">${state.currentUser.publicKey}</div>
                </div>

                <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 20px; border: 1px solid var(--border-subtle);">
                    <div style="font-size: 14px; font-weight: 600; color: white;">Sauvegarde mnémonique</div>
                    <p style="font-size: 12px; color: var(--text-muted); margin: 6px 0 12px;">Votre phrase secrète protège l'intégralité de vos sessions et contacts locaux.</p>
                    <button class="btn-secondary" style="width: 100%; font-size: 13px;" onclick="openMnemonicAuthModal()">Afficher ma phrase secrète</button>
                </div>
            </div>

            <!-- Re-authentication gate before revealing the recovery phrase: a device left
                 briefly unattended must not hand the master secret to whoever picks it up. -->
            <div class="location-modal-overlay" id="mnemonic-auth-modal">
                <div class="location-modal-card">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                        <div style="font-size: 16px; font-weight: 700; color: white; display: flex; align-items: center; gap: 8px;">
                            ${icons.lock} Confirmer votre code local
                        </div>
                        <button class="icon-btn" onclick="closeMnemonicAuthModal()" style="width: 28px; height: 28px;">✕</button>
                    </div>
                    <p style="font-size: 13px; color: var(--text-muted);">
                        Saisissez votre code d'appareil pour révéler votre phrase de récupération. (Sur un appareil réel, cette étape appellerait l'authentification native — biométrie ou code système — plutôt qu'un code saisi dans la page.)
                    </p>
                    <input type="password" inputmode="numeric" id="mnemonic-auth-pin" placeholder="Code local" maxlength="8"
                        style="width: 100%; margin: 14px 0; background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 12px 16px; border: 1px solid var(--border-subtle); color: white; font-size: 15px; letter-spacing: 3px; text-align: center;"
                        onkeydown="if(event.key==='Enter') confirmMnemonicPin()">
                    <div id="mnemonic-auth-error" style="font-size: 12px; color: var(--status-danger); min-height: 16px; margin-bottom: 8px;"></div>
                    <div style="display: flex; gap: 10px;">
                        <button class="btn-secondary" style="flex: 1;" onclick="closeMnemonicAuthModal()">Annuler</button>
                        <button class="btn-primary" style="flex: 1;" onclick="confirmMnemonicPin()">Confirmer</button>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 11. Appareils connectés
    connected_devices: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('settings')">${icons.arrowLeft}</button>
                <div class="header-title">Appareils connectés</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 20px; overflow-y: auto;">
                <div class="item-card" style="background: var(--bg-surface); margin-bottom: 12px; border: 1px solid var(--border-subtle);">
                    ${icons.user}
                    <div class="item-content">
                        <div class="item-name">Smartphone Principal (Pixel 8)</div>
                        <div class="item-sub" style="color: var(--status-success);">Session active en cours</div>
                    </div>
                </div>
                <div class="item-card" style="background: var(--bg-surface); margin-bottom: 20px; border: 1px solid var(--border-subtle);">
                    ${icons.user}
                    <div class="item-content">
                        <div class="item-name">PC Desktop (Windows)</div>
                        <div class="item-sub">Actif il y a 5 minutes</div>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 12. QR Code d'identité
    identity_qrcode: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('settings')">${icons.arrowLeft}</button>
                <div class="header-title">Mon QR Code</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 30px 20px; text-align: center; overflow-y: auto;">
                <div style="background: white; width: 220px; height: 220px; border-radius: var(--radius-lg); margin: 0 auto 20px; display: flex; align-items: center; justify-content: center; box-shadow: 0 8px 32px rgba(0,0,0,0.5);">
                    <div style="width: 180px; height: 180px; border: 4px solid black; display: grid; grid-template-columns: repeat(4, 1fr); gap: 4px; padding: 8px;">
                        <div style="background: black;"></div><div style="background: white;"></div><div style="background: black;"></div><div style="background: black;"></div>
                        <div style="background: white;"></div><div style="background: black;"></div><div style="background: white;"></div><div style="background: black;"></div>
                        <div style="background: black;"></div><div style="background: white;"></div><div style="background: black;"></div><div style="background: white;"></div>
                        <div style="background: black;"></div><div style="background: black;"></div><div style="background: white;"></div><div style="background: black;"></div>
                    </div>
                </div>

                <div style="font-size: 18px; font-weight: 700; color: white;">alex.nova</div>
                <p style="font-size: 12px; color: var(--text-muted); margin: 8px auto 24px; max-width: 260px;">Scannez ce code pour établir une connexion 1-to-1 directe sécurisée.</p>

                <button class="btn-primary" onclick="alert('Lien d\\'invitation P2P copié dans le presse-papiers !')">Partager mon identité</button>
            </div>
        </div>
    `,

    // 13. État de connexion & Diagnostics
    connection_diagnostics: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('chat')">${icons.arrowLeft}</button>
                <div class="header-title">Diagnostic Réseau</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 20px; overflow-y: auto;">
                <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 12px; border: 1px solid var(--border-subtle);">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span style="font-size: 14px; font-weight: 600; color: white;">Statut de Liaison</span>
                        <span style="color: var(--status-success); font-weight: 700; font-size: 13px;">● Connecté</span>
                    </div>
                    <div style="height: 1px; background: var(--border-subtle); margin: 10px 0;"></div>
                    <div style="display: flex; justify-content: space-between; font-size: 13px; color: var(--text-muted);">
                        <span>Mode de transport</span>
                        <span style="color: white; font-weight: 600;">Direct P2P (UDP Hole Punching)</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; font-size: 13px; color: var(--text-muted); margin-top: 6px;">
                        <span>Protocole</span>
                        <span style="color: white; font-weight: 600;">QUIC (TLS 1.3)</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; font-size: 13px; color: var(--text-muted); margin-top: 6px;">
                        <span>Latence RTT</span>
                        <span style="color: var(--status-success); font-weight: 700;">38 ms</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; font-size: 13px; color: var(--text-muted); margin-top: 6px;">
                        <span>Chiffrement E2EE</span>
                        <span style="color: var(--accent-purple-light); font-weight: 600;">Double Ratchet / ChaCha20</span>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 14. Notifications (100% 1-to-1)
    notifications: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('settings')">${icons.arrowLeft}</button>
                <div class="header-title">Notifications</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 20px;">
                <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; border: 1px solid var(--border-subtle);">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <span style="font-size: 14px; color: white;">Messages directs</span>
                        <input type="checkbox" checked style="accent-color: var(--accent-purple); width: 18px; height: 18px;">
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <span style="font-size: 14px; color: white;">Alertes nouveaux contacts</span>
                        <input type="checkbox" checked style="accent-color: var(--accent-purple); width: 18px; height: 18px;">
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <span style="font-size: 14px; color: white;">Masquer l'aperçu du contenu</span>
                        <input type="checkbox" checked style="accent-color: var(--accent-purple); width: 18px; height: 18px;">
                    </div>
                </div>
            </div>
        </div>
    `,

    // 15. Recherche globale (Aucune suggestion parasite avant saisie)
    global_search: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" onclick="navigateTo('conversations')">${icons.arrowLeft}</button>
                <div style="flex: 1; margin: 0 10px;">
                    <div class="search-input-box">
                        ${icons.search}
                        <input type="text" id="global-search-input" placeholder="Rechercher messages, contacts..." autofocus oninput="handleStrictSearch(this.value)">
                    </div>
                </div>
            </header>

            <div class="scroll-list" id="search-results-list" style="padding-top: 30px;">
                <!-- Clean State: Zero suggestions prior to typing -->
                <div style="text-align: center; color: var(--text-muted); padding: 40px 20px;">
                    <div style="width: 48px; height: 48px; border-radius: 50%; background: var(--bg-surface); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--text-dim);">
                        ${icons.search}
                    </div>
                    <div style="font-size: 15px; font-weight: 600; color: white;">Recherche locale 1-to-1</div>
                    <p style="font-size: 13px; color: var(--text-muted); margin-top: 6px; max-width: 260px; margin-left: auto; margin-right: auto;">
                        Tapez un nom de contact ou un mot-clé pour rechercher dans vos échanges locaux.
                    </p>
                </div>
            </div>
        </div>
    `
};

// --- CONTROLLER & NAVIGATION ---
function navigateTo(screenKey) {
    if (!screens[screenKey]) return;
    state.currentScreen = screenKey;

    const container = document.getElementById('screen-container');
    container.innerHTML = screens[screenKey]();

    // Update bottom nav & rail tabs
    document.querySelectorAll('.nav-tab, .rail-item').forEach(tab => {
        const tabTarget = tab.getAttribute('data-tab');
        if (tabTarget === screenKey || (screenKey.includes(tabTarget))) {
            tab.classList.add('active');
        } else {
            tab.classList.remove('active');
        }
    });

    // Control Mobile Bottom Bar Visibility (Hidden in chat, onboarding, sub-screens)
    const mobileNav = document.getElementById('mobile-nav');
    const isRootTab = ['conversations', 'contacts', 'settings'].includes(screenKey);
    if (mobileNav) {
        mobileNav.style.display = isRootTab ? 'flex' : 'none';
    }

    // Auto scroll chat & measure real latency
    if (screenKey === 'chat') {
        if (latencyMonitorInterval) clearInterval(latencyMonitorInterval);
        measureRealLatency();
        latencyMonitorInterval = setInterval(measureRealLatency, 3500);

        setTimeout(() => {
            const body = document.getElementById('chat-body');
            if (body) body.scrollTop = body.scrollHeight;
            const input = document.getElementById('chat-input');
            if (input) input.focus();
        }, 50);
    } else {
        if (latencyMonitorInterval) {
            clearInterval(latencyMonitorInterval);
            latencyMonitorInterval = null;
        }
    }
}

let latencyMonitorInterval = null;

async function measureRealLatency() {
    const t0 = performance.now();
    try {
        await fetch('/index.html', { method: 'HEAD', cache: 'no-store' });
        const t1 = performance.now();
        const measured = Math.max(1, Math.round(t1 - t0));
        state.activeContact.latencyMs = measured;
        const latencyEl = document.getElementById('chat-header-latency');
        if (latencyEl) {
            latencyEl.innerText = `${measured} ms`;
        }
        return measured;
    } catch (e) {
        const fallback = 12;
        state.activeContact.latencyMs = fallback;
        const latencyEl = document.getElementById('chat-header-latency');
        if (latencyEl) {
            latencyEl.innerText = `${fallback} ms`;
        }
        return fallback;
    }
}

function openChatWith(name, handle) {
    state.activeContact.name = name;
    state.activeContact.handle = handle;

    // 1. Clear unread notification badge on this conversation
    const conv = state.conversations.find(c => c.name === name || c.handle === handle);
    if (conv) {
        conv.unread = 0;
    }

    // 2. Mark all messages as read
    state.messages.forEach(m => {
        m.status = 'read';
    });

    // 3. Update global navigation tab badges
    updateGlobalUnreadBadges();

    navigateTo('chat');
}

// Reads the target contact from data-* attributes rather than from an inline onclick argument.
// A contact's name/handle is peer-controlled data: splicing it directly into an onclick="..."
// attribute as a JS string literal is unsafe even when HTML-escaped, because the browser
// HTML-decodes attribute text *before* treating it as JavaScript source — so an escaped quote
// would simply decode back into a real quote and break out of the string right before it runs.
// Routing the value through `dataset` (a plain string property, never re-parsed as code) avoids
// that class of bug entirely.
function openChatWithEl(el) {
    openChatWith(el.dataset.name || '', el.dataset.handle || '');
}

function openImagePreviewEl(el) {
    const url = el.dataset.url;
    if (url) window.open(url);
}

function showFileAlertEl(el) {
    const filename = el.dataset.filename || '';
    alert('Document sécurisé vérifié en local : ' + filename);
}

// --- MNEMONIC REVEAL GATE (re-authentication before showing the master recovery secret) ---
let mnemonicAuthAttempts = 0;
let mnemonicAuthLockoutUntil = 0;
const MNEMONIC_AUTH_MAX_ATTEMPTS = 3;
const MNEMONIC_AUTH_LOCKOUT_MS = 60_000;

function openMnemonicAuthModal() {
    const modal = document.getElementById('mnemonic-auth-modal');
    const errorEl = document.getElementById('mnemonic-auth-error');
    const input = document.getElementById('mnemonic-auth-pin');
    if (!modal) return;

    if (errorEl) errorEl.textContent = '';
    if (input) input.value = '';
    modal.classList.add('show');
    if (input) setTimeout(() => input.focus(), 50);
}

function closeMnemonicAuthModal() {
    const modal = document.getElementById('mnemonic-auth-modal');
    if (modal) modal.classList.remove('show');
}

function confirmMnemonicPin() {
    const errorEl = document.getElementById('mnemonic-auth-error');
    const input = document.getElementById('mnemonic-auth-pin');

    const now = Date.now();
    if (now < mnemonicAuthLockoutUntil) {
        const remaining = Math.ceil((mnemonicAuthLockoutUntil - now) / 1000);
        if (errorEl) errorEl.textContent = `Trop de tentatives. Réessayez dans ${remaining}s.`;
        return;
    }

    const entered = input ? input.value : '';
    if (entered === state.currentUser.localPin) {
        mnemonicAuthAttempts = 0;
        closeMnemonicAuthModal();
        alert('Phrase de récupération:\n\n' + state.currentUser.mnemonic);
        return;
    }

    mnemonicAuthAttempts++;
    if (mnemonicAuthAttempts >= MNEMONIC_AUTH_MAX_ATTEMPTS) {
        mnemonicAuthLockoutUntil = now + MNEMONIC_AUTH_LOCKOUT_MS;
        mnemonicAuthAttempts = 0;
        if (errorEl) errorEl.textContent = 'Trop de tentatives incorrectes. Réessayez dans 60s.';
    } else if (errorEl) {
        errorEl.textContent = 'Code incorrect.';
    }
    if (input) {
        input.value = '';
        input.focus();
    }
}

function updateGlobalUnreadBadges() {
    const totalUnread = state.conversations.reduce((sum, c) => sum + (c.unread || 0), 0);
    const railBadge = document.getElementById('rail-unread-badge');
    const mobileBadge = document.getElementById('mobile-unread-badge');

    [railBadge, mobileBadge].forEach(badge => {
        if (badge) {
            if (totalUnread > 0) {
                badge.style.display = 'inline-flex';
                badge.innerText = totalUnread;
            } else {
                badge.style.display = 'none';
            }
        }
    });
}

// --- CHAT RENDERING & DOM MUTATION HELPERS ---
function buildMessageHtml(m) {
    // m.text / m.meta / m.url ultimately come from a remote peer (message content, a shared
    // filename, a shared image) — every one is escaped before it touches innerHTML, and none
    // is ever spliced into an executable JS attribute (see the data-* + named-handler pattern
    // below), so a hostile value can only ever render as inert text.
    const safeText = escapeHtml(m.text);
    const safeMeta = escapeHtml(m.meta);

    let contentHtml = '';
    if (m.type === 'image') {
        contentHtml = `
            <div class="msg-photo-card">
                ${m.url ? `<img src="${escapeHtml(m.url)}" class="msg-image-thumb" data-url="${escapeHtml(m.url)}" onclick="openImagePreviewEl(this)" title="Cliquer pour agrandir" />` : `
                    <div class="msg-photo-preview">
                        ${icons.image}
                        <span style="font-size: 11px; font-weight: 600; color: white;">${safeText || 'Photo P2P (Chiffrée)'}</span>
                        <span style="font-size: 10px; color: var(--text-dim);">${safeMeta || '1.8 MB • Direct UDP'}</span>
                    </div>
                `}
                <div style="padding: 6px 10px; font-size: 11px; color: var(--text-muted); display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-weight: 600; color: white; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 140px;">${safeText}</span>
                    <span style="font-size: 10px; color: var(--text-dim);">${safeMeta || 'Direct UDP'}</span>
                </div>
            </div>
        `;
    } else if (m.type === 'file') {
        contentHtml = `
            <div class="msg-file-card" data-filename="${escapeHtml(m.text)}" onclick="showFileAlertEl(this)" style="cursor: pointer;">
                ${icons.file}
                <div style="overflow: hidden;">
                    <div style="font-size: 13px; font-weight: 600; color: white; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 150px;">${safeText}</div>
                    <div style="font-size: 11px; color: var(--text-muted);">${safeMeta || '2.4 MB • Document E2EE'}</div>
                </div>
            </div>
        `;
    } else if (m.type === 'voice') {
        contentHtml = `
            <div class="msg-voice-card">
                <button class="voice-play-btn" onclick="toggleVoicePlay(this)">▶</button>
                <div class="voice-waveform">
                    <div class="voice-bar" style="height: 12px;"></div>
                    <div class="voice-bar" style="height: 20px;"></div>
                    <div class="voice-bar" style="height: 15px;"></div>
                    <div class="voice-bar" style="height: 24px;"></div>
                    <div class="voice-bar" style="height: 18px;"></div>
                    <div class="voice-bar" style="height: 10px;"></div>
                    <div class="voice-bar" style="height: 22px;"></div>
                    <div class="voice-bar" style="height: 14px;"></div>
                    <div class="voice-bar" style="height: 8px;"></div>
                </div>
                <span style="font-size: 11px; color: var(--text-muted); font-family: monospace;">${safeMeta || '0:14'}</span>
            </div>
        `;
    } else if (m.type === 'location') {
        contentHtml = `
            <div class="msg-location-card">
                <div class="msg-location-header">
                    <div class="map-grid-lines"></div>
                    <div class="map-pin-pulse" style="width: 32px; height: 32px;">${icons.mapPin}</div>
                </div>
                <div class="msg-location-body">
                    <div style="font-size: 12px; font-weight: 700; color: white;">Position P2P Sécurisée</div>
                    <div style="font-size: 11px; color: var(--accent-purple-light); font-family: monospace; margin-top: 2px;">${safeText}</div>
                    <div style="font-size: 10px; color: var(--text-dim); margin-top: 4px;">${safeMeta || 'Précision GPS ~5m'}</div>
                </div>
            </div>
        `;
    } else if (m.type === 'sticker') {
        contentHtml = `<div style="font-size: 28px; padding: 4px 8px;">${safeText}</div>`;
    } else {
        contentHtml = `<div class="msg-bubble">${safeText}</div>`;
    }

    const safeId = escapeHtml(m.id);
    const statusHtml = m.isOutgoing ? `
        <span id="msg-status-${safeId}" style="display: inline-flex; align-items: center;">
            ${m.status === 'sending' ? '<span class="status-tick-sending">⏳</span>' : (m.status === 'sent' ? '<span class="status-tick-sent">✓</span>' : `<span class="status-tick-read" title="Déchiffré & Lu">${icons.checkCheck}</span>`)}
        </span>
    ` : '';

    return `
        <div class="msg-row ${m.isOutgoing ? 'msg-outgoing' : 'msg-incoming'}" id="msg-row-${safeId}">
            ${contentHtml}
            <div class="msg-meta">
                <span>${escapeHtml(m.time)}</span>
                ${statusHtml}
            </div>
        </div>
    `;
}

function appendChatMessageToBody(msg) {
    const chatBody = document.getElementById('chat-body');
    if (!chatBody) return;
    
    // Remove typing indicator if present before appending message
    const typingRow = document.getElementById('typing-indicator-row');
    if (typingRow) typingRow.remove();

    const wrapper = document.createElement('div');
    wrapper.innerHTML = buildMessageHtml(msg).trim();
    chatBody.appendChild(wrapper.firstElementChild);

    setTimeout(() => {
        chatBody.scrollTo({ top: chatBody.scrollHeight, behavior: 'smooth' });
    }, 20);
}

function setEmmaTyping(isTyping) {
    state.isTyping = isTyping;
    const chatBody = document.getElementById('chat-body');
    if (!chatBody) return;

    let typingRow = document.getElementById('typing-indicator-row');
    if (isTyping) {
        if (!typingRow) {
            typingRow = document.createElement('div');
            typingRow.id = 'typing-indicator-row';
            typingRow.className = 'typing-indicator-row';
            typingRow.innerHTML = `
                <div class="typing-indicator-bubble">
                    <div class="typing-dot"></div>
                    <div class="typing-dot"></div>
                    <div class="typing-dot"></div>
                </div>
                <span style="font-size: 11px; color: var(--text-muted); font-style: italic;">${state.activeContact.name} est en train d'écrire...</span>
            `;
            chatBody.appendChild(typingRow);
            setTimeout(() => {
                chatBody.scrollTo({ top: chatBody.scrollHeight, behavior: 'smooth' });
            }, 20);
        }
    } else {
        if (typingRow) typingRow.remove();
    }
}

function updateMessageStatus(msgId, status) {
    const target = state.messages.find(m => m.id === msgId);
    if (target) target.status = status;

    const el = document.getElementById(`msg-status-${msgId}`);
    if (el) {
        if (status === 'sending') {
            el.innerHTML = '<span class="status-tick-sending">⏳</span>';
        } else if (status === 'sent') {
            el.innerHTML = '<span class="status-tick-sent">✓</span>';
        } else {
            el.innerHTML = `<span class="status-tick-read" title="Déchiffré & Lu">${icons.checkCheck}</span>`;
        }
    }
}

function toggleVoicePlay(btn) {
    if (btn.innerText === '▶') {
        btn.innerText = '⏸';
        const bars = btn.parentElement.querySelectorAll('.voice-bar');
        bars.forEach((b, idx) => {
            b.style.transition = 'height 0.2s ease';
            b.style.height = `${(idx % 3 + 1) * 7 + 4}px`;
        });
        setTimeout(() => {
            btn.innerText = '▶';
        }, 3000);
    } else {
        btn.innerText = '▶';
    }
}

// --- DEVICE FILE PICKERS & MEDIA INTEGRATION ---
function triggerDeviceMediaPicker() {
    const input = document.getElementById('media-file-input');
    if (input) input.click();
    closePanels();
}

function handleMediaFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;

    const sizeStr = file.size > 1024 * 1024 
        ? `${(file.size / (1024 * 1024)).toFixed(1)} MB` 
        : `${(file.size / 1024).toFixed(0)} KB`;
    
    const isVideo = file.type.startsWith('video');
    const label = (isVideo ? '🎬 ' : '📷 ') + file.name;
    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const newMsgId = 'm_' + Date.now();

    const reader = new FileReader();
    reader.onload = function(e) {
        const previewUrl = e.target.result;
        const newMsg = {
            id: newMsgId,
            type: 'image',
            text: label,
            meta: `${sizeStr} • Direct UDP`,
            url: previewUrl,
            time: timeStr,
            isOutgoing: true,
            status: 'read'
        };

        state.messages.push(newMsg);
        appendChatMessageToBody(newMsg);

        // Emma peer acknowledgment
        setTimeout(() => {
            setEmmaTyping(true);
            setTimeout(() => {
                setEmmaTyping(false);
                const replyMsg = {
                    id: 'm_' + Date.now(),
                    type: 'text',
                    text: `Média "${file.name}" reçu et déchiffré en local (${sizeStr}) ! 📷✨`,
                    time: timeStr,
                    isOutgoing: false
                };
                state.messages.push(replyMsg);
                appendChatMessageToBody(replyMsg);
            }, 1200);
        }, 800);
    };

    reader.readAsDataURL(file);
    event.target.value = '';
}

function triggerDeviceDocPicker() {
    const input = document.getElementById('doc-file-input');
    if (input) input.click();
    closePanels();
}

function handleDocFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;

    const sizeStr = file.size > 1024 * 1024 
        ? `${(file.size / (1024 * 1024)).toFixed(1)} MB` 
        : `${(file.size / 1024).toFixed(0)} KB`;
    
    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const newMsgId = 'm_' + Date.now();

    const newMsg = {
        id: newMsgId,
        type: 'file',
        text: '📄 ' + file.name,
        meta: `${sizeStr} • Document E2EE`,
        time: timeStr,
        isOutgoing: true,
        status: 'read'
    };

    state.messages.push(newMsg);
    appendChatMessageToBody(newMsg);
    event.target.value = '';

    // Emma response
    setTimeout(() => {
        setEmmaTyping(true);
        setTimeout(() => {
            setEmmaTyping(false);
            const replyMsg = {
                id: 'm_' + Date.now(),
                type: 'text',
                text: `Document "${file.name}" reçu en direct P2P et intégrité SHA-256 validée ! 🛡️`,
                time: timeStr,
                isOutgoing: false
            };
            state.messages.push(replyMsg);
            appendChatMessageToBody(replyMsg);
        }, 1100);
    }, 800);
}

// --- ADVANCED WHATSAPP-LIKE VOICE RECORDER (RECORD, PAUSE, RESUME, PREVIEW & SEND) ---
let voiceRecordingTimer = null;
let voiceRecordingSeconds = 0;
let voiceWaveformAnim = null;
let isVoiceRecordingPaused = false;
let isVoicePreviewPlaying = false;
let voicePreviewTimer = null;
let voicePreviewElapsed = 0;

function startVoiceRecording() {
    closePanels();
    const normalRow = document.getElementById('normal-input-row');
    const voiceBar = document.getElementById('voice-recording-bar');
    const micBtn = document.getElementById('mic-record-btn');

    if (!voiceBar || !normalRow) return;

    normalRow.style.display = 'none';
    voiceBar.classList.remove('paused', 'previewing');
    voiceBar.classList.add('active');
    if (micBtn) micBtn.classList.add('recording');

    isVoiceRecordingPaused = false;
    isVoicePreviewPlaying = false;
    voiceRecordingSeconds = 0;
    voicePreviewElapsed = 0;

    const timerEl = document.getElementById('rec-timer');
    if (timerEl) timerEl.innerText = '00:00';

    const pauseIcon = document.getElementById('rec-pause-icon');
    if (pauseIcon) pauseIcon.innerText = '⏸';

    const previewBtn = document.getElementById('rec-preview-btn');
    if (previewBtn) previewBtn.style.display = 'none';

    startTimerAndWaveform();
}

function startTimerAndWaveform() {
    const timerEl = document.getElementById('rec-timer');
    const voiceBar = document.getElementById('voice-recording-bar');

    if (voiceRecordingTimer) clearInterval(voiceRecordingTimer);
    voiceRecordingTimer = setInterval(() => {
        voiceRecordingSeconds++;
        const mins = String(Math.floor(voiceRecordingSeconds / 60)).padStart(2, '0');
        const secs = String(voiceRecordingSeconds % 60).padStart(2, '0');
        if (timerEl) timerEl.innerText = `${mins}:${secs}`;
    }, 1000);

    // Waveform live bars
    const bars = voiceBar ? voiceBar.querySelectorAll('.rec-wave-bar') : [];
    if (voiceWaveformAnim) clearInterval(voiceWaveformAnim);
    voiceWaveformAnim = setInterval(() => {
        bars.forEach(b => {
            b.style.height = `${Math.floor(Math.random() * 16) + 6}px`;
        });
    }, 120);
}

function togglePauseVoiceRecording() {
    const voiceBar = document.getElementById('voice-recording-bar');
    const pauseIcon = document.getElementById('rec-pause-icon');
    const previewBtn = document.getElementById('rec-preview-btn');
    const previewIcon = document.getElementById('rec-preview-icon');

    if (!isVoiceRecordingPaused) {
        // --- PAUSE RECORDING ---
        isVoiceRecordingPaused = true;
        if (voiceRecordingTimer) clearInterval(voiceRecordingTimer);
        if (voiceWaveformAnim) clearInterval(voiceWaveformAnim);

        if (voiceBar) {
            voiceBar.classList.remove('previewing');
            voiceBar.classList.add('paused');
        }
        if (pauseIcon) pauseIcon.innerText = '🎙️'; // Icon to Resume
        if (previewBtn) {
            previewBtn.style.display = 'flex';
            if (previewIcon) previewIcon.innerText = '▶';
        }
    } else {
        // --- RESUME RECORDING ---
        if (isVoicePreviewPlaying) {
            stopVoicePreview();
        }
        isVoiceRecordingPaused = false;
        if (voiceBar) {
            voiceBar.classList.remove('paused', 'previewing');
        }
        if (pauseIcon) pauseIcon.innerText = '⏸';
        if (previewBtn) previewBtn.style.display = 'none';

        startTimerAndWaveform();
    }
}

function toggleVoicePreview() {
    if (isVoicePreviewPlaying) {
        stopVoicePreview();
    } else {
        startVoicePreview();
    }
}

function startVoicePreview() {
    isVoicePreviewPlaying = true;
    const voiceBar = document.getElementById('voice-recording-bar');
    const previewIcon = document.getElementById('rec-preview-icon');
    const timerEl = document.getElementById('rec-timer');

    if (voiceBar) {
        voiceBar.classList.add('previewing');
    }
    if (previewIcon) previewIcon.innerText = '⏸';

    voicePreviewElapsed = 0;
    const totalDuration = Math.max(1, voiceRecordingSeconds);

    if (voicePreviewTimer) clearInterval(voicePreviewTimer);
    voicePreviewTimer = setInterval(() => {
        voicePreviewElapsed++;
        const mins = String(Math.floor(voicePreviewElapsed / 60)).padStart(2, '0');
        const secs = String(voicePreviewElapsed % 60).padStart(2, '0');
        if (timerEl) timerEl.innerText = `${mins}:${secs}`;

        // Preview waveform active pulse
        const bars = voiceBar ? voiceBar.querySelectorAll('.rec-wave-bar') : [];
        bars.forEach((b, idx) => {
            b.style.height = `${((idx + voicePreviewElapsed) % 4 + 1) * 5 + 4}px`;
        });

        if (voicePreviewElapsed >= totalDuration) {
            stopVoicePreview();
        }
    }, 1000);
}

function stopVoicePreview() {
    isVoicePreviewPlaying = false;
    if (voicePreviewTimer) clearInterval(voicePreviewTimer);

    const voiceBar = document.getElementById('voice-recording-bar');
    const previewIcon = document.getElementById('rec-preview-icon');
    const timerEl = document.getElementById('rec-timer');

    if (voiceBar) {
        voiceBar.classList.remove('previewing');
    }
    if (previewIcon) previewIcon.innerText = '▶';

    const mins = String(Math.floor(voiceRecordingSeconds / 60)).padStart(2, '0');
    const secs = String(voiceRecordingSeconds % 60).padStart(2, '0');
    if (timerEl) timerEl.innerText = `${mins}:${secs}`;
}

function cancelVoiceRecording() {
    if (voiceRecordingTimer) clearInterval(voiceRecordingTimer);
    if (voiceWaveformAnim) clearInterval(voiceWaveformAnim);
    if (voicePreviewTimer) clearInterval(voicePreviewTimer);

    const normalRow = document.getElementById('normal-input-row');
    const voiceBar = document.getElementById('voice-recording-bar');
    const micBtn = document.getElementById('mic-record-btn');

    if (voiceBar) voiceBar.classList.remove('active', 'paused', 'previewing');
    if (normalRow) normalRow.style.display = 'flex';
    if (micBtn) micBtn.classList.remove('recording');

    isVoiceRecordingPaused = false;
    isVoicePreviewPlaying = false;
}

function stopAndSendVoiceRecording() {
    if (voiceRecordingTimer) clearInterval(voiceRecordingTimer);
    if (voiceWaveformAnim) clearInterval(voiceWaveformAnim);
    if (voicePreviewTimer) clearInterval(voicePreviewTimer);

    const durationSecs = Math.max(1, voiceRecordingSeconds);
    const mins = String(Math.floor(durationSecs / 60)).padStart(2, '0');
    const secs = String(durationSecs % 60).padStart(2, '0');
    const durationStr = `${mins}:${secs}`;

    const normalRow = document.getElementById('normal-input-row');
    const voiceBar = document.getElementById('voice-recording-bar');
    const micBtn = document.getElementById('mic-record-btn');

    if (voiceBar) voiceBar.classList.remove('active', 'paused', 'previewing');
    if (normalRow) normalRow.style.display = 'flex';
    if (micBtn) micBtn.classList.remove('recording');

    isVoiceRecordingPaused = false;
    isVoicePreviewPlaying = false;

    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const newMsgId = 'm_' + Date.now();

    const newMsg = {
        id: newMsgId,
        type: 'voice',
        text: '🎤 Note_Vocale.opus',
        meta: durationStr,
        time: timeStr,
        isOutgoing: true,
        status: 'read'
    };

    state.messages.push(newMsg);
    appendChatMessageToBody(newMsg);

    // Emma response
    setTimeout(() => {
        setEmmaTyping(true);
        setTimeout(() => {
            setEmmaTyping(false);
            const replyMsg = {
                id: 'm_' + Date.now(),
                type: 'text',
                text: `Note vocale (${durationStr}) bien écoutée en local, le son est très net ! 🎧👌`,
                time: timeStr,
                isOutgoing: false
            };
            state.messages.push(replyMsg);
            appendChatMessageToBody(replyMsg);
        }, 1200);
    }, 800);
}

// --- LOCATION CONFIRMATION & SHARING ---
let pendingCoordinates = { lat: 48.8566, lon: 2.3522, text: '48.8566° N, 2.3522° E (Paris)' };

function openLocationModal() {
    const modal = document.getElementById('location-modal');
    if (!modal) return;

    modal.classList.add('show');

    // Try real browser geolocation
    if (navigator.geolocation) {
        navigator.geolocation.getCurrentPosition(
            (pos) => {
                const lat = pos.coords.latitude.toFixed(4);
                const lon = pos.coords.longitude.toFixed(4);
                pendingCoordinates = {
                    lat: pos.coords.latitude,
                    lon: pos.coords.longitude,
                    text: `${lat}° N, ${lon}° E (Direct GPS)`
                };
                const preview = document.getElementById('loc-coords-preview');
                if (preview) preview.innerText = pendingCoordinates.text;
            },
            () => {
                // Default fallback coordinates (Paris)
                const preview = document.getElementById('loc-coords-preview');
                if (preview) preview.innerText = pendingCoordinates.text;
            },
            { timeout: 3000 }
        );
    }
}

function closeLocationModal() {
    const modal = document.getElementById('location-modal');
    if (modal) modal.classList.remove('show');
}

function confirmAndSendLocation() {
    closeLocationModal();

    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const newMsgId = 'm_' + Date.now();

    const newMsg = {
        id: newMsgId,
        type: 'location',
        text: pendingCoordinates.text,
        meta: 'Précision GPS ~5m • Chiffré P2P',
        time: timeStr,
        isOutgoing: true,
        status: 'read'
    };

    state.messages.push(newMsg);
    appendChatMessageToBody(newMsg);

    // Emma response
    setTimeout(() => {
        setEmmaTyping(true);
        setTimeout(() => {
            setEmmaTyping(false);
            const replyMsg = {
                id: 'm_' + Date.now(),
                type: 'text',
                text: 'Position bien reçue et affichée sur la carte locale chiffrée ! 📍🛡️',
                time: timeStr,
                isOutgoing: false
            };
            state.messages.push(replyMsg);
            appendChatMessageToBody(replyMsg);
        }, 1100);
    }, 800);
}



function sendRichAttachment(type, name, meta) {
    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;

    const newMsg = {
        id: 'm_' + Date.now(),
        type: type,
        text: name,
        meta: meta,
        time: timeStr,
        isOutgoing: true,
        status: 'read'
    };

    state.messages.push(newMsg);
    closePanels();
    appendChatMessageToBody(newMsg);

    // Emma response
    setTimeout(() => {
        setEmmaTyping(true);

        setTimeout(() => {
            setEmmaTyping(false);
            const replyMsg = {
                id: 'm_' + Date.now(),
                type: 'text',
                text: `Pièce jointe (${name.split(' ')[1] || name}) bien reçue et vérifiée en local ! 👌`,
                time: timeStr,
                isOutgoing: false
            };
            state.messages.push(replyMsg);
            appendChatMessageToBody(replyMsg);
        }, 1100);
    }, 800);
}

// Text send path (was referenced by the send button/Enter key but never implemented — the
// button was a dead click). Storage is raw text; escaping happens once, at render time, in
// buildMessageHtml — never here, so the value is never escaped twice.
function sendMessage() {
    const input = document.getElementById('chat-input');
    if (!input) return;
    const text = input.value.trim();
    if (!text) return;

    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;

    const newMsg = {
        id: 'm_' + Date.now(),
        type: 'text',
        text,
        time: timeStr,
        isOutgoing: true,
        status: 'sent',
    };

    state.messages.push(newMsg);
    appendChatMessageToBody(newMsg);
    input.value = '';
    input.focus();

    setTimeout(() => {
        setEmmaTyping(true);
        setTimeout(() => {
            setEmmaTyping(false);
            const replyMsg = {
                id: 'm_' + Date.now(),
                type: 'text',
                text: 'Message bien reçu, déchiffré localement via Double Ratchet.',
                time: timeStr,
                isOutgoing: false,
            };
            state.messages.push(replyMsg);
            appendChatMessageToBody(replyMsg);
        }, 1100);
    }, 700);
}

function insertEmoji(emoji) {
    const input = document.getElementById('chat-input');
    if (input) {
        input.value += emoji;
        input.focus();
    }
}

function toggleAttachmentDrawer(event) {
    if (event) event.stopPropagation();
    const panel = document.getElementById('emoji-picker-panel');
    if (panel) panel.classList.remove('show');
    const drawer = document.getElementById('attachment-drawer');
    if (drawer) drawer.classList.toggle('show');
}

function toggleEmojiPicker(event) {
    if (event) event.stopPropagation();
    const drawer = document.getElementById('attachment-drawer');
    if (drawer) drawer.classList.remove('show');
    const panel = document.getElementById('emoji-picker-panel');
    if (panel) panel.classList.toggle('show');
}

function closePanels() {
    const drawer = document.getElementById('attachment-drawer');
    if (drawer) drawer.classList.remove('show');
    const panel = document.getElementById('emoji-picker-panel');
    if (panel) panel.classList.remove('show');
}

// --- CONTACTS & STRICT SEARCH ---
function addNewContact() {
    const input = document.getElementById('add-handle-input');
    if (!input || !input.value.trim()) return;

    const handle = input.value.trim().replace('@', '');
    const name = handle.charAt(0).toUpperCase() + handle.slice(1).split('.')[0];

    state.contacts.push({
        name: name,
        handle: handle.includes('.nova') ? handle : handle + '.nova',
        online: true,
        p2pMode: 'Direct (35 ms)',
        key: '8899 AABB CCDD EEFF ... 1122'
    });

    state.conversations.unshift({
        id: 'conv_' + Date.now(),
        name: name,
        handle: handle.includes('.nova') ? handle : handle + '.nova',
        lastMsg: 'Nouvelle conversation directe',
        time: 'À l\'instant',
        unread: 0,
        online: true,
        mode: 'Direct'
    });

    alert(`Contact @${handle} ajouté avec succès.`);
    navigateTo('conversations');
}

function filterContactsList(query) {
    const container = document.getElementById('contacts-list-container');
    if (!container) return;

    const filtered = state.contacts.filter(c => 
        c.name.toLowerCase().includes(query.toLowerCase()) || 
        c.handle.toLowerCase().includes(query.toLowerCase())
    );

    container.innerHTML = filtered.map(c => `
        <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" onclick="openChatWithEl(this)">
            <div class="avatar">
                ${escapeHtml(c.name.charAt(0))}
                <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
            </div>
            <div class="item-content">
                <div class="item-name">${escapeHtml(c.name)}</div>
                <div class="item-sub">@${escapeHtml(c.handle)} • ${escapeHtml(c.p2pMode)}</div>
            </div>
        </div>
    `).join('');
}

function handleStrictSearch(query) {
    const resultsContainer = document.getElementById('search-results-list');
    if (!resultsContainer) return;

    // Strict: No suggestions before typing
    if (!query.trim()) {
        resultsContainer.innerHTML = `
            <div style="text-align: center; color: var(--text-muted); padding: 40px 20px;">
                <div style="width: 48px; height: 48px; border-radius: 50%; background: var(--bg-surface); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--text-dim);">
                    ${icons.search}
                </div>
                <div style="font-size: 15px; font-weight: 600; color: white;">Recherche locale 1-to-1</div>
                <p style="font-size: 13px; color: var(--text-muted); margin-top: 6px; max-width: 260px; margin-left: auto; margin-right: auto;">
                    Tapez un nom de contact ou un mot-clé pour rechercher dans vos échanges locaux.
                </p>
            </div>
        `;
        return;
    }

    const q = query.toLowerCase();
    const filteredContacts = state.contacts.filter(c => c.name.toLowerCase().includes(q) || c.handle.toLowerCase().includes(q));
    const filteredMessages = state.messages.filter(m => m.text.toLowerCase().includes(q));

    resultsContainer.innerHTML = `
        <div style="font-size: 12px; color: var(--text-muted); margin: 0 0 8px 12px; font-weight: 600;">CONTACTS (${filteredContacts.length})</div>
        ${filteredContacts.length > 0 ? filteredContacts.map(c => `
            <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" onclick="openChatWithEl(this)">
                <div class="avatar">${escapeHtml(c.name.charAt(0))}</div>
                <div class="item-content">
                    <div class="item-name">${escapeHtml(c.name)}</div>
                    <div class="item-sub">@${escapeHtml(c.handle)}</div>
                </div>
            </div>
        `).join('') : '<div style="font-size: 13px; color: var(--text-dim); margin-left: 12px; margin-bottom: 14px;">Aucun contact correspondant.</div>'}

        <div style="font-size: 12px; color: var(--text-muted); margin: 16px 0 8px 12px; font-weight: 600;">MESSAGES (${filteredMessages.length})</div>
        ${filteredMessages.length > 0 ? filteredMessages.map(m => `
            <div class="item-card" onclick="navigateTo('chat')">
                <div class="avatar">${icons.chat}</div>
                <div class="item-content">
                    <div class="item-name">${escapeHtml(m.text)}</div>
                    <div class="item-sub">${escapeHtml(m.time)} • Message local</div>
                </div>
            </div>
        `).join('') : '<div style="font-size: 13px; color: var(--text-dim); margin-left: 12px;">Aucun message trouvé.</div>'}
    `;
}

// --- INITIALIZATION ---
document.addEventListener('DOMContentLoaded', () => {
    // Setup rail & nav tabs
    document.querySelectorAll('[data-tab]').forEach(btn => {
        btn.addEventListener('click', () => {
            const target = btn.getAttribute('data-tab');
            navigateTo(target);
        });
    });

    // Update initial unread badges
    updateGlobalUnreadBadges();

    // Render initial screen
    navigateTo('conversations');
});

// Close drawers (attachments, emoji picker) when clicking outside
document.addEventListener('click', (event) => {
    const drawer = document.getElementById('attachment-drawer');
    const emojiPanel = document.getElementById('emoji-picker-panel');
    const attachBtn = document.getElementById('attachment-toggle-btn');
    const emojiBtn = document.getElementById('emoji-toggle-btn');

    if (drawer && drawer.classList.contains('show')) {
        if (!drawer.contains(event.target) && (!attachBtn || !attachBtn.contains(event.target))) {
            drawer.classList.remove('show');
        }
    }

    if (emojiPanel && emojiPanel.classList.contains('show')) {
        if (!emojiPanel.contains(event.target) && (!emojiBtn || !emojiBtn.contains(event.target))) {
            emojiPanel.classList.remove('show');
        }
    }
});
