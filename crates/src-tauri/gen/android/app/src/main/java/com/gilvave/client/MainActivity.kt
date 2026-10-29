package com.gilvave.client

import android.graphics.Color
import android.os.Bundle
import android.view.View
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
}
