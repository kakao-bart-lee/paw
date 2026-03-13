package dev.paw.android

import uniffi.paw_core.ping

object PawCoreBridge {
    fun describePing(): String = try {
        "connected (${ping()})"
    } catch (error: UnsatisfiedLinkError) {
        "bindings compiled, native lib missing (${error.message})"
    } catch (error: Throwable) {
        "bridge call failed (${error::class.simpleName}: ${error.message})"
    }
}
