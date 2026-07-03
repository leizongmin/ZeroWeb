package com.zeroweb.ui

import android.app.Activity
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import android.view.WindowInsets

class MainActivity : Activity(), SurfaceHolder.Callback {

    companion object {
        init {
            System.loadLibrary("zero_ui_adapter_android")
        }
    }

    // ── JNI externs（与 ui/adapters/android/src/ffi.rs C ABI 符号对应）──────

    private external fun nativeInitRuntime(): Boolean
    private external fun nativeWindowResize(width: Int, height: Int, scale: Float)
    private external fun nativeDispatchTouch(pointerId: Int, action: Int, x: Float, y: Float, timestampMs: Long)
    private external fun nativeDispatchKey(keyCode: Int, action: Int)
    private external fun nativeBackPressed(): Boolean
    private external fun nativeSoftKeyboard(height: Int, visible: Boolean)
    private external fun nativeIsRuntimeReady(): Boolean
    private external fun nativePumpEvents()
    private external fun nativeShutdown()

    private var surfaceView: SurfaceView? = null
    private var renderThread: Thread? = null
    private var running = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        surfaceView = findViewById<SurfaceView>(R.id.surface_view).apply {
            holder.addCallback(this@MainActivity)
        }
    }

    override fun onDestroy() {
        running = false
        renderThread?.join(1000)
        nativeShutdown()
        super.onDestroy()
    }

    // ── SurfaceHolder.Callback ─────────────────────────────────────────

    override fun surfaceCreated(holder: SurfaceHolder) {
        val w = surfaceView?.width ?: holder.surfaceFrame.width()
        val h = surfaceView?.height ?: holder.surfaceFrame.height()
        startRuntime(w, h)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        nativeWindowResize(width, height, resources.displayMetrics.density)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        running = false
        renderThread?.join(1000)
    }

    // ── Runtime ────────────────────────────────────────────────────────

    private fun startRuntime(width: Int, height: Int) {
        if (running) return

        if (!nativeInitRuntime()) {
            android.util.Log.e("ZeroBrowser", "Runtime init failed")
            return
        }

        nativeWindowResize(width, height, resources.displayMetrics.density)

        running = true
        renderThread = Thread {
            while (running && nativeIsRuntimeReady()) {
                nativePumpEvents()
                // Render loop: pump events → render frame via Rust side
                Thread.sleep(16) // ~60fps
            }
        }.apply {
            name = "ZeroRenderLoop"
            start()
        }
    }

    // ── Touch ──────────────────────────────────────────────────────────

    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (!nativeIsRuntimeReady()) return false

        for (i in 0 until event.pointerCount) {
            val pointerId = event.getPointerId(i)
            val x = event.getX(i)
            val y = event.getY(i)
            val actionMasked = event.actionMasked
            // Map ACTION_UP/DOWN/MOVE to simple action code per pointer
            val touchAction = when (actionMasked) {
                MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> 0
                MotionEvent.ACTION_MOVE -> 2
                MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP, MotionEvent.ACTION_CANCEL -> 1
                else -> return false
            }
            nativeDispatchTouch(pointerId, touchAction, x, y, event.eventTime)
        }
        return true
    }

    // ── Keyboard ───────────────────────────────────────────────────────

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        if (nativeIsRuntimeReady()) {
            nativeDispatchKey(keyCode, 0) // ACTION_DOWN
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (nativeIsRuntimeReady()) {
            nativeDispatchKey(keyCode, 1) // ACTION_UP
        }
        return super.onKeyUp(keyCode, event)
    }

    // ── Back ───────────────────────────────────────────────────────────

    override fun onBackPressed() {
        if (nativeIsRuntimeReady() && nativeBackPressed()) {
            return // consumed by Rust runtime
        }
        super.onBackPressed()
    }

    // ── Window insets (soft keyboard) ──────────────────────────────────

    override fun onApplyWindowInsets(insets: WindowInsets): WindowInsets {
        if (nativeIsRuntimeReady()) {
            val keyboardHeight = insets.systemWindowInsetBottom
            val visible = keyboardHeight > 0
            nativeSoftKeyboard(keyboardHeight, visible)
        }
        return super.onApplyWindowInsets(insets)
    }
}
