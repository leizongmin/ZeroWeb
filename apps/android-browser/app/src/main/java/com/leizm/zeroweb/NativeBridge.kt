package com.leizm.zeroweb

/** JNI boundary for the Rust browser host library. */
object NativeBridge {
    init {
        System.loadLibrary("zero_android_browser")
    }

    @JvmStatic
    external fun nativeVersion(): String

    @JvmStatic
    external fun nativeStartRole(role: String): Boolean

    @JvmStatic
    external fun nativeRunRole(role: String, fd: Int): Boolean

    @JvmStatic
    external fun nativeProbeDecoder(fd: Int): Boolean
}
