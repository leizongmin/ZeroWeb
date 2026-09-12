package com.leizm.zeroweb

import android.app.Service
import android.graphics.Bitmap
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Bundle
import android.os.IBinder
import android.os.ParcelFileDescriptor
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.compose.setContent
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import org.json.JSONObject
import java.nio.ByteBuffer

/** Android launcher Activity for the ZeroWeb browser process. */
class MainActivity : ComponentActivity() {
    private val serviceConnections = mutableListOf<ServiceConnection>()
    private var rendererSocket: ParcelFileDescriptor? = null
    private var rendererConnection: ServiceConnection? = null
    private var readyServiceCount by mutableStateOf(0)

    /** RFC §6.3：8 个 renderer Service 槽位类，下标即槽号（RendererServiceN = 槽 N）。 */
    private val rendererServiceClasses =
        listOf(
            RendererService0::class.java, RendererService1::class.java, RendererService2::class.java,
            RendererService3::class.java, RendererService4::class.java, RendererService5::class.java,
            RendererService6::class.java, RendererService7::class.java,
        )
    private var browserState by mutableStateOf(BrowserSnapshot.empty())
    private var browserError by mutableStateOf<String?>(null)
    private var compositorPreview by mutableStateOf<Bitmap?>(null)
    private var rendererPreview by mutableStateOf<Bitmap?>(null)
    private var compositorAttached = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        loadBrowserProfile()
        handleExternalIntent(intent)
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                if (NativeBridge.nativeGoBack()) {
                    refreshBrowserSnapshot()
                    return
                }
                isEnabled = false
                onBackPressedDispatcher.onBackPressed()
            }
        })
        bindBootstrapRoles()
        setContent {
            MaterialTheme {
                BrowserScreen(
                    nativeVersion = NativeBridge.nativeVersion(),
                    readyServiceCount = readyServiceCount,
                    snapshot = browserState,
                    error = browserError,
                    onNavigate = ::navigate,
                    onNewTab = ::newTab,
                    onSelectTab = ::selectTab,
                    onCloseTab = ::closeTab,
                    onToggleBookmark = ::toggleBookmark,
                    onGoBack = ::goBack,
                    onGoForward = ::goForward,
                    onRemoveBookmark = ::removeBookmark,
                    onClearHistory = ::clearHistory,
                    compositorPreview = compositorPreview,
                    rendererPreview = rendererPreview,
                    onPageScroll = ::scrollPage,
                )
            }
        }
    }

    override fun onDestroy() {
        rendererSocket?.close()
        rendererSocket = null
        serviceConnections.forEach(::unbindService)
        serviceConnections.clear()
        super.onDestroy()
    }

    private fun bindBootstrapRoles() {
        listOf(
            RendererService0::class.java,
            CompositorService::class.java,
            ImageDecoderService::class.java,
        ).forEach(::bindRole)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleExternalIntent(intent)
    }

    private fun loadBrowserProfile() {
        applySnapshot(NativeBridge.nativeLoadProfile(filesDir.resolve("profile").absolutePath))
    }

    private fun handleExternalIntent(intent: Intent?) {
        if (intent?.action != Intent.ACTION_VIEW) return
        val url = intent.data?.toString() ?: return
        if (url.length > 16 * 1024) {
            browserError = "外部地址过长"
            return
        }
        if (NativeBridge.nativeNewTabWithUrl(url)) {
            refreshBrowserSnapshot()
        } else {
            browserError = "仅支持 HTTP(S) 外部地址"
        }
    }

    private fun refreshBrowserSnapshot() {
        applySnapshot(NativeBridge.nativeBrowserSnapshot())
    }

    private fun navigate(url: String) {
        if (NativeBridge.nativeNavigate(url)) {
            refreshBrowserSnapshot()
            refreshRendererPreview()
        } else {
            refreshBrowserSnapshot()
            val slot = browserState?.activeRendererSlot
            if (NativeBridge.nativeRendererLinked() && slot != null &&
                !NativeBridge.nativeIsRendererAttached(slot)
            ) {
                // 渲染进程死亡/LRU 换槽后该槽未附着：按快照指示重绑对应槽
                // （android-browser goal M3 切片 1/2——断连恢复与多标签换槽）。
                rebindRenderer(slot)
                browserError = "渲染进程恢复中，请重试"
            } else {
                browserError = "仅支持有效的 HTTP(S) 地址"
            }
        }
    }

    private fun newTab() {
        if (NativeBridge.nativeNewTab()) refreshBrowserSnapshot()
    }

    private fun selectTab(id: Long) {
        if (NativeBridge.nativeSelectTab(id)) {
            refreshBrowserSnapshot()
            refreshRendererPreview()
        }
    }

    private fun closeTab(id: Long) {
        if (NativeBridge.nativeCloseTab(id)) refreshBrowserSnapshot()
    }

    private fun toggleBookmark() {
        if (NativeBridge.nativeToggleBookmark()) refreshBrowserSnapshot()
    }

    private fun goBack() {
        if (NativeBridge.nativeGoBack()) refreshBrowserSnapshot()
    }

    private fun goForward() {
        if (NativeBridge.nativeGoForward()) refreshBrowserSnapshot()
    }

    private fun removeBookmark(url: String) {
        if (NativeBridge.nativeRemoveBookmark(url)) refreshBrowserSnapshot()
    }

    private fun clearHistory() {
        if (NativeBridge.nativeClearHistory()) refreshBrowserSnapshot()
    }

    private fun applySnapshot(rawSnapshot: String) {
        runCatching { BrowserSnapshot.fromJson(rawSnapshot) }
            .onSuccess {
                browserState = it
                browserError = null
            }
            .onFailure { browserError = it.message ?: "无法读取浏览器状态" }
    }

    private fun bindRole(roleService: Class<out Service>) {
        val connection = object : ServiceConnection {
            override fun onServiceConnected(name: ComponentName, service: IBinder) {
                readyServiceCount += 1
                if (roleService in rendererServiceClasses && NativeBridge.nativeRendererLinked()) {
                    val sockets = ParcelFileDescriptor.createSocketPair()
                    IRoleService.Stub.asInterface(service).start(sockets[1])
                    rendererSocket = sockets[0]
                    android.util.Log.i("ZeroWebRole", "renderer socket connected")
                    attachRendererIfReady()
                }
                if (roleService == ImageDecoderService::class.java) {
                    val sockets = ParcelFileDescriptor.createSocketPair()
                    IRoleService.Stub.asInterface(service).start(sockets[1])
                    Thread {
                        if (NativeBridge.nativeProbeDecoder(sockets[0].detachFd())) {
                            android.util.Log.i("ZeroWebRole", "decoder probe succeeded")
                        } else {
                            android.util.Log.e("ZeroWebRole", "decoder probe failed")
                        }
                    }.start()
                }
                if (roleService == CompositorService::class.java) {
                    val sockets = ParcelFileDescriptor.createSocketPair()
                    IRoleService.Stub.asInterface(service).start(sockets[1])
                    if (NativeBridge.nativeAttachCompositor(sockets[0].detachFd(), 2, 2)) {
                        compositorAttached = true
                        compositorPreview = NativeBridge.nativeCompositorTestFrame(2, 2)?.toBitmap(2, 2)
                        android.util.Log.i("ZeroWebRole", "compositor bridge ready")
                        attachRendererIfReady()
                    } else {
                        android.util.Log.e("ZeroWebRole", "compositor bridge rejected")
                    }
                }
            }

            override fun onServiceDisconnected(name: ComponentName) {
                readyServiceCount = (readyServiceCount - 1).coerceAtLeast(0)
            }
        }
        serviceConnections += connection
        if (roleService in rendererServiceClasses) {
            rendererConnection = connection
        }
        bindService(Intent(this, roleService), connection, Context.BIND_AUTO_CREATE)
    }

    /** renderer 断连/换槽后的重绑：关旧 socket、解绑旧连接、按槽重新走 bind → socketpair → attach。 */
    private fun rebindRenderer(slot: Int) {
        rendererSocket?.close()
        rendererSocket = null
        rendererConnection?.let { connection ->
            unbindService(connection)
            serviceConnections.remove(connection)
        }
        rendererConnection = null
        bindRole(rendererServiceClasses[slot])
    }

    private fun attachRendererIfReady() {
        val socket = rendererSocket ?: return
        // attach 目标槽 = 快照指示的活动标签槽（缺省 0 兼容启动早期无快照）
        val slot = browserState?.activeRendererSlot ?: 0
        if (!compositorAttached || !NativeBridge.nativeAttachRenderer(slot, socket.detachFd())) return
        rendererSocket = null
        refreshRendererPreview()
    }

    private fun refreshRendererPreview() {
        val slot = browserState?.activeRendererSlot ?: 0
        listOf(1_000L, 5_000L, 10_000L).forEach { delayMillis ->
            window.decorView.postDelayed({
                rendererPreview = NativeBridge.nativeLatestPageFrame(slot)?.toBitmap(320, 180)
                if (rendererPreview == null) android.util.Log.e("ZeroWebRole", "renderer page frame unavailable")
                else {
                    refreshBrowserSnapshot()
                    android.util.Log.i("ZeroWebRole", "renderer page frame ready")
                }
            }, delayMillis)
        }
    }

    private fun scrollPage(deltaY: Float) {
        if (NativeBridge.nativeScroll(deltaY)) refreshRendererPreview()
    }
}

