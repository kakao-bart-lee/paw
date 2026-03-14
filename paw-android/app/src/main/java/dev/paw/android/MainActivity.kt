package dev.paw.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.lifecycle.ProcessLifecycleOwner
import dev.paw.android.presentation.bootstrap.BootstrapScreen
import dev.paw.android.presentation.bootstrap.BootstrapViewModel

class MainActivity : ComponentActivity() {
    private val viewModel by viewModels<BootstrapViewModel>()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        ProcessLifecycleOwner.get().lifecycle.addObserver(viewModel.lifecycleObserver())
        setContent {
            BootstrapScreen(viewModel = viewModel)
        }
    }
}
