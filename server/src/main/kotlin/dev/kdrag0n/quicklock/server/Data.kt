package dev.kdrag0n.quicklock.server

import kotlinx.serialization.Serializable
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.ObjectInputStream
import java.io.ObjectOutputStream

@Serializable
object EmptyObject

fun java.io.Serializable.serializeToByteArray(): ByteArray {
    val bos = ByteArrayOutputStream()
    ObjectOutputStream(bos).use {
        it.writeObject(this)
        it.flush()
        return bos.toByteArray()
    }
}

fun ByteArray.decodeSerializable(): Any? {
    val bis = ByteArrayInputStream(this)
    return ObjectInputStream(bis).use {
        it.readObject()
    }
}