@androidx.compose.runtime.Composable
private fun BrowserScreen(
    nativeVersion: String,
    readyServiceCount: Int,
    snapshot: BrowserSnapshot,
    error: String?,
    onNavigate: (String) -> Unit,
    onNewTab: () -> Unit,
    onSelectTab: (Long) -> Unit,
    onCloseTab: (Long) -> Unit,
    onToggleBookmark: () -> Unit,
    onGoBack: () -> Unit,
    onGoForward: () -> Unit,
    onRemoveBookmark: (String) -> Unit,
    onClearHistory: () -> Unit,
    compositorPreview: Bitmap?,
    rendererPreview: Bitmap?,
    onPageScroll: (Float) -> Unit,
) {
    var page by remember { mutableStateOf(BrowserPage.BROWSE) }
    BackHandler(enabled = page != BrowserPage.BROWSE) { page = BrowserPage.BROWSE }
    val activeTab = snapshot.tabs.firstOrNull { it.id == snapshot.activeTabId }
    var address by remember(activeTab?.id, activeTab?.url) { mutableStateOf(activeTab?.url.orEmpty()) }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
            .testTag("zeroWebBrowser"),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(text = stringResource(R.string.bootstrap_title), style = MaterialTheme.typography.headlineMedium)
        Text(text = if (readyServiceCount == 3) stringResource(R.string.bootstrap_ready) else stringResource(R.string.bootstrap_starting))
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            BrowserPage.entries.forEach { candidate ->
                TextButton(onClick = { page = candidate }) { Text(candidate.label) }
            }
        }
        if (page != BrowserPage.BROWSE) {
            BrowserLibraryPage(
                page = page,
                snapshot = snapshot,
                onOpenUrl = onNavigate,
                onRemoveBookmark = onRemoveBookmark,
                onClearHistory = onClearHistory,
            )
            return@Column
        }
        OutlinedTextField(
            value = address,
            onValueChange = { address = it },
            modifier = Modifier.fillMaxWidth().testTag("addressBar"),
            label = { Text("地址") },
            singleLine = true,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            TextButton(onClick = onGoBack) { Text("后退") }
            TextButton(onClick = onGoForward) { Text("前进") }
            Button(onClick = { onNavigate(address) }, modifier = Modifier.testTag("navigateButton")) { Text("前往") }
            TextButton(onClick = onNewTab) { Text("新建标签") }
            TextButton(onClick = onToggleBookmark) { Text(if (snapshot.bookmarked) "已收藏" else "收藏") }
        }
        Text(text = "标签 ${snapshot.tabs.size} · 书签 ${snapshot.bookmarkCount} · 历史 ${snapshot.historyCount} · 下载 ${snapshot.downloadCount}")
        snapshot.tabs.forEach { tab ->
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TextButton(onClick = { onSelectTab(tab.id) }, modifier = Modifier.weight(1f)) {
                    Text(if (tab.id == snapshot.activeTabId) "● ${tab.displayTitle}" else tab.displayTitle)
                }
                TextButton(onClick = { onCloseTab(tab.id) }) { Text("关闭") }
            }
        }
        Text(text = activeTab?.url ?: "新标签")
        rendererPreview?.let { preview ->
            var totalDragY = 0f
            Image(
                bitmap = preview.asImageBitmap(),
                contentDescription = "来自 renderer 与 compositor 的页面帧",
                modifier = Modifier
                    .fillMaxWidth()
                    .pointerInput(Unit) {
                        detectVerticalDragGestures(
                            onVerticalDrag = { _, amount -> totalDragY += amount },
                            onDragEnd = {
                                onPageScroll(-totalDragY)
                                totalDragY = 0f
                            },
                        )
                    }
                    .testTag("rendererPreview"),
            )
        }
        compositorPreview?.let { preview ->
            Image(
                bitmap = preview.asImageBitmap(),
                contentDescription = "来自独立 compositor 的测试帧",
                modifier = Modifier.size(96.dp).testTag("compositorPreview"),
            )
        }
        Text(text = "页面渲染器正在准备；当前 chrome 状态已由 Rust profile 持久化。")
        error?.let { Text(text = it, color = MaterialTheme.colorScheme.error) }
        Text(text = nativeVersion, style = MaterialTheme.typography.labelSmall)
    }
}

