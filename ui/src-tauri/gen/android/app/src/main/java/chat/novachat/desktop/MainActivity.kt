package chat.novachat.desktop

import android.content.Context
import android.content.Intent
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.content.ContextCompat

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

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
    multicastLock = wifiManager?.createMulticastLock("nova_chat_mdns")?.apply {
      setReferenceCounted(false)
      acquire()
    }

    // Without this, the Rust engine's P2P listener/outbox pump (running in this same process)
    // gets suspended or killed by the OS within minutes of this Activity leaving the foreground —
    // confirmed live on a real Huawei device, where a message sent from another phone never
    // arrived because this process had already been stopped. See P2pForegroundService's own doc
    // comment for why this is never explicitly stopped again.
    ContextCompat.startForegroundService(this, Intent(this, P2pForegroundService::class.java))
  }
}
