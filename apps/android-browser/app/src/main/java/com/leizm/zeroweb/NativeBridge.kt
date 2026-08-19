package com.leizm.zeroweb

/** JNI boundary for the Rust browser host library. */
object NativeBridge {
    init {
        System.loadLibrary("zero_android_browser")
    }

    @JvmStatic
    external fun nativeVersion(): String

    @JvmStatic
    external fun nativeRendererLinked(): Boolean

    @JvmStatic
    external fun nativeLoadProfile(root: String): String

    @JvmStatic
    external fun nativeBrowserSnapshot(): String

    @JvmStatic
    external fun nativeNavigate(url: String): Boolean

    @JvmStatic
    external fun nativeNewTab(): Boolean

    @JvmStatic
    external fun nativeNewTabWithUrl(url: String): Boolean

    @JvmStatic
    external fun nativeCloseTab(id: Long): Boolean

    @JvmStatic
    external fun nativeSelectTab(id: Long): Boolean

    @JvmStatic
    external fun nativeGoBack(): Boolean

    @JvmStatic
    external fun nativeGoForward(): Boolean

    @JvmStatic
    external fun nativeToggleBookmark(): Boolean

    @JvmStatic
    external fun nativeRemoveBookmark(url: String): Boolean

    @JvmStatic
    external fun nativeClearHistory(): Boolean

    @JvmStatic
    external fun nativeStartRole(role: String): Boolean

    @JvmStatic
    external fun nativeRunRole(role: String, fd: Int): Boolean

    @JvmStatic
    external fun nativeAttachCompositor(fd: Int, width: Int, height: Int): Boolean

    @JvmStatic
    external fun nativeCompositorTestFrame(width: Int, height: Int): ByteArray?

    @JvmStatic
    external fun nativeAttachRenderer(fd: Int): Boolean

    @JvmStatic
    external fun nativeLatestPageFrame(): ByteArray?

    @JvmStatic
    external fun nativeScroll(deltaY: Float): Boolean

    @JvmStatic
    external fun nativeProbeDecoder(fd: Int): Boolean

    @JvmStatic
    external fun nativeProbeCompositor(fd: Int): Boolean
}
