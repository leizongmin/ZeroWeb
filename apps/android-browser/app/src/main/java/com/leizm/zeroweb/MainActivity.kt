package com.leizm.zeroweb

import android.app.Service
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.Bundle
import android.os.IBinder
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp

/** Android launcher Activity for the ZeroWeb browser process. */
class MainActivity : ComponentActivity() {
    private val serviceConnections = mutableListOf<ServiceConnection>()
    private var readyServiceCount by mutableStateOf(0)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        bindBootstrapRoles()
        setContent {
            MaterialTheme {
                BootstrapScreen(
                    nativeVersion = NativeBridge.nativeVersion(),
                    readyServiceCount = readyServiceCount,
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

    private fun bindRole(roleService: Class<out Service>) {
        val connection = object : ServiceConnection {
            override fun onServiceConnected(name: ComponentName, service: IBinder) {
                readyServiceCount += 1
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
private fun BootstrapScreen(nativeVersion: String, readyServiceCount: Int) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
            .testTag("zeroWebBootstrap"),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text(
            text = stringResource(R.string.bootstrap_title),
            style = MaterialTheme.typography.headlineMedium,
        )
        Text(
            text = if (readyServiceCount == 3) {
                stringResource(R.string.bootstrap_ready)
            } else {
                stringResource(R.string.bootstrap_starting)
            },
        )
        Text(text = nativeVersion)
        Text(text = stringResource(R.string.bootstrap_role_summary))
    }
}
