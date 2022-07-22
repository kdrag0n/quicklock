package dev.kdrag0n.quicklock

object NativeLib {
    init {
        System.loadLibrary("quicklock")
    }

    @JvmStatic external fun init()
}
