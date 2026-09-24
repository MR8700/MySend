package chat.novachat.desktop

import android.content.Context
import android.content.Intent
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.content.ContextCompat

import android.webkit.PermissionRequest
import android.webkit.WebView

class MainActivity : TauriActivity() {
  // Android's Wi-Fi radio drops incoming multicast packets by default to save power — mDNS (used
  // by the Rust engine's nova-transport for same-Wi-Fi peer discovery, see nova-transport's
  // dht_node.rs) is entirely multicast-based, so without this lock the app can send mDNS queries
  // but never actually receive another device's replies: LAN discovery silently never works on
  // Android specifically, even though the exact same code works fine on desktop. Acquired once
  // here and held for the process's lifetime (never released) — this is a chat app whose whole
  // point is being reachable, not a one-shot LAN scan, so there is no natural point to give the
  // lock back short of the process dying, which releases it automatically anyway.
  private var multicastLock: WifiManager.MulticastLock? = null
  private var mAppWebView: WebView? = null

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    mAppWebView = webView

    // Custom WebChromeClient that automatically grants getUserMedia permissions (Microphone & Camera)
    // for voice/video calls and the QR scanner.
    webView.webChromeClient = object : android.webkit.WebChromeClient() {
      override fun onPermissionRequest(request: PermissionRequest) {
        // Run on UI thread to ensure immediate grant
        runOnUiThread {
          request.grant(request.resources)
        }
      }
    }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    // Intercept Android hardware back button and back navigation gestures.
    // Instead of terminating the activity directly when handleBackNavigation=false,
    // we query our frontend SPA navigation router. If handled by JS (modal closed or
    // screen popped), we stay in app. If JS returns false (at root screen after prompt),
    // we finish the Activity cleanly.
    onBackPressedDispatcher.addCallback(this, object : androidx.activity.OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val wv = mAppWebView
        if (wv != null) {
          wv.evaluateJavascript("(function(){ return (window.handleAndroidBack ? window.handleAndroidBack() : (window.handleBackNavigation ? window.handleBackNavigation() : false)); })()") { result ->
            if (result == "true") {
              // Handled by webview navigation or modal close
            } else {
              // At root screen and double back confirmed (or fallback): finish activity
              finish()
            }
          }
        } else {
          finish()
        }
      }
    })

    try {
      val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
      multicastLock = wifiManager?.createMulticastLock("nova_chat_mdns")?.apply {
        setReferenceCounted(false)
        acquire()
      }
    } catch (_: Throwable) {}

    try {
      // Without this, the Rust engine's P2P listener/outbox pump (running in this same process)
      // gets suspended or killed by the OS within minutes of this Activity leaving the foreground —
      // confirmed live on a real Huawei device, where a message sent from another phone never
      // arrived because this process had already been stopped. See P2pForegroundService's own doc
      // comment for why this is never explicitly stopped again.
      ContextCompat.startForegroundService(this, Intent(this, P2pForegroundService::class.java))
    } catch (_: Throwable) {}
  }
}
