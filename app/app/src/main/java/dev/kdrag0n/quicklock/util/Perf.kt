package dev.kdrag0n.quicklock.util

import android.os.SystemClock
import android.util.Log

inline fun <T> profileLog(tag: String, block: () -> T): T {
    val start = SystemClock.uptimeMillis()
    val result = block()
    val end = SystemClock.uptimeMillis()
    val duration = end - start
    Log.d("Prof-$tag", "$duration ms")
    return result
}