private fun ByteArray.toBitmap(width: Int, height: Int): Bitmap? {
    if (size != width * height * 4) return null
    return Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).also { bitmap ->
        bitmap.copyPixelsFromBuffer(ByteBuffer.wrap(this))
    }
}

@androidx.compose.runtime.Composable
private fun BrowserLibraryPage(
    page: BrowserPage,
    snapshot: BrowserSnapshot,
    onOpenUrl: (String) -> Unit,
    onRemoveBookmark: (String) -> Unit,
    onClearHistory: () -> Unit,
) {
    when (page) {
        BrowserPage.BOOKMARKS -> {
            Text(text = "书签", style = MaterialTheme.typography.titleLarge)
            if (snapshot.bookmarks.isEmpty()) Text("暂无书签")
            snapshot.bookmarks.forEach { entry ->
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    TextButton(onClick = { onOpenUrl(entry.url) }, modifier = Modifier.weight(1f)) { Text(entry.displayTitle) }
                    TextButton(onClick = { onRemoveBookmark(entry.url) }) { Text("删除") }
                }
            }
        }
        BrowserPage.HISTORY -> {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(text = "历史", style = MaterialTheme.typography.titleLarge, modifier = Modifier.weight(1f))
                TextButton(onClick = onClearHistory) { Text("清除全部") }
            }
            if (snapshot.history.isEmpty()) Text("暂无历史记录")
            snapshot.history.forEach { entry ->
                TextButton(onClick = { onOpenUrl(entry.url) }, modifier = Modifier.fillMaxWidth()) { Text(entry.displayTitle) }
            }
        }
        BrowserPage.DOWNLOADS -> {
            Text(text = "下载", style = MaterialTheme.typography.titleLarge)
            if (snapshot.downloads.isEmpty()) Text("暂无下载")
            snapshot.downloads.forEach { entry ->
                Text("${entry.filename} · ${entry.state}")
                Text(entry.url, style = MaterialTheme.typography.labelSmall)
            }
        }
        BrowserPage.BROWSE -> Unit
    }
}

