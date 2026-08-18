package com.leizm.zeroweb

import android.app.Service
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Bundle
import android.os.IBinder
import android.os.ParcelFileDescriptor
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import org.json.JSONObject

/** Android launcher Activity for the ZeroWeb browser process. */
class MainActivity : ComponentActivity() {
    private val serviceConnections = mutableListOf<ServiceConnection>()
    private var readyServiceCount by mutableStateOf(0)
    private var browserState by mutableStateOf(BrowserSnapshot.empty())
    private var browserError by mutableStateOf<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        loadBrowserProfile()
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
                )
            }
        }
    }

    override fun onDestroy() {
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

    private fun loadBrowserProfile() {
        applySnapshot(NativeBridge.nativeLoadProfile(filesDir.resolve("profile").absolutePath))
    }

    private fun refreshBrowserSnapshot() {
        applySnapshot(NativeBridge.nativeBrowserSnapshot())
    }

    private fun navigate(url: String) {
        if (NativeBridge.nativeNavigate(url)) {
            refreshBrowserSnapshot()
        } else {
            browserError = "仅支持有效的 HTTP(S) 地址"
        }
    }

    private fun newTab() {
        if (NativeBridge.nativeNewTab()) refreshBrowserSnapshot()
    }

    private fun selectTab(id: Long) {
        if (NativeBridge.nativeSelectTab(id)) refreshBrowserSnapshot()
    }

    private fun closeTab(id: Long) {
        if (NativeBridge.nativeCloseTab(id)) refreshBrowserSnapshot()
    }

    private fun toggleBookmark() {
        if (NativeBridge.nativeToggleBookmark()) refreshBrowserSnapshot()
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
                    Thread {
                        if (NativeBridge.nativeProbeCompositor(sockets[0].detachFd())) {
                            android.util.Log.i("ZeroWebRole", "compositor probe succeeded")
                        } else {
                            android.util.Log.e("ZeroWebRole", "compositor probe failed")
                        }
                    }.start()
                }
            }

            override fun onServiceDisconnected(name: ComponentName) {
                readyServiceCount = (readyServiceCount - 1).coerceAtLeast(0)
            }
        }
        serviceConnections += connection
        bindService(Intent(this, roleService), connection, Context.BIND_AUTO_CREATE)
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
) {
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
        OutlinedTextField(
            value = address,
            onValueChange = { address = it },
            modifier = Modifier.fillMaxWidth().testTag("addressBar"),
            label = { Text("地址") },
            singleLine = true,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
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
        Text(text = "页面渲染器正在准备；当前 chrome 状态已由 Rust profile 持久化。")
        error?.let { Text(text = it, color = MaterialTheme.colorScheme.error) }
        Text(text = nativeVersion, style = MaterialTheme.typography.labelSmall)
    }
}

private data class BrowserTab(val id: Long, val url: String?, val title: String?) {
    val displayTitle: String get() = title ?: url ?: "新标签"
}

private data class BrowserSnapshot(
    val activeTabId: Long?,
    val tabs: List<BrowserTab>,
    val bookmarked: Boolean,
    val bookmarkCount: Int,
    val historyCount: Int,
    val downloadCount: Int,
) {
    companion object {
        fun empty() = BrowserSnapshot(null, emptyList(), false, 0, 0, 0)

        fun fromJson(raw: String): BrowserSnapshot {
            val json = JSONObject(raw)
            check(!json.has("error")) { json.getString("error") }
            val tabs = json.getJSONArray("tabs")
            return BrowserSnapshot(
                activeTabId = if (json.isNull("activeTabId")) null else json.getLong("activeTabId"),
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
            )
        }
    }
}
