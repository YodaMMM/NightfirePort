package com.nightfire.android

import android.view.Surface

object NativeBridge {
    init {
        System.loadLibrary("dxandroid")
    }

    const val BTN_FIRE = 1 shl 0
    const val BTN_AIM = 1 shl 1
    const val BTN_RELOAD = 1 shl 2
    const val BTN_JUMP = 1 shl 3
    const val BTN_CROUCH = 1 shl 4
    const val BTN_USE = 1 shl 5
    const val BTN_WEAP_NEXT = 1 shl 6
    const val BTN_WEAP_PREV = 1 shl 7
    const val BTN_PAUSE = 1 shl 8

    const val MENU_UP = 0
    const val MENU_DOWN = 1
    const val MENU_LEFT = 2
    const val MENU_RIGHT = 3
    const val MENU_CONFIRM = 4
    const val MENU_BACK = 5
    const val MENU_SIDE = 6
    const val MENU_EXTRA = 7

    external fun nativeInit(dataPath: String): Boolean
    external fun nativeSurfaceCreated(surface: Surface, width: Int, height: Int): Boolean
    external fun nativeSurfaceChanged(width: Int, height: Int)
    external fun nativeSurfaceDestroyed()
    external fun nativeLoadLevel(archive: String): Boolean
    external fun nativeSetInput(moveX: Float, moveY: Float, lookDx: Float, lookDy: Float, buttonsMask: Int)
    external fun nativeMenuAction(actionId: Int)
    external fun nativeStepAndRender(): Int
    external fun nativeIsInMenu(): Boolean
    external fun nativeListLevels(): String
    external fun nativeGetStatus(): String
}
