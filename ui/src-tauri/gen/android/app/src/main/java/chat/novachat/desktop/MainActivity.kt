package chat.novachat.desktop

import android.os.Bundle
import android.util.Log
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import java.io.File

class MainActivity : TauriActivity() {
  private var mAppWebView: WebView? = null

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    mAppWebView = webView
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    // Install global uncaught exception logger
    Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
      Log.e("NOVA_CRASH", "FATAL EXCEPTION on thread ${thread.name}: ${throwable.message}", throwable)
      try {
        val crashFile = File(applicationContext.filesDir, "last_crash.log")
        crashFile.writeText("CRASH on ${thread.name}:\n" + Log.getStackTraceString(throwable))
      } catch (_: Throwable) {}
    }

    super.onCreate(savedInstanceState)

    // Intercept Android hardware back button and back navigation gestures.
    onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val wv = mAppWebView
        if (wv != null) {
          wv.evaluateJavascript("(function(){ return window.handleAndroidBack ? window.handleAndroidBack() : false; })()") { result ->
            if (result == "true") {
              // Handled by webview navigation or modal close
            } else {
              finish()
            }
          }
        } else {
          finish()
        }
      }
    })
  }
}
