package dev.kdrag0n.quicklock.server

import android.util.Base64
import com.squareup.moshi.JsonAdapter
import com.squareup.moshi.JsonReader
import com.squareup.moshi.JsonWriter
import com.squareup.moshi.Moshi
import dagger.Module
import dagger.Provides
import dagger.Reusable
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import retrofit2.Retrofit
import retrofit2.converter.moshi.MoshiConverterFactory

@Module
@InstallIn(SingletonComponent::class)
object ApiModule {
    @Provides
    @Reusable
    fun provideApiService(moshi: Moshi) = Retrofit.Builder().run {
        baseUrl("http://192.168.20.127:3002/")
        addConverterFactory(MoshiConverterFactory.create(moshi))
        build()
    }.create(ApiService::class.java)

    @Provides
    @Reusable
    fun provideAuditService(moshi: Moshi) = Retrofit.Builder().run {
        baseUrl("http://192.168.20.127:3002/")
        addConverterFactory(MoshiConverterFactory.create(moshi))
        build()
    }.create(AuditService::class.java)

    @Provides
    @Reusable
    fun provideMoshi() = Moshi.Builder().run {
        add(ByteArray::class.java, ByteArrayAdapter)
        build()
    }
}

private object ByteArrayAdapter : JsonAdapter<ByteArray>() {
    override fun fromJson(reader: JsonReader): ByteArray? {
        val string = reader.nextString()
        return string?.let { Base64.decode(it, Base64.NO_WRAP) }
    }

    override fun toJson(writer: JsonWriter, value: ByteArray?) {
        writer.value(value?.let { Base64.encodeToString(it, Base64.NO_WRAP) })
    }
}
