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
import android.text.InputType
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.activity.compose.setContent
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.Image
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONObject
import java.nio.ByteBuffer

/** 标签缩略图缩放目标宽（像素）：56dp @2x，缓存体量与解码成本折中。 */
private const val TAB_THUMB_WIDTH_PX = 112

/** Android launcher Activity for the ZeroWeb browser process. */
class MainActivity : ComponentActivity() {
    private val serviceConnections = mutableListOf<ServiceConnection>()
    private var rendererSocket: ParcelFileDescriptor? = null
    private var rendererConnection: ServiceConnection? = null
    private var readyServiceCount by mutableStateOf(0)

    /** 待补导航的 URL：槽重绑 attach 完成后消费（断连/换槽恢复）。 */
    private var pendingRestoreUrl: String? = null
    /** 恢复尝试节流：单标签连续 3 次未成功即停，防止 renderer 反复死亡引发循环重绑。 */
    private var restoreAttempts = 0
    /** 正在绑定中的 renderer 槽号（onServiceConnected 到达前防重复触发 rebind）。 */
    private var pendingRendererBindSlot: Int? = null

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
    /** 页面 IME 托管视图（预览下方 1dp 隐形槽），键盘开关时接管软键盘。 */
    private var pageInputView: PageInputView? = null
    private var keyboardRequested by mutableStateOf(false)
    /** 非活动标签缩略图缓存（tabId → 缩放后小图；活动标签走大预览，被逐标签无帧）。 */
    private val tabThumbnails = mutableStateMapOf<Long, ImageBitmap>()

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
                    onPageTap = ::pageTap,
                    keyboardRequested = keyboardRequested,
                    onToggleKeyboard = ::toggleKeyboard,
                    onPageInputViewCreated = ::registerPageInputView,
                    tabThumbnails = tabThumbnails,
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
        restoreActiveTabRenderer()
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
            restoreActiveTabRenderer()
        } else {
            browserError = "仅支持 HTTP(S) 外部地址"
        }
    }

    private fun refreshBrowserSnapshot() {
        applySnapshot(NativeBridge.nativeBrowserSnapshot())
    }

    private fun navigate(url: String) {
        if (NativeBridge.nativeNavigate(url)) {
            restoreAttempts = 0
            pendingRestoreUrl = null
            refreshBrowserSnapshot()
            refreshRendererPreview()
        } else {
            refreshBrowserSnapshot()
            val slot = browserState?.activeRendererSlot
            if (NativeBridge.nativeRendererLinked() && slot != null &&
                !NativeBridge.nativeIsRendererAttached(slot)
            ) {
                // 渲染进程死亡/LRU 换槽后该槽未附着：按快照指示重绑对应槽，
                // attach 完成后补导航本次目标 URL。
                pendingRestoreUrl = url
                rebindRenderer(slot)
                browserError = "渲染进程恢复中，请重试"
            } else {
                browserError = "仅支持有效的 HTTP(S) 地址"
            }
        }
    }

    /**
     * 活动标签渲染恢复（LRU 逐出切回/启动恢复/外部intent新标签）：有 URL 而无附着槽
     * 时先导航（native 侧顺带分配槽），失败则按快照指示重绑并挂起补导航。
     */
    private fun restoreActiveTabRenderer() {
        if (!NativeBridge.nativeRendererLinked()) return
        val snapshot = browserState ?: return
        val url = snapshot.tabs.firstOrNull { it.id == snapshot.activeTabId }?.url ?: return
        val slot = snapshot.activeRendererSlot
        if (slot != null && NativeBridge.nativeIsRendererAttached(slot)) {
            // 槽在：有帧直接刷新预览；无帧（attach 后从未加载本标签内容）补导航
            if (rendererPreview == null) navigate(url) else refreshRendererPreview()
            return
        }
        if (restoreAttempts >= 3) {
            android.util.Log.e("ZeroWebRole", "renderer restore exceeded attempts")
            return
        }
        if (NativeBridge.nativeNavigate(url)) {
            refreshRendererPreview()
            return
        }
        refreshBrowserSnapshot()
        val newSlot = browserState?.activeRendererSlot
        if (newSlot != null && !NativeBridge.nativeIsRendererAttached(newSlot)) {
            pendingRestoreUrl = url
            if (pendingRendererBindSlot == newSlot) {
                android.util.Log.i("ZeroWebRole", "restore pending bind of slot $newSlot")
            } else {
                rebindRenderer(newSlot)
            }
        }
    }

    private fun newTab() {
        if (NativeBridge.nativeNewTab()) refreshBrowserSnapshot()
    }

    private fun selectTab(id: Long) {
        if (NativeBridge.nativeSelectTab(id)) {
            restoreAttempts = 0
            refreshBrowserSnapshot()
            restoreActiveTabRenderer()
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
                refreshTabThumbnails()
            }
            .onFailure { browserError = it.message ?: "无法读取浏览器状态" }
    }

    /**
     * 非活动标签缩略图刷新（M4 切片 8）：后台标签帧在其非活动期间不变，故每标签
     * 只解码一次并缩放缓存；活动标签走大预览，被逐/关闭标签条目随快照清理。
     */
    private fun refreshTabThumbnails() {
        val snapshot = browserState ?: return
        val seen = mutableSetOf<Long>()
        snapshot.tabs.forEach { tab ->
            val slot = tab.rendererSlot
            if (tab.id != snapshot.activeTabId && slot != null && !tabThumbnails.containsKey(tab.id)) {
                NativeBridge.nativeLatestPageFrame(slot)?.toPageBitmap()?.let { full ->
                    val scale = TAB_THUMB_WIDTH_PX.toFloat() / full.width
                    val thumb =
                        Bitmap.createScaledBitmap(full, TAB_THUMB_WIDTH_PX, (full.height * scale).toInt().coerceAtLeast(1), true)
                    tabThumbnails[tab.id] = thumb.asImageBitmap()
                }
            }
            if (tab.id != snapshot.activeTabId) seen += tab.id
        }
        tabThumbnails.keys.removeAll { id -> id !in seen }
    }

    private fun bindRole(roleService: Class<out Service>) {
        val connection = object : ServiceConnection {
            override fun onServiceConnected(name: ComponentName, service: IBinder) {
                readyServiceCount += 1
                if (roleService in rendererServiceClasses) {
                    pendingRendererBindSlot = null
                }
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
                if (roleService == CompositorService::class.java) {
                    // compositor 进程死亡（renderer/compositor death 处理，RFC M3，
                    // 对称于 renderer 断连恢复）：作废本地附着标记并清除 native 僵尸
                    // transport，让 BIND_AUTO_CREATE 重启后的 onServiceConnected
                    // 重新走 attach 协议，不退回进程内执行。
                    compositorAttached = false
                    NativeBridge.nativeDetachCompositor()
                    android.util.Log.w("ZeroWebRole", "compositor service disconnected; native transport detached")
                }
            }
        }
        serviceConnections += connection
        val rendererSlotIndex = rendererServiceClasses.indexOf(roleService)
        if (rendererSlotIndex >= 0) {
            rendererConnection = connection
            pendingRendererBindSlot = rendererSlotIndex
        }
        bindService(Intent(this, roleService), connection, Context.BIND_AUTO_CREATE)
    }

    /** renderer 断连/换槽后的重绑：关旧 socket、解绑旧连接、按槽重新走 bind → socketpair → attach。 */
    private fun rebindRenderer(slot: Int) {
        if (restoreAttempts >= 3) {
            android.util.Log.e("ZeroWebRole", "renderer rebind exceeded attempts for slot $slot")
            browserError = "渲染进程恢复失败，请稍后重试"
            return
        }
        restoreAttempts += 1
        rendererSocket?.close()
        rendererSocket = null
        rendererConnection?.let { connection ->
            unbindService(connection)
            serviceConnections.remove(connection)
        }
        rendererConnection = null
        bindRole(rendererServiceClasses[slot])
    }

    /**
     * 页面视口（CSS 宽、高、密度）：按真实 display 推算（M3 切片 5）——CSS 宽取物理宽
     * /density 并夹在移动端典型区间，高按屏幕纵横比推算；帧像素 = css × density，
     * 单边上限 1280 防 RGBA 帧 IPC 内存放大（密度随之下调，待真机调优）。
     */
    private fun computePageViewport(): Triple<Int, Int, Float> {
        val metrics = resources.displayMetrics
        val widthPx = metrics.widthPixels.coerceAtLeast(1)
        val heightPx = metrics.heightPixels.coerceAtLeast(1)
        val cssWidth = (widthPx / metrics.density).coerceIn(320f, 480f)
        val cssHeight = (cssWidth * heightPx / widthPx).coerceIn(320f, 1024f)
        val density = minOf(metrics.density, 1280f / cssWidth, 1280f / cssHeight).coerceIn(1f, 3f)
        return Triple(cssWidth.toInt(), cssHeight.toInt(), density)
    }

    private fun attachRendererIfReady() {
        val socket = rendererSocket ?: return
        // attach 目标槽 = 快照指示的活动标签槽（缺省 0 兼容启动早期无快照）
        val slot = browserState?.activeRendererSlot ?: 0
        val (cssWidth, cssHeight, density) = computePageViewport()
        if (!compositorAttached ||
            !NativeBridge.nativeAttachRenderer(slot, socket.detachFd(), cssWidth, cssHeight, density)
        ) {
            return
        }
        rendererSocket = null
        // 断连/换槽恢复链：attach 完成即补导航被挂起的 URL（如为空则刷新预览）
        val restoreUrl = pendingRestoreUrl
        pendingRestoreUrl = null
        if (restoreUrl != null) {
            navigate(restoreUrl)
        } else {
            refreshRendererPreview()
        }
    }

    private fun refreshRendererPreview() {
        // 被逐标签（有快照但无槽）没有本标签帧：预览留空，不可错拿其他槽的帧
        val snapshot = browserState
        val slot = if (snapshot == null) 0 else snapshot.activeRendererSlot
        if (snapshot != null && slot == null) {
            rendererPreview = null
            return
        }
        val targetSlot = slot ?: 0
        listOf(1_000L, 5_000L, 10_000L).forEach { delayMillis ->
            window.decorView.postDelayed({
                rendererPreview = NativeBridge.nativeLatestPageFrame(targetSlot)?.toPageBitmap()
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

    /** 预览点击 → 活动标签渲染槽的 DOM click（坐标按预览显示区归一化）。 */
    private fun pageTap(normX: Float, normY: Float) {
        if (NativeBridge.nativePageTap(normX, normY)) refreshRendererPreview()
    }

    /** 键盘开关：开启时焦点交给页面 IME 托管视图并弹出软键盘，输入经 nativePageText/Key 落页。 */
    private fun toggleKeyboard() {
        val inputView = pageInputView
        val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
        if (inputView == null || imm == null) return
        if (keyboardRequested) {
            keyboardRequested = false
            inputView.imeRequested = false
            imm.hideSoftInputFromWindow(inputView.windowToken, 0)
            inputView.clearFocus()
        } else {
            keyboardRequested = true
            inputView.imeRequested = true
            inputView.requestFocus()
            imm.showSoftInput(inputView, InputMethodManager.SHOW_IMPLICIT)
        }
    }

    private fun registerPageInputView(view: PageInputView) {
        pageInputView = view
        view.onInputCommitted = ::refreshRendererPreview
    }
}

/**
 * 页面文本输入托管视图（RFC §IF-003 InputConnection 的最小实现）：开启键盘后接管
 * 软键盘连接，commitText → ImeEvent::Commit（CJK 主通路）、删除/回车 → 特殊键
 * keydown+keyup，全部落到活动标签渲染槽的焦点元素。
 */
private class PageInputView(context: Context) : View(context) {
    /** 键盘开关状态：关闭时 onCreateInputConnection 返回 null（IME 不弹）。 */
    var imeRequested = false

    /** 输入落地后的刷新回调（注入 Activity 的预览刷新）。 */
    var onInputCommitted: (() -> Unit)? = null

    init {
        isFocusable = true
        isFocusableInTouchMode = true
    }

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection? {
        if (!imeRequested) return null
        outAttrs.inputType = InputType.TYPE_CLASS_TEXT
        outAttrs.imeOptions = EditorInfo.IME_FLAG_NO_EXTRACT_UI
        return object : BaseInputConnection(this, false) {
            override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
                val committed = text != null && NativeBridge.nativePageText(text.toString())
                if (committed) onInputCommitted?.invoke()
                return committed
            }

            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
                val deleted = NativeBridge.nativePageKey("Backspace")
                if (deleted) onInputCommitted?.invoke()
                return deleted
            }

            override fun performEditorAction(actionCode: Int): Boolean {
                val done = NativeBridge.nativePageKey("Enter")
                if (done) onInputCommitted?.invoke()
                return done
            }
        }
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
    onPageTap: (Float, Float) -> Unit,
    keyboardRequested: Boolean,
    onToggleKeyboard: () -> Unit,
    onPageInputViewCreated: (PageInputView) -> Unit,
    tabThumbnails: Map<Long, ImageBitmap>,
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
                tabThumbnails[tab.id]?.let { thumb ->
                    Image(
                        bitmap = thumb,
                        contentDescription = null, // 装饰性缩略图：语义由标题文本承载
                        modifier = Modifier.size(56.dp, 36.dp),
                    )
                }
                TextButton(onClick = { onSelectTab(tab.id) }, modifier = Modifier.weight(1f)) {
                    Text(if (tab.id == snapshot.activeTabId) "● ${tab.displayTitle}" else tab.displayTitle)
                }
                TextButton(onClick = { onCloseTab(tab.id) }) { Text("关闭") }
            }
        }
        Text(text = activeTab?.url ?: "新标签")
        rendererPreview?.let { preview ->
            var totalDragY = 0f
            var previewSize by remember { mutableStateOf(IntSize.Zero) }
            Image(
                bitmap = preview.asImageBitmap(),
                contentDescription = "来自 renderer 与 compositor 的页面帧",
                modifier = Modifier
                    .fillMaxWidth()
                    .onSizeChanged { previewSize = it }
                    .pointerInput(Unit) {
                        detectVerticalDragGestures(
                            onVerticalDrag = { _, amount -> totalDragY += amount },
                            onDragEnd = {
                                onPageScroll(-totalDragY)
                                totalDragY = 0f
                            },
                        )
                    }
                    .pointerInput(Unit) {
                        // 点击坐标按显示区归一化，native 侧映射回页面视口（RFC M3 触摸）
                        detectTapGestures { offset ->
                            if (previewSize.width > 0 && previewSize.height > 0) {
                                onPageTap(
                                    (offset.x / previewSize.width).coerceIn(0f, 1f),
                                    (offset.y / previewSize.height).coerceIn(0f, 1f),
                                )
                            }
                        }
                    }
                    .testTag("rendererPreview"),
            )
            // 页面 IME 托管视图：1dp 隐形槽承载软键盘连接，输入经 native 落到焦点元素
            AndroidView(
                factory = { context -> PageInputView(context).also(onPageInputViewCreated) },
                modifier = Modifier.fillMaxWidth().height(1.dp),
            )
            TextButton(onClick = onToggleKeyboard) {
                Text(if (keyboardRequested) "收起键盘" else "键盘")
            }
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

/** 页面帧解码：8 字节小端 (width, height) 头 + RGBA（native 侧已做有界校验，此处防御复核）。 */
private fun ByteArray.toPageBitmap(): Bitmap? {
    if (size < 8) return null
    val header = ByteBuffer.wrap(this).order(java.nio.ByteOrder.LITTLE_ENDIAN)
    val width = header.int
    val height = header.int
    if (width <= 0 || height <= 0 || width > 4_096 || height > 4_096) return null
    val rgba = ByteBuffer.wrap(this, 8, size - 8)
    if (rgba.remaining() != width * height * 4) return null
    return Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888).also { bitmap ->
        bitmap.copyPixelsFromBuffer(rgba)
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

private data class BrowserTab(val id: Long, val url: String?, val title: String?, val rendererSlot: Int?) {
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
                        rendererSlot = if (tab.isNull("rendererSlot")) null else tab.getInt("rendererSlot"),
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
