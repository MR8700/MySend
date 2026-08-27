package chat.novachat.desktop

import android.app.Notification
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import androidx.core.app.NotificationChannelCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat

/**
 * Keeps this device reachable over the P2P network (mDNS/DHT listener, QUIC connections, the
 * outbox retry pump — all running inside the Rust engine, in this same process) while the app is
 * in the background. Without a real foreground service, Android — and especially OEM battery
 * managers (Huawei, Xiaomi, etc. are the most aggressive) — can suspend or kill the whole process
 * within seconds to minutes of the Activity leaving the foreground, silently stopping this device
 * from ever receiving messages until the user manually reopens the app. `AndroidManifest.xml`
 * already declared `FOREGROUND_SERVICE`/`FOREGROUND_SERVICE_DATA_SYNC`/`WAKE_LOCK` for exactly
 * this — this class is what actually uses them; started once from `MainActivity.onCreate` and,
 * like the multicast lock there, deliberately never stopped for the rest of the process's life —
 * a chat app's whole point is being reachable, not a one-shot task.
 *
 * `dataSync` (not e.g. `remoteMessaging`, which better fits a chat app semantically but requires
 * API 34+ branching and a second manifest permission) was chosen to keep this to one type across
 * `minSdk = 24`: Android 14+ imposes a ~6h/24h cumulative runtime cap on `dataSync` foreground
 * services specifically, so on very recent Android this service may need to be restarted by the
 * OS periodically — still a large improvement over being killed within minutes, which is the
 * actual bug this fixes (reproduced live: a message sent from one real device never reached
 * another real device in ~50 minutes with no foreground service running).
 */
class P2pForegroundService : Service() {
    private var wakeLock: PowerManager.WakeLock? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        val powerManager = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock = powerManager.newWakeLock(
            PowerManager.PARTIAL_WAKE_LOCK,
            "chat.novachat.desktop:p2p"
        ).apply {
            setReferenceCounted(false)
            acquire()
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val notification = buildNotification()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
        // START_STICKY: if the system still kills this process under extreme memory pressure, ask
        // it to recreate and restart this service (with a null Intent) once resources free up,
        // rather than leaving the device silently unreachable until the user notices and reopens
        // the app themselves.
        return START_STICKY
    }

    override fun onDestroy() {
        wakeLock?.let { if (it.isHeld) it.release() }
        wakeLock = null
        super.onDestroy()
    }

    private fun buildNotification(): Notification {
        val channel = NotificationChannelCompat.Builder(CHANNEL_ID, NotificationManagerCompat.IMPORTANCE_MIN)
            .setName("Connexion NOVA Chat")
            .setDescription("Maintient la connexion pair-à-pair privée active en arrière-plan")
            .setShowBadge(false)
            .build()
        NotificationManagerCompat.from(this).createNotificationChannel(channel)

        val launchIntent = packageManager.getLaunchIntentForPackage(packageName)
        val contentIntent = PendingIntent.getActivity(
            this,
            0,
            launchIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("NOVA Chat")
            .setContentText("Connexion privée active en arrière-plan")
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentIntent(contentIntent)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_MIN)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .build()
    }

    companion object {
        private const val NOTIFICATION_ID = 1
        private const val CHANNEL_ID = "nova_p2p_channel"
    }
}
