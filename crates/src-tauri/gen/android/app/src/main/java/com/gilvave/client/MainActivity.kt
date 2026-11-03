package com.gilvave.client

import android.graphics.Color
import android.os.Bundle
import android.view.View
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override val handleBackNavigation: Boolean = false
  private var mWebView: WebView? = null
  private lateinit var backCallback: OnBackPressedCallback

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    window.statusBarColor = Color.parseColor("#0a0e1a")
    window.navigationBarColor = Color.parseColor("#0a0e1a")

    val content = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { view, insets ->
      val topInset = insets.getInsets(
        WindowInsetsCompat.Type.statusBars() or WindowInsetsCompat.Type.displayCutout()
      ).top
      val bottomInset = insets.getInsets(WindowInsetsCompat.Type.navigationBars()).bottom
      view.setPadding(0, topInset, 0, bottomInset)
      insets
    }

    backCallback = object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val wv = mWebView
        if (wv != null) {
          wv.evaluateJavascript("(function() { return (window.__onAndroidBackPressed ? window.__onAndroidBackPressed() : false); })();") { result ->
            val consumed = result == "true" || result == "\"true\""
            if (!consumed) {
              isEnabled = false
              onBackPressedDispatcher.onBackPressed()
              isEnabled = true
            }
          }
        } else {
          isEnabled = false
          onBackPressedDispatcher.onBackPressed()
          isEnabled = true
        }
      }
    }
    onBackPressedDispatcher.addCallback(this, backCallback)
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    mWebView = webView
    WebView.setWebContentsDebuggingEnabled(true)
    webView.settings.mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW

    try {
      val packageInfo = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
        packageManager.getPackageInfo(packageName, android.content.pm.PackageManager.PackageInfoFlags.of(0))
      } else {
        @Suppress("DEPRECATION")
        packageManager.getPackageInfo(packageName, 0)
      }
      val prefs = getSharedPreferences("gilvave_webview_meta", MODE_PRIVATE)
      val lastUpdateTime = prefs.getLong("last_update_time", 0L)
      if (lastUpdateTime != packageInfo.lastUpdateTime) {
        webView.clearCache(true)
        prefs.edit().putLong("last_update_time", packageInfo.lastUpdateTime).apply()
      }
    } catch (_: Exception) {}
  }
}
