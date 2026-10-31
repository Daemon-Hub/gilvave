package com.gilvave.client

import android.graphics.Color
import android.os.Bundle
import android.view.View
import android.webkit.WebSettings
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
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
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
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
