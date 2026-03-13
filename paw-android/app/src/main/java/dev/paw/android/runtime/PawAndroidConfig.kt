package dev.paw.android.runtime

import dev.paw.android.BuildConfig

object PawAndroidConfig {
    const val defaultBaseUrl = "http://10.0.2.2:38173"

    val apiBaseUrl: String
        get() = BuildConfig.PAW_API_BASE_URL.ifBlank { defaultBaseUrl }
}
