package dev.paw.android.runtime

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.paw_core.LifecycleEvent
import uniffi.paw_core.LifecycleHint
import uniffi.paw_core.LifecycleState
import uniffi.paw_core.`lifecycleHints`

class PawLifecycleBridge : DefaultLifecycleObserver {
    private val _state = MutableStateFlow(LifecycleState.LAUNCHING)
    private val _activeHints = MutableStateFlow(computeHints(LifecycleState.ACTIVE))
    private val _backgroundHints = MutableStateFlow(computeHints(LifecycleState.BACKGROUND))

    val state: StateFlow<LifecycleState> = _state.asStateFlow()
    val activeHints: StateFlow<List<LifecycleHint>> = _activeHints.asStateFlow()
    val backgroundHints: StateFlow<List<LifecycleHint>> = _backgroundHints.asStateFlow()

    override fun onStart(owner: LifecycleOwner) {
        publish(LifecycleState.ACTIVE)
    }

    override fun onResume(owner: LifecycleOwner) {
        publish(LifecycleState.ACTIVE)
    }

    override fun onPause(owner: LifecycleOwner) {
        publish(LifecycleState.INACTIVE)
    }

    override fun onStop(owner: LifecycleOwner) {
        publish(LifecycleState.BACKGROUND)
    }

    private fun publish(nextState: LifecycleState) {
        _state.value = nextState
        _activeHints.value = computeHints(LifecycleState.ACTIVE)
        _backgroundHints.value = computeHints(LifecycleState.BACKGROUND)
    }

    private fun computeHints(state: LifecycleState): List<LifecycleHint> = `lifecycleHints`(
        LifecycleEvent(
            state = state,
            timestampMs = System.currentTimeMillis(),
            userInitiated = false,
        ),
    )
}
