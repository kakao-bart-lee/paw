package dev.paw.android.di

import android.content.Context
import dev.paw.android.data.local.AndroidDeviceKeyStore
import dev.paw.android.data.local.AndroidSecureTokenVault
import dev.paw.android.data.remote.PawApiClient
import dev.paw.android.data.repository.AuthRepositoryImpl
import dev.paw.android.data.repository.ChatRepositoryImpl
import dev.paw.android.domain.repository.AuthRepository
import dev.paw.android.domain.repository.ChatRepository
import dev.paw.android.runtime.FirebasePushRegistrar
import dev.paw.android.runtime.PawAndroidConfig

/**
 * Simple manual DI container that wires all dependencies.
 * No Hilt/Koin required -- just straightforward constructor injection.
 */
class AppContainer(context: Context) {

    private val apiClient = PawApiClient(PawAndroidConfig.apiBaseUrl)
    private val tokenVault = AndroidSecureTokenVault(context)
    private val deviceKeyStore = AndroidDeviceKeyStore(context)
    private val pushRegistrar = FirebasePushRegistrar(apiClient)

    val authRepository: AuthRepository = AuthRepositoryImpl(
        apiClient = apiClient,
        tokenVault = tokenVault,
        deviceKeyStore = deviceKeyStore,
        pushRegistrar = pushRegistrar,
    )

    val chatRepository: ChatRepository = ChatRepositoryImpl(
        apiClient = apiClient,
    )
}
