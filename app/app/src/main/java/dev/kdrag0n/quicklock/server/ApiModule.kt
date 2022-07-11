package dev.kdrag0n.quicklock.server

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
    fun provideApiService() = Retrofit.Builder().run {
        baseUrl("http://192.168.20.127:3002/")
        addConverterFactory(MoshiConverterFactory.create())
        build()
    }.create(ApiService::class.java)

    @Provides
    @Reusable
    fun provideMoshi() = Moshi.Builder().run {
        build()
    }
}
