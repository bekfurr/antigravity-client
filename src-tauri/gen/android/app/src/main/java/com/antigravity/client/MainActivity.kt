package com.antigravity.client

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.net.Uri
import android.os.Bundle
import android.text.InputType
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.webkit.CookieManager
import android.webkit.WebSettings
import android.webkit.WebView
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.appcompat.app.AlertDialog
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import org.json.JSONArray
import org.json.JSONObject

class MainActivity : TauriActivity() {

  companion object {
    private const val DEFAULT_URL = "https://antigravity.google.com"
    private const val PREFS_NAME = "antigravity_client_prefs"
    private const val KEY_BOOKMARKS = "bookmarks_json"
    private const val KEY_TOOLBAR_COLLAPSED = "toolbar_collapsed"
    private const val BG_DARK = 0xFF0B0F1C.toInt()
    private const val BAR_BG = 0xFF10172A.toInt()
    private const val BAR_BORDER = 0xFF24304D.toInt()
    private const val TEXT_PRIMARY = 0xFFEEF2FF.toInt()
    private const val TEXT_MUTED = 0xFF92A2C4.toInt()
    private const val ACCENT = 0xFF38BDF8.toInt()
  }

  override val handleBackNavigation: Boolean = true

  private var clientWebView: WebView? = null
  private var backButton: TextView? = null
  private var toolbarView: LinearLayout? = null
  private var restorePill: TextView? = null
  private var pendingIntentUrl: String? = null

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT)
    )
    super.onCreate(savedInstanceState)
    pendingIntentUrl = extractAntigravityIntentUrl(intent)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    setIntent(intent)
    val targetUrl = extractAntigravityIntentUrl(intent)
    if (targetUrl != null) {
      val webView = clientWebView
      if (webView != null) {
        webView.post { webView.loadUrl(targetUrl) }
      } else {
        pendingIntentUrl = targetUrl
      }
    }
  }

  override fun onPause() {
    super.onPause()
    CookieManager.getInstance().flush()
  }

  override fun onStop() {
    super.onStop()
    CookieManager.getInstance().flush()
  }

  override fun onWebViewHistoryChanged(webView: WebView) {
    super.onWebViewHistoryChanged(webView)
    runOnUiThread {
      updateBackButtonState(webView)
    }
  }

  @SuppressLint("SetJavaScriptEnabled")
  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    clientWebView = webView

    configureWebView(webView)

    runOnUiThread {
      attachNativeShell(webView)
      updateBackButtonState(webView)

      val launchUrl = pendingIntentUrl
      pendingIntentUrl = null
      if (launchUrl != null) {
        webView.loadUrl(launchUrl)
      }
    }
  }

  @SuppressLint("SetJavaScriptEnabled")
  private fun configureWebView(webView: WebView) {
    val cookieManager = CookieManager.getInstance()
    cookieManager.setAcceptCookie(true)
    cookieManager.setAcceptThirdPartyCookies(webView, true)

    webView.setBackgroundColor(BG_DARK)

    val settings = webView.settings
    settings.javaScriptEnabled = true
    settings.domStorageEnabled = true
    settings.databaseEnabled = true
    settings.javaScriptCanOpenWindowsAutomatically = true
    settings.setSupportMultipleWindows(false)
    settings.mediaPlaybackRequiresUserGesture = false
    settings.builtInZoomControls = true
    settings.displayZoomControls = false
    settings.useWideViewPort = true
    settings.loadWithOverviewMode = true
    settings.mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE

    // Strip Android WebView markers ("; wv" and "Version/x.x") while preserving the real
    // device model and Chromium version for better compatibility with Google web services.
    val originalUa = settings.userAgentString
    val cleanedUa = originalUa
      .replace("; wv)", ")")
      .replace(Regex("Version/\\d+\\.\\d+\\s*"), "")
      .replace(Regex("\\s{2,}"), " ")
      .trim()

    if (cleanedUa.isNotEmpty() && cleanedUa != originalUa) {
      settings.userAgentString = cleanedUa
      val existingUrl = webView.url
      if (!existingUrl.isNullOrBlank()) {
        webView.post { webView.loadUrl(existingUrl) }
      }
    }
  }

  private fun attachNativeShell(webView: WebView) {
    val existingParent = webView.parent as? ViewGroup
    existingParent?.removeView(webView)

    val root = LinearLayout(this).apply {
      orientation = LinearLayout.VERTICAL
      setBackgroundColor(BG_DARK)
      layoutParams = ViewGroup.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT,
        ViewGroup.LayoutParams.MATCH_PARENT
      )
    }

    val toolbar = buildToolbar(webView)
    toolbarView = toolbar

    val divider = View(this).apply {
      setBackgroundColor(BAR_BORDER)
      layoutParams = LinearLayout.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT,
        dp(1)
      )
    }

    val contentFrame = FrameLayout(this).apply {
      setBackgroundColor(BG_DARK)
      layoutParams = LinearLayout.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT,
        0,
        1f
      )
    }

    webView.layoutParams = FrameLayout.LayoutParams(
      ViewGroup.LayoutParams.MATCH_PARENT,
      ViewGroup.LayoutParams.MATCH_PARENT
    )
    contentFrame.addView(webView)

    val pill = TextView(this).apply {
      text = "▼"
      setTextColor(TEXT_PRIMARY)
      setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
      gravity = Gravity.CENTER
      setPadding(dp(12), dp(6), dp(12), dp(6))
      background = GradientDrawable().apply {
        setColor(0xCC10172A.toInt())
        setStroke(dp(1), BAR_BORDER)
        cornerRadius = dp(18).toFloat()
      }
      contentDescription = "Show toolbar"
      visibility = View.GONE
      setOnClickListener { setToolbarCollapsed(false, divider) }
    }
    val pillLp = FrameLayout.LayoutParams(
      ViewGroup.LayoutParams.WRAP_CONTENT,
      ViewGroup.LayoutParams.WRAP_CONTENT,
      Gravity.TOP or Gravity.END
    ).apply {
      topMargin = dp(8)
      marginEnd = dp(10)
    }
    contentFrame.addView(pill, pillLp)
    restorePill = pill

    root.addView(toolbar)
    root.addView(divider)
    root.addView(contentFrame)

    val initiallyCollapsed = prefs().getBoolean(KEY_TOOLBAR_COLLAPSED, false)
    setToolbarCollapsed(initiallyCollapsed, divider)

    // Respect status bar, camera cutout, gesture navigation bar, and soft keyboard (IME)
    ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars()
          or WindowInsetsCompat.Type.displayCutout()
          or WindowInsetsCompat.Type.ime()
      )
      v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      insets
    }

    setContentView(root)
    ViewCompat.requestApplyInsets(root)
  }

  private fun buildToolbar(webView: WebView): LinearLayout {
    val bar = LinearLayout(this).apply {
      orientation = LinearLayout.HORIZONTAL
      gravity = Gravity.CENTER_VERTICAL
      setBackgroundColor(BAR_BG)
      setPadding(dp(8), dp(4), dp(8), dp(4))
      layoutParams = LinearLayout.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT,
        dp(44)
      )
    }

    val btnBack = makeBarButton("←", "Back") {
      if (webView.canGoBack()) webView.goBack()
    }
    backButton = btnBack

    val btnHome = makeBarButton("⌂", "Home") {
      webView.loadUrl(DEFAULT_URL)
    }

    val title = TextView(this).apply {
      text = "Antigravity"
      setTextColor(TEXT_PRIMARY)
      setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
      typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
      gravity = Gravity.CENTER_VERTICAL
      setPadding(dp(8), 0, dp(8), 0)
      layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.MATCH_PARENT, 1f)
      setOnClickListener { showConnectDialog(webView) }
    }

    val btnConnect = makePillButton("Instances", "Connect to instance URL") {
      showConnectDialog(webView)
    }

    val btnReload = makeBarButton("↻", "Reload") {
      webView.reload()
    }

    val btnExternal = makeBarButton("↗", "Open in system browser") {
      val current = webView.url ?: DEFAULT_URL
      openExternalBrowser(current)
    }

    val btnCollapse = makeBarButton("▲", "Hide toolbar") {
      val divider = (toolbarView?.parent as? ViewGroup)?.getChildAt(1)
      setToolbarCollapsed(true, divider)
    }

    bar.addView(btnBack)
    bar.addView(btnHome)
    bar.addView(title)
    bar.addView(btnConnect)
    bar.addView(btnReload)
    bar.addView(btnExternal)
    bar.addView(btnCollapse)

    return bar
  }

  private fun makeBarButton(label: String, description: String, onClick: () -> Unit): TextView {
    return TextView(this).apply {
      text = label
      contentDescription = description
      setTextColor(TEXT_PRIMARY)
      setTextSize(TypedValue.COMPLEX_UNIT_SP, 16f)
      gravity = Gravity.CENTER
      minWidth = dp(36)
      minHeight = dp(36)
      setPadding(dp(6), dp(4), dp(6), dp(4))
      setOnClickListener { onClick() }
    }
  }

  private fun makePillButton(label: String, description: String, onClick: () -> Unit): TextView {
    return TextView(this).apply {
      text = label
      contentDescription = description
      setTextColor(ACCENT)
      setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
      typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
      gravity = Gravity.CENTER
      setPadding(dp(10), dp(5), dp(10), dp(5))
      background = GradientDrawable().apply {
        setColor(0x1F38BDF8)
        setStroke(dp(1), 0x4438BDF8)
        cornerRadius = dp(14).toFloat()
      }
      val lp = LinearLayout.LayoutParams(
        ViewGroup.LayoutParams.WRAP_CONTENT,
        ViewGroup.LayoutParams.WRAP_CONTENT
      ).apply {
        marginEnd = dp(4)
      }
      layoutParams = lp
      setOnClickListener { onClick() }
    }
  }

  private fun setToolbarCollapsed(collapsed: Boolean, divider: View?) {
    toolbarView?.visibility = if (collapsed) View.GONE else View.VISIBLE
    divider?.visibility = if (collapsed) View.GONE else View.VISIBLE
    restorePill?.visibility = if (collapsed) View.VISIBLE else View.GONE
    prefs().edit().putBoolean(KEY_TOOLBAR_COLLAPSED, collapsed).apply()
  }

  private fun updateBackButtonState(webView: WebView) {
    val canBack = webView.canGoBack()
    backButton?.alpha = if (canBack) 1.0f else 0.35f
    backButton?.isEnabled = canBack
  }

  private fun showConnectDialog(webView: WebView) {
    val container = LinearLayout(this).apply {
      orientation = LinearLayout.VERTICAL
      setPadding(dp(20), dp(14), dp(20), dp(6))
    }

    val urlInput = EditText(this).apply {
      hint = "https://antigravity.google.com/r/<instance-id>"
      setText(webView.url ?: DEFAULT_URL)
      inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
      setSingleLine(true)
    }
    container.addView(urlInput)

    val nameInput = EditText(this).apply {
      hint = "Optional bookmark label (e.g. Office PC)"
      inputType = InputType.TYPE_CLASS_TEXT
      setSingleLine(true)
    }
    container.addView(nameInput)

    val bookmarks = loadBookmarks()
    if (bookmarks.isNotEmpty()) {
      val header = TextView(this).apply {
        text = "Saved Remote Instances (tap to open, long-press to remove)"
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
        setTextColor(TEXT_MUTED)
        setPadding(0, dp(12), 0, dp(6))
      }
      container.addView(header)

      val listLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
      }
      val dialogHolder = arrayOfNulls<AlertDialog>(1)
      for ((name, url) in bookmarks) {
        val row = TextView(this).apply {
          text = "$name\n$url"
          setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
          setPadding(dp(10), dp(8), dp(10), dp(8))
          setOnClickListener {
            webView.loadUrl(url)
            dialogHolder[0]?.dismiss()
          }
          setOnLongClickListener {
            removeBookmark(url)
            Toast.makeText(this@MainActivity, "Removed bookmark: $name", Toast.LENGTH_SHORT).show()
            dialogHolder[0]?.dismiss()
            showConnectDialog(webView)
            true
          }
        }
        listLayout.addView(row)
      }
      val scroll = ScrollView(this).apply {
        addView(listLayout)
      }
      container.addView(scroll)

      val dialog = AlertDialog.Builder(this)
        .setTitle("Connect to Antigravity")
        .setView(container)
        .setPositiveButton("Connect") { _, _ ->
          val target = normalizeUrl(urlInput.text.toString())
          val label = nameInput.text.toString().trim()
          if (label.isNotEmpty()) {
            saveBookmark(label, target)
          }
          webView.loadUrl(target)
        }
        .setNeutralButton("Clear Session") { _, _ ->
          confirmClearSession(webView)
        }
        .setNegativeButton("Cancel", null)
        .create()
      dialogHolder[0] = dialog
      dialog.show()
    } else {
      AlertDialog.Builder(this)
        .setTitle("Connect to Antigravity")
        .setView(container)
        .setPositiveButton("Connect") { _, _ ->
          val target = normalizeUrl(urlInput.text.toString())
          val label = nameInput.text.toString().trim()
          if (label.isNotEmpty()) {
            saveBookmark(label, target)
          }
          webView.loadUrl(target)
        }
        .setNeutralButton("Clear Session") { _, _ ->
          confirmClearSession(webView)
        }
        .setNegativeButton("Cancel", null)
        .show()
    }
  }

  private fun confirmClearSession(webView: WebView) {
    AlertDialog.Builder(this)
      .setTitle("Sign Out / Clear Session")
      .setMessage("Clear saved cookies and site data? You will need to sign in again.")
      .setPositiveButton("Clear") { _, _ ->
        val cm = CookieManager.getInstance()
        cm.removeAllCookies {
          cm.flush()
          runOnUiThread {
            webView.clearCache(true)
            webView.clearHistory()
            webView.loadUrl(DEFAULT_URL)
            Toast.makeText(this, "Session cleared", Toast.LENGTH_SHORT).show()
          }
        }
      }
      .setNegativeButton("Cancel", null)
      .show()
  }

  private fun openExternalBrowser(url: String) {
    try {
      startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
    } catch (_: Exception) {
      Toast.makeText(this, "No browser app available", Toast.LENGTH_SHORT).show()
    }
  }

  private fun normalizeUrl(raw: String): String {
    val trimmed = raw.trim()
    return when {
      trimmed.isEmpty() -> DEFAULT_URL
      trimmed.startsWith("https://") || trimmed.startsWith("http://") -> trimmed
      trimmed.startsWith("/r/") -> "$DEFAULT_URL$trimmed"
      trimmed.startsWith("r/") -> "$DEFAULT_URL/$trimmed"
      trimmed.contains('.') -> "https://$trimmed"
      else -> "$DEFAULT_URL/r/$trimmed"
    }
  }

  private fun extractAntigravityIntentUrl(intent: Intent?): String? {
    if (intent?.action != Intent.ACTION_VIEW) return null
    val data = intent.data ?: return null
    return when (data.scheme?.lowercase()) {
      "https", "http" -> {
        val host = data.host?.lowercase() ?: return null
        if (host == "antigravity.google.com" || host.endsWith(".antigravity.google.com") || host == "antigravity.google") {
          data.toString()
        } else {
          null
        }
      }
      "antigravity" -> {
        val path = data.path?.trimStart('/') ?: ""
        if (path.isNotEmpty()) "$DEFAULT_URL/$path" else DEFAULT_URL
      }
      else -> null
    }
  }

  private fun prefs() = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

  private fun loadBookmarks(): List<Pair<String, String>> {
    val raw = prefs().getString(KEY_BOOKMARKS, "[]") ?: "[]"
    val list = mutableListOf<Pair<String, String>>()
    try {
      val arr = JSONArray(raw)
      for (i in 0 until arr.length()) {
        val obj = arr.getJSONObject(i)
        list.add(obj.getString("name") to obj.getString("url"))
      }
    } catch (_: Exception) {
    }
    return list
  }

  private fun saveBookmark(name: String, url: String) {
    val existing = loadBookmarks().filterNot { it.second == url }.toMutableList()
    existing.add(0, name to url)
    persistBookmarks(existing)
  }

  private fun removeBookmark(url: String) {
    val updated = loadBookmarks().filterNot { it.second == url }
    persistBookmarks(updated)
  }

  private fun persistBookmarks(items: List<Pair<String, String>>) {
    val arr = JSONArray()
    for ((name, url) in items) {
      arr.put(JSONObject().put("name", name).put("url", url))
    }
    prefs().edit().putString(KEY_BOOKMARKS, arr.toString()).apply()
  }

  private fun dp(value: Int): Int {
    return TypedValue.applyDimension(
      TypedValue.COMPLEX_UNIT_DIP,
      value.toFloat(),
      resources.displayMetrics
    ).toInt()
  }
}