private enum class BrowserPage(val label: String) {
    BROWSE("浏览"),
    BOOKMARKS("书签"),
    HISTORY("历史"),
    DOWNLOADS("下载"),
}

private data class BrowserTab(val id: Long, val url: String?, val title: String?) {
    val displayTitle: String get() = title ?: url ?: "新标签"
}

private data class BrowserEntry(val title: String, val url: String) {
    val displayTitle: String get() = if (title.isBlank()) url else title
}

private data class BrowserDownload(val filename: String, val url: String, val state: String)

private data class BrowserSnapshot(
    val activeTabId: Long?,
    val activeRendererSlot: Int?,
    val tabs: List<BrowserTab>,
    val bookmarked: Boolean,
    val bookmarkCount: Int,
    val historyCount: Int,
    val downloadCount: Int,
    val bookmarks: List<BrowserEntry>,
    val history: List<BrowserEntry>,
    val downloads: List<BrowserDownload>,
) {
    companion object {
        fun empty() =
            BrowserSnapshot(null, null, emptyList(), false, 0, 0, 0, emptyList(), emptyList(), emptyList())

        fun fromJson(raw: String): BrowserSnapshot {
            val json = JSONObject(raw)
            check(!json.has("error")) { json.getString("error") }
            val tabs = json.getJSONArray("tabs")
            return BrowserSnapshot(
                activeTabId = if (json.isNull("activeTabId")) null else json.getLong("activeTabId"),
                activeRendererSlot = if (json.isNull("activeRendererSlot")) null else json.getInt("activeRendererSlot"),
                tabs = List(tabs.length()) { index ->
                    val tab = tabs.getJSONObject(index)
                    BrowserTab(
                        id = tab.getLong("id"),
                        url = if (tab.isNull("url")) null else tab.getString("url"),
                        title = if (tab.isNull("title")) null else tab.getString("title"),
                    )
                },
                bookmarked = json.getBoolean("bookmarked"),
                bookmarkCount = json.getInt("bookmarkCount"),
                historyCount = json.getInt("historyCount"),
                downloadCount = json.getInt("downloadCount"),
                bookmarks = parseEntries(json, "bookmarks"),
                history = parseEntries(json, "history"),
                downloads = parseDownloads(json),
            )
        }

        private fun parseEntries(snapshot: JSONObject, key: String): List<BrowserEntry> {
            val entries = snapshot.getJSONArray(key)
            return List(entries.length()) { index ->
                val entry = entries.getJSONObject(index)
                BrowserEntry(entry.getString("title"), entry.getString("url"))
            }
        }

        private fun parseDownloads(snapshot: JSONObject): List<BrowserDownload> {
            val downloads = snapshot.getJSONArray("downloads")
            return List(downloads.length()) { index ->
                val download = downloads.getJSONObject(index)
                BrowserDownload(download.getString("filename"), download.getString("url"), download.getString("state"))
            }
        }
    }
}
