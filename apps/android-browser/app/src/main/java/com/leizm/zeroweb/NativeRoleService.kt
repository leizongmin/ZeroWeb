package com.leizm.zeroweb

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.os.ParcelFileDescriptor
import android.util.Log

/** Base class for private Android process roles backed by the Rust native library. */
abstract class NativeRoleService : Service() {
    private val binder = object : IRoleService.Stub() {
        override fun start(socket: ParcelFileDescriptor) {
            if (role != "renderer" && role != "image-decoder" && role != "compositor") {
                socket.close()
                return
            }
            if (!NativeBridge.nativeRunRole(role, slot, socket.detachFd())) {
                Log.e(TAG, "native role transport bootstrap rejected: $role#$slot")
            }
        }
    }

    protected abstract val role: String

    /** RFC §6.3：预声明 Service 槽位号；renderer 兼作 compositor 帧的 surface_id。 */
    protected open val slot: Int = 0

    override fun onCreate() {
        super.onCreate()
        if (!NativeBridge.nativeStartRole(role)) {
            Log.e(TAG, "native role bootstrap rejected: $role")
            stopSelf()
            return
        }
        Log.i(TAG, "native role ready: $role")
    }

    override fun onBind(intent: Intent?): IBinder = binder

    private companion object {
        const val TAG = "ZeroWebRole"
    }
}

class RendererService0 : NativeRoleService() { override val role = "renderer"; override val slot = 0 }
class RendererService1 : NativeRoleService() { override val role = "renderer"; override val slot = 1 }
class RendererService2 : NativeRoleService() { override val role = "renderer"; override val slot = 2 }
class RendererService3 : NativeRoleService() { override val role = "renderer"; override val slot = 3 }
class RendererService4 : NativeRoleService() { override val role = "renderer"; override val slot = 4 }
class RendererService5 : NativeRoleService() { override val role = "renderer"; override val slot = 5 }
class RendererService6 : NativeRoleService() { override val role = "renderer"; override val slot = 6 }
class RendererService7 : NativeRoleService() { override val role = "renderer"; override val slot = 7 }
class CompositorService : NativeRoleService() { override val role = "compositor" }
class ImageDecoderService : NativeRoleService() { override val role = "image-decoder" }
