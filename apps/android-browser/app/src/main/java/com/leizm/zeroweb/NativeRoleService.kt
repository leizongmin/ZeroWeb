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
            if (!NativeBridge.nativeRunRole(role, socket.detachFd())) {
                Log.e(TAG, "native role transport bootstrap rejected: $role")
            }
        }
    }

    protected abstract val role: String

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

class RendererService0 : NativeRoleService() { override val role = "renderer" }
class RendererService1 : NativeRoleService() { override val role = "renderer" }
class RendererService2 : NativeRoleService() { override val role = "renderer" }
class RendererService3 : NativeRoleService() { override val role = "renderer" }
class RendererService4 : NativeRoleService() { override val role = "renderer" }
class RendererService5 : NativeRoleService() { override val role = "renderer" }
class RendererService6 : NativeRoleService() { override val role = "renderer" }
class RendererService7 : NativeRoleService() { override val role = "renderer" }
class CompositorService : NativeRoleService() { override val role = "compositor" }
class ImageDecoderService : NativeRoleService() { override val role = "image-decoder" }
