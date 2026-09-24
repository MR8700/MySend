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
    chevronRight: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>`,
    download: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>`,
    copy: `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>`,
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

// --- BACKEND BRIDGE (Tauri) ---
// This UI runs inside a Tauri desktop shell (see ui/src-tauri) wrapping the real nova-engine/
// nova-transport backend — not a browser tab, and not WASM (the backend's bundled SQLite and
// QUIC/libp2p transport cannot run in a real browser sandbox at all). `window.__TAURI__` is only
// injected when actually running under Tauri; opening this file directly in a plain browser
// (e.g. while iterating on styling) leaves `hasBackend` false and every backend-dependent action
// fails honestly rather than silently doing nothing.
const tauriInvoke = window.__TAURI__ && window.__TAURI__.core ? window.__TAURI__.core.invoke : null;
const hasBackend = !!tauriInvoke;

function requireBackend() {
    if (!hasBackend) {
        alert('Backend indisponible : cette page doit être lancée via l\'application Tauri (cargo tauri dev), pas dans un navigateur.');
        return false;
    }
    return true;
}

// Global Vercel API and Supabase/Neon PostgreSQL Edge endpoint
const VERCEL_API_BASE_URL = 'https://novachat-navy-seven.vercel.app';

// Formats a safety number or verification fingerprint into clean 4-character chunks
function formatSafetyNumber(num) {
    if (!num) return 'Vérifié ✓';
    const clean = String(num).replace(/[^a-zA-Z0-9]/g, '');
    if (clean.length >= 12) {
        const chunks = clean.match(/.{1,4}/g);
        return chunks ? chunks.slice(0, 4).join('  ') : clean;
    }
    return clean;
}

// --- EVENT DELEGATION (CSP-safe) ---
// tauri.conf.json's CSP is `script-src 'self'` with no 'unsafe-inline' — inline event-handler
// attributes (onclick="...", onkeydown="...", oninput="...", onchange="...") are governed by
// script-src-attr, which falls back to script-src, so WebView2 (Chromium-based) blocks every one
// of them outright: none of this app's ~80 onclick="..." attributes ever ran. Every interactive
// element is driven instead by data-action (+ a small set of data-* arguments) and one delegated
// listener per event type below, so nothing needs to be inline.

// Disables `el` and dims it (see the `button:disabled` rule in style.css) for the duration of
// `fn()` — the "something is happening" feedback previously entirely missing from every
// backend-invoking button. Some of these calls legitimately take multiple seconds (Argon2id key
// derivation, P2P network startup with its bounded IPv6 wait), during which a button with no
// visual change at all looks exactly like a dead one.
async function runPendingAction(el, fn) {
    if (el && el.disabled) return;
    if (el) el.disabled = true;
    try {
        await fn();
    } finally {
        if (el) el.disabled = false;
    }
}

const CLICK_ACTIONS = {
    navigate: (el) => navigateTo(el.dataset.screen),
    navigateBack: () => navigateBackSafely(),
    createAccount: (el) => runPendingAction(el, createAccountReal),
    restoreAccount: (el) => runPendingAction(el, restoreAccountReal),
    openChat: (el) => openChatWithEl(el),
    callVoice: () => startRealtimeCall('voice'),
    callVideo: () => startRealtimeCall('video'),
    acceptIncomingCall: () => acceptIncomingCallReal(),
    rejectIncomingCall: () => rejectIncomingCallReal(),
    toggleCallMute: () => toggleCallMute(),
    toggleCallVideo: () => toggleCallVideo(),
    switchCallCamera: () => switchCallCamera(),
    toggleCallTorch: () => toggleCallTorch(),
    hangupActiveCall: () => hangupCall(true, 'Fin de l\'appel'),
    clearConversationsSearch: () => clearConversationsSearch(),
    clearContactsSearch: () => clearContactsSearch(),
    clearNewChatSearch: () => clearNewChatSearch(),
    startDirectChatFromQuery: (el) => startDirectChatFromQuery(el.dataset.query),
    startDirectChatFromDirectoryUser: (el) => runPendingAction(el, () => startDirectChatFromDirectoryUser(el.dataset.peerId, el.dataset.username, el.dataset.name, el.dataset.bundle)),
    triggerMediaPicker: () => triggerDeviceMediaPicker(),
    triggerDocPicker: () => triggerDeviceDocPicker(),
    startVoiceRecording: () => startVoiceRecording(),
    openLocationModal: () => { closePanels(); openLocationModal(); },
    closePanels: () => closePanels(),
    insertEmoji: (el) => insertEmoji(el.dataset.emoji),
    toggleAttachmentDrawer: (el, event) => toggleAttachmentDrawer(event),
    toggleEmojiPicker: (el, event) => toggleEmojiPicker(event),
    sendMessage: (el) => runPendingAction(el, sendMessage),
    cancelVoiceRecording: () => cancelVoiceRecording(),
    togglePauseVoiceRecording: () => togglePauseVoiceRecording(),
    toggleVoicePreview: () => toggleVoicePreview(),
    stopAndSendVoiceRecording: (el) => runPendingAction(el, stopAndSendVoiceRecording),
    closeLocationModal: () => closeLocationModal(),
    confirmAndSendLocation: (el) => runPendingAction(el, confirmAndSendLocation),
    closeMediaPreviewModal: () => closeMediaPreviewModal(),
    confirmAndSendPendingMedia: (el) => runPendingAction(el, confirmAndSendPendingMedia),
    trustActiveContact: (el) => runPendingAction(el, () => trustActiveContact(el.dataset.peerId)),
    openBlockModal: (el) => openBlockModal(el.dataset.peerId, el.dataset.name),
    closeBlockModal: () => closeBlockModal(),
    confirmBlockOnly: (el) => runPendingAction(el, confirmBlockOnly),
    confirmBlockAndDelete: (el) => runPendingAction(el, confirmBlockAndDelete),
    blockActiveContact: (el) => runPendingAction(el, blockActiveContact),
    unblockActiveContact: (el) => runPendingAction(el, () => unblockActiveContact(el.dataset.peerId)),
    deleteContact: (el) => runPendingAction(el, () => deleteContactReal(el.dataset.peerId, el.dataset.name)),
    addContact: (el) => runPendingAction(el, addContactReal),
    addActiveContactToContacts: (el) => runPendingAction(el, () => addActiveContactToContacts(el.dataset.peerId, el.dataset.name)),
    retryFailedMessage: (el) => runPendingAction(el, () => retryFailedMessageReal(el.dataset.msgId, el.dataset.convId, el.dataset.recipientId)),
    clearAppCache: () => clearAppCache(),
    inspectDirectoryUser: (el) => inspectDirectoryUser(el.dataset.peerId),
    closeInspectUserModal: () => closeInspectUserModal(),
    addInspectedUser: (el) => runPendingAction(el, addInspectedUser),
    addDirectUser: (el) => runPendingAction(el, () => addDirectUser(el.dataset.peerId, el.dataset.username, el.dataset.name, el.dataset.bundle)),
    triggerContactDirectorySearch: () => triggerContactDirectorySearchNow(),
    refreshDirectoryNodes: () => runPendingAction(null, refreshDirectoryNodesAndSearch),
    triggerContactsSearch: () => triggerContactsSearchNow(),
    toggleManualInviteAccordion: () => toggleManualInviteAccordion(),
    openMnemonicAuthModal: () => openMnemonicAuthModal(),
    closeMnemonicAuthModal: () => closeMnemonicAuthModal(),
    confirmMnemonicPin: () => confirmMnemonicPin(),
    openEditProfileModal: () => openEditProfileModal(),
    closeEditProfileModal: () => closeEditProfileModal(),
    saveProfileChanges: (el) => runPendingAction(el, saveProfileChanges),
    triggerAvatarPicker: () => { const el = document.getElementById('profile-avatar-input'); if (el) el.click(); },
    saveQrImage: () => saveQrImage(),
    shareInvitation: () => shareInvitation(),
    copyOwnBundle: () => copyOwnBundle(),
    openQrCameraScanner: () => openQrCameraScanner(),
    closeQrCameraScanner: () => closeQrCameraScanner(),
    triggerQrImagePicker: () => { const el = document.getElementById('qr-image-input'); if (el) el.click(); },
    copyOwnPeerId: () => copyOwnPeerId(),
    fillAddContactForm: (el) => fillAddContactForm(el.dataset.name, el.dataset.id),
    openImagePreview: (el) => openImagePreviewEl(el),
    closeImagePreview: () => closeImagePreview(),
    downloadCurrentImage: () => runPendingAction(null, downloadCurrentImage),
    saveAttachment: (el) => runPendingAction(el, () => saveAttachmentToDisk(el.dataset.msgId, el.dataset.filename)),
    showFileAlert: (el) => showFileAlertEl(el),
    closeMnemonicDisplayModal: () => closeMnemonicDisplayModal(),
    copyMnemonicPhrase: () => copyMnemonicPhrase(),
    confirmMnemonicSaved: () => closeMnemonicDisplayModal(),
    logout: (el) => runPendingAction(el, logoutReal),
    retryOwnBundle: (el) => runPendingAction(el, retryOwnBundleReal),
    openReportModal: (el) => openReportModal(el.dataset.peerId, el.dataset.name),
    closeReportModal: () => closeReportModal(),
    confirmCreateGroup: (el) => runPendingAction(el, confirmCreateGroupReal),
    toggleGroupMemberSelect: (el) => toggleGroupMemberSelect(el.dataset.peerId),
    shareGroupInvitation: () => shareGroupInvitation(),
    confirmLeaveGroup: (el) => runPendingAction(el, confirmLeaveGroupReal),
    confirmSubmitReport: (el) => runPendingAction(el, confirmSubmitReportReal),
    openFeedbackModal: () => openFeedbackModal(),
    closeFeedbackModal: () => closeFeedbackModal(),
    setFeedbackRating: (el) => setFeedbackRating(parseInt(el.dataset.rating, 10)),
    confirmSubmitFeedback: (el) => runPendingAction(el, confirmSubmitFeedbackReal),
    loadAdminOverview: (el) => runPendingAction(el, loadAdminOverviewReal),
    saveAdminToken: () => saveAdminToken(),
    setAdminTab: (el) => setAdminTab(el.dataset.tab),
    setAdminFilter: (el) => setAdminFilter(el.dataset.filter),
    adminBanUser: (el) => runPendingAction(el, () => adminBanUserReal(el.dataset.peerId)),
    adminUnbanUser: (el) => runPendingAction(el, () => adminUnbanUserReal(el.dataset.peerId)),
    adminSaveThreshold: (el) => runPendingAction(el, adminSaveThresholdReal),
    openSetupPinModal: () => openSetupPinModal(),
    closeSetupPinModal: () => closeSetupPinModal(),
    saveNewPin: (el) => runPendingAction(el, saveNewPin),
    appLockKeypad: (el) => handlePinDigit(el.dataset.digit),
    appLockBackspace: () => handlePinBackspace(),
    triggerBiometricsUnlock: () => triggerBiometricsUnlock(),
    openMessageActionsModal: (el) => openMessageActionsModal(el.dataset.msgId),
    closeMessageActionsModal: () => closeMessageActionsModal(),
    copyMessageText: () => copyMessageText(),
    deleteMessageForMe: (el) => runPendingAction(el, deleteMessageForMe),
    deleteMessageForEveryone: (el) => runPendingAction(el, deleteMessageForEveryone),
    shareMessageExternally: () => shareMessageExternally(),
    saveMessageMediaToDisk: (el) => runPendingAction(el, saveMessageMediaToDisk),
    openForwardMessageModal: () => openForwardMessageModal(),
    closeForwardMessageModal: () => closeForwardMessageModal(),
    forwardToContact: (el) => runPendingAction(el, () => confirmForwardMessage(el.dataset.peerId)),
    openEphemeralTimerModal: () => openEphemeralTimerModal(),
    closeEphemeralTimerModal: () => closeEphemeralTimerModal(),
    setEphemeralTimer: (el) => setConversationEphemeralTimer(el.dataset.mode),
    reportCurrentMessage: () => reportCurrentMessage(),
    openDocumentViewer: (el) => openDocumentViewer(el.dataset.msgId, el.dataset.filename),
    closeDocumentViewer: () => closeDocumentViewer(),
    openSavedDocExternally: (el) => runPendingAction(el, openSavedDocExternally),
    saveCurrentDocument: (el) => runPendingAction(el, saveCurrentDocument),
    copyDocTextContent: () => copyDocTextContent(),
    openLocationModal: (el) => openLocationModal(el.dataset.coords, el.dataset.label),
    closeLocationModal: () => closeLocationModal(),
    openInOpenStreetMap: () => openInOpenStreetMap(),
    openInGoogleMaps: () => openInGoogleMaps(),
    openInNativeGps: () => openInNativeGps(),
    copyGpsCoordinates: () => copyGpsCoordinates(),
    openVideoPlayerModal: (el) => openVideoPlayerModal(el.dataset.url, el.dataset.msgId, el.dataset.filename),
    closeVideoPlayerModal: () => closeVideoPlayerModal(),
    downloadCurrentVideo: (el) => runPendingAction(el, downloadCurrentVideo),
    closeExecutableWarningModal: () => closeExecutableWarningModal(),
    confirmOpenExecutable: (el) => runPendingAction(el, confirmOpenExecutable),
};

document.addEventListener('click', (event) => {
    const el = event.target.closest('[data-action]');
    if (!el) return;
    const handler = CLICK_ACTIONS[el.dataset.action];
    if (handler) handler(el, event);
});

// Enter-to-submit on the few inputs that had `onkeydown="if(event.key==='Enter') fn()"` — also
// blocked by the same CSP, for the same reason as onclick above.
const ENTER_SUBMIT_MAP = {
    'account-name-input': () => createAccountReal(),
    'restore-name-input': () => restoreAccountReal(),
    'chat-input': () => sendMessage(),
    'mnemonic-auth-pin': () => confirmMnemonicPin(),
    'edit-display-name-input': () => saveProfileChanges(),
    'media-caption-input': () => confirmAndSendPendingMedia(),
    'admin-token-input': () => saveAdminToken(),
    'admin-threshold-input': () => adminSaveThresholdReal(),
    'setup-pin-confirm': () => saveNewPin(),
    'contact-search-query': () => triggerContactDirectorySearchNow(),
    'contacts-search-input': () => triggerContactsSearchNow(),
    'add-display-name-input': () => addContactReal(),
    'add-bundle-input': () => addContactReal(),
    'create-group-name-input': () => confirmCreateGroupReal(),
    'create-group-desc-input': () => confirmCreateGroupReal(),
    'global-search-input': () => handleStrictSearch(document.getElementById('global-search-input')?.value || ''),
};
document.addEventListener('keydown', (event) => {
    if (state.isAppLocked) {
        if (event.key >= '0' && event.key <= '9') {
            handlePinDigit(event.key);
            event.preventDefault();
            event.stopImmediatePropagation();
            return;
        } else if (event.key === 'Backspace') {
            handlePinBackspace();
            event.preventDefault();
            event.stopImmediatePropagation();
            return;
        }
        // Block all other keys when locked
        event.preventDefault();
        event.stopImmediatePropagation();
        return;
    }
    const handler = event.key === 'Enter' && ENTER_SUBMIT_MAP[event.target.id];
    if (handler) handler();
});

const INPUT_HANDLERS = {
    'conversations-search-input': filterConversationsList,
    'new-chat-search-input': filterNewChatList,
    'contacts-search-input': filterContactsList,
    'global-search-input': handleStrictSearch,
    'contact-search-query': handleContactDirectorySearch,
    'admin-user-search-input': (val) => {
        state.adminState.searchQuery = val;
        if (state.currentScreen === 'admin_dashboard') navigateTo('admin_dashboard');
    },
};
document.addEventListener('input', (event) => {
    const handler = INPUT_HANDLERS[event.target.id];
    if (handler) handler(event.target.value);
});

const CHANGE_HANDLERS = {
    'media-file-input': handleMediaFileSelect,
    'doc-file-input': handleDocFileSelect,
    'qr-image-input': handleQrImageSelect,
    'profile-avatar-input': handleAvatarFileSelect,
    'notif-messages-chk': (e) => {
        state.notificationPrefs.notifyMessages = e.target.checked;
        localStorage.setItem('nova_notify_messages', String(e.target.checked));
        if (e.target.checked && typeof Notification !== 'undefined' && Notification.requestPermission) {
            Notification.requestPermission();
        }
    },
    'notif-contacts-chk': (e) => {
        state.notificationPrefs.notifyContacts = e.target.checked;
        localStorage.setItem('nova_notify_contacts', String(e.target.checked));
    },
    'notif-ringtone-chk': (e) => {
        state.notificationPrefs.ringtoneEnabled = e.target.checked;
        localStorage.setItem('nova_ringtone_enabled', String(e.target.checked));
    },
    'notif-app-sounds-chk': (e) => {
        state.notificationPrefs.appSounds = e.target.checked;
        localStorage.setItem('nova_app_sounds', String(e.target.checked));
    },
    'notif-hide-content-chk': (e) => {
        state.notificationPrefs.hideContent = e.target.checked;
        localStorage.setItem('nova_hide_content', String(e.target.checked));
    },
    'auto-save-media-chk': (e) => {
        state.notificationPrefs.autoSaveMedia = e.target.checked;
        localStorage.setItem('nova_auto_save_media', String(e.target.checked));
    },
    'app-lock-toggle-chk': (e) => {
        if (e.target.checked) {
            if (!state.appLock.pinHash) {
                e.target.checked = false;
                openSetupPinModal();
                return;
            }
            state.appLock.enabled = true;
            localStorage.setItem('nova_app_lock_enabled', 'true');
        } else {
            state.appLock.enabled = false;
            localStorage.setItem('nova_app_lock_enabled', 'false');
        }
        if (state.currentScreen === 'settings') navigateTo('settings');
    },
    'app-lock-timeout-select': (e) => {
        const val = parseInt(e.target.value, 10);
        state.appLock.timeoutMin = val;
        localStorage.setItem('nova_app_lock_timeout_min', String(val));
    },
    'app-lock-biometrics-chk': (e) => {
        state.appLock.biometricsEnabled = e.target.checked;
        localStorage.setItem('nova_app_lock_biometrics', String(e.target.checked));
    },
};
document.addEventListener('change', (event) => {
    const handler = CHANGE_HANDLERS[event.target.id];
    if (handler) handler(event);
});

// Global Reactive State (Strictly 1-to-1 Device Sovereignty)
const state = {
    currentScreen: 'onboarding',
    networkOnline: typeof navigator !== 'undefined' ? navigator.onLine : true,
    isServerConnected: true,
    conversationsSearchQuery: '',
    contactsSearchQuery: '',
    newChatSearchQuery: '',
    appLock: {
        enabled: localStorage.getItem('nova_app_lock_enabled') === 'true',
        pinHash: localStorage.getItem('nova_app_lock_pin_hash') || '',
        pinSalt: localStorage.getItem('nova_app_lock_pin_salt') || '',
        pinLength: parseInt(localStorage.getItem('nova_app_lock_pin_length') || '4', 10),
        timeoutMin: parseInt(localStorage.getItem('nova_app_lock_timeout_min') || '5', 10),
        biometricsEnabled: localStorage.getItem('nova_app_lock_biometrics') !== 'false',
    },
    isAppLocked: false,
    enteredPin: '',
    failedPinAttempts: 0,
    lockoutUntil: 0,
    lastUserInteraction: Date.now(),
    wentToBackgroundAt: null,
    notificationPrefs: {
        notifyMessages: localStorage.getItem('nova_notify_messages') !== 'false',
        notifyContacts: localStorage.getItem('nova_notify_contacts') !== 'false',
        ringtoneEnabled: localStorage.getItem('nova_ringtone_enabled') !== 'false',
        appSounds: localStorage.getItem('nova_app_sounds') !== 'false',
        hideContent: localStorage.getItem('nova_hide_content') === 'true',
        autoSaveMedia: localStorage.getItem('nova_auto_save_media') !== 'false',
    },
    currentUser: {
        name: '',
        username: '',
        handle: '',
        // The real, ground-truth identifier (hex-encoded Ed25519 public key) other peers use to
        // reach this device. `handle` above is a cosmetic display name only — there is no
        // handle-to-peer_id directory service, so it is never used for actual protocol
        // operations (add_contact/send_message always use peerId).
        peerId: '',
        bio: '',
        avatarDataUrl: null,
        status: 'Compte non créé',
        publicKey: '',
        mnemonic: '',
        // Hex-encoded X3DH prekey bundle, fetched on demand — see refreshOwnBundleHex().
        bundleHex: '',
        // Cryptographically signed invitation URI (nova://invite?d=...) with 24h deadline
        invitationUri: '',
        // Set by refreshOwnBundleHex() when both the signed-invitation call and the legacy
        // prekey-bundle fallback fail — distinguishes a genuine, terminal failure (show an
        // error + retry) from "still fetching" (bundleHex/invitationUri simply not populated
        // yet), which previously looked identical and left the Identity screen stuck showing
        // "Génération du lien sécurisé…" forever with a blank QR code and no way to retry.
        linkGenerationError: '',
    },
    // Set by openChatWith() when a conversation is opened. null means "no chat open" — screens
    // that render it must handle that rather than assume a contact always exists.
    activeContact: null,
    // The real `PeerConnectionInfo` last observed by nova-transport's TransportSupervisor for
    // the currently open contact (see refreshDiagnostics()) — null until at least one connection
    // attempt has happened. Never fabricated; screens that need it must handle it being null.
    currentDiagnostics: null,
    // Rendezvous/bootstrap multiaddr for first contact on a different network (see
    // set_bootstrap_addr) — populated by refreshBootstrapAddr(), edited on the Settings screen.
    // '' means none configured (mDNS/LAN-only discovery).
    bootstrapAddr: '',
    // Fallback discovery and relay server URL (e.g. wss://nova-discovery-jllv.onrender.com)
    fallbackServerUrl: '',
    // Search results from public directory query
    directorySearchResults: [],
    // User currently inspected in the directory modal
    inspectedDirectoryUser: null,
    // This device's own dialable multiaddrs (see get_own_full_listen_addrs) — typically one IPv4
    // and, when available, one IPv6 — shown read-only on Settings so the operator can copy one to
    // other devices when this one plays the rendezvous role. [] until the network has started.
    ownFullListenAddrs: [],
    // Pure 1-to-1 Sovereign Conversations — populated only by real contact/message activity.
    conversations: [],
    messages: [],
    contacts: [],
    buildInfo: {
        variant: 'user',
        is_admin: false,
        version: '1.0.0',
    },
    feedbackRating: 5,
    pendingReport: null,
    adminState: {
        token: localStorage.getItem('nova_admin_token') || '',
        overview: null,
        activeTab: 'users',
        filterStatus: 'all',
        searchQuery: '',
        isLoading: false,
        error: '',
    },
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
    // Shared empty-state for any screen that needs an open conversation (chat, contact_profile)
    // but none is open — reachable via the settings screen's direct-navigation grid even before
    // any real contact exists, now that there is no fake "Emma" always pre-selected.
    _noActiveContact: (fallback) => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="${fallback}">${icons.arrowLeft}</button>
                <div class="header-title">Aucune conversation</div>
                <div style="width: 38px;"></div>
            </header>
            <div style="padding: 40px 24px; text-align: center; color: var(--text-muted); display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1;">
                <img src="logo.png" alt="Logo" style="width: 100px; height: 100px; object-fit: contain; opacity: 0.22; filter: blur(1.5px) drop-shadow(0 0 20px rgba(139, 92, 246, 0.4)); margin-bottom: 20px;">
                <p style="font-size: 14px; line-height: 1.5; max-width: 300px;">Aucune conversation n'est ouverte. Ouvrez-en une depuis la liste des conversations ou ajoutez un contact.</p>
                <button class="btn-primary" style="margin-top: 20px;" data-action="navigate" data-screen="conversations">Voir mes conversations</button>
            </div>
        </div>
    `,

    // 1. Écran de bienvenue
    onboarding: () => `
        <div class="screen-view" style="justify-content: space-between; padding: 40px 24px; text-align: center; background: radial-gradient(circle at 50% 30%, #171A24 0%, #080A10 70%);">
            <div style="margin-top: 24px;">
                <div style="font-size: 13px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 2px; font-weight: 600;">Bienvenue sur NOVA</div>
                <p style="font-size: 15px; color: var(--text-muted); line-height: 1.4; max-width: 280px; margin: 8px auto 0;">Messagerie privée & chiffrée de bout en bout.</p>
            </div>

            <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; margin: 20px auto;">
                <img src="logo.png" alt="Logo" style="width: 120px; height: 120px; object-fit: contain; filter: drop-shadow(0 8px 30px rgba(139, 92, 246, 0.45)); margin-bottom: 16px;">
                <div style="font-size: 12px; color: var(--accent-purple-light); font-weight: 600; display: flex; align-items: center; gap: 6px;">
                    <span>🔒</span> <span>100% Chiffré & Sécurisé</span>
                </div>
            </div>

            <div>
                <button class="btn-primary" data-action="navigate" data-screen="create_account">Créer mon compte</button>
                <button class="btn-secondary" style="margin-top: 10px; width: 100%;" data-action="navigate" data-screen="restore_account">J'ai déjà un compte</button>
            </div>
        </div>
    `,

    // 2. Création de compte
    create_account: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="onboarding">${icons.arrowLeft}</button>
                <div class="header-title">Créer un compte</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px; flex: 1; display: flex; flex-direction: column; justify-content: space-between;">
                <div>
                    <div style="width: 56px; height: 56px; border-radius: 50%; background: rgba(139, 92, 246, 0.12); border: 1px solid var(--accent-purple); display: flex; align-items: center; justify-content: center; margin: 0 auto 16px; color: var(--accent-purple-light);">
                        ${icons.lock}
                    </div>

                    <label style="font-size: 13px; color: var(--text-muted); font-weight: 500;">Votre nom ou pseudo</label>
                    <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 12px 16px; margin: 8px 0 14px; border: 1px solid var(--border-subtle);">
                        <input type="text" id="account-name-input" value="${escapeHtml(state.currentUser.name)}" placeholder="Votre nom" style="background: none; border: none; color: white; font-size: 15px; width: 100%; outline: none;">
                    </div>

                    <p style="font-size: 12px; color: var(--text-muted); line-height: 1.4;">Une phrase secrète de 12 mots vous permettra de récupérer votre compte.</p>
                </div>

                <button class="btn-primary" data-action="createAccount">Créer mon compte</button>
            </div>
        </div>
    `,

    // 2b. Restauration de compte à partir d'une phrase existante
    restore_account: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="onboarding">${icons.arrowLeft}</button>
                <div class="header-title">Retrouver mon compte</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px; flex: 1; display: flex; flex-direction: column; justify-content: space-between;">
                <div>
                    <label style="font-size: 13px; color: var(--text-muted); font-weight: 500;">Votre nom</label>
                    <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 12px 16px; margin: 8px 0 14px; border: 1px solid var(--border-subtle);">
                        <input type="text" id="restore-name-input" placeholder="Votre nom" style="background: none; border: none; color: white; font-size: 15px; width: 100%; outline: none;">
                    </div>

                    <label style="font-size: 13px; color: var(--text-muted); font-weight: 500;">Vos 12 mots secrets</label>
                    <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 12px 16px; margin: 8px 0 14px; border: 1px solid var(--border-subtle);">
                        <textarea id="restore-mnemonic-input" placeholder="mot1 mot2 mot3 ..." rows="3" style="background: none; border: none; color: white; font-size: 14px; width: 100%; outline: none; resize: none; font-family: monospace;"></textarea>
                    </div>
                    <p style="font-size: 12px; color: var(--text-muted); line-height: 1.4;">Saisissez vos 12 mots dans l'ordre, séparés par un espace.</p>
                </div>

                <button class="btn-primary" data-action="restoreAccount">Retrouver mon compte</button>
            </div>
        </div>
    `,

    // 3. Liste des conversations (100% 1-to-1)
    conversations: () => `
        <div class="screen-view">
            <header class="app-header">
                <div>
                    <div class="header-title">Conversations</div>
                    <div id="user-global-status" style="font-size: 11px; display: flex; align-items: center; gap: 5px; margin-top: 2px;">
                        <span class="user-status-dot" style="width: 7px; height: 7px; border-radius: 50%; background: ${state.isServerConnected ? 'var(--status-success)' : '#ef4444'};"></span>
                        <span style="color: ${state.isServerConnected ? 'var(--status-success)' : 'var(--text-muted)'}; font-weight: 500;">${state.isServerConnected ? 'Connecté' : 'Non connecté / Hors ligne'}</span>
                    </div>
                </div>
                <div class="header-actions">
                    <button class="icon-btn" data-action="navigate" data-screen="create_group" title="Nouveau groupe">${icons.users}</button>
                    <button class="icon-btn" data-action="navigate" data-screen="add_contact" title="Ajouter un contact">${icons.plus}</button>
                </div>
            </header>

            <div class="search-bar-wrap">
                <div class="search-input-box">
                    ${icons.search}
                    <input type="text" id="conversations-search-input" placeholder="Rechercher une conversation..." value="${escapeHtml(state.conversationsSearchQuery || '')}">
                    ${state.conversationsSearchQuery ? `<button style="background: none; border: none; color: var(--text-muted); cursor: pointer; padding: 0 4px; font-size: 14px;" data-action="clearConversationsSearch">✕</button>` : ''}
                </div>
            </div>

            <div class="scroll-list" id="conversations-list-container">
                ${renderConversationsListHtml(state.conversationsSearchQuery)}
            </div>

            <button class="fab-start-chat" data-action="navigate" data-screen="new_chat" title="Démarrer une discussion">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path>
                </svg>
                <span>Démarrer une discussion</span>
            </button>
        </div>
    `,

    // 3b. Démarrer une nouvelle discussion (Modèle Google Messages LLC)
    new_chat: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="conversations">${icons.arrowLeft}</button>
                <div class="header-title">Nouvelle discussion</div>
                <button class="icon-btn" data-action="navigate" data-screen="create_group" title="Nouveau groupe">${icons.users}</button>
            </header>

            <div class="search-bar-wrap" style="padding: 10px 16px 6px;">
                <div class="search-input-box">
                    ${icons.search}
                    <input type="text" id="new-chat-search-input" placeholder="Saisir un nom, @pseudo ou identifiant..." value="${escapeHtml(state.newChatSearchQuery || '')}">
                    ${state.newChatSearchQuery ? `<button style="background: none; border: none; color: var(--text-muted); cursor: pointer; padding: 0 4px; font-size: 14px;" data-action="clearNewChatSearch">✕</button>` : ''}
                </div>
            </div>

            <div class="scroll-list" id="new-chat-results-container" style="padding: 10px 16px 80px;">
                ${renderNewChatResultsHtml(state.newChatSearchQuery)}
            </div>
        </div>
    `,

    // 4. Conversation individuelle (1-to-1)
    chat: () => { if (!state.activeContact) return screens._noActiveContact('conversations'); return `
        <div class="screen-view">
            <!-- Hidden native file pickers for real device file access -->
            <input type="file" id="media-file-input" accept="image/*,video/*" style="display: none;">
            <input type="file" id="doc-file-input" accept="*/*" style="display: none;">

            <header class="app-header">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <button class="icon-btn" data-action="navigate" data-screen="conversations">${icons.arrowLeft}</button>
                    <div class="avatar" style="width: 38px; height: 38px; font-size: 14px; cursor: pointer; ${state.activeContact.isGroup ? 'background: linear-gradient(135deg, #8b5cf6, #3b82f6);' : ''}" data-action="navigate" data-screen="${state.activeContact.isGroup ? 'group_info' : 'contact_profile'}">
                        ${state.activeContact.isGroup ? icons.users : escapeHtml(state.activeContact.name.charAt(0))}
                        <div class="status-dot ${state.activeContact.isGroup || (state.currentDiagnostics && state.currentDiagnostics.is_connected) ? 'status-online' : 'status-offline'}"></div>
                    </div>
                    <div data-action="navigate" data-screen="${state.activeContact.isGroup ? 'group_info' : 'contact_profile'}" style="cursor: pointer;">
                        <div style="font-size: 15px; font-weight: 700; color: white; display: flex; align-items: center; gap: 6px;">
                            ${state.activeContact.isGroup ? '<span>👥</span>' : ''}
                            ${escapeHtml(state.activeContact.name)}
                        </div>
                        <div id="chat-header-status" style="font-size: 11px; color: var(--accent-purple-light); font-weight: 500; display: flex; align-items: center; gap: 4px;">
                            ${state.activeContact.isGroup ? 'Groupe sécurisé (E2EE)' : chatHeaderStatusHtml()}
                        </div>
                    </div>
                </div>
                <div class="header-actions">
                    ${state.activeContact.isGroup ? `
                        <button class="icon-btn" data-action="navigate" data-screen="group_info" title="Infos du groupe">${icons.users}</button>
                    ` : `
                        <button class="icon-btn" data-name="${escapeHtml(state.activeContact.name)}" data-action="callVoice" title="Appel vocal">${icons.phone}</button>
                        <button class="icon-btn" data-name="${escapeHtml(state.activeContact.name)}" data-action="callVideo" title="Appel vidéo">${icons.video}</button>
                        <button class="icon-btn" data-action="navigate" data-screen="contact_profile" title="Infos du contact">${icons.user}</button>
                    `}
                </div>
            </header>

            ${!state.activeContact.isGroup && !state.activeContact.isContact ? `
                <div id="unknown-contact-banner" style="background: rgba(245, 158, 11, 0.12); border-bottom: 1px solid rgba(245, 158, 11, 0.3); padding: 12px 14px; display: flex; flex-direction: column; gap: 8px; z-index: 10;">
                    <div style="display: flex; align-items: center; gap: 8px;">
                        <span style="font-size: 16px;">ℹ️</span>
                        <div style="font-size: 12px; color: #fbbf24; line-height: 1.4;">
                            <strong>${escapeHtml(state.activeContact.name || state.activeContact.handle)}</strong> ne fait pas partie de vos contacts.
                        </div>
                    </div>
                    <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                        <button class="btn-secondary" style="font-size: 11px; padding: 6px 12px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.3);" data-action="openBlockModal" data-peer-id="${escapeHtml(state.activeContact.handle)}" data-name="${escapeHtml(state.activeContact.name)}">
                            Bloquer
                        </button>
                        <button class="btn-primary" style="font-size: 11px; padding: 6px 12px;" data-action="addActiveContactToContacts" data-peer-id="${escapeHtml(state.activeContact.handle)}" data-name="${escapeHtml(state.activeContact.name)}">
                            + Ajouter aux contacts
                        </button>
                        ${!state.activeContact.isTrusted ? `
                            <button class="btn-primary" style="font-size: 11px; padding: 6px 12px; background: #d97706; border-color: #f59e0b;" data-action="trustActiveContact" data-peer-id="${escapeHtml(state.activeContact.handle)}">
                                ✓ Faire confiance
                            </button>
                        ` : ''}
                    </div>
                </div>
            ` : (!state.activeContact.isTrusted ? `
                <div id="trust-contact-banner" style="background: rgba(245, 158, 11, 0.12); border-bottom: 1px solid rgba(245, 158, 11, 0.3); padding: 10px 14px; display: flex; align-items: center; justify-content: space-between; gap: 10px; z-index: 10;">
                    <div style="display: flex; align-items: center; gap: 8px; flex: 1;">
                        <span style="font-size: 16px;">⚠️</span>
                        <div style="font-size: 11px; color: #fbbf24; line-height: 1.3;">
                            Faites-vous confiance à <strong>${escapeHtml(state.activeContact.name)}</strong> ?
                        </div>
                    </div>
                    <div style="display: flex; gap: 6px;">
                        <button class="btn-secondary" style="font-size: 11px; padding: 5px 9px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.3);" data-action="openBlockModal" data-peer-id="${escapeHtml(state.activeContact.handle)}" data-name="${escapeHtml(state.activeContact.name)}">
                            Bloquer...
                        </button>
                        <button class="btn-primary" style="font-size: 11px; padding: 5px 11px; background: #d97706; border-color: #f59e0b;" data-action="trustActiveContact" data-peer-id="${escapeHtml(state.activeContact.handle)}">
                            ✓ Faire confiance
                        </button>
                    </div>
                </div>
            ` : '')}

            <div class="chat-body" id="chat-body">
                <div style="text-align: center; margin: 10px 0;">
                    <span style="background: rgba(139, 92, 246, 0.1); border: 1px solid rgba(139, 92, 246, 0.2); border-radius: var(--radius-full); padding: 4px 12px; font-size: 11px; color: var(--accent-purple-light); display: inline-flex; align-items: center; gap: 6px;">
                        ${icons.lock} Conversation privée et protégée
                    </span>
                </div>

                ${state.messages.filter(m => m.conversationId === state.activeContact.conversationId && !m.hidden).length === 0 ? `
                    <div class="empty-chat-placeholder" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 60%; margin: auto 0; text-align: center; pointer-events: none; user-select: none;">
                        <img src="logo.png" alt="Logo" style="width: 120px; height: 120px; object-fit: contain; opacity: 0.18; filter: blur(2px) drop-shadow(0 0 24px rgba(139, 92, 246, 0.5)); margin-bottom: 18px;">
                        <div style="font-size: 13px; color: var(--text-dim); font-weight: 500; opacity: 0.65;">Aucun message pour l'instant</div>
                        <div style="font-size: 11px; color: var(--text-dim); opacity: 0.45; margin-top: 4px;">Envoyez un message pour démarrer la discussion</div>
                    </div>
                ` : state.messages.filter(m => m.conversationId === state.activeContact.conversationId && !m.hidden).map(m => buildMessageHtml(m)).join('')}
            </div>

            <!-- Drawer for Multimedia Attachments -->
            <div class="attachment-drawer" id="attachment-drawer">
                <button class="drawer-option" data-action="triggerMediaPicker">
                    ${icons.image}
                    <span>Photo ou vidéo</span>
                </button>
                <button class="drawer-option" data-action="triggerDocPicker">
                    ${icons.file}
                    <span>Document</span>
                </button>
                <button class="drawer-option" data-action="startVoiceRecording">
                    ${icons.mic}
                    <span>Message vocal</span>
                </button>
                <button class="drawer-option" data-action="openLocationModal">
                    ${icons.mapPin}
                    <span>Ma position</span>
                </button>
            </div>

            <!-- Clean HD Emoji Picker Drawer -->
            <div class="emoji-picker-panel" id="emoji-picker-panel">
                <div class="emoji-picker-header">
                    <span style="font-size: 12px; font-weight: 700; color: var(--text-muted); letter-spacing: 0.5px;">ÉMOJIS</span>
                    <button class="icon-btn" data-action="closePanels" style="width: 24px; height: 24px; font-size: 12px;" title="Fermer">✕</button>
                </div>
                <div class="emoji-picker-grid">
                    ${state.emojis.map(e => `<button class="emoji-btn" data-action="insertEmoji" data-emoji="${e}" title="${e}">${e}</button>`).join('')}
                </div>
            </div>

            <!-- Secure Chat Input Bar & WhatsApp Voice Recorder -->
            <div class="chat-input-bar">
                <div class="chat-input-main-row" id="normal-input-row">
                    <button class="icon-btn" id="attachment-toggle-btn" data-action="toggleAttachmentDrawer" title="Pièces jointes">${icons.paperclip}</button>
                    <button class="icon-btn" id="emoji-toggle-btn" data-action="toggleEmojiPicker" title="Émojis">${icons.smile}</button>
                    <div class="chat-input-container">
                        <input type="text" id="chat-input" placeholder="Votre message">
                    </div>
                    <button class="send-btn" id="send-btn" data-action="sendMessage" title="Envoyer le message">${icons.send}</button>
                    <button class="mic-btn" id="mic-record-btn" data-action="startVoiceRecording" title="Enregistrer une note vocale">${icons.mic}</button>
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
                    <button class="rec-icon-btn" data-action="cancelVoiceRecording" title="Supprimer l'enregistrement">
                        ${icons.trash}
                    </button>

                    <!-- Pause / Resume button -->
                    <button class="rec-icon-btn" id="rec-pause-btn" data-action="togglePauseVoiceRecording" title="Mettre en pause / Reprendre">
                        <span id="rec-pause-icon">⏸</span>
                    </button>

                    <!-- Preview / Listen button (active when paused) -->
                    <button class="rec-icon-btn" id="rec-preview-btn" data-action="toggleVoicePreview" title="Écouter l'enregistrement" style="display: none; color: var(--accent-purple-light);">
                        <span id="rec-preview-icon">▶</span>
                    </button>

                    <!-- Send button -->
                    <button class="rec-send-btn" data-action="stopAndSendVoiceRecording" title="Envoyer la note vocale">
                        ${icons.send}
                    </button>
                </div>
            </div>

            <!-- Block & Delete Options Modal -->
            <div class="location-modal-overlay" id="block-contact-modal">
                <div class="location-modal-card" style="max-width: 360px; padding: 20px;">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px;">
                        <div style="font-size: 16px; font-weight: 700; color: white;" id="block-modal-title">
                            Bloquer ce contact ?
                        </div>
                        <button class="icon-btn" data-action="closeBlockModal" style="width: 28px; height: 28px;">✕</button>
                    </div>
                    <p style="font-size: 12px; color: var(--text-muted); line-height: 1.4; margin-bottom: 18px;" id="block-modal-desc">
                        Les futurs messages et appels de ce contact seront immédiatement rejetés. Vous pourrez le retrouver dans l'onglet Contacts pour le débloquer si nécessaire.
                    </p>

                    <div style="display: flex; flex-direction: column; gap: 10px; margin-bottom: 14px;">
                        <button class="btn-secondary" style="width: 100%; padding: 12px; font-size: 12px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.3); text-align: center;" data-action="confirmBlockOnly">
                            🚫 Bloquer uniquement
                        </button>
                        <button class="btn-secondary" style="width: 100%; padding: 12px; font-size: 12px; color: var(--status-danger); background: rgba(239, 68, 68, 0.12); border-color: rgba(239, 68, 68, 0.4); text-align: center;" data-action="confirmBlockAndDelete">
                            🗑️ Bloquer et supprimer la conversation
                        </button>
                    </div>

                    <button class="btn-secondary" style="width: 100%; font-size: 12px; padding: 10px;" data-action="closeBlockModal">
                        Annuler
                    </button>
                </div>
            </div>

            <!-- Media Preview & Confirmation Modal before sending -->
            <div class="location-modal-overlay" id="media-preview-modal">
                <div class="location-modal-card" style="max-width: 420px; padding: 20px;">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px;">
                        <div style="font-size: 16px; font-weight: 700; color: white; display: flex; align-items: center; gap: 8px;" id="media-preview-title">
                            ${icons.image} <span>Aperçu du média</span>
                        </div>
                        <button class="icon-btn" data-action="closeMediaPreviewModal" style="width: 28px; height: 28px; color: var(--text-muted);">✕</button>
                    </div>

                    <div id="media-preview-container" style="background: #090d16; border-radius: var(--radius-md); border: 1px solid var(--border-subtle); overflow: hidden; margin-bottom: 14px; display: flex; align-items: center; justify-content: center; min-height: 180px; max-height: 280px; position: relative;">
                    </div>

                    <div style="margin-bottom: 14px;">
                        <div style="background-color: var(--bg-surface-2); border-radius: var(--radius-md); padding: 10px 14px; border: 1px solid var(--border-subtle); display: flex; align-items: center; gap: 8px;">
                            <input type="text" id="media-caption-input" placeholder="Ajouter une légende... (facultatif)" style="background: none; border: none; color: white; font-size: 13px; width: 100%; outline: none;">
                        </div>
                    </div>

                    <div style="font-size: 11px; color: var(--accent-purple-light); display: flex; align-items: center; gap: 6px; margin-bottom: 16px; padding: 6px 10px; background: rgba(139, 92, 246, 0.1); border-radius: var(--radius-sm);">
                        ${icons.lock} <span>Chiffré de bout en bout pour <strong>${escapeHtml(state.activeContact.name)}</strong></span>
                    </div>

                    <div style="display: flex; gap: 10px;">
                        <button class="btn-secondary" style="flex: 1;" data-action="closeMediaPreviewModal">Annuler</button>
                        <button class="btn-primary" style="flex: 1; display: flex; align-items: center; justify-content: center; gap: 8px;" data-action="confirmAndSendPendingMedia">
                            ${icons.send}
                            <span>Envoyer</span>
                        </button>
                    </div>
                </div>
            </div>

            <!-- Location Sharing Confirmation Modal with Map Preview -->
            <div class="location-modal-overlay" id="location-modal">
                <div class="location-modal-card">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                        <div style="font-size: 16px; font-weight: 700; color: white; display: flex; align-items: center; gap: 8px;">
                            ${icons.mapPin} Partager ma position
                        </div>
                        <button class="icon-btn" data-action="closeLocationModal" style="width: 28px; height: 28px;">✕</button>
                    </div>
                    <p style="font-size: 13px; color: var(--text-muted);">
                        Voulez-vous envoyer votre position actuelle à <strong>${state.activeContact.name}</strong> ? Elle sera protégée comme le reste de vos messages, et personne d'autre ne pourra la voir.
                    </p>

                    <div class="map-radar-preview">
                        <div class="map-grid-lines"></div>
                        <div class="map-pin-pulse">${icons.mapPin}</div>
                        <div style="position: absolute; bottom: 8px; font-size: 11px; font-weight: 600; color: var(--accent-purple-light); z-index: 2;" id="loc-coords-preview">
                            48.8566° N, 2.3522° E (Paris)
                        </div>
                    </div>

                    <div style="font-size: 11px; color: var(--text-dim); text-align: center; margin-bottom: 18px;">
                        Précision d'environ 5 mètres
                    </div>

                    <div style="display: flex; gap: 10px;">
                        <button class="btn-secondary" style="flex: 1;" data-action="closeLocationModal">Annuler</button>
                        <button class="btn-primary" style="flex: 1;" data-action="confirmAndSendLocation">Partager ma position</button>
                    </div>
                </div>
            </div>
        </div>
    `; },

    // 5. Profil du contact
    contact_profile: () => { if (!state.activeContact) return screens._noActiveContact('conversations'); return `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="chat">${icons.arrowLeft}</button>
                <div class="header-title">Profil du contact</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px 20px; overflow-y: auto;">
                <div style="text-align: center; margin-bottom: 24px;">
                    <div class="avatar" style="width: 84px; height: 84px; font-size: 32px; margin: 0 auto 12px;">
                        ${escapeHtml(state.activeContact.name.charAt(0))}
                    </div>
                    <h2 style="font-size: 20px; font-weight: 700; color: white;">${escapeHtml(state.activeContact.name)}</h2>
                    <div style="font-size: 13px; color: ${state.currentDiagnostics && state.currentDiagnostics.is_connected ? 'var(--status-success)' : 'var(--text-muted)'}; margin-top: 4px; display: flex; align-items: center; justify-content: center; gap: 6px;">
                        <span class="p2p-badge-pulse" style="width:8px;height:8px;"></span> ${contactConnectionStatusText()}
                    </div>
                    <div style="margin-top: 6px;">
                        <span style="font-size: 11px; padding: 3px 8px; border-radius: var(--radius-full); ${state.activeContact.isTrusted ? 'background: rgba(34, 197, 94, 0.15); color: var(--status-success);' : 'background: rgba(245, 158, 11, 0.15); color: #fbbf24;'}">
                            ${state.activeContact.isTrusted ? '✓ Contact de confiance' : '⚠️ Non vérifié'}
                        </span>
                    </div>
                </div>

                <div style="display: flex; justify-content: space-around; margin-bottom: 24px; gap: 8px;">
                    <button class="btn-secondary" style="flex: 1; flex-direction: column; padding: 12px; font-size: 11px;" data-action="navigate" data-screen="chat">
                        ${icons.chat}
                        <span style="margin-top:4px;">Message</span>
                    </button>
                    <button class="btn-secondary" style="flex: 1; flex-direction: column; padding: 12px; font-size: 11px;" data-action="callVoice">
                        ${icons.phone}
                        <span style="margin-top:4px;">Vocal</span>
                    </button>
                    <button class="btn-secondary" style="flex: 1; flex-direction: column; padding: 12px; font-size: 11px;" data-action="callVideo">
                        ${icons.video}
                        <span style="margin-top:4px;">Vidéo</span>
                    </button>
                    <button class="btn-secondary" style="flex: 1; flex-direction: column; padding: 12px; font-size: 11px;" data-action="navigate" data-screen="shared_media">
                        ${icons.image}
                        <span style="margin-top:4px;">Médias</span>
                    </button>
                </div>

                <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 16px; border: 1px solid var(--border-subtle);">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
                        <div style="font-size: 13px; font-weight: 600; color: white; display: flex; align-items: center; gap: 6px;">
                            <span>🛡️</span> <span>Sécurité de la conversation</span>
                        </div>
                        <span style="font-size: 11px; color: var(--status-success); background: rgba(34,197,94,0.12); padding: 2px 8px; border-radius: 12px;">Chiffré E2EE</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-muted); margin-top: 6px;">Code de vérification :</div>
                    <div style="font-size: 14px; font-family: monospace; font-weight: 700; letter-spacing: 2px; color: var(--accent-purple-light); margin-top: 4px;">
                        ${formatSafetyNumber(state.activeContact.safetyNumber)}
                    </div>
                    <p style="font-size: 11px; color: var(--text-dim); margin: 6px 0 0; line-height: 1.4;">Vos messages et appels vocaux/vidéo avec ce contact sont 100% privés et inaccessibles aux tiers.</p>
                </div>

                ${!state.activeContact.isTrusted && !state.activeContact.isBlocked ? `
                    <button class="btn-primary" style="width: 100%; margin-bottom: 10px; background: #d97706; border-color: #f59e0b;" data-action="trustActiveContact" data-peer-id="${escapeHtml(state.activeContact.peerId)}">✓ Faire confiance à ce contact</button>
                ` : ''}

                ${state.activeContact.isBlocked ? `
                    <button class="btn-primary" style="width: 100%; margin-bottom: 10px;" data-action="unblockActiveContact" data-peer-id="${escapeHtml(state.activeContact.peerId)}">Débloquer ce contact</button>
                ` : `
                    <button class="btn-secondary" style="width: 100%; margin-bottom: 10px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.2);" data-action="openBlockModal" data-peer-id="${escapeHtml(state.activeContact.peerId)}" data-name="${escapeHtml(state.activeContact.name)}">Bloquer ce contact...</button>
                `}
                <button class="btn-secondary" style="width: 100%; margin-bottom: 10px; color: #fbbf24; border-color: rgba(245, 158, 11, 0.25);" data-action="openReportModal" data-peer-id="${escapeHtml(state.activeContact.peerId)}" data-name="${escapeHtml(state.activeContact.name)}">⚠️ Signaler ce contact...</button>
                <button class="btn-secondary" style="width: 100%; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.2);" data-action="deleteContact" data-peer-id="${escapeHtml(state.activeContact.peerId)}" data-name="${escapeHtml(state.activeContact.name)}">Supprimer ce contact</button>
            </div>
        </div>
    `; },

    // 6. Liste des contacts
    contacts: () => `
        <div class="screen-view">
            <header class="app-header">
                <div>
                    <div class="header-title">Contacts</div>
                    <div id="contacts-user-status" style="font-size: 11px; display: flex; align-items: center; gap: 5px; margin-top: 2px;">
                        <span class="user-status-dot" style="width: 7px; height: 7px; border-radius: 50%; background: ${state.isServerConnected ? 'var(--status-success)' : '#ef4444'};"></span>
                        <span style="color: ${state.isServerConnected ? 'var(--status-success)' : 'var(--text-muted)'}; font-weight: 500;">${state.isServerConnected ? 'Connecté' : 'Non connecté / Hors ligne'}</span>
                    </div>
                </div>
                <div class="header-actions">
                    <button class="icon-btn" data-action="navigate" data-screen="add_contact" title="Ajouter un contact">${icons.plus}</button>
                </div>
            </header>

            <div class="search-bar-wrap">
                <div class="search-input-box">
                    ${icons.search}
                    <input type="text" id="contacts-search-input" placeholder="Rechercher un contact..." value="${escapeHtml(state.contactsSearchQuery || '')}" autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false">
                    ${state.contactsSearchQuery ? `<button style="background: none; border: none; color: var(--text-muted); cursor: pointer; padding: 0 4px; font-size: 14px;" data-action="clearContactsSearch">✕</button>` : ''}
                </div>
            </div>

            <div class="scroll-list" id="contacts-list-container">
                ${state.contacts.filter(c => !c.isBlocked).length > 0 ? state.contacts.filter(c => !c.isBlocked).map(c => `
                    <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-action="openChat">
                        <div class="avatar">
                            ${escapeHtml(c.name.charAt(0))}
                            <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                        </div>
                        <div class="item-content">
                            <div class="item-name">${escapeHtml(c.name)}</div>
                            <div class="item-sub">@${escapeHtml(c.handle)}</div>
                        </div>
                    </div>
                `).join('') : `
                    <div style="text-align: center; color: var(--text-muted); padding: 60px 24px;">
                        <div style="width: 48px; height: 48px; border-radius: 50%; background: var(--bg-surface); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--text-dim);">
                            ${icons.users}
                        </div>
                        <div style="font-size: 15px; font-weight: 600; color: white;">Aucun contact actif</div>
                        <p style="font-size: 13px; color: var(--text-muted); margin-top: 6px; max-width: 260px; margin-left: auto; margin-right: auto;">Ajoutez un contact pour commencer à échanger en toute confidentialité.</p>
                        <button class="btn-primary" style="margin-top: 18px;" data-action="navigate" data-screen="add_contact">Ajouter un contact</button>
                    </div>
                `}

                ${state.contacts.some(c => c.isBlocked) ? `
                    <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 24px 0 8px 12px;">CONTACTS BLOQUÉS</div>
                    ${state.contacts.filter(c => c.isBlocked).map(c => `
                        <div class="item-card" style="opacity: 0.75;">
                            <div class="avatar" style="background: var(--bg-surface); color: var(--status-danger);">✕</div>
                            <div class="item-content">
                                <div class="item-name">${escapeHtml(c.name)}</div>
                                <div class="item-sub" style="color: var(--status-danger);">Bloqué</div>
                            </div>
                            <div style="display: flex; gap: 6px;">
                                <button class="btn-secondary" style="font-size: 11px; padding: 6px 12px;" data-action="unblockActiveContact" data-peer-id="${escapeHtml(c.handle)}">Débloquer</button>
                                <button class="btn-secondary" style="font-size: 11px; padding: 6px 12px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.2);" data-action="deleteContact" data-peer-id="${escapeHtml(c.handle)}" data-name="${escapeHtml(c.name)}">Supprimer</button>
                            </div>
                        </div>
                    `).join('')}
                ` : ''}
            </div>
        </div>
    `,

    // 7. Ajouter un contact
    add_contact: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="contacts">${icons.arrowLeft}</button>
                <div class="header-title">Ajouter un contact</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 20px 16px; overflow-y: auto; padding-bottom: 90px;">
                <!-- Fast Actions: QR Code Scanner and Display -->
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 20px;">
                    <button class="btn-secondary" style="font-size: 12px; padding: 12px; display: flex; align-items: center; justify-content: center; gap: 8px;" data-action="openQrCameraScanner">
                        ${icons.qr}
                        <span>Scanner un QR</span>
                    </button>
                    <button class="btn-secondary" style="font-size: 12px; padding: 12px; display: flex; align-items: center; justify-content: center; gap: 8px;" data-action="navigate" data-screen="identity">
                        ${icons.user}
                        <span>Mon QR Code</span>
                    </button>
                </div>

                <!-- Online Search in Directory -->
                <div style="margin-bottom: 20px;">
                    <div style="font-size: 13px; font-weight: 600; color: white; margin-bottom: 8px;">Recherche dans l'annuaire</div>
                    <div class="search-input-box">
                        ${icons.search}
                        <input type="text" id="contact-search-query" placeholder="Nom, @pseudo ou identifiant..." autocomplete="off" autocorrect="off" autocapitalize="off" spellcheck="false">
                    </div>
                </div>

                <!-- Dynamic Search Results -->
                <div id="directory-search-results" style="margin-bottom: 20px;"></div>

                <!-- Direct / Manual Addition -->
                <div style="background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-lg); padding: 18px 16px; margin-bottom: 22px;">
                    <div style="font-size: 13px; font-weight: 700; color: white; margin-bottom: 12px;">Ajouter un contact par lien ou @pseudo</div>

                    <label style="font-size: 12px; color: var(--text-muted);">Nom ou surnom *</label>
                    <div class="search-input-box" style="margin: 6px 0 12px;">
                        <input type="text" id="add-display-name-input" placeholder="ex: Alice, Paul..." autocomplete="off" autocorrect="off" autocapitalize="words" spellcheck="false">
                    </div>

                    <label style="font-size: 12px; color: var(--text-muted);">@Pseudo ou lien d'invitation *</label>
                    <div style="background-color: var(--bg-surface-2); border-radius: var(--radius-md); padding: 10px 12px; margin: 6px 0 14px; border: 1px solid var(--border-subtle);">
                        <input type="text" id="add-bundle-input" placeholder="ex: @alice ou collez le lien partagé..." style="width: 100%; background: none; border: none; color: white; font-size: 13px; outline: none;">
                    </div>

                    <button class="btn-primary" style="width: 100%; padding: 12px; font-size: 13px; font-weight: 700; display: flex; align-items: center; justify-content: center; gap: 8px;" data-action="addContact">
                        ${icons.plus}
                        <span>Ajouter le contact</span>
                    </button>
                </div>
            </div>

            <!-- Inspect Directory User Profile Modal -->
            <div class="location-modal-overlay" id="inspect-user-modal">
                <div class="location-modal-card" style="max-width: 360px;" id="inspect-user-card">
                    <!-- Injected dynamically by inspectDirectoryUser() -->
                </div>
            </div>

            <!-- Live Camera QR Scanner Modal Overlay -->
            <div class="qr-scanner-overlay" id="qr-camera-modal">
                <div class="qr-scanner-header">
                    <div style="display: flex; align-items: center; gap: 8px;">
                        ${icons.qr} <span>Scanner un QR Code</span>
                    </div>
                    <button class="icon-btn" data-action="closeQrCameraScanner" style="width: 32px; height: 32px; color: white;">✕</button>
                </div>

                <div class="qr-scanner-viewport">
                    <video id="qr-scanner-video" class="qr-scanner-video" playsinline autoplay muted></video>
                    <canvas id="qr-scanner-canvas" style="display: none;"></canvas>
                    <div class="qr-scanner-frame"></div>
                    <div class="qr-scanner-corners"></div>
                    <div class="qr-scanner-laser"></div>
                    <div id="qr-camera-error" style="display: none; position: absolute; inset: 20px; text-align: center; color: var(--text-muted); font-size: 12px; align-items: center; justify-content: center; flex-direction: column; gap: 10px; background: rgba(0,0,0,0.85); border-radius: var(--radius-md);">
                        <span>Caméra indisponible ou permission non accordée.</span>
                        <button class="btn-secondary" style="font-size: 12px; padding: 8px 14px;" data-action="triggerQrImagePicker">Importer une capture d'écran</button>
                    </div>
                </div>

                <div class="qr-scanner-footer">
                    <p style="font-size: 12px; color: var(--text-muted); text-align: center; margin: 0;">
                        Cadrez le QR code à l'écran, ou choisissez une image depuis votre appareil.
                    </p>
                    <div style="display: flex; width: 100%; gap: 10px;">
                        <input type="file" id="qr-image-input" accept="image/*" style="display: none;">
                        <button class="btn-secondary" style="flex: 1; font-size: 12px; padding: 11px;" data-action="triggerQrImagePicker">
                            <span>🖼 Importer une image</span>
                        </button>
                        <button class="btn-secondary" style="flex: 1; font-size: 12px; padding: 11px;" data-action="closeQrCameraScanner">
                            <span>Fermer</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    `,

    // Nouveau Groupe Souverain P2P
    create_group: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="conversations">${icons.arrowLeft}</button>
                <div class="header-title">Nouveau groupe</div>
                <div style="width: 38px;"></div>
            </header>

            <div class="scroll-content" style="padding: 16px;">
                <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 18px; border: 1px solid var(--border-subtle); margin-bottom: 20px;">
                    <div style="display: flex; gap: 14px; align-items: center; margin-bottom: 16px;">
                        <div style="width: 52px; height: 52px; border-radius: 50%; background: linear-gradient(135deg, #8b5cf6, #3b82f6); display: flex; align-items: center; justify-content: center; color: white; flex-shrink: 0;">
                            ${icons.users}
                        </div>
                        <div style="flex: 1;">
                            <label style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 700; letter-spacing: 0.5px;">Nom du groupe *</label>
                            <input type="text" id="create-group-name-input" placeholder="ex: Projet Nova, Famille..." style="width: 100%; background: var(--bg-surface-2); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 10px 12px; color: white; font-size: 14px; margin-top: 4px; outline: none;">
                        </div>
                    </div>

                    <div>
                        <label style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 700; letter-spacing: 0.5px;">Description (facultative)</label>
                        <input type="text" id="create-group-desc-input" placeholder="Objectif ou sujet du groupe" style="width: 100%; background: var(--bg-surface-2); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 10px 12px; color: white; font-size: 13px; margin-top: 4px; outline: none;">
                    </div>
                </div>

                <div style="margin-bottom: 16px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                        <span style="font-size: 12px; font-weight: 700; color: white; text-transform: uppercase; letter-spacing: 0.5px;">Sélectionner les membres</span>
                        <span style="font-size: 11px; color: var(--accent-purple-light);" id="group-selected-count">0 sélectionné(s)</span>
                    </div>

                    <div style="background: var(--bg-surface); border-radius: var(--radius-md); border: 1px solid var(--border-subtle); overflow: hidden; max-height: 280px; overflow-y: auto;" id="group-contacts-selection-list">
                        ${renderGroupContactsSelectionHtml()}
                    </div>
                </div>

                <div style="padding: 10px 12px; background: rgba(139, 92, 246, 0.08); border-radius: var(--radius-md); border: 1px solid rgba(139, 92, 246, 0.2); margin-bottom: 20px; font-size: 11px; color: var(--text-muted); display: flex; gap: 8px; align-items: center;">
                    <span style="color: var(--accent-purple-light); font-size: 14px;">🔒</span>
                    <span>Messages et fichiers chiffrés de bout en bout (E2EE).</span>
                </div>

                <button class="btn-primary" style="width: 100%; padding: 14px; font-size: 14px; font-weight: 700;" data-action="confirmCreateGroup">Créer le groupe</button>
            </div>
        </div>
    `,

    // Détails et gestion du groupe (Membres, Invitation, Quitter)
    group_info: () => {
        if (!state.activeContact || !state.activeContact.isGroup) return screens._noActiveContact('conversations');
        return `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="chat">${icons.arrowLeft}</button>
                <div class="header-title">Infos du groupe</div>
                <div style="width: 38px;"></div>
            </header>

            <div class="scroll-content" style="padding: 16px;" id="group-info-container">
                <div style="text-align: center; padding: 20px 0 24px;">
                    <div style="width: 72px; height: 72px; border-radius: 50%; background: linear-gradient(135deg, #8b5cf6, #3b82f6); display: flex; align-items: center; justify-content: center; color: white; margin: 0 auto 12px; font-size: 28px;">
                        ${icons.users}
                    </div>
                    <div style="font-size: 18px; font-weight: 800; color: white;">${escapeHtml(state.activeContact.name)}</div>
                    <div style="font-size: 12px; color: var(--accent-purple-light); margin-top: 4px;">Groupe chiffré (E2EE)</div>
                </div>

                <div style="display: flex; gap: 10px; margin-bottom: 24px;">
                    <button class="btn-primary" style="flex: 1; padding: 10px; font-size: 12px; display: flex; align-items: center; justify-content: center; gap: 6px;" data-action="shareGroupInvitation">
                        ${icons.share || icons.copy} <span>Inviter des amis</span>
                    </button>
                </div>

                <div style="margin-bottom: 24px;">
                    <div style="font-size: 12px; font-weight: 700; color: white; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">Membres du groupe</div>
                    <div style="background: var(--bg-surface); border-radius: var(--radius-md); border: 1px solid var(--border-subtle); overflow: hidden;" id="group-members-list">
                        <div style="text-align: center; padding: 20px; color: var(--text-dim); font-size: 12px;">Chargement des membres...</div>
                    </div>
                </div>

                <div style="border-top: 1px solid var(--border-subtle); padding-top: 20px;">
                    <button class="btn-secondary" style="width: 100%; padding: 12px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.3); font-weight: 600;" data-action="confirmLeaveGroup">
                        Quitter et supprimer le groupe
                    </button>
                </div>
            </div>
        </div>
        `;
    },

    // 8. Médias partagés (1-to-1)
    shared_media: () => { if (!state.activeContact) return screens._noActiveContact('conversations'); const conversationId = state.activeContact.conversationId;
        const media = state.messages.filter(m => m.conversationId === conversationId && (m.type === 'image' || m.type === 'video'));
        const files = state.messages.filter(m => m.conversationId === conversationId && m.type === 'file');
        // Thumbnails render with whatever url each message already has; anything not loaded yet
        // is fetched now and this screen re-renders once each one arrives (see ensureAttachmentLoaded).
        media.filter(m => !m.url).forEach(ensureAttachmentLoaded);
        return `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="chat">${icons.arrowLeft}</button>
                <div class="header-title">Médias partagés</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 16px; overflow-y: auto;">
                <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 8px;">Photos & vidéos de cette conversation</div>
                ${media.length > 0 ? `
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-bottom: 16px;">
                        ${media.map(m => m.type === 'video'
                            ? `<div style="aspect-ratio: 1; border-radius: var(--radius-sm); overflow: hidden;"><video src="${escapeHtml(m.url || '')}" style="width: 100%; height: 100%; object-fit: cover;" muted></video></div>`
                            : `<div style="aspect-ratio: 1; border-radius: var(--radius-sm); overflow: hidden;"><img src="${escapeHtml(m.url || '')}" style="width: 100%; height: 100%; object-fit: cover;"></div>`
                        ).join('')}
                    </div>
                ` : `<div style="font-size: 13px; color: var(--text-dim); margin-bottom: 16px;">Aucun média partagé.</div>`}

                <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 8px;">Fichiers de cette conversation</div>
                ${files.length > 0 ? files.map(m => `
                    <div style="background: var(--bg-surface); padding: 12px; border-radius: var(--radius-md); display: flex; align-items: center; gap: 12px; border: 1px solid var(--border-subtle); margin-bottom: 8px;">
                        ${icons.file}
                        <div>
                            <div style="font-size: 14px; font-weight: 600; color: white;">${escapeHtml(m.text)}</div>
                            <div style="font-size: 11px; color: var(--text-muted);">${escapeHtml(m.meta)}</div>
                        </div>
                    </div>
                `).join('') : `<div style="font-size: 13px; color: var(--text-dim);">Aucun fichier partagé.</div>`}
            </div>
        </div>
    `; },

    // 9. Paramètres (100% orienté utilisateur, simple et élégant)
    settings: () => `
        <div class="screen-view">
            <header class="app-header">
                <div class="header-title">Paramètres</div>
            </header>

            <div style="padding: 16px; overflow-y: auto; padding-bottom: 90px;">
                <!-- Full Profile Card -->
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 18px; border: 1px solid var(--border-subtle); margin-bottom: 20px;">
                    <div style="display: flex; align-items: center; gap: 14px; margin-bottom: 12px;">
                        <div class="avatar" style="width: 56px; height: 56px; font-size: 22px; background: var(--accent-purple); color: white; overflow: hidden; padding: 0;">
                            ${state.currentUser.avatarDataUrl ? `<img src="${state.currentUser.avatarDataUrl}" style="width: 100%; height: 100%; object-fit: cover;">` : escapeHtml(state.currentUser.name.charAt(0) || '?')}
                        </div>
                        <div style="flex: 1;">
                            <div style="font-size: 17px; font-weight: 700; color: white;">${escapeHtml(state.currentUser.name) || 'Mon compte'}</div>
                            ${state.currentUser.handle ? `<div style="font-size: 13px; color: var(--accent-purple-light);">@${escapeHtml(state.currentUser.handle)}</div>` : ''}
                            <div style="font-size: 11px; color: var(--text-muted); margin-top: 2px; display: flex; align-items: center; gap: 6px;">
                                <span class="status-dot ${state.networkOnline ? 'status-online' : 'status-offline'}" style="width: 7px; height: 7px; display: inline-block;"></span>
                                <span id="profile-network-status-text">${escapeHtml(state.currentUser.status)}</span>
                            </div>
                        </div>
                        <button class="icon-btn" data-action="openEditProfileModal" title="Modifier mon profil">${icons.pencil || '✎'}</button>
                    </div>
                    <div style="font-size: 12px; color: var(--text-muted); line-height: 1.4; border-top: 1px solid var(--border-subtle); padding-top: 10px; display: flex; justify-content: space-between; align-items: center;">
                        <span>${escapeHtml(state.currentUser.bio) || 'Aucune biographie définie.'}</span>
                        <button class="btn-secondary" data-action="openEditProfileModal" style="font-size: 11px; padding: 4px 10px; margin-left: 8px;">Modifier</button>
                    </div>
                </div>

                <!-- Account & Profile Sharing Shortcuts -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">COMPTE & SÉCURITÉ</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); overflow: hidden; margin-bottom: 24px; border: 1px solid var(--border-subtle);">
                    <div class="item-card" data-action="navigate" data-screen="identity">
                        ${icons.qr}
                        <div class="item-content">
                            <div class="item-name">Mon QR Code & Lien de profil</div>
                            <div class="item-sub">Partagez votre compte facilement avec vos proches</div>
                        </div>
                        ${icons.chevronRight}
                    </div>
                    <div class="item-card" data-action="openMnemonicAuthModal" style="border-top: 1px solid var(--border-subtle);">
                        ${icons.shield}
                        <div class="item-content">
                            <div class="item-name">Sauvegarde du compte</div>
                            <div class="item-sub">Afficher ma phrase secrète de 12 mots</div>
                        </div>
                        ${icons.chevronRight}
                    </div>
                </div>

                <!-- Notifications & Sounds -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">NOTIFICATIONS & SONS</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 16px; border: 1px solid var(--border-subtle); margin-bottom: 24px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <div>
                            <div style="font-size: 14px; color: white;">Sonnerie des appels</div>
                            <div style="font-size: 11px; color: var(--text-muted);">Faire sonner lors d'un appel entrant</div>
                        </div>
                        <input type="checkbox" id="notif-ringtone-chk" ${state.notificationPrefs.ringtoneEnabled !== false ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <div>
                            <div style="font-size: 14px; color: white;">Notifications des messages</div>
                            <div style="font-size: 11px; color: var(--text-muted);">Alerte pour chaque nouveau message reçu</div>
                        </div>
                        <input type="checkbox" id="notif-messages-chk" ${state.notificationPrefs.notifyMessages ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <div>
                            <div style="font-size: 14px; color: white;">Sons dans l'application</div>
                            <div style="font-size: 11px; color: var(--text-muted);">Bips lors de l'envoi et de la réception</div>
                        </div>
                        <input type="checkbox" id="notif-app-sounds-chk" ${state.notificationPrefs.appSounds !== false ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <div>
                            <div style="font-size: 14px; color: white;">Masquer l'aperçu du texte</div>
                            <div style="font-size: 11px; color: var(--text-muted);">Ne pas afficher le texte du message dans les alertes</div>
                        </div>
                        <input type="checkbox" id="notif-hide-content-chk" ${state.notificationPrefs.hideContent ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                    </div>
                </div>

                <!-- App Lock & Inactivity Timeout -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">SÉCURITÉ & VERROUILLAGE</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 16px; border: 1px solid var(--border-subtle); margin-bottom: 24px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: ${state.appLock.enabled ? '14px' : '0'};">
                        <div>
                            <div style="font-size: 14px; font-weight: 600; color: white;">Verrouillage de l'application</div>
                            <div style="font-size: 11px; color: var(--text-muted);">Exiger un code PIN ou empreinte après inactivité</div>
                        </div>
                        <input type="checkbox" id="app-lock-toggle-chk" ${state.appLock.enabled ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                    </div>

                    ${state.appLock.enabled ? `
                        <div style="border-top: 1px solid var(--border-subtle); padding-top: 14px; margin-top: 12px;">
                            <label style="font-size: 12px; color: var(--text-muted); display: block; margin-bottom: 6px;">Délai d'inactivité avant verrouillage</label>
                            <select id="app-lock-timeout-select" style="width: 100%; box-sizing: border-box; background: var(--bg-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); padding: 10px 12px; color: white; font-size: 13px; margin-bottom: 14px; outline: none;">
                                <option value="0" ${state.appLock.timeoutMin === 0 ? 'selected' : ''}>⚡ Immédiatement (quand on quitte l'app)</option>
                                <option value="3" ${state.appLock.timeoutMin === 3 ? 'selected' : ''}>⏱️ 3 minutes d'inactivité</option>
                                <option value="5" ${state.appLock.timeoutMin === 5 ? 'selected' : ''}>⏱️ 5 minutes d'inactivité</option>
                                <option value="10" ${state.appLock.timeoutMin === 10 ? 'selected' : ''}>⏱️ 10 minutes d'inactivité</option>
                                <option value="15" ${state.appLock.timeoutMin === 15 ? 'selected' : ''}>⏱️ 15 minutes d'inactivité</option>
                                <option value="30" ${state.appLock.timeoutMin === 30 ? 'selected' : ''}>⏱️ 30 minutes d'inactivité</option>
                                <option value="60" ${state.appLock.timeoutMin === 60 ? 'selected' : ''}>⏱️ 60 minutes d'inactivité</option>
                            </select>

                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
                                <div>
                                    <div style="font-size: 13px; color: white;">Empreinte / Biométrie</div>
                                    <div style="font-size: 11px; color: var(--text-muted);">Déverrouiller avec le capteur de l'appareil</div>
                                </div>
                                <input type="checkbox" id="app-lock-biometrics-chk" ${state.appLock.biometricsEnabled ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                            </div>

                            <button class="btn-secondary" style="width: 100%; font-size: 12px; padding: 10px;" data-action="openSetupPinModal">🔑 Modifier mon code PIN</button>
                        </div>
                    ` : ''}
                </div>

                <!-- Storage & Dedicated Media -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">STOCKAGE & DONNÉES</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 16px; border: 1px solid var(--border-subtle); margin-bottom: 24px;">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;">
                        <div style="font-size: 13px; font-weight: 600; color: white;">Dossier des médias reçus</div>
                        <span style="font-size: 11px; background: rgba(34, 197, 94, 0.15); color: var(--status-success); padding: 2px 8px; border-radius: var(--radius-full); font-weight: 600;">Automatique</span>
                    </div>
                    <div style="font-size: 12px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
                        Vos photos, vidéos et documents reçus sont automatiquement enregistrés dans le dossier <strong>Téléchargements/NOVA</strong> pour ne jamais les perdre.
                    </div>
                    <div style="font-size: 11px; color: var(--accent-purple-light); font-family: monospace; word-break: break-all; margin-bottom: 12px;" id="settings-media-folder-path"></div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
                        <div>
                            <div style="font-size: 13px; color: white;">Téléchargement automatique</div>
                            <div style="font-size: 11px; color: var(--text-muted);">Enregistrer les médias dès réception</div>
                        </div>
                        <input type="checkbox" id="auto-save-media-chk" ${state.notificationPrefs.autoSaveMedia !== false ? 'checked' : ''} style="accent-color: var(--accent-purple); width: 18px; height: 18px; cursor: pointer;">
                    </div>
                    <button class="btn-secondary" style="width: 100%; font-size: 12px; padding: 9px;" data-action="clearAppCache">🧹 Vider le cache temporaire</button>
                </div>

                <!-- User Feedback & Stars (All Users) -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">VOTRE AVIS SUR L'APPLICATION</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 16px; border: 1px solid var(--border-subtle); margin-bottom: 24px;">
                    <div style="font-size: 13px; font-weight: 600; color: white; margin-bottom: 4px;">Donnez votre avis & vos étoiles ⭐</div>
                    <div style="font-size: 12px; color: var(--text-muted); line-height: 1.4; margin-bottom: 12px;">
                        Votre avis et vos suggestions nous aident directement à perfectionner la qualité de NOVA.
                    </div>
                    <button class="btn-primary" style="width: 100%; font-size: 13px; padding: 10px;" data-action="openFeedbackModal">⭐ Noter et donner mon avis</button>
                </div>

                <!-- Admin Supervision Console (ONLY visible in Admin build) -->
                ${state.buildInfo && state.buildInfo.is_admin ? `
                <div style="font-size: 12px; color: #fbbf24; font-weight: 700; margin: 0 0 8px 6px; display: flex; align-items: center; gap: 6px;">
                    <span>🛡️</span>
                    <span>CONSOLE D'ADMINISTRATION (BUILD ADMIN)</span>
                </div>
                <div style="background: rgba(245, 158, 11, 0.08); border-radius: var(--radius-lg); padding: 16px; border: 1px solid rgba(245, 158, 11, 0.3); margin-bottom: 24px;">
                    <div style="font-size: 13px; font-weight: 700; color: #fbbf24; margin-bottom: 4px;">Supervision & Modération active</div>
                    <div style="font-size: 12px; color: var(--text-muted); line-height: 1.4; margin-bottom: 12px;">
                        Accédez au tableau de bord des utilisateurs sains, signalés et bannis, gérez l'auto-quarantaine et lisez les retours.
                    </div>
                    <button class="btn-primary" style="width: 100%; font-size: 13px; padding: 11px; background: #d97706; border-color: #f59e0b;" data-action="navigate" data-screen="admin_dashboard">🛡️ Ouvrir la Console Admin</button>
                </div>
                ` : ''}

                <!-- About & Sovereignty -->
                <div style="font-size: 12px; color: var(--text-muted); font-weight: 600; margin: 0 0 8px 6px;">À PROPOS</div>
                <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 16px; border: 1px solid var(--border-subtle); margin-bottom: 24px; text-align: center;">
                    <div style="font-size: 15px; font-weight: 700; color: white; margin-bottom: 4px;">NOVA Messagerie</div>
                    <div style="font-size: 12px; color: var(--accent-purple-light); margin-bottom: 8px;">Version 1.0.0 • Chiffrement de bout en bout</div>
                    <p style="font-size: 11px; color: var(--text-muted); line-height: 1.4; margin: 0;">
                        Toutes vos communications sont protégées et chiffrées. Aucun intermédiaire ne peut lire vos messages ni écouter vos appels.
                    </p>
                </div>

                <button class="btn-secondary" style="width: 100%; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.2); padding: 12px;" data-action="logout">Se déconnecter de cet appareil</button>
            </div>

            <!-- Profile Edit Modal -->
            <div class="location-modal-overlay" id="edit-profile-modal">
                <div class="location-modal-card" style="max-width: 360px;">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
                        <div style="font-size: 16px; font-weight: 700; color: white;">Modifier mon profil</div>
                        <button class="icon-btn" data-action="closeEditProfileModal" style="width: 28px; height: 28px;">✕</button>
                    </div>

                    <div style="text-align: center; margin-bottom: 18px;">
                        <input type="file" id="profile-avatar-input" accept="image/*" style="display: none;">
                        <div class="avatar" id="edit-profile-avatar-preview" data-action="triggerAvatarPicker" style="width: 72px; height: 72px; font-size: 26px; margin: 0 auto 8px; cursor: pointer; position: relative; overflow: hidden; border: 2px dashed var(--accent-purple-light);">
                            ${state.currentUser.avatarDataUrl ? `<img src="${state.currentUser.avatarDataUrl}" style="width: 100%; height: 100%; object-fit: cover;">` : escapeHtml(state.currentUser.name.charAt(0) || '?')}
                        </div>
                        <button class="btn-secondary" data-action="triggerAvatarPicker" style="font-size: 11px; padding: 6px 12px;">Choisir une photo</button>
                    </div>

                    <label style="font-size: 12px; color: var(--text-muted);">Nom d'affichage</label>
                    <div style="background-color: var(--bg-elevated); border-radius: var(--radius-md); padding: 10px 14px; margin: 6px 0 14px; border: 1px solid var(--border-subtle);">
                        <input type="text" id="edit-display-name-input" value="${escapeHtml(state.currentUser.name)}" placeholder="Votre nom" style="background: none; border: none; color: white; font-size: 14px; width: 100%; outline: none;">
                    </div>

                    <label style="font-size: 12px; color: var(--text-muted);">Biographie / Statut</label>
                    <div style="background-color: var(--bg-elevated); border-radius: var(--radius-md); padding: 10px 14px; margin: 6px 0 18px; border: 1px solid var(--border-subtle);">
                        <textarea id="edit-bio-input" rows="2" placeholder="Quelques mots sur vous..." style="background: none; border: none; color: white; font-size: 13px; width: 100%; outline: none; resize: none;">${escapeHtml(state.currentUser.bio)}</textarea>
                    </div>

                    <div style="display: flex; gap: 10px;">
                        <button class="btn-secondary" style="flex: 1;" data-action="closeEditProfileModal">Annuler</button>
                        <button class="btn-primary" style="flex: 1;" data-action="saveProfileChanges">Enregistrer</button>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 10. Mon identité — regroupe le code à partager, la phrase secrète et l'appareil actif
    identity: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="settings">${icons.arrowLeft}</button>
                <div class="header-title">Mon identité</div>
                <div style="width: 38px;"></div>
            </header>

            <div style="padding: 24px 20px; overflow-y: auto; padding-bottom: 90px;">
                <div style="text-align: center; margin-bottom: 20px;">
                    <!-- The invitation ticket (signed bundle + rendezvous addresses) needs a fairly
                         high-density QR (~120-140 modules/side) to fit its ~1.5-2KB of data. Showing it
                         at only 200 CSS px (the previous size) put well under 2 physical pixels per
                         module on a typical phone screen — too fine-grained for another phone's camera
                         to resolve at all, which is what made the live scanner spin forever without ever
                         detecting a code ("ça scanne à l'infini"). 300px, plus the lower error-correction
                         level set in renderOwnQrCode(), meaningfully increases the physical size of each
                         module. -->
                    <div style="background: white; width: 300px; height: 300px; max-width: 100%; border-radius: var(--radius-lg); margin: 0 auto 14px; display: flex; align-items: center; justify-content: center; box-shadow: 0 8px 32px rgba(0,0,0,0.5); overflow: hidden; padding: 14px; box-sizing: border-box;">
                        ${state.currentUser.linkGenerationError ? `
                            <div style="color: #B91C1C; text-align: center; padding: 8px;">
                                <div style="font-size: 28px; margin-bottom: 6px;">⚠️</div>
                                <div style="font-size: 12px; font-weight: 600; margin-bottom: 6px;">Échec de génération du lien</div>
                                <div style="font-size: 10px; color: #7F1D1D; word-break: break-word; margin-bottom: 8px; max-height: 70px; overflow-y: auto;">${escapeHtml(state.currentUser.linkGenerationError)}</div>
                                <button class="btn-secondary" style="font-size: 11px; padding: 6px 10px;" data-action="retryOwnBundle">Réessayer</button>
                            </div>
                        ` : `<canvas id="my-qr-canvas" width="640" height="640" style="width: 272px; height: 272px;"></canvas>`}
                    </div>

                    <div style="display: inline-flex; align-items: center; gap: 6px; background: rgba(168, 85, 247, 0.15); border: 1px solid var(--accent-purple-light); padding: 4px 12px; border-radius: 20px; font-size: 11px; color: var(--accent-purple-light); margin-bottom: 8px;">
                        <span>🔒 Profil sécurisé</span> • <span>✨ Chiffré de bout en bout</span>
                    </div>

                    <div style="font-size: 18px; font-weight: 700; color: white;">${escapeHtml(state.currentUser.name) || 'Mon compte'}</div>
                    <p style="font-size: 12px; color: var(--text-muted); margin: 6px auto 0; max-width: 290px;">
                        Faites scanner ce QR code pour vous ajouter, ou partagez votre lien avec vos amis.
                    </p>
                </div>

                <!-- Action buttons: Save Image & Share -->
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
                    <button class="btn-secondary" style="font-size: 12px; padding: 10px;" data-action="saveQrImage">
                        <span>📥 Enregistrer l'image</span>
                    </button>
                    <button class="btn-secondary" style="font-size: 12px; padding: 10px;" data-action="shareInvitation">
                        <span>🔗 Partager le lien</span>
                    </button>
                </div>

                <div style="background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 14px; border: 1px solid var(--border-subtle); text-align: left;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                        <span style="font-size: 12px; color: var(--text-muted); font-weight: 600;">Mon profil :</span>
                        <span style="font-size: 11px; color: var(--status-success); background: rgba(34,197,94,0.12); padding: 2px 8px; border-radius: 12px;">Sécurisé ✓</span>
                    </div>
                    <div style="font-size: 16px; font-weight: 700; color: white;">${escapeHtml(state.currentUser.name || 'Mon compte')}</div>
                    <div style="font-size: 13px; color: var(--accent-purple-light); margin-top: 2px;">@${escapeHtml(state.currentUser.username || state.currentUser.name || 'utilisateur')}</div>
                    <div style="display: flex; gap: 8px; margin-top: 14px;">
                        <button class="btn-primary" style="flex: 1; font-size: 12px; padding: 10px;" data-action="copyOwnBundle">
                            <span>📋 Copier mon lien</span>
                        </button>
                        <button class="btn-secondary" style="flex: 1; font-size: 12px; padding: 10px;" data-action="shareInvitation">
                            <span>🔗 Partager</span>
                        </button>
                    </div>
                </div>

                <div style="height: 1px; background: var(--border-subtle); margin: 24px 0;"></div>

                <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; margin-bottom: 16px; border: 1px solid var(--border-subtle);">
                    <div style="font-size: 14px; font-weight: 600; color: white;">Sauvegarde du compte</div>
                    <p style="font-size: 12px; color: var(--text-muted); margin: 6px 0 12px;">Votre phrase secrète de 12 mots vous permet de récupérer votre compte et vos contacts en cas de perte de téléphone.</p>
                    <button class="btn-secondary" style="width: 100%; font-size: 13px;" data-action="openMnemonicAuthModal">Afficher ma phrase secrète</button>
                </div>

                <div class="item-card" style="background: var(--bg-surface); border: 1px solid var(--border-subtle);">
                    ${icons.user}
                    <div class="item-content">
                        <div class="item-name">Cet appareil</div>
                        <div class="item-sub" style="color: var(--status-success);">Votre compte est actif ici</div>
                    </div>
                </div>
                <p style="font-size: 12px; color: var(--text-muted); text-align: center; margin-top: 8px;">Un compte ne peut être utilisé que sur un seul appareil à la fois.</p>
            </div>

            <!-- Re-authentication gate before revealing the recovery phrase: a device left
                 briefly unattended must not hand the master secret to whoever picks it up. -->
            <div class="location-modal-overlay" id="mnemonic-auth-modal">
                <div class="location-modal-card">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;">
                        <div style="font-size: 16px; font-weight: 700; color: white; display: flex; align-items: center; gap: 8px;">
                            ${icons.lock} Confirmez que c'est bien vous
                        </div>
                        <button class="icon-btn" data-action="closeMnemonicAuthModal" style="width: 28px; height: 28px;">✕</button>
                    </div>
                    <p style="font-size: 13px; color: var(--text-muted);">
                        Entrez votre code pour afficher votre phrase secrète.
                    </p>
                    <input type="password" inputmode="numeric" id="mnemonic-auth-pin" placeholder="Votre code" maxlength="8"
                        style="width: 100%; margin: 14px 0; background-color: var(--bg-surface); border-radius: var(--radius-md); padding: 12px 16px; border: 1px solid var(--border-subtle); color: white; font-size: 15px; letter-spacing: 3px; text-align: center;">
                    <div id="mnemonic-auth-error" style="font-size: 12px; color: var(--status-danger); min-height: 16px; margin-bottom: 8px;"></div>
                    <div style="display: flex; gap: 10px;">
                        <button class="btn-secondary" style="flex: 1;" data-action="closeMnemonicAuthModal">Annuler</button>
                        <button class="btn-primary" style="flex: 1;" data-action="confirmMnemonicPin">Confirmer</button>
                    </div>
                </div>
            </div>
        </div>
    `,

    // 11. Recherche globale (Aucune suggestion parasite avant saisie)
    global_search: () => `
        <div class="screen-view">
            <header class="app-header">
                <button class="icon-btn" data-action="navigate" data-screen="conversations">${icons.arrowLeft}</button>
                <div style="flex: 1; margin: 0 10px;">
                    <div class="search-input-box">
                        ${icons.search}
                        <input type="text" id="global-search-input" placeholder="Rechercher messages, contacts..." autofocus>
                    </div>
                </div>
            </header>

            <div class="scroll-list" id="search-results-list" style="padding-top: 30px;">
                <!-- Clean State: Zero suggestions prior to typing -->
                <div style="text-align: center; color: var(--text-muted); padding: 40px 20px;">
                    <div style="width: 48px; height: 48px; border-radius: 50%; background: var(--bg-surface); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--text-dim);">
                        ${icons.search}
                    </div>
                    <div style="font-size: 15px; font-weight: 600; color: white;">Rechercher</div>
                    <p style="font-size: 13px; color: var(--text-muted); margin-top: 6px; max-width: 260px; margin-left: auto; margin-right: auto;">
                        Tapez un nom ou un mot pour retrouver un contact ou un message.
                    </p>
                </div>
            </div>
        </div>
    `,

    // 16. Console d'Administration (Build Admin uniquement)
    admin_dashboard: () => {
        if (!state.buildInfo || !state.buildInfo.is_admin) {
            return `
                <div class="screen-view">
                    <header class="app-header">
                        <button class="icon-btn" data-action="navigate" data-screen="settings">${icons.arrowLeft}</button>
                        <div class="header-title">Accès non autorisé</div>
                        <div style="width: 38px;"></div>
                    </header>
                    <div style="padding: 40px 20px; text-align: center; color: var(--text-muted);">
                        <div style="font-size: 36px; margin-bottom: 12px;">🔒</div>
                        <div style="font-size: 16px; font-weight: 700; color: white; margin-bottom: 8px;">Build Utilisateur Standard</div>
                        <p style="font-size: 13px; line-height: 1.5; max-width: 320px; margin: 0 auto;">Cette fonctionnalité de modération nécessite un build administrateur compilé avec l'option admin.</p>
                        <button class="btn-primary" style="margin-top: 20px;" data-action="navigate" data-screen="settings">Retour aux paramètres</button>
                    </div>
                </div>
            `;
        }

        const overview = state.adminState.overview;
        if (overview) {
            overview.users = overview.users || [];
            overview.reports = overview.reports || [];
            overview.feedbacks = overview.feedbacks || [];
            overview.total_users = overview.total_users ?? overview.users.length;
            overview.healthy_users = overview.healthy_users ?? overview.users.filter(u => u.status === 'healthy').length;
            overview.reported_users = overview.reported_users ?? overview.users.filter(u => u.status === 'reported').length;
            overview.banned_users = overview.banned_users ?? overview.users.filter(u => u.status === 'banned').length;
            overview.average_rating = typeof overview.average_rating === 'number' ? overview.average_rating : 5.0;
            overview.total_feedbacks = overview.total_feedbacks ?? overview.feedbacks.length;
            overview.auto_ban_threshold = overview.auto_ban_threshold ?? 3;
        }

        const activeTab = state.adminState.activeTab || 'users';
        const filterStatus = state.adminState.filterStatus || 'all';

        let filteredUsers = overview ? overview.users : [];
        if (filterStatus !== 'all') {
            filteredUsers = filteredUsers.filter(u => u.status === filterStatus);
        }
        if (state.adminState.searchQuery) {
            const q = state.adminState.searchQuery.toLowerCase();
            filteredUsers = filteredUsers.filter(u => (u.display_name || '').toLowerCase().includes(q) || (u.username || '').toLowerCase().includes(q) || (u.peer_id || '').toLowerCase().includes(q));
        }

        return `
            <div class="screen-view" style="background: #0d0f17;">
                <header class="app-header" style="background: #141724; border-bottom: 1px solid rgba(245, 158, 11, 0.2);">
                    <button class="icon-btn" data-action="navigate" data-screen="settings">${icons.arrowLeft}</button>
                    <div class="header-title" style="color: #fbbf24; display: flex; align-items: center; gap: 6px;">
                        <span>🛡️</span> <span>Console Administrateur</span>
                    </div>
                    <button class="icon-btn" data-action="loadAdminOverview" title="Actualiser">${icons.refresh || '🔄'}</button>
                </header>

                <div style="padding: 16px; overflow-y: auto; padding-bottom: 90px;">
                    <!-- Admin Token Auth Bar -->
                    <div style="background: var(--bg-surface); border-radius: var(--radius-lg); padding: 14px; border: 1px solid var(--border-subtle); margin-bottom: 16px;">
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                            <div style="font-size: 12px; font-weight: 700; color: white;">Clé secrète Administrateur</div>
                            <span style="font-size: 11px; padding: 2px 8px; border-radius: var(--radius-full); ${overview ? 'background: rgba(34, 197, 94, 0.15); color: var(--status-success);' : 'background: rgba(239, 68, 68, 0.15); color: var(--status-danger);'}">
                                ${overview ? '✓ Connecté au serveur' : '⚠️ Non authentifié'}
                            </span>
                        </div>
                        <div style="display: flex; gap: 8px;">
                            <input type="password" id="admin-token-input" value="${escapeHtml(state.adminState.token)}" placeholder="Entrez le jeton admin (ex: NOVA_ADMIN_SECRET)..." style="flex: 1; background: var(--bg-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); padding: 8px 12px; color: white; font-size: 12px; font-family: monospace; outline: none;">
                            <button class="btn-primary" style="padding: 8px 14px; font-size: 12px; background: #d97706; border-color: #f59e0b;" data-action="saveAdminToken">Valider</button>
                        </div>
                    </div>

                    ${overview ? `
                        <!-- Stats Grid -->
                        <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; margin-bottom: 16px;">
                            <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 12px; border: 1px solid var(--border-subtle);">
                                <div style="font-size: 11px; color: var(--text-muted);">Total Utilisateurs</div>
                                <div style="font-size: 20px; font-weight: 800; color: white; margin-top: 2px;">${overview.total_users}</div>
                                <div style="font-size: 11px; color: var(--status-success); margin-top: 2px;">🟢 ${overview.healthy_users} sains</div>
                            </div>
                            <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 12px; border: 1px solid var(--border-subtle);">
                                <div style="font-size: 11px; color: var(--text-muted);">Signalements / Bannis</div>
                                <div style="font-size: 20px; font-weight: 800; color: #fbbf24; margin-top: 2px;">${overview.reported_users} <span style="font-size: 14px; color: var(--status-danger);">/ ${overview.banned_users} bannis</span></div>
                                <div style="font-size: 11px; color: var(--text-muted); margin-top: 2px;">${overview.reports.length} plaintes totales</div>
                            </div>
                            <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 12px; border: 1px solid var(--border-subtle); grid-column: span 2; display: flex; justify-content: space-between; align-items: center;">
                                <div>
                                    <div style="font-size: 11px; color: var(--text-muted);">Satisfaction & Avis de l'App</div>
                                    <div style="font-size: 18px; font-weight: 800; color: #fbbf24; margin-top: 2px;">
                                        ⭐ ${overview.average_rating.toFixed(1)} / 5.0
                                    </div>
                                </div>
                                <div style="font-size: 12px; color: var(--text-muted); text-align: right;">
                                    <strong>${overview.total_feedbacks}</strong> retours reçus
                                </div>
                            </div>
                        </div>

                        <!-- Tab Navigation -->
                        <div style="display: flex; background: var(--bg-surface); border-radius: var(--radius-md); padding: 4px; border: 1px solid var(--border-subtle); margin-bottom: 16px; gap: 4px;">
                            <button class="btn-secondary" style="flex: 1; padding: 8px 4px; font-size: 11px; font-weight: 600; ${activeTab === 'users' ? 'background: #d97706; color: white; border-color: #f59e0b;' : 'border-color: transparent;'}" data-action="setAdminTab" data-tab="users">👥 Comptes</button>
                            <button class="btn-secondary" style="flex: 1; padding: 8px 4px; font-size: 11px; font-weight: 600; ${activeTab === 'reports' ? 'background: #d97706; color: white; border-color: #f59e0b;' : 'border-color: transparent;'}" data-action="setAdminTab" data-tab="reports">⚠️ Signalements (${overview.reports.length})</button>
                            <button class="btn-secondary" style="flex: 1; padding: 8px 4px; font-size: 11px; font-weight: 600; ${activeTab === 'feedback' ? 'background: #d97706; color: white; border-color: #f59e0b;' : 'border-color: transparent;'}" data-action="setAdminTab" data-tab="feedback">⭐ Avis (${overview.feedbacks.length})</button>
                            <button class="btn-secondary" style="flex: 1; padding: 8px 4px; font-size: 11px; font-weight: 600; ${activeTab === 'settings' ? 'background: #d97706; color: white; border-color: #f59e0b;' : 'border-color: transparent;'}" data-action="setAdminTab" data-tab="settings">⚙️ Seuil</button>
                        </div>

                        <!-- Tab 1: Users List -->
                        ${activeTab === 'users' ? `
                            <div style="margin-bottom: 12px; display: flex; gap: 6px; flex-wrap: wrap;">
                                <button class="btn-secondary" style="padding: 4px 10px; font-size: 11px; ${filterStatus === 'all' ? 'background: var(--bg-elevated); color: white;' : 'color: var(--text-muted);'}" data-action="setAdminFilter" data-filter="all">Tous (${overview.users.length})</button>
                                <button class="btn-secondary" style="padding: 4px 10px; font-size: 11px; ${filterStatus === 'healthy' ? 'background: var(--bg-elevated); color: var(--status-success);' : 'color: var(--text-muted);'}" data-action="setAdminFilter" data-filter="healthy">🟢 Sains (${overview.healthy_users})</button>
                                <button class="btn-secondary" style="padding: 4px 10px; font-size: 11px; ${filterStatus === 'reported' ? 'background: var(--bg-elevated); color: #fbbf24;' : 'color: var(--text-muted);'}" data-action="setAdminFilter" data-filter="reported">🟡 Signalés (${overview.reported_users})</button>
                                <button class="btn-secondary" style="padding: 4px 10px; font-size: 11px; ${filterStatus === 'banned' ? 'background: var(--bg-elevated); color: var(--status-danger);' : 'color: var(--text-muted);'}" data-action="setAdminFilter" data-filter="banned">🔴 Bannis (${overview.banned_users})</button>
                            </div>

                            <div style="display: flex; flex-direction: column; gap: 8px;">
                                ${filteredUsers.length > 0 ? filteredUsers.map(u => `
                                    <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 12px; border: 1px solid ${u.status === 'banned' ? 'rgba(239, 68, 68, 0.3)' : (u.status === 'reported' ? 'rgba(245, 158, 11, 0.3)' : 'var(--border-subtle)')};">
                                        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
                                            <div style="display: flex; align-items: center; gap: 10px;">
                                                <div class="avatar" style="width: 38px; height: 38px; font-size: 15px; background: var(--bg-elevated);">
                                                    ${u.avatar_data_url ? `<img src="${u.avatar_data_url}" style="width: 100%; height: 100%; object-fit: cover;">` : escapeHtml(u.display_name.charAt(0) || '?')}
                                                </div>
                                                <div>
                                                    <div style="font-size: 14px; font-weight: 700; color: white;">${escapeHtml(u.display_name)}</div>
                                                    <div style="font-size: 11px; color: var(--accent-purple-light);">@${escapeHtml(u.username)}</div>
                                                </div>
                                            </div>
                                            <div style="text-align: right;">
                                                <span style="font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: var(--radius-full); ${u.status === 'banned' ? 'background: rgba(239, 68, 68, 0.2); color: var(--status-danger);' : (u.status === 'reported' ? 'background: rgba(245, 158, 11, 0.2); color: #fbbf24;' : 'background: rgba(34, 197, 94, 0.15); color: var(--status-success);')}">
                                                    ${u.status === 'banned' ? 'Banni' : (u.status === 'reported' ? `Signalé (${u.report_count})` : 'Sain')}
                                                </span>
                                                <div style="font-size: 10px; color: var(--text-muted); margin-top: 3px;">${u.is_online ? '🟢 En ligne' : '⚪ Hors ligne'}</div>
                                            </div>
                                        </div>
                                        <div style="font-size: 10px; font-family: monospace; color: var(--text-dim); word-break: break-all; margin-bottom: 8px; background: var(--bg-elevated); padding: 4px 8px; border-radius: 4px;">
                                            ${escapeHtml(u.peer_id)}
                                        </div>
                                        <div style="display: flex; gap: 8px; justify-content: flex-end;">
                                            ${u.status === 'banned' ? `
                                                <button class="btn-primary" style="font-size: 11px; padding: 5px 12px; background: var(--status-success); border-color: var(--status-success);" data-action="adminUnbanUser" data-peer-id="${escapeHtml(u.peer_id)}">Débloquer</button>
                                            ` : `
                                                <button class="btn-secondary" style="font-size: 11px; padding: 5px 12px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.3);" data-action="adminBanUser" data-peer-id="${escapeHtml(u.peer_id)}">Bannir</button>
                                            `}
                                        </div>
                                    </div>
                                `).join('') : `<div style="text-align: center; color: var(--text-muted); padding: 30px;">Aucun utilisateur dans cette catégorie.</div>`}
                            </div>
                        ` : ''}

                        <!-- Tab 2: Reports List -->
                        ${activeTab === 'reports' ? `
                            <div style="display: flex; flex-direction: column; gap: 10px;">
                                ${overview.reports.length > 0 ? overview.reports.map(r => `
                                    <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 14px; border: 1px solid rgba(245, 158, 11, 0.25);">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                                            <span style="font-size: 11px; font-weight: 700; background: rgba(245, 158, 11, 0.15); color: #fbbf24; padding: 2px 8px; border-radius: 4px; text-transform: uppercase;">${escapeHtml(r.category)}</span>
                                            <span style="font-size: 11px; color: var(--text-muted);">${new Date(r.timestamp_utc * 1000).toLocaleString('fr-FR')}</span>
                                        </div>
                                        <div style="font-size: 13px; font-weight: 700; color: white; margin-bottom: 4px;">Motif : ${escapeHtml(r.reason)}</div>
                                        ${r.comment ? `<div style="font-size: 12px; color: var(--text-muted); margin-bottom: 8px; background: var(--bg-elevated); padding: 8px; border-radius: 4px;">« ${escapeHtml(r.comment)} »</div>` : ''}
                                        <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 8px;">
                                            Cible : <span style="font-family: monospace; color: var(--accent-purple-light);">${escapeHtml(r.target_peer_id.slice(0, 16))}...</span><br>
                                            Signaleur : <span style="font-family: monospace;">${escapeHtml(r.reporter_peer_id.slice(0, 16))}...</span>
                                        </div>
                                        <button class="btn-secondary" style="width: 100%; font-size: 11px; color: var(--status-danger); border-color: rgba(239, 68, 68, 0.3);" data-action="adminBanUser" data-peer-id="${escapeHtml(r.target_peer_id)}">Bannir cet utilisateur cible</button>
                                    </div>
                                `).join('') : `<div style="text-align: center; color: var(--text-muted); padding: 40px;">Aucun signalement enregistré. La communauté est saine !</div>`}
                            </div>
                        ` : ''}

                        <!-- Tab 3: Feedback List -->
                        ${activeTab === 'feedback' ? `
                            <div style="display: flex; flex-direction: column; gap: 10px;">
                                ${overview.feedbacks.length > 0 ? overview.feedbacks.map(f => `
                                    <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 14px; border: 1px solid var(--border-subtle);">
                                        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                            <div style="font-size: 16px; color: #fbbf24;">${'★'.repeat(f.rating)}${'☆'.repeat(5 - f.rating)}</div>
                                            <span style="font-size: 11px; color: var(--text-muted);">${new Date(f.timestamp_utc * 1000).toLocaleDateString('fr-FR')}</span>
                                        </div>
                                        <div style="font-size: 11px; color: var(--accent-purple-light); font-weight: 600; text-transform: uppercase; margin-bottom: 4px;">${escapeHtml(f.category)}</div>
                                        <div style="font-size: 13px; color: white; line-height: 1.4;">${escapeHtml(f.comment) || '<em style="color: var(--text-dim);">Aucun commentaire textuel</em>'}</div>
                                    </div>
                                `).join('') : `<div style="text-align: center; color: var(--text-muted); padding: 40px;">Aucun avis utilisateur pour l'instant.</div>`}
                            </div>
                        ` : ''}

                        <!-- Tab 4: Auto-Quarantine Settings -->
                        ${activeTab === 'settings' ? `
                            <div style="background: var(--bg-surface); border-radius: var(--radius-md); padding: 16px; border: 1px solid var(--border-subtle);">
                                <div style="font-size: 14px; font-weight: 700; color: white; margin-bottom: 6px;">Seuil de mise en quarantaine automatique</div>
                                <p style="font-size: 12px; color: var(--text-muted); line-height: 1.4; margin-bottom: 16px;">
                                    Définit le nombre de signalements distincts requis pour qu'un compte soit automatiquement suspendu du répertoire et des relais sans validation humaine préalable.
                                </p>
                                <div style="display: flex; align-items: center; gap: 12px; margin-bottom: 16px;">
                                    <input type="number" id="admin-threshold-input" min="1" max="50" value="${overview.auto_ban_threshold}" style="width: 80px; background: var(--bg-elevated); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); padding: 8px 12px; color: white; font-size: 15px; font-weight: 700; text-align: center; outline: none;">
                                    <span style="font-size: 13px; color: var(--text-muted);">signalements distincts</span>
                                </div>
                                <button class="btn-primary" style="width: 100%; background: #d97706; border-color: #f59e0b;" data-action="adminSaveThreshold">Enregistrer le seuil</button>
                            </div>
                        ` : ''}
                    ` : `
                        <div style="text-align: center; padding: 40px 20px; color: var(--text-muted);">
                            <div style="font-size: 32px; margin-bottom: 8px;">🔐</div>
                            <div style="font-size: 15px; font-weight: 700; color: white; margin-bottom: 6px;">Authentification requise</div>
                            <p style="font-size: 12px; max-width: 300px; margin: 0 auto 16px;">Entrez votre jeton secret d'administration pour charger les données de supervision.</p>
                        </div>
                    `}
                </div>
            </div>
        `;
    }
};

// --- CONTROLLER & NAVIGATION ---
// Every screen except these three requires a created/restored identity (state.currentUser.peerId).
// Without this gate, a device with no account yet could still reach conversations/contacts/
// settings/etc. — all rendering empty or non-functional since there is no peerId to act as —
// instead of being told to create an account first.
const PRE_AUTH_SCREENS = new Set(['onboarding', 'create_account', 'restore_account']);
let conversationsPollInterval = null;
// Set while replaying a browser/Android-back navigation (see the `popstate` listener below) so
// navigateTo() doesn't push a *new* history entry on top of the one the back action just landed
// on — that would turn one back-press into a no-op (pop one, push one right back).
let suppressHistoryPush = false;
// Explicit in-memory screen stack for reliable back navigation (sub-screens -> root)
const navStack = [];
let lastBackPressTime = 0;

async function navigateTo(screenKey) {
    closeQrCameraScanner();
    closeMediaPreviewModal();
    if (document.activeElement && typeof document.activeElement.blur === 'function') {
        document.activeElement.blur();
    }
    if (!screens[screenKey]) return;
    if (!state.currentUser.peerId && !PRE_AUTH_SCREENS.has(screenKey)) {
        if (state.currentScreen !== 'onboarding') {
            alert('Vous devez d\'abord créer votre compte (ou en restaurer un) pour accéder à cette fonctionnalité.');
        }
        screenKey = 'onboarding';
    }
    state.currentScreen = screenKey;

    // Refresh from the real backend *before* rendering, for screens whose content it owns —
    // otherwise the screen would render last-known (possibly stale) local state first.
    if (hasBackend) {
        if (screenKey === 'conversations') {
            await refreshConversationsFromBackend();
            await refreshContactsFromBackend();
        } else if (screenKey === 'contacts') {
            await refreshContactsFromBackend();
            tauriInvoke('publish_directory_profile').catch(() => {});
        } else if (screenKey === 'add_contact') {
            tauriInvoke('publish_directory_profile').catch(() => {});
        } else if (screenKey === 'chat' && state.activeContact) {
            await refreshMessagesFromBackend(state.activeContact.conversationId);
            await refreshDiagnostics(state.activeContact.peerId);
            // Historical attachments (photos, videos, voice notes) sent/received before this
            // chat was opened only have an attachmentId, not a fetched url — kick off loading
            // for all of them now, each re-rendering its own bubble in place once it arrives.
            state.messages
                .filter(m => m.conversationId === state.activeContact.conversationId && m.attachmentId && !m.url)
                .forEach(ensureAttachmentLoaded);
        } else if (screenKey === 'contact_profile' && state.activeContact) {
            await refreshDiagnostics(state.activeContact.peerId);
        } else if (screenKey === 'shared_media' && state.activeContact) {
            // Reachable directly (e.g. a deep link) without the chat screen having populated
            // state.messages for this conversation first.
            await refreshMessagesFromBackend(state.activeContact.conversationId);
        } else if (screenKey === 'identity') {
            await refreshOwnBundleHex();
            tauriInvoke('publish_directory_profile').catch(() => {});
        } else if (screenKey === 'settings') {
            await refreshMediaFolderPath();
        } else if (screenKey === 'create_group') {
            await refreshContactsFromBackend();
            state.selectedGroupMemberIds = new Set();
        } else if (screenKey === 'group_info' && state.activeContact) {
            await refreshGroupMembersInfo(state.activeContact.peerId);
        } else if (screenKey === 'admin_dashboard') {
            if (state.adminState.token && !state.adminState.overview) {
                await loadAdminOverviewReal();
            }
        }
    }

    const container = document.getElementById('screen-container');
    container.innerHTML = screens[screenKey]();

    if (screenKey === 'identity') {
        renderOwnQrCode();
    }

    // On mobile webviews, dynamically injected inputs with 'autofocus' often don't trigger the virtual keyboard.
    // Explicitly focusing after DOM injection ensures keyboard and text cursor appear reliably.
    setTimeout(() => {
        const autofocusInput = container.querySelector('input[autofocus]');
        if (autofocusInput) {
            autofocusInput.focus();
        }
    }, 100);

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

    // Auto scroll chat, and poll for incoming messages + live connection diagnostics
    if (screenKey === 'chat') {
        if (messagePollInterval) clearInterval(messagePollInterval);
        if (hasBackend && state.activeContact) {
            const conversationId = state.activeContact.conversationId;
            const peerId = state.activeContact.peerId;
            messagePollInterval = setInterval(async () => {
                try {
                    await tauriInvoke('drain_relay');
                } catch (_) {}
                const newOnes = await refreshMessagesFromBackend(conversationId);
                await refreshDiagnostics(peerId);
                // Only touch the DOM if this conversation is still the one open
                if (state.currentScreen === 'chat' && state.activeContact && state.activeContact.conversationId === conversationId) {
                    if (newOnes.length > 0) {
                        const chatBody = document.getElementById('chat-body');
                        const isNearBottom = chatBody ? (chatBody.scrollHeight - chatBody.scrollTop - chatBody.clientHeight < 140) : true;
                        newOnes.forEach(appendChatMessageToBody);
                        newOnes.filter(m => m.attachmentId && !m.url).forEach(ensureAttachmentLoaded);
                        if (chatBody && isNearBottom) {
                            chatBody.scrollTo({ top: chatBody.scrollHeight, behavior: 'smooth' });
                        }
                    }
                    const statusEl = document.getElementById('chat-header-status');
                    if (statusEl) statusEl.innerHTML = chatHeaderStatusHtml();
                }
            }, 800);
        }

        setTimeout(() => {
            const body = document.getElementById('chat-body');
            if (body) {
                body.scrollTop = body.scrollHeight;
                // Automatically dismiss keyboard when touching / scrolling message history
                body.addEventListener('touchstart', () => {
                    const input = document.getElementById('chat-input');
                    if (input && document.activeElement === input) {
                        input.blur();
                    }
                }, { passive: true });
            }
        }, 100);
    } else if (messagePollInterval) {
        clearInterval(messagePollInterval);
        messagePollInterval = null;
    }

    // Keep the conversations list (unread badges, last-message previews) live without destructive re-renders
    if (screenKey === 'conversations') {
        if (conversationsPollInterval) clearInterval(conversationsPollInterval);
        if (hasBackend) {
            conversationsPollInterval = setInterval(async () => {
                try {
                    await tauriInvoke('drain_relay');
                } catch (_) {}
                await refreshConversationsFromBackend();
                updateGlobalUnreadBadges();
                if (state.currentScreen === 'conversations') {
                    updateConversationsListDom();
                }
            }, 1500);
        }
    } else if (conversationsPollInterval) {
        clearInterval(conversationsPollInterval);
        conversationsPollInterval = null;
    }

    // Record this screen in the browser/WebView history so the Android hardware/gesture back
    // button has something to go back TO. Without this, the WebView's history stays completely
    // empty (this SPA never used to call pushState), so Tauri's default Android back-button
    // handling (go back in WebView history, else close the app) had nothing to go back to and
    // closed the entire app from any screen — see the `popstate` listener below for the other
    // half of this fix.
    // Maintain in-memory navigation stack for safe back navigation
    if (!suppressHistoryPush) {
        if (navStack.length === 0 || navStack[navStack.length - 1] !== screenKey) {
            navStack.push(screenKey);
        }
        if (history.state && history.state.screen === screenKey) {
            // Re-render of the same screen (e.g. a poll refresh) — nothing to add to history.
        } else if (history.state && history.state.screen) {
            history.pushState({ screen: screenKey }, '', '#' + screenKey);
        } else {
            // First navigation since page load: replace rather than push
            history.replaceState({ screen: screenKey }, '', '#' + screenKey);
        }
    }
}

// All possible modals, overlays and drawers present across the app
const ALL_MODAL_IDS = [
    'qr-camera-modal',
    'media-preview-modal',
    'location-modal',
    'location-action-modal',
    'mnemonic-auth-modal',
    'mnemonic-display-modal',
    'image-preview-modal',
    'edit-profile-modal',
    'inspect-user-modal',
    'block-contact-modal',
    'report-modal',
    'feedback-modal',
    'setup-pin-modal',
    'message-actions-modal',
    'forward-message-modal',
    'ephemeral-timer-modal',
    'document-viewer-modal',
    'video-player-modal',
    'executable-warning-modal',
    'incoming-call-modal',
];

// True while any modal/drawer/sheet is actually visible in the DOM
function isAnyOverlayOpen() {
    if (ALL_MODAL_IDS.some(id => {
        const el = document.getElementById(id);
        return el && el.classList.contains('show');
    })) return true;
    const drawer = document.getElementById('attachment-drawer');
    const emojiPanel = document.getElementById('emoji-picker-panel');
    return !!((drawer && drawer.classList.contains('show')) || (emojiPanel && emojiPanel.classList.contains('show')));
}

// Closes any open modal/drawer/sheet cleanly
function closeAllActiveOverlays() {
    closeQrCameraScanner();
    closeMediaPreviewModal();
    closeLocationModal();
    closeMnemonicAuthModal();
    closeMnemonicDisplayModal();
    closeImagePreview();
    closeEditProfileModal();
    closeInspectUserModal();
    closeBlockModal();
    closeReportModal();
    closeFeedbackModal();
    closeSetupPinModal();
    closeMessageActionsModal();
    closeForwardMessageModal();
    closeEphemeralTimerModal();
    closeDocumentViewer();
    closeVideoPlayerModal();
    closeExecutableWarningModal();
    closePanels();
    const incomingCallModal = document.getElementById('incoming-call-modal');
    if (incomingCallModal && incomingCallModal.classList.contains('show')) {
        rejectIncomingCallReal();
    }
}

// Safely navigates backwards in the application hierarchy.
// Returns true if the back action was consumed inside the app (modal closed or screen changed),
// or false if the app is at the root screen and exit is requested.
function navigateBackSafely() {
    // 1. If any modal, sheet or drawer is open, dismiss it first
    if (isAnyOverlayOpen()) {
        closeAllActiveOverlays();
        return true;
    }

    // 2. Active call in progress: prompt or hangup
    const activeCallModal = document.getElementById('active-call-modal');
    if (activeCallModal && activeCallModal.classList.contains('show')) {
        hangupCall(true, 'Fin de l\'appel');
        return true;
    }

    // 3. Sub-screens: navigate back to previous screen
    const rootScreen = state.currentUser.peerId ? 'conversations' : 'onboarding';
    if (state.currentScreen !== rootScreen) {
        // Pop current screen from nav stack
        if (navStack.length > 0 && navStack[navStack.length - 1] === state.currentScreen) {
            navStack.pop();
        }
        const prevScreen = navStack.length > 0 ? navStack.pop() : rootScreen;
        suppressHistoryPush = true;
        navigateTo(prevScreen).finally(() => {
            suppressHistoryPush = false;
        });
        return true;
    }

    // 4. Already at root screen ('conversations' or 'onboarding')
    const now = Date.now();
    if (now - lastBackPressTime < 2000) {
        // Double-tap confirmed: allow app to close
        return false;
    }
    lastBackPressTime = now;
    showToastNotification('Appuyez à nouveau pour quitter NOVA');
    return true;
}

// Bridge function invoked by Android native layer (MainActivity.kt)
window.handleAndroidBack = function() {
    return navigateBackSafely();
};
window.handleBackNavigation = window.handleAndroidBack;

// Handles browser/WebView popstate events
let handlingPopState = false;
window.addEventListener('popstate', (event) => {
    if (handlingPopState) return;
    if (isAnyOverlayOpen()) {
        closeAllActiveOverlays();
        history.pushState({ screen: state.currentScreen }, '', '#' + state.currentScreen);
        return;
    }

    handlingPopState = true;
    const targetScreen = (event.state && event.state.screen) || (state.currentUser.peerId ? 'conversations' : 'onboarding');
    suppressHistoryPush = true;
    navigateTo(targetScreen).finally(() => {
        suppressHistoryPush = false;
        handlingPopState = false;
    });
});

// Applies the identity nova-engine just created/restored to local UI state.
function applyAccountInfo(name, peerId, mnemonic, networkActive, bio, avatarDataUrl) {
    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '');
    state.currentUser.name = name;
    state.currentUser.username = slug;
    state.currentUser.handle = slug + '.nova';
    state.currentUser.peerId = peerId;
    state.currentUser.publicKey = peerId;
    state.currentUser.mnemonic = mnemonic || '';
    if (typeof bio === 'string') state.currentUser.bio = bio;
    if (typeof avatarDataUrl === 'string' || avatarDataUrl === null) state.currentUser.avatarDataUrl = avatarDataUrl;
    state.currentUser.status = networkActive
        ? 'Connecté au réseau NOVA'
        : 'Compte prêt — pas de connexion réseau pour l\'instant';
}

let pendingAvatarDataUrl = null;

function openEditProfileModal() {
    const modal = document.getElementById('edit-profile-modal');
    if (!modal) return;
    pendingAvatarDataUrl = state.currentUser.avatarDataUrl || null;
    modal.classList.add('show');
}

function closeEditProfileModal() {
    const modal = document.getElementById('edit-profile-modal');
    if (modal) modal.classList.remove('show');
    pendingAvatarDataUrl = null;
}

function handleAvatarFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
        const img = new Image();
        img.onload = () => {
            const canvas = document.createElement('canvas');
            const maxDim = 256;
            let width = img.width;
            let height = img.height;
            if (width > height) {
                if (width > maxDim) {
                    height = Math.round((height * maxDim) / width);
                    width = maxDim;
                }
            } else {
                if (height > maxDim) {
                    width = Math.round((width * maxDim) / height);
                    height = maxDim;
                }
            }
            canvas.width = width;
            canvas.height = height;
            const ctx = canvas.getContext('2d');
            ctx.drawImage(img, 0, 0, width, height);
            const dataUrl = canvas.toDataURL('image/jpeg', 0.85);
            pendingAvatarDataUrl = dataUrl;

            const preview = document.getElementById('edit-profile-avatar-preview');
            if (preview) {
                preview.innerHTML = `<img src="${dataUrl}" style="width: 100%; height: 100%; object-fit: cover;">`;
            }
        };
        img.src = e.target.result;
    };
    reader.readAsDataURL(file);
}

async function saveProfileChanges() {
    const nameInput = document.getElementById('edit-display-name-input');
    const bioInput = document.getElementById('edit-bio-input');
    const newName = (nameInput && nameInput.value.trim()) || state.currentUser.name;
    const newBio = (bioInput && bioInput.value.trim()) || '';

    if (!newName) {
        alert('Entrez un nom pour votre profil.');
        return;
    }

    try {
        if (hasBackend) {
            await tauriInvoke('update_user_profile', {
                displayName: newName,
                bio: newBio,
                avatarDataUrl: pendingAvatarDataUrl,
            });
        }
        state.currentUser.name = newName;
        state.currentUser.bio = newBio;
        state.currentUser.avatarDataUrl = pendingAvatarDataUrl;

        // Synchronisation immédiate vers Vercel
        syncProfileToVercel(state.currentUser.peerId, newName, newName, state.currentUser.bundleHex, pendingAvatarDataUrl);

        closeEditProfileModal();
        if (state.currentScreen === 'settings') {
            navigateTo('settings');
        }
    } catch (e) {
        alert('Échec de la sauvegarde du profil : ' + e);
    }
}

async function createAccountReal() {
    if (!requireBackend()) return;
    const input = document.getElementById('account-name-input');
    const name = (input && input.value.trim()) || '';
    if (!name) {
        alert('Entrez un nom pour continuer.');
        return;
    }
    try {
        const info = await tauriInvoke('create_account', { username: name });
        applyAccountInfo(name, info.peer_id, info.mnemonic, info.network_active);
        
        // Démarrage instantané du canal Vercel prioritaire
        sendPresenceHeartbeat('online');
        refreshContactsPresence();
        startVercelSignalPolling();
        syncProfileToVercel(info.peer_id, name, name, state.currentUser.bundleHex, null);

        const networkNote = info.network_active
            ? ''
            : '\n\n(Pas de connexion réseau détectée pour l\'instant — votre compte est bien créé et sauvegardé, l\'app réessaiera de se connecter automatiquement.)';
        showMnemonicDisplayModal(info.mnemonic, async () => {
            await refreshConversationsFromBackend();
            navigateTo('conversations');
        });
    } catch (e) {
        alert(String(e).includes('already exists')
            ? 'Vous avez déjà un compte sur cet appareil. Pour en créer un nouveau, supprimez d\'abord l\'actuel (Réglages → Mon identité).'
            : 'Impossible de créer le compte pour le moment : ' + e);
    }
}

async function restoreAccountReal() {
    if (!requireBackend()) return;
    const nameInput = document.getElementById('restore-name-input');
    const mnemonicInput = document.getElementById('restore-mnemonic-input');
    const name = (nameInput && nameInput.value.trim()) || '';
    const mnemonic = (mnemonicInput && mnemonicInput.value.trim()) || '';
    if (!name) {
        alert('Entrez un nom pour continuer.');
        return;
    }
    if (mnemonic.split(/\s+/).filter(Boolean).length !== 12) {
        alert('Vérifiez votre phrase secrète : il doit y avoir exactement 12 mots, séparés par des espaces.');
        return;
    }
    try {
        const info = await tauriInvoke('restore_account', { mnemonic, username: name });
        applyAccountInfo(name, info.peer_id, mnemonic, info.network_active);
        
        // Démarrage instantané du canal Vercel prioritaire
        sendPresenceHeartbeat('online');
        refreshContactsPresence();
        startVercelSignalPolling();
        syncProfileToVercel(info.peer_id, name, name, state.currentUser.bundleHex, null);

        if (!info.network_active) {
            alert('Compte retrouvé ! Pas de connexion réseau détectée pour l\'instant — l\'app réessaiera de se connecter automatiquement.');
        }
        await refreshConversationsFromBackend();
        navigateTo('conversations');
    } catch (e) {
        alert(String(e).includes('already exists')
            ? 'Vous avez déjà un compte sur cet appareil. Pour en retrouver un autre, supprimez d\'abord l\'actuel (Réglages → Mon identité).'
            : 'Impossible de retrouver ce compte — vérifiez que les 12 mots sont corrects. ' + e);
    }
}

// Logs this device out (see the Rust-side `logout` command / `NovaEngine::clear_identity`):
// there is no server account to log back into — identity is device-bound — so this wipes the
// local identity, contacts, conversations and messages rather than merely hiding the screen.
// Irreversible without the mnemonic, hence the confirmation.
async function logoutReal() {
    if (!requireBackend()) return;
    if (!confirm('Se déconnecter effacera définitivement l\'identité, les contacts et les messages de cet appareil (il n\'y a pas de compte distant à récupérer — seule votre phrase de récupération, si vous l\'avez notée, permet de revenir). Continuer ?')) {
        return;
    }
    try {
        await tauriInvoke('logout');
    } catch (e) {
        alert('Échec de la déconnexion : ' + e);
        return;
    }
    state.currentUser = {
        name: '', username: '', handle: '', peerId: '', bio: '',
        status: 'Compte non créé', publicKey: '', mnemonic: '', bundleHex: '', linkGenerationError: '',
    };
    try {
        localStorage.removeItem('nova_pin_hash');
        localStorage.removeItem('nova_pin_salt');
    } catch (_) {}
    state.activeContact = null;
    state.currentDiagnostics = null;
    state.conversations = [];
    state.messages = [];
    state.contacts = [];
    updateGlobalUnreadBadges();
    navigateTo('onboarding');
}

function openChatWith(name, handle, conversationId) {
    const isGroup = (conversationId && conversationId.startsWith('group_')) || (handle && handle.startsWith('group_'));
    const actualConvId = conversationId || (isGroup ? (handle.startsWith('group_') ? handle : 'group_' + handle) : 'conv_' + handle);
    const peerId = isGroup ? actualConvId.replace(/^group_/, '') : handle;

    if (isGroup) {
        state.activeContact = {
            name: name || 'Groupe',
            handle: peerId,
            peerId: peerId,
            conversationId: actualConvId,
            isGroup: true,
            publicKey: 'Chiffré E2EE',
            safetyNumber: 'Chiffrement E2EE',
            isOnline: true,
            p2pMode: 'Groupe chiffré',
            isBlocked: false,
            isTrusted: true,
        };
        const contact = state.contacts.find(c => c.name === name || c.handle === handle || (c.peerId && c.peerId === handle) || (c.peerId && c.peerId === peerId) || (c.handle && c.handle === peerId));
        const resolvedPeerId = contact ? (contact.handle || contact.peerId) : peerId;
        state.activeContact = {
            name: contact ? contact.name : name,
            handle: resolvedPeerId,
            peerId: resolvedPeerId,
            conversationId: actualConvId,
            isGroup: false,
            isContact: !!contact,
            publicKey: (contact && contact.key) || 'Non disponible',
            safetyNumber: (contact && contact.safetyNumber) || 'Non disponible',
            isOnline: !!(contact && contact.online),
            p2pMode: (contact && contact.p2pMode) || 'Relais serveur',
            isBlocked: !!(contact && contact.isBlocked),
            isTrusted: !!(contact && contact.isTrusted),
        };
    }
    state.currentDiagnostics = null;

    // 1. Clear unread notification badge on this conversation
    const conv = state.conversations.find(c => c.name === name || c.handle === handle);
    if (conv) {
        conv.unread = 0;
    }

    // 2. Mark this conversation's messages as read (not other conversations')
    state.messages.forEach(m => {
        if (m.conversationId === state.activeContact.conversationId) {
            m.status = 'read';
        }
    });

    // 3. Update global navigation tab badges
    updateGlobalUnreadBadges();

    navigateTo('chat');
}

// Real per-peer connection state from nova-transport's TransportSupervisor — populated only by
// an actual connection attempt (dial or send), never fabricated. `null` means "no attempt yet",
// rendered as an honest "unknown" state rather than defaulted to looking connected or offline.
async function refreshDiagnostics(peerId) {
    if (!hasBackend || !peerId) {
        state.currentDiagnostics = null;
        return;
    }
    try {
        state.currentDiagnostics = await tauriInvoke('get_diagnostics', { peerId });
    } catch (e) {
        console.error('refreshDiagnostics failed', e);
        state.currentDiagnostics = null;
    }
}

// Plain-language connection description — shown on the contact profile, never in the chat
// header itself (that just shows a simple online/offline dot, see chatHeaderStatusHtml).
function transportModeLabel(mode) {
    switch (mode) {
        case 'DirectQuic': return 'connexion directe';
        case 'RelayedOpaque': return 'connexion via un relais';
        case 'Disconnected': return 'non connecté';
        default: return 'statut inconnu';
    }
}

function chatHeaderStatusHtml() {
    if (!state.activeContact) return '';
    if (state.activeContact.isGroup) {
        return `<span style="color: var(--accent-purple-light); font-size: 11px;">Groupe chiffré</span>`;
    }
    if (state.activeContact.online) {
        return `<span style="color: var(--status-success); font-size: 11px; display: inline-flex; align-items: center; gap: 4px;"><span style="width: 7px; height: 7px; border-radius: 50%; background: var(--status-success); display: inline-block;"></span> En ligne</span>`;
    }
    return `<span style="color: var(--text-muted); font-size: 11px; display: inline-flex; align-items: center; gap: 4px;"><span style="width: 7px; height: 7px; border-radius: 50%; background: var(--text-dim); display: inline-block;"></span> Hors ligne</span>`;
}

function contactConnectionStatusText() {
    if (!state.activeContact) return 'Hors ligne';
    if (state.activeContact.online) return 'En ligne • Chiffré';
    return 'Hors ligne • Vu récemment';
}

// --- LIVE PRESENCE & NETWORK CONNECTIVITY (Vercel Edge & Neon Postgres) ---

async function refreshContactsPresence() {
    if (!state.contacts || state.contacts.length === 0) return;
    const peerIds = state.contacts.map(c => c.handle || c.peerId).filter(Boolean);
    if (peerIds.length === 0) return;

    try {
        const resp = await fetch(`${VERCEL_API_BASE_URL}/api/presence?peer_ids=${encodeURIComponent(peerIds.join(','))}`);
        if (resp.ok) {
            const data = await resp.json();
            if (data && data.presence) {
                let updated = false;
                for (const c of state.contacts) {
                    const id = c.handle || c.peerId;
                    if (data.presence[id] !== undefined) {
                        const newOnline = !!data.presence[id];
                        if (c.online !== newOnline) {
                            c.online = newOnline;
                            updated = true;
                        }
                    }
                }
                if (state.activeContact) {
                    const activeId = state.activeContact.handle || state.activeContact.peerId;
                    if (data.presence[activeId] !== undefined) {
                        state.activeContact.online = !!data.presence[activeId];
                        const headerStatus = document.getElementById('chat-header-status');
                        if (headerStatus) headerStatus.innerHTML = chatHeaderStatusHtml();
                    }
                }
                if (updated && state.currentScreen === 'contacts') {
                    updateContactsListStatusDots();
                }
            }
        }
    } catch (_) {}
}

function updateContactsListStatusDots() {
    const cards = document.querySelectorAll('#contacts-list-container .item-card');
    cards.forEach(card => {
        const handle = card.dataset.handle;
        if (!handle) return;
        const c = state.contacts.find(item => item.handle === handle || item.peerId === handle);
        if (c) {
            const dot = card.querySelector('.status-dot');
            if (dot) {
                dot.className = `status-dot ${c.online ? 'status-online' : 'status-offline'}`;
            }
        }
    });
}

async function checkServerConnectivity() {
    try {
        const ctrl = new AbortController();
        const timeoutId = setTimeout(() => ctrl.abort(), 4000);
        const resp = await fetch(`${VERCEL_API_BASE_URL}/api/health`, { signal: ctrl.signal });
        clearTimeout(timeoutId);
        state.isServerConnected = resp.ok;
    } catch (_) {
        state.isServerConnected = false;
    }
    updateConnectionIndicators();
}

async function sendPresenceHeartbeat(status = 'online') {
    if (!state.currentUser.peerId) return;
    try {
        const ctrl = new AbortController();
        const timeoutId = setTimeout(() => ctrl.abort(), 4000);
        const resp = await fetch(`${VERCEL_API_BASE_URL}/api/presence`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                peer_id: state.currentUser.peerId,
                status: status
            }),
            signal: ctrl.signal
        });
        clearTimeout(timeoutId);
        if (resp.ok) {
            state.isServerConnected = true;
        } else {
            state.isServerConnected = false;
        }
    } catch (_) {
        state.isServerConnected = false;
    }
    updateConnectionIndicators();
}

let vercelSignalPollTimeout = null;

async function pollVercelCallSignals() {
    if (!state.currentUser.peerId) return;
    try {
        const resp = await fetch(`${VERCEL_API_BASE_URL}/api/signal?peer_id=${encodeURIComponent(state.currentUser.peerId)}`, {
            cache: 'no-store'
        });
        if (resp.ok) {
            const data = await resp.json();
            if (data && Array.isArray(data.signals) && data.signals.length > 0) {
                for (const sig of data.signals) {
                    await handleIncomingCallSignal(sig, null, null);
                }
            }
        }
    } catch (_) {}
}

function startVercelSignalPolling() {
    if (vercelSignalPollTimeout) clearTimeout(vercelSignalPollTimeout);
    const getInterval = () => (currentCall || pendingIncomingCall) ? 800 : 1500;

    const tick = async () => {
        if (state.currentUser.peerId) {
            await pollVercelCallSignals();
            vercelSignalPollTimeout = setTimeout(tick, getInterval());
        } else {
            vercelSignalPollTimeout = setTimeout(tick, 3000);
        }
    };
    tick();
}

function updateConnectionIndicators() {
    const offlineBanner = document.getElementById('offline-banner');
    const isOnline = navigator.onLine && state.isServerConnected;
    if (offlineBanner) {
        if (!navigator.onLine) {
            offlineBanner.innerText = "📡 Pas de connexion réseau — Vérifiez votre Wi-Fi ou vos données mobiles.";
            offlineBanner.classList.add('show');
        } else if (!state.isServerConnected) {
            offlineBanner.innerText = "⚠️ Non connecté au serveur — Reconnexion en cours...";
            offlineBanner.classList.add('show');
        } else {
            offlineBanner.classList.remove('show');
        }
    }

    const statusText = isOnline ? 'Connecté' : (!navigator.onLine ? 'Hors ligne' : 'Non connecté / Hors ligne');
    const statusColor = isOnline ? 'var(--status-success)' : '#ef4444';

    ['user-global-status', 'contacts-user-status'].forEach(id => {
        const el = document.getElementById(id);
        if (el) {
            el.innerHTML = `
                <span class="user-status-dot" style="width: 7px; height: 7px; border-radius: 50%; background: ${statusColor};"></span>
                <span style="color: ${statusColor}; font-weight: 500;">${statusText}</span>
            `;
        }
    });

    const badges = document.querySelectorAll('.p2p-badge-pulse');
    badges.forEach(b => {
        b.style.background = isOnline ? 'var(--status-success)' : 'var(--status-danger)';
        b.title = isOnline ? 'Connecté et sécurisé' : 'Non connecté';
    });
}

async function syncProfileToVercel(peerId, username, displayName, bundleHex, avatarDataUrl) {
    if (!peerId) return;
    try {
        await fetch(`${VERCEL_API_BASE_URL}/api/directory`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                peer_id: peerId,
                username: username || displayName || 'utilisateur',
                display_name: displayName || username || 'Utilisateur NOVA',
                avatar_data_url: avatarDataUrl || null,
                prekey_bundle_hex: bundleHex || 'none',
            }),
        });
    } catch (e) {
        console.warn('syncProfileToVercel error:', e);
    }
}

// Fetches this device's real, signature-verifiable X3DH invitation ticket (valid for 24h).
async function refreshOwnBundleHex() {
    if (!hasBackend) return;
    try {
        const uri = await tauriInvoke('get_own_invitation_uri', { ttlSeconds: 86400 });
        state.currentUser.invitationUri = uri;
        state.currentUser.bundleHex = uri;
        state.currentUser.linkGenerationError = '';
    } catch (e) {
        console.error('get_own_invitation_uri failed, fallback to raw prekey bundle', e);
        try {
            state.currentUser.bundleHex = await tauriInvoke('get_own_prekey_bundle_hex');
            state.currentUser.invitationUri = state.currentUser.bundleHex;
            state.currentUser.linkGenerationError = '';
        } catch (e2) {
            console.error('refreshOwnBundleHex fallback failed', e2);
            state.currentUser.bundleHex = '';
            state.currentUser.invitationUri = '';
            state.currentUser.linkGenerationError = String((e2 && e2.message) || e2 || e || 'erreur inconnue');
        }
    }
}

// Re-runs refreshOwnBundleHex() from the "Réessayer" button shown when it previously failed,
// then re-renders the Identity screen so the QR code / link / error state reflect the outcome.
async function retryOwnBundleReal() {
    await refreshOwnBundleHex();
    if (state.currentScreen === 'identity') {
        navigateTo('identity');
    }
}



function formatMessageTime(timestampUtc) {
    return new Date(timestampUtc * 1000).toLocaleTimeString('fr-FR', { hour: '2-digit', minute: '2-digit' });
}

// Replaces the local conversation list with what nova-engine's local (encrypted) storage
// actually has — called before rendering the conversations screen, and after any action that
// changes it (adding a contact, sending a message).
async function refreshConversationsFromBackend() {
    if (!hasBackend) return;
    try {
        const convos = await tauriInvoke('get_conversations');
        state.conversations = convos.map(c => ({
            id: c.id,
            name: c.title,
            handle: c.peer_id,
            lastMsg: c.last_message_text,
            time: c.last_message_time_utc ? formatMessageTime(c.last_message_time_utc) : '',
            unread: c.unread_count,
            online: false,
            mode: 'DHT',
        }));
    } catch (e) {
        console.error('refreshConversationsFromBackend failed', e);
    }
}

function renderConversationsListHtml(query) {
    const q = (query || '').trim().toLowerCase();
    let filtered = state.conversations;
    if (q) {
        filtered = state.conversations.filter(c =>
            (c.name && c.name.toLowerCase().includes(q)) ||
            (c.handle && c.handle.toLowerCase().includes(q)) ||
            (c.lastMsg && c.lastMsg.toLowerCase().includes(q))
        );
    }

    if (filtered.length > 0) {
        return filtered.map(c => {
            const isGroup = (c.id && c.id.startsWith('group_')) || (c.handle && c.handle.startsWith('group_'));
            return `
            <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-conv-id="${escapeHtml(c.id || '')}" data-action="openChat">
                <div class="avatar" style="${isGroup ? 'background: linear-gradient(135deg, #8b5cf6, #3b82f6);' : ''}">
                    ${isGroup ? icons.users : escapeHtml(c.name.charAt(0))}
                    <div class="status-dot ${isGroup ? 'status-online' : (c.online ? 'status-online' : 'status-offline')}"></div>
                </div>
                <div class="item-content">
                    <div class="item-header">
                        <span class="item-name" style="${c.unread > 0 ? 'font-weight: 700; color: white;' : ''}">
                            ${isGroup ? '<span style="color: var(--accent-purple-light); font-size: 11px; margin-right: 4px;">👥</span>' : ''}
                            ${escapeHtml(c.name)}
                        </span>
                        <span class="item-time" style="${c.unread > 0 ? 'color: var(--accent-purple-light); font-weight: 600;' : ''}">${escapeHtml(c.time)}</span>
                    </div>
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 2px;">
                        <span class="item-sub" style="${c.unread > 0 ? 'color: var(--text-main); font-weight: 600;' : ''}">${escapeHtml(c.lastMsg || '')}</span>
                        ${c.unread > 0 ? `<span class="badge-unread">${escapeHtml(String(c.unread))}</span>` : ''}
                    </div>
                </div>
            </div>
        `;
        }).join('');
    }

    if (q) {
        return `
            <div style="text-align: center; color: var(--text-muted); padding: 30px 16px;">
                <div style="font-size: 14px; font-weight: 600; color: white;">Aucun échange correspondant</div>
                <p style="font-size: 12px; color: var(--text-muted); margin: 6px 0 16px;">Aucune conversation locale ne contient « ${escapeHtml(query)} ».</p>
                <div style="display: flex; flex-direction: column; gap: 8px; max-width: 300px; margin: 0 auto;">
                    <button class="btn-primary" style="font-size: 12px; padding: 10px 14px; display: flex; align-items: center; justify-content: center; gap: 6px;" data-action="fillAddContactForm" data-name="${escapeHtml(query)}" data-id="${escapeHtml(query)}">
                        <span>🔍</span>
                        <span>Chercher / Ajouter « ${escapeHtml(query)} »</span>
                    </button>
                    <button class="btn-secondary" style="font-size: 11px; padding: 8px 12px;" data-action="navigate" data-screen="contacts">
                        Voir la liste des contacts
                    </button>
                </div>
            </div>
            <div id="conversations-search-contacts-preview" style="margin-top: 10px;"></div>
        `;
    }

    return `
        <div class="empty-conversations-state">
            <div class="empty-conversations-bubble">
                <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
                    <line x1="8" y1="10" x2="16" y2="10"></line>
                    <line x1="8" y1="14" x2="12" y2="14"></line>
                </svg>
            </div>
            <div style="font-size: 17px; font-weight: 700; color: white; margin-bottom: 8px;">Aucune discussion</div>
            <p style="font-size: 13px; color: var(--text-muted); margin-bottom: 24px; max-width: 290px; line-height: 1.5;">
                Démarrez une conversation chiffrée de bout en bout avec vos contacts ou recherchez un correspondant sur le réseau.
            </p>
            <button class="btn-primary" style="padding: 12px 24px; font-size: 13.5px; font-weight: 600; display: inline-flex; align-items: center; justify-content: center; gap: 8px; border-radius: 24px; box-shadow: 0 4px 16px rgba(124, 58, 237, 0.35);" data-action="navigate" data-screen="new_chat">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path>
                </svg>
                <span>Démarrer une discussion</span>
            </button>
        </div>
    `;
}

function renderNewChatResultsHtml(query) {
    const rawQ = (query || '').trim();
    const q = rawQ.toLowerCase().replace(/^@+/, '');

    let html = '';

    // Quick Action row (Google Messages LLC style)
    if (!q) {
        html += `
            <div style="margin-bottom: 12px;">
                <div class="new-chat-quick-action" data-action="navigate" data-screen="create_group">
                    <div class="new-chat-quick-icon" style="background: rgba(139, 92, 246, 0.18); color: var(--accent-purple-light);">
                        ${icons.users}
                    </div>
                    <div>
                        <div style="font-size: 13.5px; font-weight: 600; color: white;">Nouveau groupe</div>
                        <div style="font-size: 11px; color: var(--text-muted);">Créer un échange chiffré à plusieurs</div>
                    </div>
                </div>
                <div class="new-chat-quick-action" data-action="navigate" data-screen="add_contact">
                    <div class="new-chat-quick-icon" style="background: rgba(56, 189, 248, 0.18); color: #38bdf8;">
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <rect x="3" y="3" width="7" height="7"></rect>
                            <rect x="14" y="3" width="7" height="7"></rect>
                            <rect x="14" y="14" width="7" height="7"></rect>
                            <rect x="3" y="14" width="7" height="7"></rect>
                        </svg>
                    </div>
                    <div>
                        <div style="font-size: 13.5px; font-weight: 600; color: white;">Ajouter par lien ou QR Code</div>
                        <div style="font-size: 11px; color: var(--text-muted);">Coller une clé d'invitation ou scanner</div>
                    </div>
                </div>
            </div>
        `;
    }

    // Direct Action if user is typing
    if (rawQ) {
        html += `
            <div style="margin-bottom: 14px;">
                <div class="new-chat-quick-action" style="border-color: rgba(139, 92, 246, 0.4); background: rgba(139, 92, 246, 0.08);" data-action="startDirectChatFromQuery" data-query="${escapeHtml(rawQ)}">
                    <div class="new-chat-quick-icon" style="background: var(--accent-purple); color: white;">
                        ${icons.chat}
                    </div>
                    <div style="flex: 1; min-width: 0;">
                        <div style="font-size: 13.5px; font-weight: 600; color: white;">Écrire directement à</div>
                        <div style="font-size: 12px; color: var(--accent-purple-light); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">« ${escapeHtml(rawQ)} »</div>
                    </div>
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--accent-purple-light)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="9 18 15 12 9 6"></polyline>
                    </svg>
                </div>
            </div>
        `;
    }

    // Local Contacts
    let matchingContacts = state.contacts.filter(c => !c.isBlocked);
    if (q) {
        matchingContacts = matchingContacts.filter(c =>
            (c.name && c.name.toLowerCase().includes(q)) ||
            (c.handle && c.handle.toLowerCase().includes(q))
        );
    }

    if (matchingContacts.length > 0) {
        html += `
            <div class="new-chat-section-header">Contacts (${matchingContacts.length})</div>
            <div style="display: flex; flex-direction: column; gap: 4px;">
                ${matchingContacts.map(c => `
                    <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-conv-id="conv_${escapeHtml(c.handle)}" data-action="openChat">
                        <div class="avatar">
                            ${escapeHtml(c.name.charAt(0).toUpperCase())}
                            <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                        </div>
                        <div class="item-content">
                            <div class="item-header">
                                <span class="item-name" style="font-size: 14px; font-weight: 600; color: white;">${escapeHtml(c.name)}</span>
                            </div>
                            <div class="item-sub" style="font-size: 11.5px; color: var(--text-muted);">
                                ${escapeHtml(c.handle.length > 16 ? c.handle.slice(0, 8) + '...' + c.handle.slice(-6) : c.handle)}
                            </div>
                        </div>
                        <div style="color: var(--accent-purple-light); font-size: 12px; font-weight: 600; padding: 4px 10px; background: rgba(139, 92, 246, 0.12); border-radius: 12px;">
                            Discuter
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    } else if (!q) {
        html += `
            <div class="new-chat-section-header">Contacts (0)</div>
            <div style="text-align: center; color: var(--text-muted); padding: 24px 12px; background: var(--bg-surface); border: 1px dashed var(--border-subtle); border-radius: var(--radius-md);">
                <div style="font-size: 13px; color: white; font-weight: 600; margin-bottom: 4px;">Aucun contact enregistré</div>
                <div style="font-size: 11.5px; color: var(--text-muted); max-width: 260px; margin: 0 auto 12px;">
                    Tapez un @pseudo ou un identifiant ci-dessus pour chercher un utilisateur sur le réseau et lui écrire directement.
                </div>
                <button class="btn-secondary" style="font-size: 11px; padding: 6px 14px;" data-action="navigate" data-screen="add_contact">
                    Ajouter un contact par invitation
                </button>
            </div>
        `;
    }

    // Directory Search Results Container (populated asynchronously)
    html += `<div id="new-chat-directory-results" style="margin-top: 14px;"></div>`;

    return html;
}

let newChatSearchDebounceTimer = null;

function filterNewChatList(query) {
    state.newChatSearchQuery = query || '';
    const container = document.getElementById('new-chat-results-container');
    if (container) {
        container.innerHTML = renderNewChatResultsHtml(state.newChatSearchQuery);
    }

    const rawQ = (query || '').trim();
    if (!rawQ) return;

    if (newChatSearchDebounceTimer) clearTimeout(newChatSearchDebounceTimer);
    newChatSearchDebounceTimer = setTimeout(async () => {
        const dirContainer = document.getElementById('new-chat-directory-results');
        if (!dirContainer) return;

        const qLower = rawQ.toLowerCase().replace(/^@+/, '');
        if (hasBackend) {
            try {
                const dirUsers = await tauriInvoke('search_directory', { query: qLower }) || [];
                state.directorySearchResults = dirUsers;
                const notContacts = dirUsers.filter(u =>
                    !state.contacts.some(c => c.handle === u.peer_id || c.peerId === u.peer_id) &&
                    u.peer_id !== state.currentUser.peerId
                );

                if (notContacts.length > 0) {
                    dirContainer.innerHTML = `
                        <div class="new-chat-section-header">Réseau de découverte (${notContacts.length})</div>
                        <div style="display: flex; flex-direction: column; gap: 6px;">
                            ${notContacts.slice(0, 6).map(u => `
                                <div class="item-card" style="background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); padding: 10px 12px; display: flex; align-items: center; gap: 12px;">
                                    <div class="avatar" style="width: 38px; height: 38px; font-size: 15px; border-radius: 50%;">
                                        ${escapeHtml((u.display_name || u.username || '?').charAt(0).toUpperCase())}
                                        <div class="status-dot ${u.is_online ? 'status-online' : 'status-offline'}"></div>
                                    </div>
                                    <div class="item-content" style="flex: 1; min-width: 0;">
                                        <div class="item-name" style="font-size: 13.5px; font-weight: 600; color: white;">${escapeHtml(u.display_name || u.username)}</div>
                                        <div class="item-sub" style="font-size: 11px; color: var(--accent-purple-light);">@${escapeHtml(u.username)}</div>
                                    </div>
                                    <button class="btn-primary" style="font-size: 12px; padding: 7px 14px; border-radius: 16px; display: inline-flex; align-items: center; gap: 6px;" data-action="startDirectChatFromDirectoryUser" data-peer-id="${escapeHtml(u.peer_id)}" data-username="${escapeHtml(u.username)}" data-name="${escapeHtml(u.display_name)}" data-bundle="${escapeHtml(u.prekey_bundle_hex)}">
                                        <span>💬</span>
                                        <span>Discuter</span>
                                    </button>
                                </div>
                            `).join('')}
                        </div>
                    `;
                } else {
                    dirContainer.innerHTML = `
                        <div style="text-align: center; color: var(--text-dim); padding: 16px 8px; font-size: 12px;">
                            Aucun utilisateur supplémentaire trouvé sur le réseau pour « ${escapeHtml(rawQ)} ».
                        </div>
                    `;
                }
            } catch (err) {
                console.warn('New chat directory search error:', err);
            }
        }
    }, 220);
}

function clearNewChatSearch() {
    state.newChatSearchQuery = '';
    const input = document.getElementById('new-chat-search-input');
    if (input) input.value = '';
    const container = document.getElementById('new-chat-results-container');
    if (container) {
        container.innerHTML = renderNewChatResultsHtml('');
    }
}

async function startDirectChatFromDirectoryUser(peerId, username, name, bundleHex) {
    if (!requireBackend()) return;
    try {
        const existing = state.contacts.find(c => c.handle === peerId || c.peerId === peerId);
        if (!existing && bundleHex) {
            await tauriInvoke('add_contact', {
                username: username || '',
                displayName: name || username || 'Contact',
                bundleHex: bundleHex
            });
            await refreshContactsFromBackend();
        }
        openChatWith(name || username || 'Contact', peerId, 'conv_' + peerId);
    } catch (e) {
        console.error('startDirectChatFromDirectoryUser error', e);
        openChatWith(name || username || 'Contact', peerId, 'conv_' + peerId);
    }
}

async function startDirectChatFromQuery(rawQ) {
    const q = (rawQ || '').trim();
    if (!q) return;

    // 1. Check local contacts
    const qLower = q.toLowerCase().replace(/^@+/, '');
    const localContact = state.contacts.find(c =>
        c.name.toLowerCase() === qLower ||
        c.handle.toLowerCase() === qLower ||
        (c.peerId && c.peerId.toLowerCase() === qLower)
    );
    if (localContact) {
        openChatWith(localContact.name, localContact.handle, 'conv_' + localContact.handle);
        return;
    }

    // 2. Check cached directory search results
    if (state.directorySearchResults && state.directorySearchResults.length > 0) {
        const dirMatch = state.directorySearchResults.find(u =>
            u.peer_id.toLowerCase() === qLower ||
            (u.username && u.username.toLowerCase() === qLower)
        );
        if (dirMatch) {
            await startDirectChatFromDirectoryUser(dirMatch.peer_id, dirMatch.username, dirMatch.display_name, dirMatch.prekey_bundle_hex);
            return;
        }
    }

    // 3. Search directory live
    if (hasBackend) {
        try {
            const results = await tauriInvoke('search_directory', { query: qLower }) || [];
            if (results.length > 0) {
                const best = results[0];
                await startDirectChatFromDirectoryUser(best.peer_id, best.username, best.display_name, best.prekey_bundle_hex);
                return;
            }
        } catch (e) {
            console.warn('startDirectChatFromQuery directory search failed', e);
        }
    }

    // 4. Hex peer ID
    if (/^[0-9a-fA-F]{16,}$/.test(q)) {
        openChatWith('Pair ' + q.slice(0, 6) + '...' + q.slice(-4), q, 'conv_' + q);
        return;
    }

    // 5. Fallback: navigate to add_contact prefilled
    fillAddContactForm(q, q);
}

let conversationsSearchDebounceTimer = null;

function filterConversationsList(query) {
    state.conversationsSearchQuery = query || '';
    const container = document.getElementById('conversations-list-container');
    if (container) {
        container.innerHTML = renderConversationsListHtml(state.conversationsSearchQuery);
    }

    // Also look up contacts & directory if typing a query
    const rawQ = (query || '').trim();
    if (!rawQ) return;

    if (conversationsSearchDebounceTimer) clearTimeout(conversationsSearchDebounceTimer);
    conversationsSearchDebounceTimer = setTimeout(async () => {
        const previewContainer = document.getElementById('conversations-search-contacts-preview');
        if (!previewContainer) return;

        const qLower = rawQ.toLowerCase().replace(/^@+/, '');
        const matchingContacts = state.contacts.filter(c =>
            !c.isBlocked &&
            (c.name.toLowerCase().includes(qLower) || c.handle.toLowerCase().includes(qLower))
        );

        let html = '';
        if (matchingContacts.length > 0) {
            html += `
                <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); margin: 10px 0 8px 4px;">CONTACTS CORRESPONDANTS (${matchingContacts.length})</div>
                ${matchingContacts.map(c => `
                    <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-action="openChat">
                        <div class="avatar">
                            ${escapeHtml(c.name.charAt(0))}
                            <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                        </div>
                        <div class="item-content">
                            <div class="item-name">${escapeHtml(c.name)}</div>
                            <div class="item-sub">@${escapeHtml(c.handle)}</div>
                        </div>
                    </div>
                `).join('')}
            `;
        }

        // Search directory if backend available
        if (hasBackend) {
            try {
                const dirUsers = await tauriInvoke('search_directory', { query: qLower }) || [];
                const notContacts = dirUsers.filter(u => !state.contacts.some(c => c.handle === u.peer_id || c.peerId === u.peer_id) && u.peer_id !== state.currentUser.peerId);
                if (notContacts.length > 0) {
                    html += `
                        <div style="font-size: 11px; font-weight: 600; color: var(--accent-purple-light); margin: 14px 0 8px 4px;">ANNUAIRE DE DÉCOUVERTE (${notContacts.length})</div>
                        ${notContacts.slice(0, 5).map(u => `
                            <div class="item-card" style="background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); margin-bottom: 6px; padding: 10px 12px;">
                                <div class="avatar" style="width: 36px; height: 36px; font-size: 14px;">
                                    ${escapeHtml(u.display_name.charAt(0) || '?')}
                                    <div class="status-dot ${u.is_online ? 'status-online' : 'status-offline'}"></div>
                                </div>
                                <div class="item-content" style="cursor: pointer;" data-action="inspectDirectoryUser" data-peer-id="${escapeHtml(u.peer_id)}">
                                    <div class="item-name" style="font-size: 13px; font-weight: 600; color: white;">${escapeHtml(u.display_name)}</div>
                                    <div class="item-sub" style="color: var(--accent-purple-light); font-size: 11px;">@${escapeHtml(u.username)}</div>
                                </div>
                                <button class="btn-primary" style="font-size: 10px; padding: 5px 10px;" data-action="addDirectUser" data-peer-id="${escapeHtml(u.peer_id)}" data-username="${escapeHtml(u.username)}" data-name="${escapeHtml(u.display_name)}" data-bundle="${escapeHtml(u.prekey_bundle_hex)}">Ajouter</button>
                            </div>
                        `).join('')}
                    `;
                }
            } catch (err) {
                console.warn('Conversations search directory query error:', err);
            }
        }

        if (html && document.getElementById('conversations-search-contacts-preview')) {
            document.getElementById('conversations-search-contacts-preview').innerHTML = html;
        }
    }, 200);
}

function clearConversationsSearch() {
    state.conversationsSearchQuery = '';
    const input = document.getElementById('conversations-search-input');
    if (input) input.value = '';
    const container = document.getElementById('conversations-list-container');
    if (container) {
        container.innerHTML = renderConversationsListHtml('');
    }
}

function clearContactsSearch() {
    state.contactsSearchQuery = '';
    const input = document.getElementById('contacts-search-input');
    if (input) input.value = '';
    filterContactsList('');
    if (state.currentScreen === 'contacts') render();
}

function updateNetworkOnlineStatus(isOnline) {
    state.networkOnline = !!isOnline;
    state.currentUser.status = isOnline
        ? 'Connecté au réseau NOVA'
        : 'Hors ligne — Déconnecté du réseau';

    const statusEl = document.getElementById('profile-network-status-text');
    if (statusEl) {
        statusEl.innerText = state.currentUser.status;
    }
    const railDot = document.querySelector('.rail-status-dot');
    if (railDot) {
        railDot.className = `rail-status-dot ${isOnline ? 'status-online' : 'status-offline'}`;
    }
}

window.addEventListener('online', () => updateNetworkOnlineStatus(true));
window.addEventListener('offline', () => updateNetworkOnlineStatus(false));

function updateConversationsListDom() {
    const scrollList = document.getElementById('conversations-list-container') || document.querySelector('.screen-view .scroll-list');
    if (!scrollList || state.currentScreen !== 'conversations') return;
    const previousScroll = scrollList.scrollTop;
    scrollList.innerHTML = renderConversationsListHtml(state.conversationsSearchQuery);
    scrollList.scrollTop = previousScroll;
}

// --- LOCATION SHARING (small JSON envelope over the text pipeline) ---
// A shared location is just a couple of coordinates — small enough to travel as a JSON envelope
// inside the ordinary text pipeline (X3DH/Double Ratchet encrypted exactly like any other text
// message) without needing the binary media pipeline below. Real media (image/video/audio/file)
// used to travel this same way as a base64 blob, but as of the 2026-08-22 audit's J4 fix that
// goes through send_media/nova_engine::send_media instead — real chunked binary transfer, a
// dedicated encrypted-at-rest attachment table, and no more decrypting megabytes of embedded
// media on every search (see MAX_MEDIA_BYTES and sendMediaMessage below).
const STRUCTURED_MARKER = '__NOVA_STRUCTURED_MSG_V1__:';

function encodeStructuredMessage(obj) {
    return STRUCTURED_MARKER + JSON.stringify(obj);
}

function decodeStructuredMessage(text) {
    if (typeof text !== 'string' || !text.startsWith(STRUCTURED_MARKER)) return null;
    try {
        return JSON.parse(text.slice(STRUCTURED_MARKER.length));
    } catch (e) {
        return null;
    }
}

// Overall ceiling on one media message, now that real chunking (nova_protocol::MEDIA_CHUNK_SIZE
// per packet) removes the old single-packet 5 MB wall — chosen to keep a send/receive bounded to
// a reasonable amount of time and local storage, not because of any protocol limit.
const MAX_MEDIA_BYTES = 100 * 1024 * 1024; // 100 Mo

// Maps the Rust engine's real DbMessageStatus (serialized as its variant name — "Sent",
// "Delivered", "Read", "Failed", ...) onto the small set of visual states the message bubble
// understands. Incoming messages are always shown as simply "read" (no delivery-tick semantics
// apply to something already received). "sending" is a purely local/optimistic state set before
// the backend call resolves — see sendMessage — never something the backend reports.
function mapUiMessageStatus(m) {
    if (!m.is_outgoing) return 'read';
    switch (m.status) {
        case 'Failed':
            return 'failed';
        case 'Delivered':
        case 'Read':
            return 'delivered';
        default:
            return 'sent';
    }
}

// nova_protocol::MessageContentType's variant name (as serialized to JSON) -> the UI's message
// bubble type. "Audio" maps to 'voice' (a recorded note, this app's only current audio path);
// "Text"/"SystemNotification" and anything unrecognized fall through to plain text.
function contentTypeToUiType(contentType) {
    switch (contentType) {
        case 'Image': return 'image';
        case 'Video': return 'video';
        case 'Audio': return 'voice';
        case 'File': return 'file';
        default: return 'text';
    }
}

function mapBackendMessage(m, conversationId) {
    const base = {
        id: m.id,
        conversationId,
        recipientId: m.recipient_id,
        time: formatMessageTime(m.timestamp_utc),
        isOutgoing: m.is_outgoing,
        status: mapUiMessageStatus(m),
    };
    const structured = decodeStructuredMessage(m.text_content);
    if (structured) {
        if (structured.kind === 'location') {
            return { ...base, type: 'location', text: structured.label, meta: 'Précision d\'environ 5 mètres' };
        }
        if (structured.kind === 'call_log') {
            return { ...base, type: 'call', text: structured.label || 'Appel', meta: structured.duration || 'Terminé' };
        }
        if (structured.kind === 'call_signal') {
            handleIncomingCallSignal(structured, m, conversationId);
            return { ...base, type: 'call_signal', hidden: true, text: '' };
        }
    }

    const uiType = contentTypeToUiType(m.content_type);
    if (uiType !== 'text' && m.attachment) {
        return {
            ...base,
            type: uiType,
            text: m.text_content,
            meta: formatFileSize(m.attachment.size_bytes),
            attachmentId: m.id,
            mimeType: m.attachment.mime_type,
            url: undefined, // filled in lazily — see ensureAttachmentLoaded
        };
    }
    return { ...base, type: 'text', text: m.text_content };
}

// Fetches one message's attachment bytes on demand (never eagerly with the rest of the
// conversation — see nova_engine::NovaEngine::get_attachment_data) and re-renders that one
// bubble in place once loaded. Safe to call repeatedly: a no-op once `msg.url` is already set.
async function ensureAttachmentLoaded(msg) {
    if (!msg.attachmentId || msg.url || !hasBackend) return;
    try {
        const dataBase64 = await tauriInvoke('get_attachment_data', { messageId: msg.attachmentId });
        if (!dataBase64) return;
        msg.url = `data:${msg.mimeType || 'application/octet-stream'};base64,${dataBase64}`;

        // In the chat view, swap just this one bubble in place (keeps scroll position). On any
        // other screen showing this same message (e.g. "Médias partagés"), there's no single
        // bubble to target, so re-render that whole screen instead.
        const existingRow = document.getElementById(`msg-row-${msg.id}`);
        if (existingRow) {
            const rebuilt = document.createElement('div');
            rebuilt.innerHTML = buildMessageHtml(msg).trim();
            existingRow.replaceWith(rebuilt.firstElementChild);
        } else if (state.currentScreen === 'shared_media' || state.currentScreen === 'chat') {
            const container = document.getElementById('screen-container');
            if (container) container.innerHTML = screens[state.currentScreen]();
        }
    } catch (e) {
        console.error('Échec du chargement de la pièce jointe', msg.id, e);
    }
}

// Fetches the full message history for one conversation from local storage and merges it into
// `state.messages`, returning only the messages new since the last fetch (by id) — the chat-open
// polling loop uses this to append incrementally instead of destroying scroll position and input
// focus with a full re-render on every poll.
async function refreshMessagesFromBackend(conversationId) {
    if (!hasBackend || !conversationId) return [];
    let fetched;
    try {
        fetched = await tauriInvoke('get_messages', { conversationId });
    } catch (e) {
        console.error('refreshMessagesFromBackend failed', e);
        return [];
    }

    const previouslyKnownIds = new Set(
        state.messages.filter(m => m.conversationId === conversationId).map(m => m.id)
    );
    const mapped = fetched.map(m => mapBackendMessage(m, conversationId));

    const newOnes = mapped.filter(m => !previouslyKnownIds.has(m.id));
    if (newOnes.length > 0 && previouslyKnownIds.size > 0) {
        newOnes.filter(m => !m.isOutgoing).forEach(maybeNotifyIncomingMessage);
    }

    state.messages = state.messages.filter(m => m.conversationId !== conversationId).concat(mapped);
    return newOnes;
}

// Renders a real, scannable QR code encoding the cryptographically signed invitation ticket.
function renderOwnQrCode() {
    const canvas = document.getElementById('my-qr-canvas');
    const textToEncode = state.currentUser.invitationUri || state.currentUser.bundleHex;
    if (!canvas || !textToEncode || typeof QRCode === 'undefined') return;
    // `scale` (integer pixels per module) is used instead of a fixed `width` on purpose: qrcode.js
    // picks the scale as width/(moduleCount + 2*margin) and floor-rounds each module's pixel span
    // independently, so a `width` that doesn't divide evenly into the module count (the overwhelmingly
    // common case, since module count varies with invitation length) produces a jittery grid where
    // modules are inconsistently 1px narrower/wider than their neighbors. That's invisible to the eye
    // at a glance but breaks the uniform sampling grid both jsQR and real camera scanners rely on —
    // this was the actual cause of QR codes that looked fine but scanned unreliably or not at all.
    // `margin: 4` matches the ISO/IEC 18004 minimum quiet zone (the previous margin: 2 halved that,
    // which independently hurt real-camera recognition). `scale: 4` keeps modules physically large.
    // toCanvas() sets canvas.style.width/height to match the raster size it just produced, so the CSS
    // display size (272px, chosen so live camera-scanning gets a consistently sized on-screen code
    // regardless of how large the underlying raster ends up being for a given invitation's module
    // count) is explicitly restored below after rendering. 'L' (7% redundancy) keeps the QR version —
    // and so the module count — as low as this ~1.5-2KB invitation ticket allows, since fewer, larger
    // modules is what actually makes live camera scanning reliable.
    QRCode.toCanvas(canvas, textToEncode, { errorCorrectionLevel: 'L', margin: 4, scale: 4 }, (err) => {
        if (err) {
            // The link itself was fetched fine, but the QR library couldn't encode it (e.g. the
            // invitation ticket's content — bundle + rendezvous addresses — exceeded the QR
            // format's max capacity). Previously this left the canvas permanently blank with only
            // a console.error, indistinguishable from "still loading" — surface it visibly instead,
            // same as a link-fetch failure, so the user has a "Réessayer" instead of a dead screen.
            console.error('QR render failed', err);
            state.currentUser.linkGenerationError = String(err.message || err);
            if (state.currentScreen === 'identity') {
                const container = document.getElementById('screen-container');
                if (container) container.innerHTML = screens.identity();
            }
            return;
        }
        canvas.style.width = '272px';
        canvas.style.height = '272px';
    });
}

// Saves/shares the rendered QR as a PNG. The synthetic `<a download>` click this used to rely on
// exclusively works on desktop WebView2 but is silently a no-op on Android's WebView (there is no
// download manager hook for a `data:` URI there) — exactly the "le bouton enregistré est
// silencieux" symptom. Mobile gets the Web Share API (Level 2, file sharing) instead, which hands
// the PNG to Android's native share sheet (Save to Files/Gallery, send in another app, etc.) — the
// same mechanism `shareInvitation()` already uses for text, just extended with a `files` array.
// Every path now also gives explicit success/failure feedback, since a platform-specific save
// mechanism failing silently was the other half of the complaint, independent of *which*
// mechanism was used.
async function saveQrImage() {
    const canvas = document.getElementById('my-qr-canvas');
    if (!canvas) {
        alert('QR code indisponible.');
        return;
    }
    const filename = `nova_invite_${(state.currentUser.username || 'contact')}.png`;

    const blob = await new Promise((resolve) => canvas.toBlob(resolve, 'image/png'));
    if (!blob) {
        alert("Échec de l'enregistrement : impossible de générer l'image du QR code.");
        return;
    }
    const file = new File([blob], filename, { type: 'image/png' });

    if (navigator.canShare && navigator.canShare({ files: [file] })) {
        try {
            await navigator.share({ files: [file], title: 'QR code NOVA Chat' });
            return;
        } catch (e) {
            if (e && e.name === 'AbortError') return; // user cancelled the share sheet — not a failure
            console.error('navigator.share(files) failed, falling back', e);
        }
    }

    // Under Tauri (desktop or Android), write the file for real via the fs plugin instead of the
    // synthetic <a download> blob-URL click below — that click() never throws even when it
    // silently does nothing, which is exactly what happens on Android's WebView (no
    // download-manager hook for a blob:/data: URI there): the button used to claim "Image
    // enregistrée" unconditionally regardless of whether anything was actually written to disk.
    if (hasBackend && window.__TAURI__.fs && window.__TAURI__.path) {
        try {
            const bytes = new Uint8Array(await blob.arrayBuffer());
            await window.__TAURI__.fs.writeFile(filename, bytes, {
                baseDir: window.__TAURI__.path.BaseDirectory.Download,
            });
            alert(`Image enregistrée : dossier Téléchargements/${filename}`);
        } catch (e) {
            console.error('Native file save failed', e);
            alert("Échec de l'enregistrement de l'image. Utilisez plutôt « Partager le lien ».");
        }
        return;
    }

    // Plain-browser fallback only (e.g. iterating on styling outside the Tauri shell, where
    // hasBackend is false) — a real <a download> click reliably works in an actual browser tab.
    try {
        const link = document.createElement('a');
        link.download = filename;
        link.href = URL.createObjectURL(blob);
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        setTimeout(() => URL.revokeObjectURL(link.href), 10000);
        alert('Image enregistrée dans vos téléchargements.');
    } catch (e) {
        console.error('QR image save failed', e);
        alert("Échec de l'enregistrement de l'image. Utilisez plutôt « Partager le lien ».");
    }
}

async function shareInvitation() {
    const text = state.currentUser.invitationUri || state.currentUser.bundleHex;
    if (!text) {
        alert('Créez d\'abord votre compte pour partager votre invitation.');
        return;
    }
    const shareData = {
        title: 'Invitation NOVA Chat',
        text: `Ajoutez-moi sur NOVA Chat avec mon lien sécurisé (valable 24h) :\n${text}`,
    };
    if (navigator.share) {
        try {
            await navigator.share(shareData);
            return;
        } catch (e) {
            // Cancelled or not supported, fallback to copy
        }
    }
    copyOwnBundle();
}

function copyOwnBundle() {
    const text = state.currentUser.invitationUri || state.currentUser.bundleHex;
    if (!text) {
        alert('Créez d\'abord votre compte pour obtenir votre lien.');
        return;
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).catch(() => {});
    }
    const textarea = document.getElementById('my-bundle-output');
    if (textarea) {
        textarea.select();
        document.execCommand('copy');
    }
    alert('Lien d\'invitation sécurisé copié ! Vous pouvez l\'envoyer par SMS, WhatsApp ou tout autre moyen.');
}

function copyOwnPeerId() {
    const peerId = state.currentUser.peerId;
    if (!peerId) {
        alert('Identifiant indisponible. Créez d\'abord votre compte.');
        return;
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(peerId).catch(() => {});
    }
    alert('Votre identifiant public (Peer ID) a été copié dans le presse-papier !');
}

function fillAddContactForm(name, idOrBundle) {
    state.prefillContact = { name: name || '', id: idOrBundle || '' };
    navigateTo('add_contact').then(() => {
        const nameInput = document.getElementById('add-display-name-input');
        const bundleInput = document.getElementById('add-bundle-input');
        if (nameInput && state.prefillContact && state.prefillContact.name) {
            nameInput.value = state.prefillContact.name;
        }
        if (bundleInput && state.prefillContact && state.prefillContact.id) {
            bundleInput.value = state.prefillContact.id;
        }
        if (bundleInput && bundleInput.value) {
            bundleInput.focus();
        } else if (nameInput) {
            nameInput.focus();
        }
        state.prefillContact = null;
    });
}

function maybeNotifyIncomingMessage(msg) {
    CallAudio.playMessageReceivedSound();
    if (!state.notificationPrefs.notifyMessages) return;
    if (typeof Notification === 'undefined' || Notification.permission !== 'granted') return;
    const title = state.notificationPrefs.hideContent
        ? 'NOVA Chat'
        : (state.activeContact ? `Nouveau message de ${state.activeContact.name}` : 'Nouveau message reçu');
    const body = state.notificationPrefs.hideContent
        ? 'Nouveau message chiffré reçu'
        : (msg.type === 'text' ? msg.text : `[${msg.type}]`);
    try {
        new Notification(title, { body, icon: 'favicon.ico' });
    } catch (e) {
        // Notification API unavailable or denied in certain WebViews
    }
}

async function refreshContactsFromBackend() {
    if (!hasBackend) return;
    try {
        const contacts = await tauriInvoke('get_contacts');
        state.contacts = contacts.map(c => ({
            name: c.display_name,
            handle: c.peer_id,
            peerId: c.peer_id,
            online: c.is_online,
            isBlocked: c.is_blocked,
            isTrusted: c.is_trusted,
            p2pMode: c.is_blocked ? 'Contact bloqué' : (c.is_trusted ? 'Contact de confiance' : 'Contact non vérifié'),
            key: c.peer_id,
            safetyNumber: c.safety_number,
        }));
    } catch (e) {
        console.error('refreshContactsFromBackend failed', e);
    }
}

async function addActiveContactToContacts(peerId, name) {
    if (!requireBackend()) return;
    const targetPeerId = peerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    const targetName = name || (state.activeContact && state.activeContact.name) || 'Contact';
    if (!targetPeerId) return;

    try {
        let bundleHex = '';
        let username = '';
        let displayName = targetName;
        try {
            const results = await tauriInvoke('search_directory', { query: targetPeerId });
            if (results && results.length > 0) {
                const match = results.find(u => u.peer_id.toLowerCase() === targetPeerId.toLowerCase()) || results[0];
                bundleHex = match.prekey_bundle_hex;
                username = match.username;
                displayName = match.display_name || targetName;
            }
        } catch (_) {}

        await tauriInvoke('add_contact', {
            username: username || '',
            displayName: displayName || 'Contact',
            bundleHex: bundleHex || ''
        });

        await refreshContactsFromBackend();

        if (state.activeContact && (state.activeContact.peerId === targetPeerId || state.activeContact.handle === targetPeerId)) {
            state.activeContact.isContact = true;
            state.activeContact.name = displayName;
            const banner = document.getElementById('unknown-contact-banner') || document.getElementById('trust-contact-banner');
            if (banner) banner.remove();
        }

        alert('« ' + displayName + ' » a été ajouté à vos contacts avec succès.');
    } catch (e) {
        alert('Erreur lors de l\'ajout aux contacts : ' + e);
    }
}

async function trustActiveContact(peerId) {
    const targetPeerId = peerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    const targetName = (state.activeContact && state.activeContact.name) || 'Contact';
    if (!targetPeerId || !requireBackend()) return;
    try {
        if (state.activeContact && !state.activeContact.isContact) {
            await addActiveContactToContacts(targetPeerId, targetName);
        }
        await tauriInvoke('trust_contact', { peerId: targetPeerId });
        if (state.activeContact && (state.activeContact.peerId === targetPeerId || state.activeContact.handle === targetPeerId)) {
            state.activeContact.isTrusted = true;
            const banner = document.getElementById('unknown-contact-banner') || document.getElementById('trust-contact-banner');
            if (banner) banner.remove();
        }
        await refreshContactsFromBackend();
        alert('Contact marqué comme de confiance.');
        if (state.currentScreen === 'contact_profile') {
            navigateTo('contact_profile');
        }
    } catch (e) {
        alert('Échec de la validation de confiance : ' + e);
    }
}

function openBlockModal(peerId, name) {
    const targetPeerId = peerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    const targetName = name || (state.activeContact && state.activeContact.name) || 'ce contact';
    state.pendingBlockPeerId = targetPeerId;
    state.pendingBlockName = targetName;

    const modal = document.getElementById('block-contact-modal');
    const title = document.getElementById('block-modal-title');
    const desc = document.getElementById('block-modal-desc');
    if (title) title.innerText = `Bloquer ${targetName} ?`;
    if (desc) desc.innerHTML = `Les futurs messages et appels de <strong>${escapeHtml(targetName)}</strong> seront immédiatement rejetés. Vous pourrez retrouver ce contact dans l'onglet Contacts pour le débloquer si nécessaire.`;
    if (modal) modal.classList.add('show');
}

function closeBlockModal() {
    const modal = document.getElementById('block-contact-modal');
    if (modal) modal.classList.remove('show');
    state.pendingBlockPeerId = null;
    state.pendingBlockName = null;
}

async function confirmBlockOnly() {
    const peerId = state.pendingBlockPeerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    if (!peerId || !requireBackend()) return;
    try {
        await tauriInvoke('block_contact', { peerId });
        closeBlockModal();
        await refreshContactsFromBackend();
        await refreshConversationsFromBackend();
        alert('Ce contact a été bloqué au niveau du moteur Rust. Ses paquets seront désormais ignorés.');
        state.activeContact = null;
        navigateTo('conversations');
    } catch (e) {
        alert('Échec du blocage : ' + e);
    }
}

async function confirmBlockAndDelete() {
    const peerId = state.pendingBlockPeerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    if (!peerId || !requireBackend()) return;
    try {
        await tauriInvoke('block_and_delete_conversation', { peerId });
        closeBlockModal();
        await refreshContactsFromBackend();
        await refreshConversationsFromBackend();
        alert('Contact bloqué et conversation supprimée définitivement.');
        state.activeContact = null;
        navigateTo('conversations');
    } catch (e) {
        alert('Échec du blocage et suppression : ' + e);
    }
}

async function blockActiveContact() {
    openBlockModal();
}

async function unblockActiveContact(peerId) {
    const targetPeerId = peerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    if (!targetPeerId || !requireBackend()) return;
    try {
        await tauriInvoke('unblock_contact', { peerId: targetPeerId });
        await refreshContactsFromBackend();
        await refreshConversationsFromBackend();
        alert('Contact débloqué avec succès.');
        if (state.currentScreen === 'contact_profile') {
            const found = state.contacts.find(c => c.handle === targetPeerId || c.peerId === targetPeerId);
            if (found) {
                openChatWith(found.name, found.handle);
                navigateTo('contact_profile');
                return;
            }
        }
        navigateTo('contacts');
    } catch (e) {
        alert('Échec du déblocage : ' + e);
    }
}

// --- REPORTING & FEEDBACK (User features) ---
function openReportModal(peerId, name) {
    closeInspectUserModal();
    const targetPeerId = peerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    const targetName = name || (state.activeContact && state.activeContact.name) || 'ce contact';
    state.pendingReport = { targetPeerId, targetName };

    const modal = document.getElementById('report-modal');
    const title = document.getElementById('report-modal-title');
    const comment = document.getElementById('report-comment-input');
    if (title) title.innerText = `Signaler ${targetName}`;
    if (comment) comment.value = '';
    if (modal) modal.classList.add('show');
}

function closeReportModal() {
    const modal = document.getElementById('report-modal');
    if (modal) modal.classList.remove('show');
    state.pendingReport = null;
}

async function confirmSubmitReportReal() {
    if (!state.pendingReport || !state.pendingReport.targetPeerId) return;
    const select = document.getElementById('report-category-select');
    const commentInput = document.getElementById('report-comment-input');
    const category = select ? select.value : 'other';
    const comment = commentInput ? commentInput.value.trim() : '';
    const reason = select ? select.options[select.selectedIndex].text : 'Signalement utilisateur';

    try {
        const msg = await tauriInvoke('report_user', {
            targetPeerId: state.pendingReport.targetPeerId,
            reason,
            category,
            comment: comment || null,
        });
        closeReportModal();
        showToast(msg || 'Signalement envoyé avec succès.');
    } catch (e) {
        alert('Erreur lors du signalement : ' + e);
    }
}

function openFeedbackModal() {
    state.feedbackRating = 5;
    updateFeedbackStarsUI();
    const comment = document.getElementById('feedback-comment-input');
    if (comment) comment.value = '';
    const modal = document.getElementById('feedback-modal');
    if (modal) modal.classList.add('show');
}

function closeFeedbackModal() {
    const modal = document.getElementById('feedback-modal');
    if (modal) modal.classList.remove('show');
}

function setFeedbackRating(rating) {
    state.feedbackRating = Math.max(1, Math.min(5, rating));
    updateFeedbackStarsUI();
}

function updateFeedbackStarsUI() {
    const container = document.getElementById('feedback-stars-container');
    if (!container) return;
    const stars = container.querySelectorAll('.star-btn');
    stars.forEach((btn, idx) => {
        if (idx < state.feedbackRating) {
            btn.style.color = '#fbbf24';
        } else {
            btn.style.color = '#4b5563';
        }
    });
}

async function confirmSubmitFeedbackReal() {
    const select = document.getElementById('feedback-category-select');
    const commentInput = document.getElementById('feedback-comment-input');
    const category = select ? select.value : 'general';
    const comment = commentInput ? commentInput.value.trim() : '';

    try {
        const msg = await tauriInvoke('submit_app_feedback', {
            rating: state.feedbackRating,
            category,
            comment: comment || null,
        });
        closeFeedbackModal();
        showToast(msg || 'Merci pour votre avis !');
    } catch (e) {
        alert('Erreur lors de l\'envoi de votre avis : ' + e);
    }
}

// --- ADMIN MODERATION ENGINE (Conditional Admin Build) ---
async function saveAdminToken() {
    const input = document.getElementById('admin-token-input');
    if (!input) return;
    const token = input.value.trim();
    state.adminState.token = token;
    localStorage.setItem('nova_admin_token', token);
    await loadAdminOverviewReal();
}

async function loadAdminOverviewReal() {
    if (!state.adminState.token) {
        showToast('Veuillez renseigner votre clé secrète admin.');
        return;
    }
    state.adminState.isLoading = true;
    try {
        const overview = await tauriInvoke('admin_fetch_overview', {
            adminToken: state.adminState.token,
        });
        state.adminState.overview = overview;
        state.adminState.error = '';
        if (state.currentScreen === 'admin_dashboard') {
            navigateTo('admin_dashboard');
        }
        showToast('Données de supervision actualisées.');
    } catch (e) {
        state.adminState.error = String(e);
        alert('Erreur d\'authentification admin : ' + e);
    } finally {
        state.adminState.isLoading = false;
    }
}

function setAdminTab(tab) {
    state.adminState.activeTab = tab;
    if (state.currentScreen === 'admin_dashboard') {
        navigateTo('admin_dashboard');
    }
}

function setAdminFilter(filter) {
    state.adminState.filterStatus = filter;
    if (state.currentScreen === 'admin_dashboard') {
        navigateTo('admin_dashboard');
    }
}

async function adminBanUserReal(peerId) {
    if (!peerId) return;
    const reason = prompt('Motif de suspension du compte :', 'Non-respect des conditions d\'utilisation');
    if (reason === null) return;
    try {
        await tauriInvoke('admin_ban_user', {
            adminToken: state.adminState.token,
            peerId,
            reason: reason || null,
        });
        showToast('Utilisateur suspendu.');
        await loadAdminOverviewReal();
    } catch (e) {
        alert('Échec de suspension : ' + e);
    }
}

async function adminUnbanUserReal(peerId) {
    if (!peerId) return;
    if (!confirm('Voulez-vous réhabiliter cet utilisateur et restaurer son accès au répertoire ?')) return;
    try {
        await tauriInvoke('admin_unban_user', {
            adminToken: state.adminState.token,
            peerId,
        });
        showToast('Utilisateur réhabilité.');
        await loadAdminOverviewReal();
    } catch (e) {
        alert('Échec du déblocage : ' + e);
    }
}

async function adminSaveThresholdReal() {
    const input = document.getElementById('admin-threshold-input');
    if (!input) return;
    const val = parseInt(input.value, 10);
    if (isNaN(val) || val < 1) {
        alert('Veuillez entrer un nombre valide supérieur ou égal à 1.');
        return;
    }
    try {
        const msg = await tauriInvoke('admin_update_settings', {
            adminToken: state.adminState.token,
            autoBanThreshold: val,
        });
        showToast(msg || 'Seuil enregistré.');
        await loadAdminOverviewReal();
    } catch (e) {
        alert('Échec de mise à jour du seuil : ' + e);
    }
}

async function initAppBuildInfo() {
    try {
        const info = await tauriInvoke('get_app_build_info');
        if (info) {
            state.buildInfo = info;
            console.log('App build info:', info);
        }
    } catch (e) {
        console.warn('Could not fetch build info:', e);
    }
}

// --- INVIOLABLE APP LOCK ENGINE (PIN, BIOMETRICS & INACTIVITY TIMEOUT) ---
async function hashPin(pin, salt) {
    const enc = new TextEncoder();
    const data = enc.encode(salt + ':' + pin + ':nova_app_lock_v1');
    const hashBuf = await crypto.subtle.digest('SHA-256', data);
    return Array.from(new Uint8Array(hashBuf)).map(b => b.toString(16).padStart(2, '0')).join('');
}

function lockApp() {
    if (!state.appLock.enabled || !state.appLock.pinHash) return;
    state.isAppLocked = true;
    state.enteredPin = '';
    updateLockScreenUI();
    const overlay = document.getElementById('app-lock-screen');
    if (overlay) {
        overlay.style.display = 'flex';
    }
    if (state.appLock.biometricsEnabled) {
        setTimeout(triggerBiometricsUnlock, 300);
    }
}

function unlockApp() {
    state.isAppLocked = false;
    state.enteredPin = '';
    state.failedPinAttempts = 0;
    state.lockoutUntil = 0;
    state.lastUserInteraction = Date.now();
    state.wentToBackgroundAt = null;
    const overlay = document.getElementById('app-lock-screen');
    if (overlay) {
        overlay.style.display = 'none';
    }
}

function updateLockScreenUI() {
    const dotsContainer = document.getElementById('app-lock-dots');
    const errorMsg = document.getElementById('app-lock-error-msg');
    const biometricsBtn = document.getElementById('btn-lock-biometrics');
    if (biometricsBtn) {
        biometricsBtn.style.visibility = state.appLock.biometricsEnabled ? 'visible' : 'hidden';
    }

    if (dotsContainer) {
        const totalDots = state.appLock.pinLength || 4;
        let html = '';
        for (let i = 0; i < totalDots; i++) {
            html += `<div class="pin-dot ${i < state.enteredPin.length ? 'filled' : ''}"></div>`;
        }
        dotsContainer.innerHTML = html;
    }

    if (errorMsg) {
        if (state.lockoutUntil > Date.now()) {
            const remSec = Math.ceil((state.lockoutUntil - Date.now()) / 1000);
            errorMsg.innerText = `Trop d'échecs. Réessayez dans ${remSec}s...`;
        } else if (state.failedPinAttempts > 0 && state.enteredPin.length === 0) {
            errorMsg.innerText = `Code PIN incorrect (${5 - state.failedPinAttempts} essais restants)`;
        } else {
            errorMsg.innerText = '';
        }
    }
}

async function handlePinDigit(digit) {
    if (state.lockoutUntil > Date.now()) {
        updateLockScreenUI();
        return;
    }
    const maxLen = state.appLock.pinLength || 6;
    if (state.enteredPin.length >= maxLen) return;
    state.enteredPin += String(digit);
    updateLockScreenUI();

    const expectedLen = state.appLock.pinLength || 4;
    if (state.enteredPin.length >= expectedLen) {
        const computedHash = await hashPin(state.enteredPin, state.appLock.pinSalt);
        if (computedHash === state.appLock.pinHash) {
            unlockApp();
            return;
        } else if (state.enteredPin.length >= (state.appLock.pinLength || 6)) {
            handleWrongPin();
        }
    }
}

function handlePinBackspace() {
    if (state.enteredPin.length > 0) {
        state.enteredPin = state.enteredPin.slice(0, -1);
        updateLockScreenUI();
    }
}

function handleWrongPin() {
    state.failedPinAttempts += 1;
    const dotsContainer = document.getElementById('app-lock-dots');
    const errorMsg = document.getElementById('app-lock-error-msg');
    if (dotsContainer) {
        dotsContainer.querySelectorAll('.pin-dot').forEach(d => d.classList.add('error'));
    }
    if (state.failedPinAttempts >= 5) {
        state.lockoutUntil = Date.now() + 30000;
        if (errorMsg) errorMsg.innerText = "Trop de tentatives erronées. Verrouillé pendant 30 secondes.";
    } else {
        if (errorMsg) errorMsg.innerText = `Code PIN incorrect (${5 - state.failedPinAttempts} essais restants)`;
    }

    setTimeout(() => {
        state.enteredPin = '';
        updateLockScreenUI();
    }, 600);
}

async function triggerBiometricsUnlock() {
    if (!state.appLock.enabled || !state.appLock.biometricsEnabled) return;
    if (window.PublicKeyCredential && typeof navigator.credentials !== 'undefined' && navigator.credentials.get) {
        try {
            const challenge = new Uint8Array(32);
            crypto.getRandomValues(challenge);
            const assertion = await navigator.credentials.get({
                publicKey: {
                    challenge,
                    timeout: 60000,
                    userVerification: 'preferred',
                    allowCredentials: [],
                }
            });
            if (assertion) {
                unlockApp();
            }
        } catch (e) {
            console.log('Biometrics auth fallback to PIN:', e);
        }
    }
}

function openSetupPinModal() {
    const modal = document.getElementById('setup-pin-modal');
    const input1 = document.getElementById('setup-pin-input');
    const input2 = document.getElementById('setup-pin-confirm');
    const err = document.getElementById('setup-pin-error');
    if (input1) input1.value = '';
    if (input2) input2.value = '';
    if (err) err.innerText = '';
    if (modal) modal.classList.add('show');
}

function closeSetupPinModal() {
    const modal = document.getElementById('setup-pin-modal');
    if (modal) modal.classList.remove('show');
}

async function saveNewPin() {
    const input1 = document.getElementById('setup-pin-input');
    const input2 = document.getElementById('setup-pin-confirm');
    const err = document.getElementById('setup-pin-error');
    const p1 = input1 ? input1.value.trim() : '';
    const p2 = input2 ? input2.value.trim() : '';

    if (!/^\d{4,6}$/.test(p1)) {
        if (err) err.innerText = 'Le code PIN doit comporter 4 à 6 chiffres numériques.';
        return;
    }
    if (p1 !== p2) {
        if (err) err.innerText = 'Les deux codes saisis ne correspondent pas.';
        return;
    }

    const salt = Array.from(crypto.getRandomValues(new Uint8Array(16))).map(b => b.toString(16).padStart(2, '0')).join('');
    const hash = await hashPin(p1, salt);

    state.appLock.pinHash = hash;
    state.appLock.pinSalt = salt;
    state.appLock.pinLength = p1.length;
    state.appLock.enabled = true;
    localStorage.setItem('nova_app_lock_enabled', 'true');
    localStorage.setItem('nova_app_lock_pin_hash', hash);
    localStorage.setItem('nova_app_lock_pin_salt', salt);
    localStorage.setItem('nova_app_lock_pin_length', String(p1.length));

    closeSetupPinModal();
    showToast('Code PIN de verrouillage configuré avec succès.');
    if (state.currentScreen === 'settings') {
        navigateTo('settings');
    }
}

function registerActivityListener() {
    const onUserActivity = () => {
        state.lastUserInteraction = Date.now();
    };

    ['pointerdown', 'keydown', 'touchstart', 'mousemove', 'scroll', 'click'].forEach(evt => {
        window.addEventListener(evt, onUserActivity, { passive: true, capture: true });
    });

    document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
            state.wentToBackgroundAt = Date.now();
            if (state.appLock.enabled && state.appLock.timeoutMin === 0) {
                lockApp();
            }
        } else {
            if (hasBackend) {
                tauriInvoke('drain_relay').then(async () => {
                    if (state.currentScreen === 'conversations') {
                        await refreshConversationsFromBackend();
                        updateGlobalUnreadBadges();
                        updateConversationsListDom();
                    } else if (state.currentScreen === 'chat' && state.activeContact) {
                        await refreshMessagesFromBackend(state.activeContact.conversationId);
                    }
                }).catch(() => {});
            }
            if (state.appLock.enabled && !state.isAppLocked) {
                const elapsed = Date.now() - (state.wentToBackgroundAt || state.lastUserInteraction);
                const timeoutMs = (state.appLock.timeoutMin || 0) * 60 * 1000;
                if (state.appLock.timeoutMin === 0 || elapsed >= timeoutMs) {
                    lockApp();
                }
            }
        }
    });

    // Global background drain and badge synchronization for non-chat/non-conversations screens
    setInterval(async () => {
        if (!hasBackend) return;
        if (state.currentScreen !== 'chat' && state.currentScreen !== 'conversations') {
            try {
                const count = await tauriInvoke('drain_relay');
                if (count > 0) {
                    await refreshConversationsFromBackend();
                    updateGlobalUnreadBadges();
                }
            } catch (_) {}
        }
    }, 3500);

    setInterval(() => {
        if (state.activeCall && (state.activeCall.state === 'connected' || state.activeCall.state === 'ringing')) {
            state.lastUserInteraction = Date.now();
            return;
        }
        if (state.appLock.enabled && !state.isAppLocked && state.appLock.pinHash) {
            const timeoutMs = (state.appLock.timeoutMin || 5) * 60 * 1000;
            if (timeoutMs > 0 && Date.now() - state.lastUserInteraction >= timeoutMs) {
                lockApp();
            }
        }
        if (state.isAppLocked && state.lockoutUntil > 0) {
            updateLockScreenUI();
        }
    }, 1000);
}

// Permanently removes a contact — unlike block/unblock, this also wipes the entire conversation
// history with them (see nova-storage's delete_contact), so it always asks for confirmation
// first, same as logoutReal()'s "this is destructive" confirm() pattern.
async function deleteContactReal(peerId, name) {
    const targetPeerId = peerId || (state.activeContact && (state.activeContact.peerId || state.activeContact.handle));
    if (!targetPeerId || !requireBackend()) return;
    const displayName = name || (state.activeContact && state.activeContact.name) || 'ce contact';
    if (!confirm(`Supprimer ${displayName} ? Tout l'historique de conversation avec ce contact sera aussi effacé définitivement. Cette action est irréversible.`)) {
        return;
    }
    try {
        await tauriInvoke('delete_contact', { peerId: targetPeerId });
        await refreshContactsFromBackend();
        await refreshConversationsFromBackend();
        if (state.activeContact && (state.activeContact.peerId === targetPeerId || state.activeContact.handle === targetPeerId)) {
            state.activeContact = null;
        }
        navigateTo('conversations');
    } catch (e) {
        alert('Échec de la suppression : ' + e);
    }
}

// Reads the target contact from data-* attributes rather than from an inline onclick argument.
// A contact's name/handle is peer-controlled data: splicing it directly into an onclick="..."
// attribute as a JS string literal is unsafe even when HTML-escaped, because the browser
// HTML-decodes attribute text *before* treating it as JavaScript source — so an escaped quote
// would simply decode back into a real quote and break out of the string right before it runs.
// Routing the value through `dataset` (a plain string property, never re-parsed as code) avoids
// that class of bug entirely.
function openChatWithEl(el) {
    openChatWith(el.dataset.name || '', el.dataset.handle || '', el.dataset.convId || '');
}

function openImagePreviewEl(el) {
    const url = el.dataset.url;
    if (!url) return;
    // Validation stricte : autoriser UNIQUEMENT data:image/ ou blob: (rejet de tout javascript:, file:, ou schémas externes)
    if (url.startsWith('data:image/') || url.startsWith('blob:')) {
        const win = window.open();
        if (win) {
            win.document.write(`
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Aperçu Image - NOVA Chat</title>
                    <meta charset="utf-8">
                    <style>
                        body { margin: 0; background: #080A10; display: flex; align-items: center; justify-content: center; height: 100vh; font-family: sans-serif; color: white; }
                        img { max-width: 95vw; max-height: 95vh; object-fit: contain; border-radius: 8px; box-shadow: 0 4px 24px rgba(0,0,0,0.85); }
                    </style>
                </head>
                <body>
                    <img src="${escapeHtml(url)}" alt="Aperçu image NOVA" />
                </body>
                </html>
            `);
            win.document.close();
        }
    } else {
        console.warn('Tentative d\'ouverture d\'une URL non autorisée bloquée :', url);
    }
}

function showFileAlertEl(el) {
    const filename = el.dataset.filename || '';
    alert('Document reçu : ' + filename);
}

function alertCallTo(el, kind) {
    alert('Les appels ' + (kind === 'video' ? 'vidéo' : 'vocaux') + ' ne sont pas encore proposés dans cette version.');
}

// --- MNEMONIC REVEAL GATE (authentification par PIN haché cryptographiquement) ---
let mnemonicAuthAttempts = 0;
let mnemonicAuthLockoutUntil = 0;
const MNEMONIC_AUTH_MAX_ATTEMPTS = 3;
const MNEMONIC_AUTH_LOCKOUT_MS = 60_000;

async function hashPinCode(pin, saltHex) {
    const enc = new TextEncoder();
    const data = enc.encode(saltHex + ':' + pin);
    const hashBuf = await crypto.subtle.digest('SHA-256', data);
    return Array.from(new Uint8Array(hashBuf)).map(b => b.toString(16).padStart(2, '0')).join('');
}

function openMnemonicAuthModal() {
    const modal = document.getElementById('mnemonic-auth-modal');
    const errorEl = document.getElementById('mnemonic-auth-error');
    const input = document.getElementById('mnemonic-auth-pin');
    const subtitle = modal ? modal.querySelector('p') : null;
    if (!modal) return;

    if (errorEl) errorEl.textContent = '';
    if (input) input.value = '';

    const hasStoredPin = !!localStorage.getItem('nova_pin_hash');
    if (subtitle) {
        subtitle.textContent = hasStoredPin
            ? 'Entrez votre code de sécurité pour afficher votre phrase secrète.'
            : 'Définissez votre code de sécurité (4 à 8 chiffres) pour protéger votre phrase secrète :';
    }

    modal.classList.add('show');
    if (input) setTimeout(() => input.focus(), 50);
}

function closeMnemonicAuthModal() {
    const modal = document.getElementById('mnemonic-auth-modal');
    if (modal) modal.classList.remove('show');
}

async function confirmMnemonicPin() {
    const errorEl = document.getElementById('mnemonic-auth-error');
    const input = document.getElementById('mnemonic-auth-pin');

    const now = Date.now();
    if (now < mnemonicAuthLockoutUntil) {
        const remaining = Math.ceil((mnemonicAuthLockoutUntil - now) / 1000);
        if (errorEl) errorEl.textContent = `Trop de tentatives. Réessayez dans ${remaining}s.`;
        return;
    }

    const entered = input ? input.value.trim() : '';
    if (!entered || entered.length < 4) {
        if (errorEl) errorEl.textContent = 'Le code doit comporter au moins 4 chiffres.';
        return;
    }

    const storedHash = localStorage.getItem('nova_pin_hash');
    let storedSalt = localStorage.getItem('nova_pin_salt');

    if (!storedHash) {
        // Premier enregistrement du code PIN
        const saltBytes = new Uint8Array(16);
        crypto.getRandomValues(saltBytes);
        storedSalt = Array.from(saltBytes).map(b => b.toString(16).padStart(2, '0')).join('');
        const newHash = await hashPinCode(entered, storedSalt);
        localStorage.setItem('nova_pin_hash', newHash);
        localStorage.setItem('nova_pin_salt', storedSalt);

        mnemonicAuthAttempts = 0;
        closeMnemonicAuthModal();
        if (state.currentUser.mnemonic) {
            showMnemonicDisplayModal(state.currentUser.mnemonic);
        } else {
            showToastNotification('Code de sécurité configuré.');
        }
        return;
    }

    // Vérification du code PIN existant
    const computedHash = await hashPinCode(entered, storedSalt);
    if (computedHash === storedHash) {
        mnemonicAuthAttempts = 0;
        closeMnemonicAuthModal();
        if (state.currentUser.mnemonic) {
            showMnemonicDisplayModal(state.currentUser.mnemonic);
        } else {
            showToastNotification('Créez d\'abord votre compte pour obtenir une phrase secrète.');
        }
        return;
    }

    mnemonicAuthAttempts++;
    if (mnemonicAuthAttempts >= MNEMONIC_AUTH_MAX_ATTEMPTS) {
        mnemonicAuthLockoutUntil = now + MNEMONIC_AUTH_LOCKOUT_MS;
        mnemonicAuthAttempts = 0;
        if (errorEl) errorEl.textContent = 'Trop de tentatives incorrectes. Verrouillé 60 secondes.';
    } else if (errorEl) {
        const remainingTries = MNEMONIC_AUTH_MAX_ATTEMPTS - mnemonicAuthAttempts;
        errorEl.textContent = `Code incorrect (${remainingTries} essai(s) restant(s)).`;
    }
    if (input) {
        input.value = '';
        input.focus();
    }
}

// --- FLOATING TOAST NOTIFICATION ---
let toastTimer = null;
function showToastNotification(message) {
    const toast = document.getElementById('toast-notification');
    if (!toast) return;
    toast.textContent = message;
    toast.classList.add('show');
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
        toast.classList.remove('show');
    }, 2800);
}
const showToast = showToastNotification;

// --- 12-WORD MNEMONIC MODAL & CLIPBOARD COPY ---
let pendingMnemonicConfirmCallback = null;

function showMnemonicDisplayModal(mnemonic, onConfirm) {
    const modal = document.getElementById('mnemonic-display-modal');
    const grid = document.getElementById('mnemonic-words-grid');
    const btnLabel = document.getElementById('copy-mnemonic-btn-label');
    if (!modal || !grid) {
        if (onConfirm) onConfirm();
        return;
    }
    pendingMnemonicConfirmCallback = onConfirm || null;
    const cleanMnemonic = (mnemonic || state.currentUser.mnemonic || '').trim();
    const words = cleanMnemonic.split(/\s+/).filter(Boolean);

    grid.innerHTML = words.map((word, i) => `
        <div class="mnemonic-chip">
            <span class="mnemonic-chip-num">${i + 1}.</span>
            <span class="mnemonic-chip-word">${escapeHtml(word)}</span>
        </div>
    `).join('');

    if (btnLabel) btnLabel.textContent = 'Copier la phrase secrète (12 mots)';
    modal.classList.add('show');
}

function closeMnemonicDisplayModal() {
    const modal = document.getElementById('mnemonic-display-modal');
    if (modal) modal.classList.remove('show');
    if (pendingMnemonicConfirmCallback) {
        const cb = pendingMnemonicConfirmCallback;
        pendingMnemonicConfirmCallback = null;
        cb();
    }
}

async function copyMnemonicPhrase() {
    const mnemonic = (state.currentUser.mnemonic || '').trim();
    if (!mnemonic) {
        showToastNotification('Aucune phrase secrète disponible');
        return;
    }
    try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(mnemonic);
        } else {
            const textarea = document.createElement('textarea');
            textarea.value = mnemonic;
            document.body.appendChild(textarea);
            textarea.select();
            document.execCommand('copy');
            document.body.removeChild(textarea);
        }
        const btnLabel = document.getElementById('copy-mnemonic-btn-label');
        if (btnLabel) btnLabel.textContent = '✓ 12 mots copiés !';
        showToastNotification('✓ Phrase de récupération (12 mots) copiée !');
    } catch (e) {
        showToastNotification('Phrase : ' + mnemonic);
    }
}

// --- IMAGE PREVIEW & MEDIA DISK STORAGE ---
let currentPreviewImageData = null;

function openImagePreviewEl(el) {
    const url = el.getAttribute('data-url') || el.src;
    const msgId = el.getAttribute('data-msg-id') || '';
    const filename = el.getAttribute('data-filename') || 'photo.jpg';
    if (!url) return;

    currentPreviewImageData = { url, msgId, filename };
    const modal = document.getElementById('image-preview-modal');
    const img = document.getElementById('image-preview-img');
    const title = document.getElementById('image-preview-filename');
    if (modal && img) {
        img.src = url;
        if (title) title.textContent = filename;
        modal.classList.add('show');
    }
}

function closeImagePreview() {
    const modal = document.getElementById('image-preview-modal');
    if (modal) modal.classList.remove('show');
    currentPreviewImageData = null;
}

async function downloadCurrentImage() {
    if (!currentPreviewImageData) return;
    if (currentPreviewImageData.msgId) {
        await saveAttachmentToDisk(currentPreviewImageData.msgId, currentPreviewImageData.filename);
    } else if (currentPreviewImageData.url) {
        const dateStr = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 15);
        const nameParts = (currentPreviewImageData.filename || 'photo.jpg').split('.');
        const ext = nameParts.length > 1 ? nameParts.pop() : 'jpg';
        const stem = nameParts.join('.') || 'photo';
        const link = document.createElement('a');
        link.download = `${stem}_${dateStr}.${ext}`;
        link.href = currentPreviewImageData.url;
        link.click();
        showToastNotification('✓ Photo téléchargée');
    }
}

async function saveAttachmentToDisk(msgId, suggestedFilename) {
    if (!hasBackend) {
        showToastNotification('Enregistrement réservé à l\'application native');
        return;
    }
    try {
        const savedPath = await tauriInvoke('save_attachment_to_disk', {
            messageId: String(msgId),
            suggestedFilename: suggestedFilename || null
        });
        const basename = savedPath.split(/[\\/]/).pop();
        showToastNotification(`✓ Enregistré dans Téléchargements/NOVA/${basename}`);
    } catch (e) {
        showToastNotification(`Erreur d'enregistrement : ${e}`);
    }
}

// --- DOCUMENT VIEWER, GPS LOCATION & VIDEO PLAYER SUITE ---
let currentViewerDoc = { msgId: null, filename: null, textContent: '', path: null };
let currentLocationData = { coords: '0,0', label: 'Position partagée' };
let currentVideoData = { url: null, msgId: null, filename: null };

function getFileIconForExtension(filename) {
    const ext = (filename || '').split('.').pop().toLowerCase();
    if (ext === 'pdf') return '📕';
    if (ext === 'doc' || ext === 'docx') return '📘';
    if (ext === 'xls' || ext === 'xlsx') return '📊';
    if (ext === 'ppt' || ext === 'pptx') return '📈';
    if (ext === 'apk' || ext === 'aab') return '📱';
    if (['exe', 'msi', 'dmg', 'app', 'deb', 'rpm', 'bin', 'iso'].includes(ext)) return '⚙️';
    if (['txt', 'md', 'markdown', 'log', 'json', 'csv', 'xml', 'yaml', 'yml', 'js', 'html', 'css', 'rs', 'py', 'sh', 'bat'].includes(ext)) return '📝';
    if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz'].includes(ext)) return '🗂️';
    if (['mp3', 'wav', 'ogg', 'm4a', 'aac', 'flac'].includes(ext)) return '🎵';
    if (['mp4', 'webm', 'mov', 'mkv', 'avi'].includes(ext)) return '🎥';
    return '📎';
}

async function openDocumentViewer(msgId, filename) {
    if (!msgId) return;
    currentViewerDoc = { msgId, filename: filename || 'document', textContent: '', path: null };

    const modal = document.getElementById('document-viewer-modal');
    const nameEl = document.getElementById('doc-viewer-filename');
    const sizeEl = document.getElementById('doc-viewer-filesize');
    const iconEl = document.getElementById('doc-viewer-icon');
    const textEl = document.getElementById('doc-viewer-text');
    const unsupportedEl = document.getElementById('doc-viewer-unsupported');
    const copyBtn = document.getElementById('btn-copy-doc-text');

    if (nameEl) nameEl.textContent = filename || 'document';
    if (iconEl) iconEl.textContent = getFileIconForExtension(filename);

    const ext = (filename || '').split('.').pop().toLowerCase();
    const isTextReadable = ['txt', 'md', 'markdown', 'json', 'csv', 'log', 'xml', 'yaml', 'yml', 'js', 'html', 'css', 'rs', 'py'].includes(ext);

    if (textEl) textEl.textContent = 'Chargement du contenu...';
    if (unsupportedEl) unsupportedEl.style.display = 'none';
    if (copyBtn) copyBtn.style.display = isTextReadable ? 'inline-flex' : 'none';

    if (modal) modal.classList.add('show');

    try {
        if (hasBackend) {
            const dataBase64 = await tauriInvoke('get_attachment_data', { messageId: String(msgId) });
            if (dataBase64) {
                const binStr = atob(dataBase64);
                const len = binStr.length;
                const bytes = new Uint8Array(len);
                for (let i = 0; i < len; i++) {
                    bytes[i] = binStr.charCodeAt(i);
                }
                if (sizeEl) sizeEl.textContent = formatFileSize(bytes.length);

                if (isTextReadable) {
                    const decoder = new TextDecoder('utf-8');
                    const text = decoder.decode(bytes);
                    currentViewerDoc.textContent = text;
                    if (textEl) {
                        textEl.textContent = text;
                        textEl.style.display = 'block';
                    }
                    if (unsupportedEl) unsupportedEl.style.display = 'none';
                } else {
                    if (textEl) textEl.style.display = 'none';
                    if (unsupportedEl) unsupportedEl.style.display = 'block';
                }
            } else {
                if (textEl) textEl.textContent = 'Impossible de charger les données du document.';
            }
        }
    } catch (e) {
        if (textEl) textEl.textContent = 'Erreur lors du chargement : ' + e;
    }
}

function closeDocumentViewer() {
    const modal = document.getElementById('document-viewer-modal');
    if (modal) modal.classList.remove('show');
    currentViewerDoc = { msgId: null, filename: null, textContent: '', path: null };
}

let pendingExecutableCallback = null;

function isExecutableOrRiskyFile(filename) {
    const ext = (filename || '').split('.').pop().toLowerCase();
    return [
        'apk', 'aab', 'xapk',
        'exe', 'msi', 'bat', 'cmd', 'ps1', 'vbs', 'wsf', 'scr', 'pif', 'reg', 'hta',
        'sh', 'bin', 'app', 'dmg', 'pkg', 'deb', 'rpm',
        'docm', 'xlsm', 'pptm', 'iso'
    ].includes(ext);
}

function openExecutableWarningModal(msgId, filename, onConfirm) {
    pendingExecutableCallback = onConfirm;
    const modal = document.getElementById('executable-warning-modal');
    const nameEl = document.getElementById('exec-warning-filename');
    const sizeEl = document.getElementById('exec-warning-size');
    const untrustedBox = document.getElementById('exec-warning-untrusted-box');

    if (nameEl) nameEl.textContent = filename || 'application.apk';
    if (sizeEl && currentViewerDoc.filesize) sizeEl.textContent = currentViewerDoc.filesize;

    const isUntrusted = state.activeContact && !state.activeContact.isTrusted;
    if (untrustedBox) {
        untrustedBox.style.display = isUntrusted ? 'block' : 'none';
    }

    if (modal) modal.classList.add('show');
}

function closeExecutableWarningModal() {
    const modal = document.getElementById('executable-warning-modal');
    if (modal) modal.classList.remove('show');
    pendingExecutableCallback = null;
}

async function confirmOpenExecutable() {
    const cb = pendingExecutableCallback;
    closeExecutableWarningModal();
    if (typeof cb === 'function') {
        await cb();
    }
}

async function proceedOpenSavedDocExternally() {
    if (!currentViewerDoc.msgId || !hasBackend) return;
    try {
        await tauriInvoke('save_and_open_attachment', {
            messageId: String(currentViewerDoc.msgId),
            suggestedFilename: currentViewerDoc.filename || null
        });
        showToastNotification('✓ Ouverture dans l\'application système...');
    } catch (e) {
        showToastNotification(`Échec de l'ouverture : ${e}`);
    }
}

async function openSavedDocExternally() {
    if (!currentViewerDoc.msgId || !hasBackend) return;
    if (isExecutableOrRiskyFile(currentViewerDoc.filename)) {
        openExecutableWarningModal(currentViewerDoc.msgId, currentViewerDoc.filename, proceedOpenSavedDocExternally);
    } else {
        await proceedOpenSavedDocExternally();
    }
}

async function saveCurrentDocument() {
    if (!currentViewerDoc.msgId) return;
    await saveAttachmentToDisk(currentViewerDoc.msgId, currentViewerDoc.filename);
}

async function copyDocTextContent() {
    if (!currentViewerDoc.textContent) return;
    try {
        await navigator.clipboard.writeText(currentViewerDoc.textContent);
        showToastNotification('✓ Contenu du document copié');
    } catch (e) {
        showToastNotification('Texte copié');
    }
}

// GPS Location Modal Handlers
function openLocationModal(coords, label) {
    currentLocationData = { coords: coords || '0,0', label: label || 'Position partagée' };
    const modal = document.getElementById('location-action-modal');
    const labelEl = document.getElementById('loc-modal-label');
    const coordsEl = document.getElementById('loc-modal-coords');

    if (labelEl) labelEl.textContent = currentLocationData.label;
    if (coordsEl) coordsEl.textContent = currentLocationData.coords;

    if (modal) modal.classList.add('show');
}

function closeLocationModal() {
    const modal = document.getElementById('location-action-modal');
    if (modal) modal.classList.remove('show');
}

function parseCoordinates(str) {
    if (!str) return { lat: 0, lon: 0 };
    const parts = str.split(',').map(s => parseFloat(s.trim()));
    if (parts.length >= 2 && !isNaN(parts[0]) && !isNaN(parts[1])) {
        return { lat: parts[0], lon: parts[1] };
    }
    return { lat: 0, lon: 0 };
}

function openInOpenStreetMap() {
    const { lat, lon } = parseCoordinates(currentLocationData.coords);
    const url = `https://www.openstreetmap.org/?mlat=${lat}&mlon=${lon}#map=16/${lat}/${lon}`;
    window.open(url, '_blank');
    closeLocationModal();
}

function openInGoogleMaps() {
    const { lat, lon } = parseCoordinates(currentLocationData.coords);
    const url = `https://www.google.com/maps?q=${lat},${lon}`;
    window.open(url, '_blank');
    closeLocationModal();
}

function openInNativeGps() {
    const { lat, lon } = parseCoordinates(currentLocationData.coords);
    const geoUri = `geo:${lat},${lon}?q=${lat},${lon}`;
    window.open(geoUri, '_blank');
    closeLocationModal();
}

async function copyGpsCoordinates() {
    try {
        await navigator.clipboard.writeText(currentLocationData.coords);
        showToastNotification('✓ Coordonnées GPS copiées');
    } catch (e) {
        showToastNotification('Coordonnées copiées');
    }
    closeLocationModal();
}

// Video Player Modal Handlers
function openVideoPlayerModal(url, msgId, filename) {
    currentVideoData = { url, msgId, filename: filename || 'video.mp4' };
    const modal = document.getElementById('video-player-modal');
    const videoEl = document.getElementById('video-player-element');
    const titleEl = document.getElementById('video-player-title');

    if (titleEl) titleEl.textContent = currentVideoData.filename;
    if (videoEl) {
        if (url) {
            videoEl.src = url;
            videoEl.play().catch(() => {});
        } else if (msgId && hasBackend) {
            // Lazy load if attachment blob is not yet cached
            tauriInvoke('get_attachment_data', { messageId: String(msgId) }).then(dataBase64 => {
                if (dataBase64) {
                    const videoUrl = `data:video/mp4;base64,${dataBase64}`;
                    currentVideoData.url = videoUrl;
                    videoEl.src = videoUrl;
                    videoEl.play().catch(() => {});
                }
            }).catch(console.error);
        }
    }

    if (modal) modal.classList.add('show');
}

function closeVideoPlayerModal() {
    const modal = document.getElementById('video-player-modal');
    const videoEl = document.getElementById('video-player-element');
    if (videoEl) {
        videoEl.pause();
        videoEl.src = '';
    }
    if (modal) modal.classList.remove('show');
    currentVideoData = { url: null, msgId: null, filename: null };
}

async function downloadCurrentVideo() {
    if (currentVideoData.msgId) {
        await saveAttachmentToDisk(currentVideoData.msgId, currentVideoData.filename);
    } else if (currentVideoData.url) {
        const link = document.createElement('a');
        link.download = currentVideoData.filename || 'video.mp4';
        link.href = currentVideoData.url;
        link.click();
        showToastNotification('✓ Vidéo téléchargée');
    }
}

async function refreshMediaFolderPath() {
    if (!hasBackend) return;
    try {
        const folderPath = await tauriInvoke('get_media_folder_path');
        const el = document.getElementById('settings-media-folder-path');
        if (el) el.textContent = folderPath;
    } catch (e) {
        // Non critical
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
    const safeText = escapeHtml(m.text);
    const safeMeta = escapeHtml(m.meta);
    const safeId = escapeHtml(m.id);

    let contentHtml = '';
    if (m.type === 'image') {
        contentHtml = `
            <div class="msg-photo-card">
                ${m.url ? `<img src="${escapeHtml(m.url)}" class="msg-image-thumb" data-url="${escapeHtml(m.url)}" data-msg-id="${safeId}" data-filename="${safeText || 'photo.jpg'}" data-action="openImagePreview" title="Cliquer pour agrandir" />` : `
                    <div class="msg-photo-preview">
                        ${icons.image}
                        <span style="font-size: 11px; font-weight: 600; color: white;">${safeText || 'Photo'}</span>
                        <span style="font-size: 10px; color: var(--text-dim);">${safeMeta || '1,8 Mo'}</span>
                    </div>
                `}
                <div style="padding: 6px 10px; font-size: 11px; color: var(--text-muted); display: flex; justify-content: space-between; align-items: center; gap: 8px;">
                    <span style="font-weight: 600; color: white; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 130px;">${safeText || 'Photo'}</span>
                    <div style="display: flex; align-items: center; gap: 6px;">
                        <span style="font-size: 10px; color: var(--text-dim);">${safeMeta || ''}</span>
                        <button class="icon-btn" style="width: 24px; height: 24px; color: var(--accent-purple-light); padding: 0;" data-action="saveAttachment" data-msg-id="${safeId}" data-filename="${safeText || 'photo.jpg'}" title="Enregistrer dans Téléchargements/NOVA">
                            ${icons.download}
                        </button>
                    </div>
                </div>
            </div>
        `;
    } else if (m.type === 'file') {
        const fileIcon = getFileIconForExtension(m.text || m.meta || '');
        contentHtml = `
            <div class="msg-file-card" data-msg-id="${safeId}" data-filename="${safeText || 'document'}" data-action="openDocumentViewer" style="cursor: pointer; display: flex; align-items: center; justify-content: space-between; gap: 10px;" title="Cliquer pour afficher / ouvrir">
                <div style="display: flex; align-items: center; gap: 10px; overflow: hidden; flex: 1;">
                    <span style="font-size: 20px; flex-shrink: 0;">${fileIcon}</span>
                    <div style="overflow: hidden; flex: 1;">
                        <div style="font-size: 13px; font-weight: 600; color: white; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 140px;">${safeText}</div>
                        <div style="font-size: 11px; color: var(--text-muted);">${safeMeta || 'Document'}</div>
                    </div>
                </div>
                <button class="icon-btn" style="width: 26px; height: 26px; color: var(--accent-purple-light); padding: 0; flex-shrink: 0;" data-action="saveAttachment" data-msg-id="${safeId}" data-filename="${safeText || 'document'}" title="Enregistrer dans Téléchargements/NOVA">
                    ${icons.download}
                </button>
            </div>
        `;
    } else if (m.type === 'video') {
        contentHtml = `
            <div class="msg-photo-card">
                ${m.url ? `
                    <div style="position: relative; cursor: pointer;" data-action="openVideoPlayerModal" data-url="${escapeHtml(m.url)}" data-msg-id="${safeId}" data-filename="${safeText || 'video.mp4'}" title="Agrandir la vidéo">
                        <video src="${escapeHtml(m.url)}" controls preload="metadata" class="msg-image-thumb" style="background:#000; display: block;"></video>
                    </div>
                ` : `
                    <div class="msg-photo-preview" data-action="openVideoPlayerModal" data-url="" data-msg-id="${safeId}" data-filename="${safeText || 'video.mp4'}" style="cursor: pointer;">
                        ${icons.video}
                        <span style="font-size: 11px; font-weight: 600; color: white;">${safeText || 'Vidéo'}</span>
                        <span style="font-size: 10px; color: var(--text-dim);">${safeMeta || ''}</span>
                    </div>
                `}
                <div style="padding: 6px 10px; font-size: 11px; color: var(--text-muted); display: flex; justify-content: space-between; align-items: center; gap: 8px;">
                    <span style="font-weight: 600; color: white; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 130px;">${safeText || 'Vidéo'}</span>
                    <div style="display: flex; align-items: center; gap: 6px;">
                        <span style="font-size: 10px; color: var(--text-dim);">${safeMeta || ''}</span>
                        <button class="icon-btn" style="width: 24px; height: 24px; color: var(--accent-purple-light); padding: 0;" data-action="saveAttachment" data-msg-id="${safeId}" data-filename="${safeText || 'video.mp4'}" title="Enregistrer dans Téléchargements/NOVA">
                            ${icons.download}
                        </button>
                    </div>
                </div>
            </div>
        `;
    } else if (m.type === 'voice') {
        contentHtml = `
            <div class="msg-voice-card" style="flex-direction: column; align-items: stretch; gap: 4px;">
                <div style="display: flex; align-items: center; gap: 6px;">
                    ${m.url ? `<audio controls preload="none" src="${escapeHtml(m.url)}" style="flex: 1; height: 36px;"></audio>` : `<span style="font-size: 12px; color: var(--text-dim);">Audio indisponible</span>`}
                    <button class="icon-btn" style="width: 24px; height: 24px; color: var(--text-muted); padding: 0;" data-action="saveAttachment" data-msg-id="${safeId}" data-filename="note_vocale_${safeId}.webm" title="Enregistrer la note vocale">
                        ${icons.download}
                    </button>
                </div>
                <span style="font-size: 11px; color: var(--text-muted); font-family: monospace;">${safeMeta || ''}</span>
            </div>
        `;
    } else if (m.type === 'location') {
        contentHtml = `
            <div class="msg-location-card" data-action="openLocationModal" data-coords="${safeText}" data-label="${safeMeta || 'Position partagée'}" style="cursor: pointer;" title="Cliquer pour afficher la carte">
                <div class="msg-location-header">
                    <div class="map-grid-lines"></div>
                    <div class="map-pin-pulse" style="width: 32px; height: 32px;">${icons.mapPin}</div>
                </div>
                <div class="msg-location-body">
                    <div style="font-size: 12px; font-weight: 700; color: white;">📍 Position partagée (Cliquer)</div>
                    <div style="font-size: 11px; color: var(--accent-purple-light); font-family: monospace; margin-top: 2px;">${safeText}</div>
                    <div style="font-size: 10px; color: var(--text-dim); margin-top: 4px;">${safeMeta || 'OpenStreetMap / Google Maps / GPS'}</div>
                </div>
            </div>
        `;
    } else if (m.type === 'call') {
        const isVideo = m.meta && m.meta.toLowerCase().includes('vidéo');
        contentHtml = `
            <div class="msg-call-card" style="display: flex; align-items: center; gap: 12px; padding: 10px 14px; background: rgba(139, 92, 246, 0.12); border-radius: var(--radius-md); border: 1px solid rgba(139, 92, 246, 0.25);">
                <div style="width: 38px; height: 38px; border-radius: 50%; background: linear-gradient(135deg, var(--accent-purple), #6366f1); color: white; display: flex; align-items: center; justify-content: center; flex-shrink: 0;">
                    ${isVideo ? icons.video : icons.phone}
                </div>
                <div>
                    <div style="font-size: 13px; font-weight: 700; color: white;">${safeText}</div>
                    <div style="font-size: 11px; color: var(--accent-purple-light); font-family: monospace; margin-top: 2px;">${safeMeta || 'Appel terminé'}</div>
                </div>
            </div>
        `;
    } else if (m.type === 'sticker') {
        contentHtml = `<div style="font-size: 28px; padding: 4px 8px;">${safeText}</div>`;
    } else {
        contentHtml = `<div class="msg-bubble">${safeText}</div>`;
    }

    const statusHtml = m.isOutgoing ? `
        <span id="msg-status-${safeId}" style="display: inline-flex; align-items: center;">
            ${messageStatusTickHtml(m)}
        </span>
    ` : '';

    return `
        <div class="msg-row ${m.isOutgoing ? 'msg-outgoing' : 'msg-incoming'}" id="msg-row-${safeId}" data-msg-id="${safeId}">
            <div class="msg-bubble-wrap">
                ${contentHtml}
                <button class="msg-more-btn" data-action="openMessageActionsModal" data-msg-id="${safeId}" title="Options du message">⋮</button>
            </div>
            <div class="msg-meta">
                <span>${escapeHtml(m.time)}</span>
                ${statusHtml}
            </div>
        </div>
    `;
}

function appendChatMessageToBody(msg) {
    if (!msg || msg.hidden || msg.type === 'call_signal') return;
    const chatBody = document.getElementById('chat-body');
    if (!chatBody) return;

    // Prevent duplicate DOM messages caused by concurrent polling
    const existing = document.getElementById(`msg-row-${msg.id}`);
    if (existing) {
        if (msg.isOutgoing && msg.status) {
            updateMessageStatus(msg.id, msg.status);
        }
        return;
    }
    
    // Remove typing indicator or empty chat placeholder if present before appending message
    const typingRow = document.getElementById('typing-indicator-row');
    if (typingRow) typingRow.remove();
    const emptyPlaceholder = chatBody.querySelector('.empty-chat-placeholder');
    if (emptyPlaceholder) emptyPlaceholder.remove();

    const wrapper = document.createElement('div');
    wrapper.innerHTML = buildMessageHtml(msg).trim();
    const newRow = wrapper.firstElementChild;
    chatBody.appendChild(newRow);

    if (msg.attachmentId && !msg.url) {
        ensureAttachmentLoaded(msg); // fire-and-forget — re-renders this bubble once loaded
    }

    // Reliable smooth scrolling anchored to bottom on both mobile & desktop
    requestAnimationFrame(() => {
        chatBody.scrollTop = chatBody.scrollHeight;
        if (newRow && typeof newRow.scrollIntoView === 'function') {
            newRow.scrollIntoView({ behavior: 'smooth', block: 'end' });
        }
    });
}

// Renders the delivery-tick (or failure + retry affordance) for one outgoing message bubble.
// A "failed" message shows a distinct icon and an inline "Réessayer" button — see
// retryFailedMessageReal — instead of silently staying stuck at a "sending" tick forever, which
// is what happened before the outbox had any give-up state to report at all.
function messageStatusTickHtml(m) {
    if (m.status === 'sending') return '<span class="status-tick-sending" title="Envoi en cours">⏳</span>';
    if (m.status === 'failed') {
        return `<span class="status-tick-failed" title="Échec de l'envoi">⚠</span>
            <button class="status-retry-btn" data-action="retryFailedMessage"
                data-msg-id="${escapeHtml(m.id)}" data-conv-id="${escapeHtml(m.conversationId)}" data-recipient-id="${escapeHtml(m.recipientId || '')}">Réessayer</button>`;
    }
    if (m.status === 'delivered' || m.status === 'read') return `<span class="status-tick-read" title="Reçu / Lu">${icons.checkCheck}</span>`;
    return '<span class="status-tick-sent" title="Envoyé">✓</span>';
}

function updateMessageStatus(msgId, status) {
    const target = state.messages.find(m => m.id === msgId);
    if (target) target.status = status;

    const el = document.getElementById(`msg-status-${msgId}`);
    if (el && target) {
        el.innerHTML = messageStatusTickHtml(target);
    }
}

// Re-attempts delivery of a message the outbox already gave up on (status "failed") — resets
// its bookkeeping in the backend under the same message id and puts it back in "sending" state
// in the UI while the fresh attempt is in flight.
async function retryFailedMessageReal(msgId, convId, recipientId) {
    if (!msgId || !convId || !recipientId || !requireBackend()) return;
    updateMessageStatus(msgId, 'sending');
    try {
        await tauriInvoke('retry_failed_message', {
            conversationId: convId,
            recipientPeerId: recipientId,
            messageId: msgId,
        });
        updateMessageStatus(msgId, 'sent');
    } catch (e) {
        updateMessageStatus(msgId, 'failed');
        alert('Le renvoi a échoué : ' + e);
    }
}

// --- WHATSAPP-STYLE MESSAGE CONTEXT ACTIONS (COPY, DELETE, FORWARD, SHARE, SAVE, EPHEMERAL) ---
function openMessageActionsModal(msgId) {
    if (!msgId) return;
    const msg = state.messages.find(m => m.id === msgId);
    if (!msg) return;
    state.selectedMessageId = msgId;

    const modal = document.getElementById('message-actions-modal');
    const preview = document.getElementById('msg-actions-preview');
    const saveMediaBtn = document.getElementById('btn-msg-save-media');
    const deleteEveryoneBtn = document.getElementById('btn-msg-delete-everyone');
    const reportBtn = document.getElementById('btn-msg-report');

    if (preview) {
        let snippet = msg.text || '';
        if (msg.type === 'photo') snippet = '📷 Photo: ' + (msg.text || msg.meta || 'Image');
        else if (msg.type === 'video') snippet = '🎥 Vidéo: ' + (msg.text || msg.meta || 'Vidéo');
        else if (msg.type === 'file') snippet = '📎 Fichier: ' + (msg.text || msg.meta || 'Document');
        else if (msg.type === 'voice') snippet = '🎤 Note vocale (' + (msg.meta || 'Audio') + ')';
        else if (msg.type === 'location') snippet = '📍 Position: ' + (msg.text || 'Coordonnées GPS');
        else if (msg.type === 'call') snippet = '📞 ' + (msg.text || 'Appel');
        preview.innerText = snippet || '(Message)';
    }

    if (saveMediaBtn) {
        const isMedia = msg.attachmentId || msg.type === 'photo' || msg.type === 'video' || msg.type === 'file' || msg.type === 'voice';
        saveMediaBtn.style.display = isMedia ? 'flex' : 'none';
    }

    if (deleteEveryoneBtn) {
        deleteEveryoneBtn.style.display = msg.isOutgoing ? 'flex' : 'none';
    }

    if (reportBtn) {
        reportBtn.style.display = !msg.isOutgoing ? 'flex' : 'none';
    }

    if (modal) modal.classList.add('show');
}

function closeMessageActionsModal() {
    const modal = document.getElementById('message-actions-modal');
    if (modal) modal.classList.remove('show');
}

async function copyMessageText() {
    if (!state.selectedMessageId) return;
    const msg = state.messages.find(m => m.id === state.selectedMessageId);
    if (!msg) return;

    const textToCopy = msg.text || msg.meta || '';
    if (textToCopy) {
        try {
            await navigator.clipboard.writeText(textToCopy);
            showToast('Message copié dans le presse-papier.');
        } catch (e) {
            showToast('Texte copié.');
        }
    }
    closeMessageActionsModal();
}

async function deleteMessageForMe() {
    if (!state.selectedMessageId) return;
    const msgId = state.selectedMessageId;
    closeMessageActionsModal();

    try {
        if (hasBackend) {
            await tauriInvoke('delete_message', { messageId: msgId });
        }
        state.messages = state.messages.filter(m => m.id !== msgId);
        const row = document.getElementById(`msg-row-${msgId}`);
        if (row) {
            row.style.transition = 'opacity 0.2s ease, transform 0.2s ease';
            row.style.opacity = '0';
            row.style.transform = 'scale(0.9)';
            setTimeout(() => row.remove(), 200);
        }
        showToast('Message supprimé pour vous.');
    } catch (e) {
        alert('Échec de suppression : ' + e);
    }
}

async function deleteMessageForEveryone() {
    if (!state.selectedMessageId) return;
    const msgId = state.selectedMessageId;
    const msg = state.messages.find(m => m.id === msgId);
    if (!msg) return;
    closeMessageActionsModal();

    if (!confirm('Voulez-vous supprimer ce message pour tous les participants ?')) return;

    try {
        msg.text = '🚫 Ce message a été supprimé';
        msg.type = 'text';
        msg.url = null;
        msg.attachmentId = null;

        const row = document.getElementById(`msg-row-${msgId}`);
        if (row) {
            const bubble = row.querySelector('.msg-bubble');
            if (bubble) {
                bubble.innerHTML = `<em style="color: var(--text-dim);">🚫 Ce message a été supprimé</em>`;
            }
        }
        showToast('Message supprimé pour tous.');
    } catch (e) {
        alert('Échec de la révocation : ' + e);
    }
}

async function shareMessageExternally() {
    if (!state.selectedMessageId) return;
    const msg = state.messages.find(m => m.id === state.selectedMessageId);
    if (!msg) return;
    closeMessageActionsModal();

    const text = msg.text || msg.meta || 'Message NOVA';
    if (navigator.share) {
        try {
            await navigator.share({
                title: 'Message NOVA',
                text: text,
            });
        } catch (e) {
            // Dismissed
        }
    } else {
        await navigator.clipboard.writeText(text);
        showToast('Texte copié (partage externe non supporté sur ce navigateur).');
    }
}

async function saveMessageMediaToDisk() {
    if (!state.selectedMessageId) return;
    const msg = state.messages.find(m => m.id === state.selectedMessageId);
    if (!msg) return;
    closeMessageActionsModal();

    const filename = msg.filename || msg.meta || `media_${msg.id}`;
    await saveAttachmentToDisk(msg.attachmentId || msg.id, filename);
}

function openForwardMessageModal() {
    if (!state.selectedMessageId) return;
    closeMessageActionsModal();

    const modal = document.getElementById('forward-message-modal');
    const container = document.getElementById('forward-contacts-list');
    if (!container || !modal) return;

    const contacts = state.contacts.filter(c => c.handle !== state.currentUser.peerId);
    if (contacts.length === 0) {
        container.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 20px;">Aucun contact disponible pour le transfert.</div>`;
    } else {
        container.innerHTML = contacts.map(c => `
            <div class="item-card" style="padding: 10px; cursor: pointer;" data-action="forwardToContact" data-peer-id="${escapeHtml(c.handle)}">
                <div class="avatar" style="width: 36px; height: 36px; font-size: 14px;">${escapeHtml(c.name.charAt(0))}</div>
                <div class="item-content">
                    <div class="item-name" style="font-size: 13px;">${escapeHtml(c.name)}</div>
                    <div class="item-sub" style="font-size: 11px;">@${escapeHtml(c.handle.slice(0, 16))}...</div>
                </div>
            </div>
        `).join('');
    }

    modal.classList.add('show');
}

function closeForwardMessageModal() {
    const modal = document.getElementById('forward-message-modal');
    if (modal) modal.classList.remove('show');
}

async function confirmForwardMessage(targetPeerId) {
    if (!state.selectedMessageId || !targetPeerId) return;
    const msg = state.messages.find(m => m.id === state.selectedMessageId);
    if (!msg) return;
    closeForwardMessageModal();

    try {
        const text = msg.text || msg.meta || '';
        const targetConvId = 'conv_' + targetPeerId;
        await tauriInvoke('send_message', {
            conversationId: targetConvId,
            recipientPeerId: targetPeerId,
            text: text ? `[Transféré] ${text}` : '[Média transféré]',
        });
        showToast('Message transféré avec succès.');
    } catch (e) {
        alert('Échec du transfert : ' + e);
    }
}

function openEphemeralTimerModal() {
    closeMessageActionsModal();
    const modal = document.getElementById('ephemeral-timer-modal');
    if (modal) modal.classList.add('show');
}

function closeEphemeralTimerModal() {
    const modal = document.getElementById('ephemeral-timer-modal');
    if (modal) modal.classList.remove('show');
}

function setConversationEphemeralTimer(mode) {
    if (!state.activeContact) return;
    const convId = state.activeContact.conversationId;
    localStorage.setItem('nova_ephemeral_' + convId, mode);
    closeEphemeralTimerModal();

    let label = 'Désactivé';
    if (mode === '3_msgs') label = '3 textos (A ↔ B ↔ A)';
    else if (mode === '24h') label = '24 heures';
    else if (mode === '7d') label = '7 jours';

    showToast(`Messages éphémères : ${label}`);
    checkEphemeralPurge(convId);
}

function checkEphemeralPurge(convId) {
    const mode = localStorage.getItem('nova_ephemeral_' + convId);
    if (!mode || mode === 'off') return;

    if (mode === '3_msgs') {
        const convMessages = state.messages.filter(m => m.conversationId === convId);
        if (convMessages.length > 3) {
            const toPurge = convMessages.slice(0, convMessages.length - 3);
            toPurge.forEach(async (m) => {
                state.messages = state.messages.filter(x => x.id !== m.id);
                const row = document.getElementById(`msg-row-${m.id}`);
                if (row) row.remove();
                if (hasBackend) {
                    try { await tauriInvoke('delete_message', { messageId: m.id }); } catch (_) {}
                }
            });
        }
    }
}

function reportCurrentMessage() {
    if (!state.selectedMessageId) return;
    const msg = state.messages.find(m => m.id === state.selectedMessageId);
    closeMessageActionsModal();
    if (!msg) return;

    const senderPeerId = msg.senderId || (state.activeContact && state.activeContact.peerId);
    const senderName = state.activeContact ? state.activeContact.name : 'cet utilisateur';
    openReportModal(senderPeerId, senderName);
}

// Touch long-press listener for mobile message bubbles
let touchTimer = null;
document.addEventListener('touchstart', (e) => {
    const row = e.target.closest('.msg-row');
    if (!row || !row.dataset.msgId) return;
    touchTimer = setTimeout(() => {
        openMessageActionsModal(row.dataset.msgId);
    }, 450);
}, { passive: true });

document.addEventListener('touchend', () => {
    if (touchTimer) {
        clearTimeout(touchTimer);
        touchTimer = null;
    }
}, { passive: true });

document.addEventListener('touchmove', () => {
    if (touchTimer) {
        clearTimeout(touchTimer);
        touchTimer = null;
    }
}, { passive: true });

// --- DEVICE FILE PICKERS & MEDIA INTEGRATION ---
function triggerDeviceMediaPicker() {
    const input = document.getElementById('media-file-input');
    if (input) input.click();
    closePanels();
}

function formatFileSize(bytes) {
    return bytes > 1024 * 1024 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB` : `${(bytes / 1024).toFixed(0)} KB`;
}

// Sends a File's actual bytes through the real chunked media pipeline (see
// nova_engine::NovaEngine::send_media) — split into MEDIA_CHUNK_SIZE wire packets on the Rust
// side, stored encrypted in their own attachment table, never inline in the message text.
async function sendFileAsStructuredMessage(file, mediaType, label, metaTextOverride, existingDataUrl, rawFileName, rawCaption) {
    if (!state.activeContact || !requireBackend()) return;
    if (file.size > MAX_MEDIA_BYTES) {
        alert(`Fichier trop volumineux (${formatFileSize(file.size)}). Limite actuelle : ${formatFileSize(MAX_MEDIA_BYTES)}.`);
        return;
    }

    const dataUrl = existingDataUrl || await new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = (e) => resolve(e.target.result);
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(file);
    });
    const commaIdx = dataUrl.indexOf(',');
    const dataBase64 = commaIdx >= 0 ? dataUrl.slice(commaIdx + 1) : dataUrl;
    // The dataUrl's own mime prefix reflects what was actually encoded (e.g. an image compressed
    // to JPEG via compressImageIfNeeded, even though `file.type` is still the original PNG) —
    // trust that first, falling back to the source file's type only when there's no dataUrl mime.
    const mimeType = (dataUrl.match(/^data:([^;]+);/) || [])[1] || file.type || 'application/octet-stream';

    const sizeStr = formatFileSize(file.size);
    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const metaText = metaTextOverride || sizeStr;

    const actualFileName = rawFileName || file.name || label || 'file';
    const actualCaption = (rawCaption !== undefined && rawCaption !== null) ? rawCaption : (label || actualFileName);

    try {
        const record = await tauriInvoke('send_media', {
            conversationId: state.activeContact.conversationId,
            recipientPeerId: state.activeContact.peerId,
            contentType: mediaType,
            fileName: actualFileName,
            mimeType,
            dataBase64,
            caption: actualCaption,
        });
        const newMsg = {
            id: record.id,
            conversationId: state.activeContact.conversationId,
            type: mediaType,
            text: actualCaption,
            meta: metaText,
            // Already have the bytes locally (we just uploaded them) — no need to round-trip
            // through get_attachment_data for our own just-sent message.
            url: (mediaType === 'image' || mediaType === 'voice' || mediaType === 'video') ? dataUrl : undefined,
            attachmentId: record.id,
            mimeType,
            time: timeStr,
            isOutgoing: true,
            status: 'sent',
        };
        state.messages.push(newMsg);
        appendChatMessageToBody(newMsg);
    } catch (e) {
        alert('Échec de l\'envoi du fichier : ' + e);
    }
}

// --- SYSTEMATIC MEDIA PREVIEW & CONFIRMATION BEFORE SENDING ---
let pendingMedia = null;

function openMediaPreviewModal(mediaInfo) {
    pendingMedia = mediaInfo;
    const modal = document.getElementById('media-preview-modal');
    const container = document.getElementById('media-preview-container');
    const titleEl = document.getElementById('media-preview-title');
    const captionInput = document.getElementById('media-caption-input');
    if (!modal || !container) return;

    if (captionInput) captionInput.value = '';

    if (mediaInfo.isImage) {
        if (titleEl) titleEl.innerHTML = `${icons.image} <span>Aperçu de la photo</span>`;
        container.innerHTML = `<img src="${mediaInfo.dataUrl}" style="max-width: 100%; max-height: 260px; object-fit: contain; border-radius: var(--radius-sm);" alt="${escapeHtml(mediaInfo.name)}">`;
    } else if (mediaInfo.isVideo) {
        if (titleEl) titleEl.innerHTML = `${icons.image} <span>Aperçu de la vidéo</span>`;
        container.innerHTML = `<video src="${mediaInfo.dataUrl}" controls autoplay muted style="max-width: 100%; max-height: 260px; border-radius: var(--radius-sm);"></video>`;
    } else {
        if (titleEl) titleEl.innerHTML = `${icons.file} <span>Aperçu du document</span>`;
        container.innerHTML = `
            <div style="padding: 24px 20px; text-align: center; display: flex; flex-direction: column; align-items: center; gap: 8px;">
                <div style="width: 56px; height: 56px; border-radius: var(--radius-md); background: var(--bg-surface-2); display: flex; align-items: center; justify-content: center; color: var(--accent-purple-light); border: 1px solid var(--border-subtle);">
                    ${icons.file}
                </div>
                <div style="font-weight: 600; font-size: 14px; color: white; word-break: break-all; max-width: 260px;">${escapeHtml(mediaInfo.name)}</div>
                <div style="font-size: 12px; color: var(--text-muted);">${formatFileSize(mediaInfo.size)}</div>
            </div>
        `;
    }

    modal.classList.add('show');
    closePanels();
}

function closeMediaPreviewModal() {
    const modal = document.getElementById('media-preview-modal');
    if (modal) modal.classList.remove('show');
    const container = document.getElementById('media-preview-container');
    if (container) container.innerHTML = '';
    pendingMedia = null;
}

async function confirmAndSendPendingMedia() {
    if (!pendingMedia || !state.activeContact) {
        closeMediaPreviewModal();
        return;
    }
    const captionInput = document.getElementById('media-caption-input');
    const caption = (captionInput && captionInput.value.trim()) || '';
    const { file, mediaType, dataUrl, name, size } = pendingMedia;
    closeMediaPreviewModal();

    const label = caption ? `${caption} (${name})` : (mediaType === 'image' ? (name.match(/\.(mp4|webm|mov)$/i) ? '🎬 ' : '📷 ') + name : '📄 ' + name);
    await sendFileAsStructuredMessage(file, mediaType, label, formatFileSize(size), dataUrl, name, caption || label);
}

async function compressImageIfNeeded(file, maxDimension = 2048, quality = 0.82) {
    if (!file.type.startsWith('image/') || file.type === 'image/gif' || file.type === 'image/svg+xml') {
        return new Promise((resolve) => {
            const r = new FileReader();
            r.onload = (e) => resolve({ dataUrl: e.target.result, size: file.size, name: file.name });
            r.readAsDataURL(file);
        });
    }

    return new Promise((resolve) => {
        const reader = new FileReader();
        reader.onload = (e) => {
            const img = new Image();
            img.onload = () => {
                let width = img.width;
                let height = img.height;

                if (width > maxDimension || height > maxDimension) {
                    if (width > height) {
                        height = Math.round((height * maxDimension) / width);
                        width = maxDimension;
                    } else {
                        width = Math.round((width * maxDimension) / height);
                        height = maxDimension;
                    }
                }

                const canvas = document.createElement('canvas');
                canvas.width = width;
                canvas.height = height;
                const ctx = canvas.getContext('2d');
                ctx.drawImage(img, 0, 0, width, height);

                const compressedDataUrl = canvas.toDataURL('image/jpeg', quality);
                const approxBytes = Math.round((compressedDataUrl.length * 3) / 4);
                const safeName = file.name.replace(/\.[^/.]+$/, "") + ".jpg";
                resolve({ dataUrl: compressedDataUrl, size: approxBytes, name: safeName });
            };
            img.onerror = () => {
                resolve({ dataUrl: e.target.result, size: file.size, name: file.name });
            };
            img.src = e.target.result;
        };
        reader.readAsDataURL(file);
    });
}

async function handleMediaFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    event.target.value = '';
    if (!file) return;

    const isVideo = file.type.startsWith('video') || /\.(mp4|webm|mov)$/i.test(file.name);
    const isImage = file.type.startsWith('image') || /\.(png|jpe?g|gif|webp|bmp|svg)$/i.test(file.name);
    const mediaType = isVideo ? 'video' : (isImage ? 'image' : 'file');

    // Auto-compress high-resolution camera photos if image
    if (isImage) {
        const processed = await compressImageIfNeeded(file);
        if (processed.size > MAX_MEDIA_BYTES) {
            alert(`Image trop volumineuse après compression (${formatFileSize(processed.size)}). Limite : ${formatFileSize(MAX_MEDIA_BYTES)}.`);
            return;
        }
        openMediaPreviewModal({
            file,
            mediaType,
            dataUrl: processed.dataUrl,
            name: processed.name,
            size: processed.size,
            isVideo: false,
            isImage: true,
        });
        return;
    }

    if (file.size > MAX_MEDIA_BYTES) {
        alert(`Fichier trop volumineux (${formatFileSize(file.size)}). Limite actuelle : ${formatFileSize(MAX_MEDIA_BYTES)}.`);
        return;
    }

    const reader = new FileReader();
    reader.onload = (e) => {
        const dataUrl = e.target.result;
        openMediaPreviewModal({
            file,
            mediaType,
            dataUrl,
            name: file.name,
            size: file.size,
            isVideo,
            isImage,
        });
    };
    reader.readAsDataURL(file);
}

function triggerDeviceDocPicker() {
    const input = document.getElementById('doc-file-input');
    if (input) input.click();
    closePanels();
}

function handleDocFileSelect(event) {
    const file = event.target.files && event.target.files[0];
    event.target.value = '';
    if (!file) return;

    if (file.size > MAX_MEDIA_BYTES) {
        alert(`Document trop volumineux (${formatFileSize(file.size)}). Limite actuelle : ${formatFileSize(MAX_MEDIA_BYTES)}.`);
        return;
    }

    const reader = new FileReader();
    reader.onload = (e) => {
        const dataUrl = e.target.result;
        openMediaPreviewModal({
            file,
            mediaType: 'file',
            dataUrl,
            name: file.name,
            size: file.size,
            isVideo: false,
            isImage: false,
        });
    };
    reader.readAsDataURL(file);
}

// --- WHATSAPP-LIKE VOICE RECORDER (RECORD, PAUSE, RESUME & SEND) ---
// Records real microphone audio via MediaRecorder — the resulting bytes are what actually gets
// sent (see stopAndSendVoiceRecording), not a simulation. The waveform bars during recording are
// a cosmetic animation, not a real-time visualization of the captured audio.
let voiceRecordingTimer = null;
let voiceRecordingSeconds = 0;
let voiceWaveformAnim = null;
let isVoiceRecordingPaused = false;
let isVoicePreviewPlaying = false;
let voicePreviewTimer = null;
let voicePreviewElapsed = 0;
let activeMediaRecorder = null;
let activeMicStream = null;
let recordedAudioChunks = [];

async function startVoiceRecording() {
    closePanels();
    const normalRow = document.getElementById('normal-input-row');
    const voiceBar = document.getElementById('voice-recording-bar');
    const micBtn = document.getElementById('mic-record-btn');

    if (!voiceBar || !normalRow || !state.activeContact) return;

    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        alert('Micro indisponible dans ce contexte.');
        return;
    }

    let stream;
    try {
        stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    } catch (e) {
        alert('Accès au microphone refusé ou indisponible : ' + e);
        return;
    }

    activeMicStream = stream;
    recordedAudioChunks = [];

    // Negotiate optimal audio MIME type across Android WebView, iOS WebKit, and Desktop browsers
    let recorderOptions = {};
    if (typeof MediaRecorder !== 'undefined' && MediaRecorder.isTypeSupported) {
        if (MediaRecorder.isTypeSupported('audio/webm;codecs=opus')) {
            recorderOptions = { mimeType: 'audio/webm;codecs=opus' };
        } else if (MediaRecorder.isTypeSupported('audio/mp4')) {
            recorderOptions = { mimeType: 'audio/mp4' };
        } else if (MediaRecorder.isTypeSupported('audio/aac')) {
            recorderOptions = { mimeType: 'audio/aac' };
        } else if (MediaRecorder.isTypeSupported('audio/ogg;codecs=opus')) {
            recorderOptions = { mimeType: 'audio/ogg;codecs=opus' };
        }
    }

    try {
        activeMediaRecorder = new MediaRecorder(stream, recorderOptions);
    } catch (err) {
        activeMediaRecorder = new MediaRecorder(stream);
    }

    activeMediaRecorder.ondataavailable = (e) => {
        if (e.data && e.data.size > 0) recordedAudioChunks.push(e.data);
    };
    activeMediaRecorder.start();

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

function stopMicCapture() {
    if (activeMediaRecorder && activeMediaRecorder.state !== 'inactive') {
        try { activeMediaRecorder.stop(); } catch (e) { /* already stopped */ }
    }
    if (activeMicStream) {
        activeMicStream.getTracks().forEach(t => t.stop());
    }
    activeMicStream = null;
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
        if (activeMediaRecorder && activeMediaRecorder.state === 'recording') activeMediaRecorder.pause();
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
        if (activeMediaRecorder && activeMediaRecorder.state === 'paused') activeMediaRecorder.resume();
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
    stopMicCapture();
    recordedAudioChunks = [];

    const normalRow = document.getElementById('normal-input-row');
    const voiceBar = document.getElementById('voice-recording-bar');
    const micBtn = document.getElementById('mic-record-btn');

    if (voiceBar) voiceBar.classList.remove('active', 'paused', 'previewing');
    if (normalRow) normalRow.style.display = 'flex';
    if (micBtn) micBtn.classList.remove('recording');

    isVoiceRecordingPaused = false;
    isVoicePreviewPlaying = false;
}

async function stopAndSendVoiceRecording() {
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

    const recorder = activeMediaRecorder;
    if (!state.activeContact || !recorder) {
        stopMicCapture();
        return;
    }

    const mimeType = recorder.mimeType || 'audio/webm';
    const blob = await new Promise((resolve) => {
        if (recorder.state === 'inactive') {
            resolve(new Blob(recordedAudioChunks, { type: mimeType }));
            return;
        }
        recorder.addEventListener('stop', () => {
            resolve(new Blob(recordedAudioChunks, { type: mimeType }));
        }, { once: true });
        recorder.stop();
    });
    stopMicCapture();
    recordedAudioChunks = [];

    if (blob.size === 0) {
        alert('Aucun son enregistré.');
        return;
    }

    const voiceFileName = `voice_${Date.now()}.webm`;
    await sendFileAsStructuredMessage(blob, 'voice', `🎤 Note vocale (${durationStr})`, durationStr, null, voiceFileName, `🎤 Note vocale (${durationStr})`);
}

// =============================================================================
// REAL-TIME P2P VOICE & VIDEO CALLS (SOVEREIGN WebRTC + NOVA ENCRYPTED SIGNALING)
// =============================================================================

const CallAudio = {
    ctx: null,
    intervalId: null,

    _init() {
        if (!this.ctx) {
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            if (AudioContext) {
                this.ctx = new AudioContext();
            }
        }
        if (this.ctx && this.ctx.state === 'suspended') {
            this.ctx.resume().catch(() => {});
        }
    },

    playOutgoingRinging() {
        this.stop();
        this._init();
        if (!this.ctx) return;

        const beep = () => {
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                const osc = this.ctx.createOscillator();
                const gain = this.ctx.createGain();
                osc.type = 'sine';
                osc.frequency.setValueAtTime(425, now);
                gain.gain.setValueAtTime(0.06, now);
                gain.gain.exponentialRampToValueAtTime(0.001, now + 1.2);
                osc.connect(gain);
                gain.connect(this.ctx.destination);
                osc.start(now);
                osc.stop(now + 1.2);
            } catch (e) {}
        };

        beep();
        this.intervalId = setInterval(beep, 3500);
    },

    playIncomingRingtone() {
        if (state.notificationPrefs && state.notificationPrefs.ringtoneEnabled === false) return;
        this.stop();
        this._init();
        if (!this.ctx) return;

        const ring = () => {
            if (!this.ctx) return;
            try {
                const now = this.ctx.currentTime;
                [523.25, 659.25, 783.99].forEach((freq, i) => {
                    const osc = this.ctx.createOscillator();
                    const gain = this.ctx.createGain();
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(freq, now + i * 0.14);
                    gain.gain.setValueAtTime(0.08, now + i * 0.14);
                    gain.gain.exponentialRampToValueAtTime(0.001, now + i * 0.14 + 0.7);
                    osc.connect(gain);
                    gain.connect(this.ctx.destination);
                    osc.start(now + i * 0.14);
                    osc.stop(now + i * 0.14 + 0.7);
                });
            } catch (e) {}
        };

        ring();
        this.intervalId = setInterval(ring, 2400);
    },

    playConnectedChime() {
        this.stop();
        this._init();
        if (!this.ctx) return;
        try {
            const now = this.ctx.currentTime;
            const osc = this.ctx.createOscillator();
            const gain = this.ctx.createGain();
            osc.type = 'sine';
            osc.frequency.setValueAtTime(587.33, now);
            osc.frequency.exponentialRampToValueAtTime(880.00, now + 0.18);
            gain.gain.setValueAtTime(0.1, now);
            gain.gain.exponentialRampToValueAtTime(0.001, now + 0.35);
            osc.connect(gain);
            gain.connect(this.ctx.destination);
            osc.start(now);
            osc.stop(now + 0.35);
        } catch (e) {}
    },

    playEndCallBeep() {
        this.stop();
        this._init();
        if (!this.ctx) return;
        try {
            const now = this.ctx.currentTime;
            const osc = this.ctx.createOscillator();
            const gain = this.ctx.createGain();
            osc.type = 'sine';
            osc.frequency.setValueAtTime(440, now);
            osc.frequency.exponentialRampToValueAtTime(220, now + 0.22);
            gain.gain.setValueAtTime(0.1, now);
            gain.gain.exponentialRampToValueAtTime(0.001, now + 0.3);
            osc.connect(gain);
            gain.connect(this.ctx.destination);
            osc.start(now);
            osc.stop(now + 0.3);
        } catch (e) {}
    },

    playMessageSentSound() {
        if (state.notificationPrefs && state.notificationPrefs.appSounds === false) return;
        this._init();
        if (!this.ctx) return;
        try {
            const now = this.ctx.currentTime;
            const osc = this.ctx.createOscillator();
            const gain = this.ctx.createGain();
            osc.type = 'sine';
            osc.frequency.setValueAtTime(600, now);
            osc.frequency.exponentialRampToValueAtTime(900, now + 0.08);
            gain.gain.setValueAtTime(0.05, now);
            gain.gain.exponentialRampToValueAtTime(0.001, now + 0.12);
            osc.connect(gain);
            gain.connect(this.ctx.destination);
            osc.start(now);
            osc.stop(now + 0.12);
        } catch (e) {}
    },

    playMessageReceivedSound() {
        if (state.notificationPrefs && state.notificationPrefs.appSounds === false) return;
        this._init();
        if (!this.ctx) return;
        try {
            const now = this.ctx.currentTime;
            const osc = this.ctx.createOscillator();
            const gain = this.ctx.createGain();
            osc.type = 'sine';
            osc.frequency.setValueAtTime(800, now);
            osc.frequency.exponentialRampToValueAtTime(1050, now + 0.09);
            gain.gain.setValueAtTime(0.06, now);
            gain.gain.exponentialRampToValueAtTime(0.001, now + 0.15);
            osc.connect(gain);
            gain.connect(this.ctx.destination);
            osc.start(now);
            osc.stop(now + 0.15);
        } catch (e) {}
    },

    stop() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }
};

function clearAppCache() {
    try {
        let freed = 0;
        state.messages.forEach(m => {
            if (m.url && m.url.startsWith('blob:')) {
                URL.revokeObjectURL(m.url);
                m.url = undefined;
                freed++;
            }
        });
        alert('Cache nettoyé avec succès ! ' + (freed > 0 ? (freed + ' aperçu(s) libéré(s).') : 'La mémoire est propre.'));
    } catch (e) {
        alert('Nettoyage du cache terminé.');
    }
}

let currentCall = null;
let pendingIncomingCall = null;
let callDurationSeconds = 0;
let callDurationTimer = null;
let pendingIceCandidates = [];
const processedCallSignalIds = new Set();

const RTC_ICE_CONFIG = {
    iceServers: [
        // STUN servers (Fast P2P NAT punch for Wi-Fi and open networks)
        { urls: 'stun:stun.l.google.com:19302' },
        { urls: 'stun:stun1.l.google.com:19302' },
        { urls: 'stun:stun2.l.google.com:19302' },
        { urls: 'stun:stun.cloudflare.com:3478' },
        { urls: 'stun:stun.services.mozilla.com' },

        // Free OpenRelay / Metered TURN servers (Traverse symmetric NATs on 4G/5G mobile carriers)
        {
            urls: 'turn:openrelay.metered.ca:80',
            username: 'openrelayproject',
            credential: 'openrelayproject'
        },
        {
            urls: 'turn:openrelay.metered.ca:443',
            username: 'openrelayproject',
            credential: 'openrelayproject'
        },
        {
            urls: 'turn:openrelay.metered.ca:443?transport=tcp',
            username: 'openrelayproject',
            credential: 'openrelayproject'
        },
        {
            urls: 'turns:openrelay.metered.ca:443?transport=tcp',
            username: 'openrelayproject',
            credential: 'openrelayproject'
        }
    ],
    iceCandidatePoolSize: 10
};

async function sendCallSignalToPeer(recipientPeerId, signalObj) {
    if (!recipientPeerId) return;

    // 1. Priorité absolue : Acheminement ultra-rapide (<100ms) via Edge Serverless Vercel
    const vercelPromise = fetch(`${VERCEL_API_BASE_URL}/api/signal`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            target_peer_id: recipientPeerId,
            sender_peer_id: state.currentUser.peerId || '',
            signal: signalObj
        })
    }).catch(err => console.warn('Vercel call signal delivery warning:', err));

    // 2. Canaux complémentaires P2P / Transport local si backend disponible
    if (hasBackend) {
        try {
            const payload = encodeStructuredMessage(signalObj);
            const conversationId = (state.activeContact && state.activeContact.conversationId) || ('conv_' + recipientPeerId);
            await tauriInvoke('send_message', {
                conversationId,
                recipientPeerId,
                text: payload,
            }).catch(() => {});
        } catch (e) {
            console.warn('send_message fallback warning:', e);
        }
    }

    await vercelPromise;
}

async function startRealtimeCall(type) {
    if (!state.activeContact) return;
    if (!requireBackend()) return;

    if (currentCall && currentCall.status !== 'ended') {
        alert('Un appel est déjà en cours.');
        return;
    }

    const peerId = state.activeContact.peerId || state.activeContact.handle;
    const peerName = state.activeContact.name;
    const callId = 'call_' + Date.now() + '_' + Math.random().toString(36).substring(2, 9);

    const constraints = {
        audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true },
        video: type === 'video' ? { facingMode: 'user', width: { ideal: 1280 }, height: { ideal: 720 } } : false
    };

    let localStream;
    try {
        localStream = await navigator.mediaDevices.getUserMedia(constraints);
    } catch (e) {
        alert(`Impossible d'accéder au micro/caméra : ${e.message}`);
        return;
    }

    const pc = new RTCPeerConnection(RTC_ICE_CONFIG);
    localStream.getTracks().forEach(track => pc.addTrack(track, localStream));

    currentCall = {
        callId,
        peerId,
        peerName,
        type,
        status: 'outgoing',
        isCaller: true,
        pc,
        localStream,
        isMuted: false,
        isVideoOff: false,
        facingMode: 'user',
        startTime: null
    };

    setupPeerConnectionHandlers(pc, callId, peerId, type);

    const localVideo = document.getElementById('call-local-video');
    const localPip = document.getElementById('call-local-pip');
    if (type === 'video' && localVideo) {
        localVideo.srcObject = localStream;
        if (localPip) localPip.style.display = 'block';
    } else if (localPip) {
        localPip.style.display = 'none';
    }

    showActiveCallModal(peerName, type, 'Sonnerie en cours...');
    CallAudio.playOutgoingRinging();

    try {
        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);
        await sendCallSignalToPeer(peerId, {
            kind: 'call_signal',
            signalId: 'sig_' + Date.now(),
            signalType: 'offer',
            callId,
            callType: type,
            sdp: offer,
            senderName: state.currentUser.name || 'Utilisateur NOVA'
        });
    } catch (e) {
        console.error('Échec de la création de l\'offre WebRTC', e);
        hangupCall(false, 'Erreur de connexion');
    }
}

function setupPeerConnectionHandlers(pc, callId, peerId, type) {
    pc.ontrack = (event) => {
        const [remoteStream] = event.streams;
        const remoteAudio = document.getElementById('call-remote-audio');
        const remoteVideo = document.getElementById('call-remote-video');
        if (remoteAudio) {
            remoteAudio.srcObject = remoteStream;
            remoteAudio.play().catch(() => {});
        }
        if (type === 'video' && remoteVideo) {
            remoteVideo.srcObject = remoteStream;
            remoteVideo.style.display = 'block';
            remoteVideo.play().catch(() => {});
            const voiceCenter = document.getElementById('call-voice-center');
            if (voiceCenter) voiceCenter.style.display = 'none';
        }
    };

    pc.onicecandidate = (event) => {
        if (event.candidate && currentCall && currentCall.callId === callId) {
            sendCallSignalToPeer(peerId, {
                kind: 'call_signal',
                signalId: 'sig_' + Date.now() + '_' + Math.random().toString(36).substring(2, 6),
                signalType: 'candidate',
                callId,
                candidate: event.candidate.toJSON()
            });
        }
    };

    pc.onconnectionstatechange = () => {
        if (!currentCall || currentCall.callId !== callId) return;
        if (pc.connectionState === 'connected') {
            CallAudio.playConnectedChime();
            setCallConnectedState();
        } else if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected' || pc.connectionState === 'closed') {
            hangupCall(false, 'Connexion terminée');
        }
    };
}

function showActiveCallModal(peerName, type, statusText) {
    const modal = document.getElementById('active-call-modal');
    if (!modal) return;

    const nameEl = document.getElementById('active-call-name');
    const avatarEl = document.getElementById('active-call-avatar');
    const statusEl = document.getElementById('active-call-status');
    const timerEl = document.getElementById('active-call-timer');
    const barName = document.getElementById('call-bar-peer-name');
    const barDuration = document.getElementById('call-bar-duration');

    const isVideo = type === 'video';
    const typePrefix = isVideo ? '📹 Appel Vidéo' : '📞 Appel Vocal';

    if (nameEl) nameEl.innerText = peerName;
    if (avatarEl) avatarEl.innerText = peerName.charAt(0).toUpperCase();
    if (statusEl) statusEl.innerText = `${typePrefix} • ${statusText || (isVideo ? 'Connexion vidéo...' : 'Sonnerie en cours...')}`;
    if (timerEl) timerEl.innerText = '00:00';
    if (barName) barName.innerText = `${typePrefix} — ${peerName}`;
    if (barDuration) barDuration.innerText = statusText || '00:00';

    const videoBtn = document.getElementById('call-btn-video');
    const flipBtn = document.getElementById('call-btn-camera-flip');
    const voiceCenter = document.getElementById('call-voice-center');
    const localPip = document.getElementById('call-local-pip');

    if (!isVideo) {
        if (videoBtn) videoBtn.style.display = 'none';
        if (flipBtn) flipBtn.style.display = 'none';
        if (localPip) localPip.style.display = 'none';
        if (voiceCenter) voiceCenter.style.display = 'flex';
    } else {
        if (videoBtn) videoBtn.style.display = 'flex';
        if (flipBtn) flipBtn.style.display = 'flex';
        if (localPip) localPip.style.display = 'block';
    }

    updateTorchButtonVisibility();
    modal.classList.add('show');
}

function setCallConnectedState() {
    if (!currentCall) return;
    currentCall.status = 'connected';
    currentCall.startTime = Date.now();

    const statusEl = document.getElementById('active-call-status');
    if (statusEl) statusEl.innerText = 'Connecté (chiffré)';

    if (callDurationTimer) clearInterval(callDurationTimer);
    callDurationSeconds = 0;
    callDurationTimer = setInterval(() => {
        callDurationSeconds++;
        const mins = String(Math.floor(callDurationSeconds / 60)).padStart(2, '0');
        const secs = String(callDurationSeconds % 60).padStart(2, '0');
        const timeStr = `${mins}:${secs}`;
        const timer1 = document.getElementById('active-call-timer');
        const timer2 = document.getElementById('call-bar-duration');
        if (timer1) timer1.innerText = timeStr;
        if (timer2) timer2.innerText = timeStr;
    }, 1000);
}

async function handleIncomingCallSignal(signal, msgRecord, conversationId) {
    if (!signal || !signal.callId) return;
    if (signal.signalId && processedCallSignalIds.has(signal.signalId)) return;
    if (signal.signalId) processedCallSignalIds.add(signal.signalId);

    const senderPeerId = msgRecord ? (msgRecord.sender_id || msgRecord.recipient_id) : (signal.senderPeerId || '');

    if (signal.signalType === 'offer') {
        if (currentCall && currentCall.status !== 'ended') {
            await sendCallSignalToPeer(senderPeerId, {
                kind: 'call_signal',
                signalId: 'sig_' + Date.now(),
                signalType: 'busy',
                callId: signal.callId
            });
            return;
        }

        pendingIncomingCall = {
            callId: signal.callId,
            callType: signal.callType || 'voice',
            sdp: signal.sdp,
            senderPeerId,
            senderName: signal.senderName || 'Contact',
            conversationId
        };
        pendingIceCandidates = [];

        const isVideo = signal.callType === 'video';
        const contactMatch = state.contacts.find(c => c.handle === senderPeerId || c.peerId === senderPeerId);
        const displayName = (contactMatch && contactMatch.name) || pendingIncomingCall.senderName || 'Contact';

        const nameEl = document.getElementById('incoming-call-name');
        const avatarEl = document.getElementById('incoming-call-avatar');
        const iconEl = document.getElementById('incoming-call-type-icon');
        const labelEl = document.getElementById('incoming-call-type-label');

        if (nameEl) nameEl.innerText = displayName;
        if (avatarEl) avatarEl.innerText = displayName.charAt(0).toUpperCase();
        if (iconEl) iconEl.innerText = isVideo ? '📹' : '📞';
        if (labelEl) labelEl.innerText = isVideo ? 'Appel vidéo entrant...' : 'Appel vocal entrant...';

        const acceptBtnSpan = document.querySelector('#incoming-call-modal .btn-accept span');
        if (acceptBtnSpan) {
            acceptBtnSpan.innerText = isVideo ? 'Décrocher Vidéo' : 'Décrocher';
        }

        const incomingModal = document.getElementById('incoming-call-modal');
        if (incomingModal) incomingModal.classList.add('show');
        CallAudio.playIncomingRingtone();
    } else if (signal.signalType === 'answer') {
        if (currentCall && currentCall.callId === signal.callId && currentCall.isCaller && currentCall.pc) {
            CallAudio.stop();
            try {
                await currentCall.pc.setRemoteDescription(new RTCSessionDescription(signal.sdp));
                for (const cand of pendingIceCandidates) {
                    await currentCall.pc.addIceCandidate(new RTCIceCandidate(cand)).catch(() => {});
                }
                pendingIceCandidates = [];
            } catch (e) {
                console.error('Erreur lors de la définition de la réponse WebRTC', e);
            }
        }
    } else if (signal.signalType === 'candidate') {
        if (signal.candidate) {
            if (currentCall && currentCall.callId === signal.callId && currentCall.pc && currentCall.pc.remoteDescription) {
                currentCall.pc.addIceCandidate(new RTCIceCandidate(signal.candidate)).catch(() => {});
            } else {
                pendingIceCandidates.push(signal.candidate);
            }
        }
    } else if (signal.signalType === 'reject') {
        if (currentCall && currentCall.callId === signal.callId) {
            hangupCall(false, 'Appel refusé');
        }
    } else if (signal.signalType === 'busy') {
        if (currentCall && currentCall.callId === signal.callId) {
            hangupCall(false, 'Correspondant déjà en ligne');
        }
    } else if (signal.signalType === 'hangup') {
        if (currentCall && currentCall.callId === signal.callId) {
            hangupCall(false, 'Appel terminé par le correspondant');
        }
        if (pendingIncomingCall && pendingIncomingCall.callId === signal.callId) {
            rejectIncomingCallReal(false);
        }
    }
}

async function acceptIncomingCallReal() {
    if (!pendingIncomingCall) return;
    const callData = pendingIncomingCall;
    pendingIncomingCall = null;
    CallAudio.stop();

    const incomingModal = document.getElementById('incoming-call-modal');
    if (incomingModal) incomingModal.classList.remove('show');

    const constraints = {
        audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true },
        video: callData.callType === 'video' ? { facingMode: 'user', width: { ideal: 1280 }, height: { ideal: 720 } } : false
    };

    let localStream;
    try {
        localStream = await navigator.mediaDevices.getUserMedia(constraints);
    } catch (e) {
        alert(`Impossible d'accéder au micro/caméra : ${e.message}`);
        await sendCallSignalToPeer(callData.senderPeerId, {
            kind: 'call_signal',
            signalId: 'sig_' + Date.now(),
            signalType: 'reject',
            callId: callData.callId
        });
        return;
    }

    const pc = new RTCPeerConnection(RTC_ICE_CONFIG);
    localStream.getTracks().forEach(track => pc.addTrack(track, localStream));

    currentCall = {
        callId: callData.callId,
        peerId: callData.senderPeerId,
        peerName: callData.senderName,
        type: callData.callType,
        status: 'connecting',
        isCaller: false,
        pc,
        localStream,
        isMuted: false,
        isVideoOff: false,
        facingMode: 'user',
        startTime: null,
        conversationId: callData.conversationId
    };

    setupPeerConnectionHandlers(pc, callData.callId, callData.senderPeerId, callData.callType);

    const localVideo = document.getElementById('call-local-video');
    const localPip = document.getElementById('call-local-pip');
    if (callData.callType === 'video' && localVideo) {
        localVideo.srcObject = localStream;
        if (localPip) localPip.style.display = 'block';
    } else if (localPip) {
        localPip.style.display = 'none';
    }

    showActiveCallModal(callData.senderName, callData.callType, 'Connexion sécurisée...');

    try {
        await pc.setRemoteDescription(new RTCSessionDescription(callData.sdp));
        for (const cand of pendingIceCandidates) {
            await pc.addIceCandidate(new RTCIceCandidate(cand)).catch(() => {});
        }
        pendingIceCandidates = [];

        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);

        await sendCallSignalToPeer(callData.senderPeerId, {
            kind: 'call_signal',
            signalId: 'sig_' + Date.now(),
            signalType: 'answer',
            callId: callData.callId,
            sdp: answer
        });
    } catch (e) {
        console.error('Échec de la réponse WebRTC', e);
        hangupCall(false, 'Erreur de négociation');
    }
}

async function rejectIncomingCallReal(notify = true) {
    if (!pendingIncomingCall) return;
    const callData = pendingIncomingCall;
    pendingIncomingCall = null;
    CallAudio.stop();

    const incomingModal = document.getElementById('incoming-call-modal');
    if (incomingModal) incomingModal.classList.remove('show');

    if (notify && callData.senderPeerId) {
        await sendCallSignalToPeer(callData.senderPeerId, {
            kind: 'call_signal',
            signalId: 'sig_' + Date.now(),
            signalType: 'reject',
            callId: callData.callId
        });
    }
}

async function hangupCall(notifyPeer = true, reason = '') {
    CallAudio.stop();
    if (callDurationTimer) {
        clearInterval(callDurationTimer);
        callDurationTimer = null;
    }

    const modal1 = document.getElementById('incoming-call-modal');
    const modal2 = document.getElementById('active-call-modal');
    if (modal1) modal1.classList.remove('show');
    if (modal2) modal2.classList.remove('show');

    if (!currentCall) return;
    const callSnapshot = currentCall;
    currentCall = null;

    if (notifyPeer && callSnapshot.peerId) {
        sendCallSignalToPeer(callSnapshot.peerId, {
            kind: 'call_signal',
            signalId: 'sig_' + Date.now(),
            signalType: 'hangup',
            callId: callSnapshot.callId
        });
    }

    if (callSnapshot.localStream) {
        callSnapshot.localStream.getTracks().forEach(track => {
            try { track.stop(); } catch (e) {}
        });
    }

    if (callSnapshot.pc) {
        try { callSnapshot.pc.close(); } catch (e) {}
    }

    const localVideo = document.getElementById('call-local-video');
    const remoteVideo = document.getElementById('call-remote-video');
    const remoteAudio = document.getElementById('call-remote-audio');
    const torchBtn = document.getElementById('call-btn-torch');
    if (localVideo) localVideo.srcObject = null;
    if (remoteVideo) remoteVideo.srcObject = null;
    if (remoteAudio) remoteAudio.srcObject = null;
    if (torchBtn) {
        torchBtn.classList.remove('active-torch');
        torchBtn.style.display = 'none';
    }

    CallAudio.playEndCallBeep();

    if (callDurationSeconds > 0) {
        const mins = String(Math.floor(callDurationSeconds / 60)).padStart(2, '0');
        const secs = String(callDurationSeconds % 60).padStart(2, '0');
        const durationStr = `${mins}:${secs}`;
        const callLabel = callSnapshot.type === 'video' ? 'Appel vidéo' : 'Appel vocal';

        if (callSnapshot.isCaller) {
            sendStructuredCallSummary(callSnapshot.peerId, callLabel, `${durationStr} (sécurisé)`);
        }
    }
}

async function sendStructuredCallSummary(peerId, label, duration) {
    if (!hasBackend || !peerId) return;
    try {
        const payload = encodeStructuredMessage({
            kind: 'call_log',
            label,
            duration
        });
        const conversationId = (state.activeContact && state.activeContact.conversationId) || ('conv_' + peerId);
        await tauriInvoke('send_message', {
            conversationId,
            recipientPeerId: peerId,
            text: payload,
        });
        await refreshConversationsFromBackend();
        if (state.activeContact && state.activeContact.conversationId) {
            await refreshMessagesFromBackend(state.activeContact.conversationId);
        }
    } catch (e) {
        console.error('sendStructuredCallSummary failed', e);
    }
}

function toggleCallMute() {
    if (!currentCall || !currentCall.localStream) return;
    currentCall.isMuted = !currentCall.isMuted;
    currentCall.localStream.getAudioTracks().forEach(track => {
        track.enabled = !currentCall.isMuted;
    });

    const muteBtn = document.getElementById('call-btn-mute');
    const muteLabel = document.getElementById('call-mute-label');
    const pipMuted = document.getElementById('pip-self-muted');

    if (muteBtn) muteBtn.classList.toggle('active-off', currentCall.isMuted);
    if (muteLabel) muteLabel.innerText = currentCall.isMuted ? 'Coupé' : 'Micro';
    if (pipMuted) pipMuted.style.display = currentCall.isMuted ? 'block' : 'none';
}

function toggleCallVideo() {
    if (!currentCall || !currentCall.localStream) return;
    currentCall.isVideoOff = !currentCall.isVideoOff;
    currentCall.localStream.getVideoTracks().forEach(track => {
        track.enabled = !currentCall.isVideoOff;
    });

    const videoBtn = document.getElementById('call-btn-video');
    const videoLabel = document.getElementById('call-video-label');
    const localPip = document.getElementById('call-local-pip');

    if (videoBtn) videoBtn.classList.toggle('active-off', currentCall.isVideoOff);
    if (videoLabel) videoLabel.innerText = currentCall.isVideoOff ? 'Éteinte' : 'Caméra';
    if (localPip) localPip.style.opacity = currentCall.isVideoOff ? '0.3' : '1';
}

async function switchCallCamera() {
    if (!currentCall || !currentCall.localStream || currentCall.type !== 'video') return;

    if (currentCall.isTorchOn) {
        toggleCallTorch(false);
    }

    const nextMode = (currentCall.facingMode === 'user') ? 'environment' : 'user';
    currentCall.facingMode = nextMode;

    const oldVideoTrack = currentCall.localStream.getVideoTracks()[0];
    if (oldVideoTrack) {
        currentCall.localStream.removeTrack(oldVideoTrack);
        try { oldVideoTrack.stop(); } catch (e) {}
    }

    let newStream = null;
    try {
        newStream = await navigator.mediaDevices.getUserMedia({
            video: {
                facingMode: { ideal: nextMode },
                width: { ideal: 1280 },
                height: { ideal: 720 }
            }
        });
    } catch (err1) {
        try {
            newStream = await navigator.mediaDevices.getUserMedia({
                video: { facingMode: nextMode }
            });
        } catch (err2) {
            try {
                newStream = await navigator.mediaDevices.getUserMedia({ video: true });
            } catch (err3) {
                console.error('All camera switch attempts failed', err3);
            }
        }
    }

    if (newStream) {
        const newVideoTrack = newStream.getVideoTracks()[0];
        if (newVideoTrack) {
            currentCall.localStream.addTrack(newVideoTrack);

            if (currentCall.pc) {
                const sender = currentCall.pc.getSenders().find(s => s.track && s.track.kind === 'video');
                if (sender) {
                    await sender.replaceTrack(newVideoTrack);
                }
            }

            const localVideo = document.getElementById('call-local-video');
            if (localVideo) {
                localVideo.srcObject = currentCall.localStream;
                localVideo.style.transform = currentCall.facingMode === 'user' ? 'scaleX(-1)' : 'none';
            }
        }
    }

    updateTorchButtonVisibility();
}

function updateTorchButtonVisibility() {
    const torchBtn = document.getElementById('call-btn-torch');
    if (!torchBtn) return;
    if (currentCall && currentCall.type === 'video' && currentCall.facingMode === 'environment') {
        torchBtn.style.display = 'flex';
    } else {
        torchBtn.style.display = 'none';
        if (currentCall && currentCall.isTorchOn) {
            toggleCallTorch(false);
        }
    }
}

async function toggleCallTorch(forceState) {
    if (!currentCall || !currentCall.localStream || currentCall.type !== 'video') return;
    const videoTrack = currentCall.localStream.getVideoTracks()[0];
    if (!videoTrack) return;

    const desiredState = typeof forceState === 'boolean' ? forceState : !currentCall.isTorchOn;

    try {
        if (typeof videoTrack.applyConstraints === 'function') {
            await videoTrack.applyConstraints({
                advanced: [{ torch: desiredState }]
            });
            currentCall.isTorchOn = desiredState;
        }
    } catch (e) {
        console.warn('Torch constraint not supported on this track/camera', e);
        currentCall.isTorchOn = false;
    }

    const torchBtn = document.getElementById('call-btn-torch');
    const torchLabel = document.getElementById('call-torch-label');
    if (torchBtn) {
        torchBtn.classList.toggle('active-torch', !!currentCall.isTorchOn);
    }
    if (torchLabel) {
        torchLabel.innerText = currentCall.isTorchOn ? 'Allumé' : 'Flash';
    }
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
                    text: `${lat}° N, ${lon}° E (position exacte)`
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

async function confirmAndSendLocation() {
    closeLocationModal();
    if (!state.activeContact) return;
    if (!requireBackend()) return;

    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const payload = encodeStructuredMessage({
        kind: 'location',
        lat: pendingCoordinates.lat,
        lon: pendingCoordinates.lon,
        label: pendingCoordinates.text,
    });

    try {
        const record = await tauriInvoke('send_message', {
            conversationId: state.activeContact.conversationId,
            recipientPeerId: state.activeContact.peerId,
            text: payload,
        });
        const newMsg = {
            id: record.id,
            conversationId: state.activeContact.conversationId,
            type: 'location',
            text: pendingCoordinates.text,
            meta: 'Précision d\'environ 5 mètres',
            time: timeStr,
            isOutgoing: true,
            status: 'sent',
        };
        state.messages.push(newMsg);
        appendChatMessageToBody(newMsg);
    } catch (e) {
        alert('Échec de l\'envoi de la position : ' + e);
    }
}

// Text send path, wired to the send button and the Enter key. Storage is raw text; escaping
// happens once, at render time, in buildMessageHtml — never here, so the value is never escaped
// twice. Encrypts (X3DH/Double Ratchet) and queues the message for real delivery via nova-transport.
// `status` stays 'sending' until the outbox pump actually hands it off (direct or via relay) —
// the next poll of get_messages picks up the authoritative status from local storage.
async function sendMessage() {
    const input = document.getElementById('chat-input');
    if (!input || !state.activeContact) return;
    if (!requireBackend()) return;
    const text = input.value.trim();
    if (!text) return;

    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const conversationId = state.activeContact.conversationId;

    try {
        let record;
        if (state.activeContact.isGroup) {
            record = await tauriInvoke('send_group_message', {
                groupId: state.activeContact.peerId,
                text,
            });
        } else {
            record = await tauriInvoke('send_message', {
                conversationId,
                recipientPeerId: state.activeContact.peerId,
                text,
            });
        }
        const newMsg = {
            id: record.id,
            conversationId,
            type: 'text',
            text: record.text_content,
            time: timeStr,
            isOutgoing: true,
            status: 'sent',
        };
        state.messages.push(newMsg);
        appendChatMessageToBody(newMsg);
        CallAudio.playMessageSentSound();
        input.value = '';
        checkEphemeralPurge(conversationId);
    } catch (e) {
        alert('Échec de l\'envoi : ' + e);
    }
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

// --- LIVE CAMERA & IMAGE QR SCANNER ---
let qrScannerStream = null;
let qrScannerAnimId = null;

function showQrScannerError(message) {
    const errBox = document.getElementById('qr-camera-error');
    if (!errBox) return;
    const msgEl = errBox.querySelector('span');
    if (msgEl) msgEl.textContent = message;
    errBox.style.display = 'flex';
}

async function openQrCameraScanner() {
    const modal = document.getElementById('qr-camera-modal');
    const video = document.getElementById('qr-scanner-video');
    const errBox = document.getElementById('qr-camera-error');
    if (!modal || !video) return;

    // Ignore a second trigger while a scan session is already active (e.g. a rapid double-tap on
    // the "Scanner" button before the first getUserMedia() promise settles) — without this, the
    // first camera stream's tracks were never stopped (closeQrCameraScanner() was never called
    // in between), leaking the camera lock and battery, and some mobile browsers reject the
    // second concurrent getUserMedia() call outright even though permission was already granted.
    if (qrScannerStream) return;

    modal.classList.add('show');
    if (errBox) errBox.style.display = 'none';

    if (typeof jsQR === 'undefined') {
        // The live per-frame scan loop below used to check this exact same condition silently
        // and just loop forever doing nothing if it was ever true ("ça scanne à l'infini" with a
        // camera that visibly works but never reacts) — checked once here instead, now failing
        // exactly as visibly as the image-import path (handleQrImageSelect) already did.
        showQrScannerError('Décodeur QR indisponible.');
        return;
    }

    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
        showQrScannerError('Caméra indisponible ou permission non accordée.');
        return;
    }

    try {
        qrScannerStream = await navigator.mediaDevices.getUserMedia({
            video: {
                facingMode: { ideal: 'environment' },
                // Higher than the previous 1280x720 ideal: the invitation QR's own module count
                // is already at the low end error-correction can allow (see renderOwnQrCode), so
                // squeezing more real camera pixels per module here is the other half of making a
                // scan of another phone's on-screen code actually resolve instead of coming back
                // empty every single frame ("ça scanne à l'infini"). Still just a hint — falls back
                // to whatever the device actually offers, same as the plain getUserMedia retry below.
                width: { ideal: 1920 },
                height: { ideal: 1080 },
            }
        });
        video.srcObject = qrScannerStream;
        await video.play();
        requestQrScanFrame();
    } catch (err) {
        console.warn('getUserMedia with constraints failed, trying default video', err);
        try {
            qrScannerStream = await navigator.mediaDevices.getUserMedia({ video: true });
            video.srcObject = qrScannerStream;
            await video.play();
            requestQrScanFrame();
        } catch (err2) {
            console.error('Camera access completely failed or denied', err2);
            showQrScannerError('Caméra indisponible ou permission non accordée.');
        }
    }
}

function closeQrCameraScanner() {
    const modal = document.getElementById('qr-camera-modal');
    if (modal) modal.classList.remove('show');

    if (qrScannerAnimId) {
        cancelAnimationFrame(qrScannerAnimId);
        qrScannerAnimId = null;
    }

    if (qrScannerStream) {
        try {
            qrScannerStream.getTracks().forEach(track => track.stop());
        } catch (e) {}
        qrScannerStream = null;
    }

    const video = document.getElementById('qr-scanner-video');
    if (video) {
        video.srcObject = null;
    }
}

function requestQrScanFrame() {
    if (!qrScannerStream) return;
    qrScannerAnimId = requestAnimationFrame(scanQrCameraFrame);
}

function scanQrCameraFrame() {
    const video = document.getElementById('qr-scanner-video');
    const canvas = document.getElementById('qr-scanner-canvas');
    if (!video || !canvas || video.readyState < 2) {
        requestQrScanFrame();
        return;
    }

    canvas.width = video.videoWidth || 640;
    canvas.height = video.videoHeight || 480;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height);

    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    if (typeof jsQR !== 'undefined') {
        const code = jsQR(imageData.data, imageData.width, imageData.height, {
            inversionAttempts: 'dontInvert',
        });
        if (code && code.data && code.data.trim().length > 0) {
            const detected = code.data.trim();
            const bundleInput = document.getElementById('add-bundle-input');
            if (bundleInput) bundleInput.value = detected;
            closeQrCameraScanner();
            if (navigator.vibrate) {
                navigator.vibrate(100);
            }
            if (detected.startsWith('nova://group-invite')) {
                setTimeout(() => {
                    if (confirm('Lien d\'invitation à un groupe détecté ! Souhaitez-vous rejoindre ce groupe immédiatement ?')) {
                        addContactReal();
                    }
                }, 100);
            }
            return;
        }
    }

    requestQrScanFrame();
}

// Decodes a QR code from an imported image file or screenshot and fills the bundle input.
function handleQrImageSelect(event) {
    const file = event.target.files && event.target.files[0];
    event.target.value = '';
    if (!file) return;
    if (typeof jsQR === 'undefined') {
        alert('Décodeur QR indisponible.');
        return;
    }

    const img = new Image();
    img.onload = () => {
        // Downscale large smartphone photos (e.g. 12MP-48MP) to max 1024px for fast, reliable jsQR decoding
        const maxDim = 1024;
        let width = img.width;
        let height = img.height;
        if (width > maxDim || height > maxDim) {
            if (width > height) {
                height = Math.round((height * maxDim) / width);
                width = maxDim;
            } else {
                width = Math.round((width * maxDim) / height);
                height = maxDim;
            }
        }

        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');
        ctx.drawImage(img, 0, 0, width, height);
        const imageData = ctx.getImageData(0, 0, width, height);
        const result = jsQR(imageData.data, imageData.width, imageData.height);
        URL.revokeObjectURL(img.src);
        if (!result) {
            alert('Aucun QR code détecté dans cette image. Vérifiez que la photo est nette et bien cadrée.');
            return;
        }

        const bundleInput = document.getElementById('add-bundle-input');
        if (bundleInput) bundleInput.value = result.data.trim();
        closeQrCameraScanner();
        if (result.data.trim().startsWith('nova://group-invite')) {
            setTimeout(() => {
                if (confirm('Lien d\'invitation à un groupe détecté ! Souhaitez-vous rejoindre ce groupe immédiatement ?')) {
                    addContactReal();
                }
            }, 100);
        }
    };
    img.onerror = () => alert('Impossible de charger cette image.');
    img.src = URL.createObjectURL(file);
}

// Adds a contact from an invitation URI or pasted X3DH prekey bundle. The backend verifies the
// invitation's unforgeable signature and expiry deadline before accepting it.
async function addContactReal() {
    if (!requireBackend()) return;
    const displayNameInput = document.getElementById('add-display-name-input');
    const bundleInput = document.getElementById('add-bundle-input');
    const displayName = (displayNameInput && displayNameInput.value.trim()) || '';
    const bundleHex = (bundleInput && bundleInput.value.trim()) || '';

    if (!displayName && !bundleHex.startsWith('nova://group-invite')) {
        alert('Entrez un nom pour ce contact.');
        return;
    }
    if (!bundleHex) {
        alert('Collez le lien ou code reçu de votre contact.');
        return;
    }

    if (bundleHex.startsWith('nova://group-invite')) {
        try {
            const group = await tauriInvoke('join_group_by_invitation', { uri: bundleHex });
            await refreshConversationsFromBackend();
            alert(`Vous avez rejoint le groupe "${group.name}" !`);
            openChatWith(group.name, group.id, 'group_' + group.id);
            return;
        } catch (e) {
            alert('Impossible de rejoindre le groupe — vérifiez le lien d\'invitation : ' + e);
            return;
        }
    }

    try {
        await tauriInvoke('add_contact', {
            username: displayName.toLowerCase().replace(/[^a-z0-9]+/g, ''),
            displayName: displayName,
            bundleHex: bundleHex,
        });
        await refreshContactsFromBackend();
        await refreshConversationsFromBackend();
        navigateTo('contacts');
        alert(`Le contact « ${displayName} » a bien été ajouté.`);
    } catch (e) {
        alert('Impossible d\'ajouter ce contact — vérifiez que le code a été copié en entier et sans erreur. ' + e);
    }
}

// --- ZERO-CONFIG DIRECTORY SEARCH & USER PROFILE INSPECTION ---
let contactSearchDebounceTimer = null;
let mainContactsSearchDebounceTimer = null;

async function performDirectorySearch(query, isRetry) {
    const container = document.getElementById('directory-search-results');
    if (!container) return;
    const rawQ = (query || '').trim();
    const q = rawQ.replace(/^@+/, '').trim();
    if (!q) {
        container.innerHTML = `
            <div style="text-align: center; color: var(--text-dim); padding: 30px 16px; font-size: 13px;">
                <div style="font-size: 28px; margin-bottom: 8px;">🔍</div>
                <div style="color: white; font-weight: 600; margin-bottom: 4px;">Recherche dans l'annuaire</div>
                <div>Tapez un nom ou un @pseudo pour trouver et ajouter un contact instantanément.</div>
            </div>
        `;
        state.directorySearchResults = [];
        return;
    }

    container.innerHTML = `
        <div style="text-align: center; color: var(--accent-purple-light); padding: 20px 16px; font-size: 13px;">
            <div style="margin-bottom: 6px;">⏳</div>
            Recherche en cours...
        </div>
    `;

    let results = [];

    // 1. High-speed query to Vercel Edge API
    try {
        const resp = await fetch(`${VERCEL_API_BASE_URL}/api/directory?query=${encodeURIComponent(q)}`);
        if (resp.ok) {
            const data = await resp.json();
            if (data && Array.isArray(data.results)) {
                results = data.results.map(r => ({
                    peer_id: r.peer_id,
                    username: r.username,
                    display_name: r.display_name,
                    avatar_data_url: r.avatar_data_url,
                    prekey_bundle_hex: r.prekey_bundle_hex,
                    is_online: (Date.now() - Number(r.last_updated_at)) < 300000,
                }));
            }
        }
    } catch (_) {}

    // 2. Fallback to Tauri backend if Vercel fetch returned empty or failed
    if (results.length === 0 && hasBackend) {
        try {
            const backendResults = await tauriInvoke('search_directory', { query: q });
            if (backendResults && backendResults.length > 0) {
                results = backendResults;
            }
        } catch (e) {
            console.warn('search_directory backend fallback failed', e);
        }
    }

    state.directorySearchResults = results;

    if (results.length === 0) {
        container.innerHTML = `
            <div style="text-align: center; color: var(--text-muted); padding: 24px 16px; font-size: 13px; background: var(--bg-surface); border-radius: var(--radius-md); border: 1px solid var(--border-subtle);">
                <div style="color: white; font-weight: 600; margin-bottom: 4px;">Aucun utilisateur trouvé</div>
                <div style="font-size: 12px; margin-bottom: 12px;">Aucun contact ne correspond à « ${escapeHtml(rawQ)} ».</div>
                <button class="btn-primary" style="font-size: 12px; padding: 10px 16px;" data-action="fillAddContactForm" data-name="${escapeHtml(rawQ)}" data-id="${escapeHtml(rawQ)}">➕ Ajouter « ${escapeHtml(rawQ)} » manuellement</button>
            </div>
        `;
        return;
    }

    container.innerHTML = results.map(u => {
        const isSelf = u.peer_id === state.currentUser.peerId;
        const isAlreadyContact = state.contacts.some(c => c.handle === u.peer_id || c.peerId === u.peer_id);
        return `
            <div class="item-card" style="background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); margin-bottom: 8px; padding: 12px 14px;">
                <div class="avatar" style="width: 44px; height: 44px; font-size: 18px; position: relative;">
                    ${u.avatar_data_url ? `<img src="${u.avatar_data_url}" style="width: 100%; height: 100%; object-fit: cover;">` : escapeHtml(u.display_name.charAt(0) || u.username.charAt(0) || '?')}
                    <div class="status-dot ${u.is_online ? 'status-online' : 'status-offline'}" style="position: absolute; bottom: -2px; right: -2px;"></div>
                </div>
                <div class="item-content" style="cursor: pointer;" data-action="inspectDirectoryUser" data-peer-id="${escapeHtml(u.peer_id)}">
                    <div class="item-name" style="font-size: 14px; font-weight: 600; color: white;">${escapeHtml(u.display_name)}</div>
                    <div class="item-sub" style="color: var(--accent-purple-light); font-size: 12px;">@${escapeHtml(u.username)} • <span style="font-size: 11px; color: ${u.is_online ? 'var(--status-success)' : 'var(--text-muted)'};">${u.is_online ? '🟢 En ligne' : '⚪ Hors ligne'}</span></div>
                </div>
                <div style="display: flex; gap: 6px;">
                    <button class="btn-secondary" style="font-size: 11px; padding: 6px 10px;" data-action="inspectDirectoryUser" data-peer-id="${escapeHtml(u.peer_id)}">Profil</button>
                    ${isSelf ? `
                        <button class="btn-secondary" style="font-size: 11px; padding: 6px 10px; opacity: 0.6;" disabled>C'est vous</button>
                    ` : isAlreadyContact ? `
                        <button class="btn-secondary" style="font-size: 11px; padding: 6px 10px; color: var(--status-success);" data-action="openChat" data-name="${escapeHtml(u.display_name)}" data-handle="${escapeHtml(u.peer_id)}">Discuter</button>
                    ` : `
                        <button class="btn-primary" style="font-size: 11px; padding: 6px 12px;" data-action="addDirectUser" data-peer-id="${escapeHtml(u.peer_id)}" data-username="${escapeHtml(u.username)}" data-name="${escapeHtml(u.display_name)}" data-bundle="${escapeHtml(u.prekey_bundle_hex)}">Ajouter</button>
                    `}
                </div>
            </div>
        `;
    }).join('');
}

function handleContactDirectorySearch(query) {
    if (contactSearchDebounceTimer) clearTimeout(contactSearchDebounceTimer);
    contactSearchDebounceTimer = setTimeout(() => {
        performDirectorySearch(query);
    }, 150);
}

function triggerContactDirectorySearchNow() {
    if (contactSearchDebounceTimer) clearTimeout(contactSearchDebounceTimer);
    const input = document.getElementById('contact-search-query');
    performDirectorySearch(input ? input.value : '');
}

// Called when user clicks "Chercher" or presses Enter from the contacts screen
function triggerContactsSearchNow() {
    const input = document.getElementById('contacts-search-input');
    const q = input ? input.value.trim() : '';
    filterContactsList(q);
}

async function refreshDirectoryNodesAndSearch() {
    if (hasBackend) {
        try {
            await tauriInvoke('refresh_remote_seed_nodes');
            await tauriInvoke('publish_directory_profile');
        } catch (_) {}
    }
    triggerContactDirectorySearchNow();
}

function inspectDirectoryUser(peerId) {
    const user = state.directorySearchResults.find(u => u.peer_id === peerId);
    if (!user) return;
    state.inspectedDirectoryUser = user;
    const modal = document.getElementById('inspect-user-modal');
    const card = document.getElementById('inspect-user-card');
    if (!modal || !card) return;

    const isSelf = user.peer_id === state.currentUser.peerId;
    const isAlreadyContact = state.contacts.some(c => c.handle === user.peer_id || c.peerId === user.peer_id);

    card.innerHTML = `
        <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;">
            <div style="font-size: 16px; font-weight: 700; color: white;">Profil de l'utilisateur</div>
            <button class="icon-btn" data-action="closeInspectUserModal" style="width: 28px; height: 28px;">✕</button>
        </div>

        <div style="text-align: center; margin-bottom: 18px;">
            <div class="avatar" style="width: 72px; height: 72px; font-size: 28px; margin: 0 auto 10px; position: relative;">
                ${user.avatar_data_url ? `<img src="${user.avatar_data_url}" style="width: 100%; height: 100%; object-fit: cover;">` : escapeHtml(user.display_name.charAt(0) || '?')}
                <div class="status-dot ${user.is_online ? 'status-online' : 'status-offline'}" style="position: absolute; bottom: 2px; right: 2px; width: 14px; height: 14px;"></div>
            </div>
            <div style="font-size: 18px; font-weight: 700; color: white;">${escapeHtml(user.display_name)}</div>
            <div style="font-size: 13px; color: var(--accent-purple-light); margin-top: 2px;">@${escapeHtml(user.username)}</div>
            <div style="font-size: 11px; color: ${user.is_online ? 'var(--status-success)' : 'var(--text-muted)'}; margin-top: 4px;">
                ${user.is_online ? '🟢 En ligne sur le réseau NOVA' : '⚪ Vu récemment'}
            </div>
        </div>

        <div style="background: var(--bg-elevated); border-radius: var(--radius-md); padding: 14px; margin-bottom: 16px; border: 1px solid var(--border-subtle); display: flex; align-items: center; justify-content: space-between;">
            <div style="display: flex; align-items: center; gap: 8px;">
                <span style="font-size: 16px;">🔒</span>
                <div>
                    <div style="font-size: 12px; font-weight: 600; color: white;">Profil vérifié</div>
                    <div style="font-size: 11px; color: var(--text-muted);">Messages et appels 100% chiffrés</div>
                </div>
            </div>
            <span style="font-size: 11px; color: var(--status-success); background: rgba(34,197,94,0.12); padding: 3px 8px; border-radius: 12px; font-weight: 600;">Sécurisé</span>
        </div>

        <div style="display: flex; gap: 10px;">
            <button class="btn-secondary" style="flex: 1;" data-action="closeInspectUserModal">Fermer</button>
            ${isSelf ? `
                <button class="btn-secondary" style="flex: 1; opacity: 0.6;" disabled>C'est vous</button>
            ` : isAlreadyContact ? `
                <button class="btn-primary" style="flex: 1;" data-action="openChat" data-name="${escapeHtml(user.display_name)}" data-handle="${escapeHtml(user.peer_id)}">Ouvrir la discussion</button>
            ` : `
                <button class="btn-primary" style="flex: 1;" data-action="addInspectedUser">Ajouter aux contacts</button>
            `}
        </div>
        ${!isSelf ? `
            <button class="btn-secondary" style="width: 100%; margin-top: 10px; font-size: 11px; color: #fbbf24; border-color: rgba(245, 158, 11, 0.3);" data-action="openReportModal" data-peer-id="${escapeHtml(user.peer_id)}" data-name="${escapeHtml(user.display_name)}">⚠️ Signaler cet utilisateur</button>
        ` : ''}
    `;

    modal.classList.add('show');
}

function closeInspectUserModal() {
    const modal = document.getElementById('inspect-user-modal');
    if (modal) modal.classList.remove('show');
    state.inspectedDirectoryUser = null;
}

async function addInspectedUser() {
    const user = state.inspectedDirectoryUser;
    if (!user) return;
    closeInspectUserModal();
    await addDirectUser(user.peer_id, user.username, user.display_name, user.prekey_bundle_hex);
}

async function addDirectUser(peerId, username, displayName, bundleHex) {
    if (!requireBackend()) return;
    try {
        await tauriInvoke('add_contact', {
            username: username || displayName || 'contact',
            displayName: displayName || username || 'Contact',
            bundleHex: bundleHex || peerId,
        });
        await refreshContactsFromBackend();
        await refreshConversationsFromBackend();
        openChatWith(displayName || username, peerId);
    } catch (e) {
        alert('Échec de l\'ajout du contact : ' + e);
    }
}

function toggleManualInviteAccordion() {
    const body = document.getElementById('manual-invite-body');
    const chevron = document.getElementById('manual-invite-chevron');
    if (!body) return;
    const isHidden = body.style.display === 'none';
    body.style.display = isHidden ? 'block' : 'none';
    if (chevron) chevron.innerText = isHidden ? '▲' : '▼';
}

function filterContactsList(query) {
    state.contactsSearchQuery = query || '';
    const container = document.getElementById('contacts-list-container');
    if (!container) return;

    const rawQ = (query || '').trim();
    if (!rawQ) {
        container.innerHTML = state.contacts.filter(c => !c.isBlocked).length > 0 ? state.contacts.filter(c => !c.isBlocked).map(c => `
            <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-action="openChat">
                <div class="avatar">
                    ${escapeHtml(c.name.charAt(0))}
                    <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                </div>
                <div class="item-content">
                    <div class="item-name">${escapeHtml(c.name)}</div>
                    <div class="item-sub">@${escapeHtml(c.handle)}</div>
                </div>
            </div>
        `).join('') : `
            <div style="text-align: center; color: var(--text-muted); padding: 60px 24px;">
                <div style="width: 48px; height: 48px; border-radius: 50%; background: var(--bg-surface); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--text-dim);">
                    ${icons.users}
                </div>
                <div style="font-size: 15px; font-weight: 600; color: white;">Aucun contact actif</div>
                <p style="font-size: 13px; color: var(--text-muted); margin-top: 6px; max-width: 260px; margin-left: auto; margin-right: auto;">Ajoutez un contact pour commencer à échanger en toute confidentialité.</p>
                <button class="btn-primary" style="margin-top: 18px;" data-action="navigate" data-screen="add_contact">Ajouter un contact</button>
            </div>
        `;
        return;
    }

    const q = rawQ.toLowerCase().replace(/^@+/, '');
    const filteredLocal = state.contacts.filter(c =>
        !c.isBlocked &&
        (c.name.toLowerCase().includes(q) ||
        c.handle.toLowerCase().includes(q))
    );

    let localHtml = '';
    if (filteredLocal.length > 0) {
        localHtml = `
            <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); margin: 6px 0 8px 4px;">VOS CONTACTS (${filteredLocal.length})</div>
            ${filteredLocal.map(c => `
                <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-action="openChat">
                    <div class="avatar">
                        ${escapeHtml(c.name.charAt(0))}
                        <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                    </div>
                    <div class="item-content">
                        <div class="item-name">${escapeHtml(c.name)}</div>
                        <div class="item-sub">@${escapeHtml(c.handle)}</div>
                    </div>
                </div>
            `).join('')}
        `;
    }

    container.innerHTML = `
        ${localHtml}
        <div id="contacts-global-dir-section" style="margin-top: 16px;">
            <div style="font-size: 11px; font-weight: 600; color: var(--accent-purple-light); margin: 6px 0 8px 4px;">ANNUAIRE GLOBAL</div>
            <div id="contacts-global-dir-results" style="padding: 12px; text-align: center; color: var(--text-dim); font-size: 12px; background: var(--bg-surface); border-radius: var(--radius-md); border: 1px solid var(--border-subtle);">
                Recherche de « ${escapeHtml(rawQ)} » sur l'annuaire...
            </div>
        </div>
    `;

    if (mainContactsSearchDebounceTimer) clearTimeout(mainContactsSearchDebounceTimer);
    mainContactsSearchDebounceTimer = setTimeout(async () => {
        const dirResultsContainer = document.getElementById('contacts-global-dir-results');
        if (!dirResultsContainer) return;
        let results = [];
        if (hasBackend) {
            try {
                results = await tauriInvoke('search_directory', { query: q }) || [];
            } catch (e) {
                console.error('search_directory in contacts failed', e);
            }
        }
        if (results.length === 0) {
            if (filteredLocal.length === 0) {
                dirResultsContainer.innerHTML = `
                    <div style="color: white; font-weight: 500; margin-bottom: 4px;">Aucun résultat pour « ${escapeHtml(rawQ)} »</div>
                    <div style="color: var(--text-muted); font-size: 11px; margin-bottom: 10px;">Aucun contact local ou distant correspondant.</div>
                    <div style="display: flex; flex-direction: column; gap: 8px; max-width: 320px; margin: 0 auto;">
                        <button class="btn-primary" style="font-size: 11px; padding: 8px 12px;" data-action="fillAddContactForm" data-name="${escapeHtml(rawQ)}" data-id="${escapeHtml(rawQ)}">➕ Ajouter « ${escapeHtml(rawQ)} » comme contact</button>
                        <button class="btn-secondary" style="font-size: 11px; padding: 6px 12px;" data-action="navigate" data-screen="add_contact">Aller à l'écran Ajouter un contact</button>
                    </div>
                `;
            } else {
                dirResultsContainer.innerHTML = `
                    <div style="color: var(--text-dim); font-size: 11px;">Aucun autre utilisateur trouvé dans l'annuaire global.</div>
                `;
            }
            return;
        }

        dirResultsContainer.innerHTML = results.map(u => {
            const shortId = u.peer_id.slice(0, 8) + '…' + u.peer_id.slice(-6);
            const isSelf = u.peer_id === state.currentUser.peerId;
            const isAlreadyContact = state.contacts.some(c => c.handle === u.peer_id || c.peerId === u.peer_id);
            return `
                <div class="item-card" style="background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); margin-bottom: 8px; padding: 10px 12px;">
                    <div class="avatar" style="width: 40px; height: 40px; font-size: 16px; position: relative;">
                        ${u.avatar_data_url ? `<img src="${u.avatar_data_url}" style="width: 100%; height: 100%; object-fit: cover;">` : escapeHtml(u.display_name.charAt(0) || u.username.charAt(0) || '?')}
                        <div class="status-dot ${u.is_online ? 'status-online' : 'status-offline'}" style="position: absolute; bottom: -2px; right: -2px;"></div>
                    </div>
                    <div class="item-content" style="cursor: pointer;" data-action="inspectDirectoryUser" data-peer-id="${escapeHtml(u.peer_id)}">
                        <div class="item-name" style="font-size: 13px; font-weight: 600; color: white;">${escapeHtml(u.display_name)}</div>
                        <div class="item-sub" style="color: var(--accent-purple-light); font-size: 11px;">@${escapeHtml(u.username)} • <span style="font-family: monospace; font-size: 10px; color: var(--text-dim);">${shortId}</span></div>
                    </div>
                    <div style="display: flex; gap: 6px;">
                        ${isSelf ? `
                            <button class="btn-secondary" style="font-size: 10px; padding: 4px 8px; opacity: 0.6;" disabled>Moi</button>
                        ` : isAlreadyContact ? `
                            <button class="btn-secondary" style="font-size: 10px; padding: 4px 8px; color: var(--status-success);" data-action="openChat" data-name="${escapeHtml(u.display_name)}" data-handle="${escapeHtml(u.peer_id)}">Discuter</button>
                        ` : `
                            <button class="btn-primary" style="font-size: 10px; padding: 4px 10px;" data-action="addDirectUser" data-peer-id="${escapeHtml(u.peer_id)}" data-username="${escapeHtml(u.username)}" data-name="${escapeHtml(u.display_name)}" data-bundle="${escapeHtml(u.prekey_bundle_hex)}">Ajouter</button>
                        `}
                    </div>
                </div>
            `;
        }).join('');
    }, 150);
}

async function handleStrictSearch(query) {
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

    // Full-text message search runs against the encrypted local database (see
    // nova_engine::NovaEngine::search_messages), not just whatever conversation happens to be
    // loaded in state.messages — otherwise a message from any conversation other than the one
    // currently open would never be found.
    let searchedMessages = [];
    if (hasBackend) {
        try {
            searchedMessages = await tauriInvoke('search_messages', { query });
        } catch (e) {
            console.error('search_messages failed', e);
        }
    }
    // Bail out if the input has moved on to a different query while this awaited (avoids a
    // slower earlier search clobbering a faster later one's results).
    const searchInput = document.getElementById('global-search-input');
    if (searchInput && searchInput.value !== query) return;

    resultsContainer.innerHTML = `
        <div style="font-size: 12px; color: var(--text-muted); margin: 0 0 8px 12px; font-weight: 600;">CONVERSATIONS & CONTACTS (${filteredContacts.length})</div>
        ${filteredContacts.length > 0 ? filteredContacts.map(c => `
            <div class="item-card" data-name="${escapeHtml(c.name)}" data-handle="${escapeHtml(c.handle)}" data-action="openChat">
                <div class="avatar">${escapeHtml(c.name.charAt(0))}</div>
                <div class="item-content">
                    <div class="item-name">${escapeHtml(c.name)}</div>
                    <div class="item-sub">@${escapeHtml(c.handle)}</div>
                </div>
            </div>
        `).join('') : '<div style="font-size: 13px; color: var(--text-dim); margin-left: 12px; margin-bottom: 14px;">Aucun contact correspondant.</div>'}

        <div style="font-size: 12px; color: var(--text-muted); margin: 16px 0 8px 12px; font-weight: 600;">MESSAGES (${searchedMessages.length})</div>
        ${searchedMessages.length > 0 ? searchedMessages.map(m => {
            // conversation_id follows nova-engine's `conv_<peer_id>` convention (see the field
            // comment on state.contacts in openChatWith) — the peer is whichever side of the
            // message isn't us.
            const peerId = m.is_outgoing ? m.recipient_id : m.sender_id;
            const contact = state.contacts.find(c => c.handle === peerId);
            return `
            <div class="item-card" data-name="${escapeHtml(contact ? contact.name : '')}" data-handle="${escapeHtml(peerId)}" data-action="openChat">
                <div class="avatar">${icons.chat}</div>
                <div class="item-content">
                    <div class="item-name">${escapeHtml(contact ? contact.name : peerId)}</div>
                    <div class="item-sub">${escapeHtml(m.text_content)}</div>
                </div>
            </div>
        `; }).join('') : '<div style="font-size: 13px; color: var(--text-dim); margin-left: 12px;">Aucun message trouvé.</div>'}
    `;
}

// --- INITIALIZATION ---
document.addEventListener('DOMContentLoaded', async () => {
    // Setup rail & nav tabs
    document.querySelectorAll('[data-tab]').forEach(btn => {
        btn.addEventListener('click', () => {
            const target = btn.getAttribute('data-tab');
            navigateTo(target);
        });
    });

    // Update initial unread badges
    updateGlobalUnreadBadges();

    // Real-Time Push Event Listeners from Tauri Backend
    if (window.__TAURI__ && window.__TAURI__.event) {
        window.__TAURI__.event.listen('nova://message-received', async (event) => {
            const msg = event.payload;
            if (!msg) return;

            // Priorité absolue : Détection immédiate des signaux d'appel entrant
            if (msg.text_content) {
                const structured = decodeStructuredMessage(msg.text_content);
                if (structured && structured.kind === 'call_signal') {
                    await handleIncomingCallSignal(structured, msg, msg.conversation_id);
                    return;
                }
            }

            // 1. If currently in this chat, append it immediately
            if (state.currentScreen === 'chat' && state.activeContact && state.activeContact.conversationId === msg.conversation_id) {
                const newOnes = await refreshMessagesFromBackend(msg.conversation_id);
                if (newOnes.length > 0) {
                    const chatBody = document.getElementById('chat-body');
                    const isNearBottom = chatBody ? (chatBody.scrollHeight - chatBody.scrollTop - chatBody.clientHeight < 140) : true;
                    newOnes.forEach(appendChatMessageToBody);
                    newOnes.filter(m => m.attachmentId && !m.url).forEach(ensureAttachmentLoaded);
                    if (chatBody && isNearBottom) {
                        chatBody.scrollTo({ top: chatBody.scrollHeight, behavior: 'smooth' });
                    }
                }
            }

            // 2. Refresh conversations and update unread badges
            await refreshConversationsFromBackend();
            updateGlobalUnreadBadges();
            if (state.currentScreen === 'conversations') {
                updateConversationsListDom();
            }
        });

        window.__TAURI__.event.listen('nova://conversation-updated', async () => {
            await refreshConversationsFromBackend();
            updateGlobalUnreadBadges();
            if (state.currentScreen === 'conversations') {
                updateConversationsListDom();
            }
        });
    }

    // If this device already created/restored an identity in a previous session, resume it
    // straight from local encrypted storage — a "sovereign" identity app that made you re-type
    // your 12-word mnemonic every single launch would defeat a good part of the point.
    let resumed = null;
    if (hasBackend) {
        await initAppBuildInfo();
        try {
            resumed = await tauriInvoke('try_resume_session');
        } catch (e) {
            console.error('try_resume_session failed', e);
        }
    }
    if (resumed) {
        applyAccountInfo(
            resumed.display_name || resumed.username,
            resumed.peer_id,
            resumed.mnemonic,
            resumed.network_active,
            resumed.bio || '',
            resumed.avatar_data_url || null
        );
        try {
            await refreshContactsFromBackend();
        } catch (e) {
            console.warn('refreshContactsFromBackend failed', e);
        }
        try {
            await refreshConversationsFromBackend();
        } catch (e) {
            console.warn('refreshConversationsFromBackend failed', e);
        }

        // Démarrage instantané du canal Vercel prioritaire (Edge Signaling & Présence)
        try {
            sendPresenceHeartbeat('online');
            refreshContactsPresence();
            startVercelSignalPolling();
            syncProfileToVercel(resumed.peer_id, resumed.username, resumed.display_name, state.currentUser.bundleHex, resumed.avatar_data_url);
        } catch (e) {
            console.warn('Vercel startup sync non-blocking error:', e);
        }
    }

    try {
        registerActivityListener();
        if (state.appLock && state.appLock.enabled && state.appLock.pinHash) {
            lockApp();
        }
    } catch (e) {
        console.warn('Activity/AppLock init warning:', e);
    }

    // Initialisation de la connectivité et présence périodique (toutes les 10s)
    try {
        checkServerConnectivity();
        if (state.currentUser.peerId) {
            sendPresenceHeartbeat('online');
            refreshContactsPresence();
            startVercelSignalPolling();
        }
    } catch (e) {
        console.warn('Server connectivity init warning:', e);
    }

    setInterval(() => {
        if (state.currentUser.peerId) {
            try {
                sendPresenceHeartbeat('online');
                refreshContactsPresence();
            } catch (_) {}
        }
    }, 10000);

    window.addEventListener('online', () => {
        try {
            checkServerConnectivity();
            if (state.currentUser.peerId) {
                sendPresenceHeartbeat('online');
                refreshContactsPresence();
            }
        } catch (_) {}
    });

    window.addEventListener('offline', () => {
        try {
            state.isServerConnected = false;
            updateConnectionIndicators();
        } catch (_) {}
    });

    window.addEventListener('beforeunload', () => {
        try {
            if (state.currentUser.peerId && navigator.sendBeacon) {
                navigator.sendBeacon(`${VERCEL_API_BASE_URL}/api/presence`, JSON.stringify({
                    peer_id: state.currentUser.peerId,
                    status: 'offline'
                }));
            }
        } catch (_) {}
    });

    navigateTo(state.currentUser.peerId ? 'conversations' : 'onboarding');
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

// Automatically scroll chat messages when mobile keyboard opens/resizes
if (window.visualViewport) {
    window.visualViewport.addEventListener('resize', () => {
        if (state.currentScreen === 'chat') {
            const body = document.getElementById('chat-body');
            if (body) {
                body.scrollTop = body.scrollHeight;
            }
        }
    });
}

// Alerts the user when this device has no Wi-Fi/mobile data connectivity at all —
// `navigator.onLine` reflects the OS's own network state (Android's ConnectivityManager
// underneath the WebView, same on desktop), no native code needed. Without this, adding a
// contact or sending a message while genuinely offline just queued silently in the outbox with
// no feedback at all — indistinguishable from "the app is broken" until the outbox eventually
// gives up, several minutes later (see OUTBOX_GIVE_UP_AFTER_SECS in nova-engine).  Note this only
// detects "no network interface up at all" (no Wi-Fi, no mobile data) — a Wi-Fi connected to a
// router with no internet uplink still reports online here, same as every browser; that's a
// different, harder problem (real reachability, not link state) than what was asked for.
function updateOfflineBanner() {
    const banner = document.getElementById('offline-banner');
    if (banner) banner.classList.toggle('show', !navigator.onLine);
}
window.addEventListener('online', updateOfflineBanner);
window.addEventListener('offline', updateOfflineBanner);
updateOfflineBanner();

// ============================================================================
// P2P SOVEREIGN GROUP CHAT LOGIC & UI HELPERS
// ============================================================================

state.selectedGroupMemberIds = new Set();

function renderGroupContactsSelectionHtml() {
    if (!state.contacts || state.contacts.length === 0) {
        return `
            <div style="text-align: center; padding: 20px; color: var(--text-muted); font-size: 13px;">
                Aucun contact enregistré.<br>
                <span style="font-size: 11px; color: var(--text-dim);">Vous pourrez créer le groupe et inviter des contacts plus tard par lien d'invitation.</span>
            </div>
        `;
    }

    return state.contacts.map(c => {
        const isSelected = state.selectedGroupMemberIds.has(c.peerId);
        return `
            <div class="item-card" data-action="toggleGroupMemberSelect" data-peer-id="${escapeHtml(c.peerId)}" style="cursor: pointer; padding: 10px 14px; border-bottom: 1px solid var(--border-subtle); display: flex; align-items: center; justify-content: space-between;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <div class="avatar" style="width: 34px; height: 34px; font-size: 13px;">
                        ${escapeHtml(c.name.charAt(0))}
                        <div class="status-dot ${c.online ? 'status-online' : 'status-offline'}"></div>
                    </div>
                    <div>
                        <div style="font-size: 13px; font-weight: 600; color: white;">${escapeHtml(c.name)}</div>
                        <div style="font-size: 10px; color: var(--text-dim); font-family: monospace;">${escapeHtml(c.peerId.slice(0, 12))}...</div>
                    </div>
                </div>
                <div style="width: 20px; height: 20px; border-radius: 4px; border: 2px solid ${isSelected ? 'var(--accent-purple)' : 'var(--border-subtle)'}; background: ${isSelected ? 'var(--accent-purple)' : 'transparent'}; display: flex; align-items: center; justify-content: center; color: white; font-size: 12px;">
                    ${isSelected ? '✓' : ''}
                </div>
            </div>
        `;
    }).join('');
}

function toggleGroupMemberSelect(peerId) {
    if (!peerId) return;
    if (state.selectedGroupMemberIds.has(peerId)) {
        state.selectedGroupMemberIds.delete(peerId);
    } else {
        state.selectedGroupMemberIds.add(peerId);
    }

    const countEl = document.getElementById('group-selected-count');
    if (countEl) {
        countEl.textContent = `${state.selectedGroupMemberIds.size} sélectionné(s)`;
    }

    const listEl = document.getElementById('group-contacts-selection-list');
    if (listEl) {
        listEl.innerHTML = renderGroupContactsSelectionHtml();
    }
}

async function confirmCreateGroupReal() {
    if (!requireBackend()) return;
    const nameInput = document.getElementById('create-group-name-input');
    const descInput = document.getElementById('create-group-desc-input');
    const name = nameInput ? nameInput.value.trim() : '';
    const description = descInput && descInput.value.trim() ? descInput.value.trim() : null;

    if (!name) {
        alert('Veuillez spécifier un nom pour le groupe.');
        if (nameInput) nameInput.focus();
        return;
    }

    const members = Array.from(state.selectedGroupMemberIds);

    try {
        const group = await tauriInvoke('create_group', {
            name,
            description,
            avatarDataUrl: null,
            members,
        });

        await refreshConversationsFromBackend();
        openChatWith(group.name, group.id, 'group_' + group.id);
    } catch (e) {
        alert('Erreur lors de la création du groupe : ' + e);
    }
}

async function refreshGroupMembersInfo(groupId) {
    if (!hasBackend || !groupId) return;
    const container = document.getElementById('group-members-list');
    if (!container) return;

    try {
        const members = await tauriInvoke('get_group_members', { groupId });
        if (!members || members.length === 0) {
            container.innerHTML = `<div style="text-align: center; padding: 20px; color: var(--text-dim); font-size: 12px;">Aucun membre répertorié.</div>`;
            return;
        }

        container.innerHTML = members.map(m => `
            <div style="padding: 12px 14px; border-bottom: 1px solid var(--border-subtle); display: flex; align-items: center; justify-content: space-between;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <div class="avatar" style="width: 34px; height: 34px; font-size: 13px;">
                        ${escapeHtml(m.display_name.charAt(0))}
                        <div class="status-dot ${m.is_online ? 'status-online' : 'status-offline'}"></div>
                    </div>
                    <div>
                        <div style="font-size: 13px; font-weight: 600; color: white; display: flex; align-items: center; gap: 6px;">
                            ${escapeHtml(m.display_name)}
                            ${m.peer_id === state.currentUser.peerId ? '<span style="font-size: 10px; background: rgba(139, 92, 246, 0.2); color: var(--accent-purple-light); padding: 1px 6px; border-radius: 4px;">Vous</span>' : ''}
                        </div>
                        <div style="font-size: 10px; color: var(--text-dim); font-family: monospace;">${escapeHtml(m.peer_id.slice(0, 12))}...</div>
                    </div>
                </div>
                <div>
                    <span style="font-size: 11px; font-weight: 600; color: ${m.role === 'owner' ? '#f59e0b' : (m.role === 'admin' ? 'var(--accent-purple-light)' : 'var(--text-muted)')}; text-transform: uppercase;">
                        ${escapeHtml(m.role)}
                    </span>
                </div>
            </div>
        `).join('');
    } catch (e) {
        container.innerHTML = `<div style="text-align: center; padding: 20px; color: var(--status-danger); font-size: 12px;">Échec chargement membres: ${escapeHtml(String(e))}</div>`;
    }
}

async function shareGroupInvitation() {
    if (!state.activeContact || !state.activeContact.isGroup || !requireBackend()) return;
    try {
        const uri = await tauriInvoke('create_group_invitation', {
            groupId: state.activeContact.peerId,
            ttlSeconds: 86400 * 7, // 7 days
        });

        if (navigator.share) {
            try {
                await navigator.share({
                    title: `Invitation au groupe ${state.activeContact.name}`,
                    text: `Rejoignez notre groupe sécurisé "${state.activeContact.name}" sur NOVA Chat via ce lien pair-à-pair :\n${uri}`,
                });
                return;
            } catch (err) {}
        }

        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(uri);
        }
        alert(`Lien d'invitation au groupe copié dans le presse-papiers !\n\n${uri}`);
    } catch (e) {
        alert('Erreur lors de la génération de l\'invitation : ' + e);
    }
}

async function confirmLeaveGroupReal() {
    if (!state.activeContact || !state.activeContact.isGroup || !requireBackend()) return;
    if (!confirm(`Êtes-vous sûr de vouloir quitter le groupe "${state.activeContact.name}" ? Vous ne recevrez plus ses messages.`)) {
        return;
    }

    try {
        await tauriInvoke('leave_group', { groupId: state.activeContact.peerId });
        await refreshConversationsFromBackend();
        navigateTo('conversations');
    } catch (e) {
        alert('Erreur pour quitter le groupe : ' + e);
    }
}

