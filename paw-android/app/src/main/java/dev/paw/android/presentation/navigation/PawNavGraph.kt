package dev.paw.android.presentation.navigation

import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import dev.paw.android.presentation.agent.AgentDetailScreen
import dev.paw.android.presentation.agent.AgentHubScreen
import dev.paw.android.presentation.auth.DeviceRegisterScreen
import dev.paw.android.presentation.auth.LoginMethodScreen
import dev.paw.android.presentation.auth.OtpVerifyScreen
import dev.paw.android.presentation.auth.PhoneInputScreen
import dev.paw.android.presentation.auth.UsernameSetupScreen
import dev.paw.android.presentation.auth.WelcomeScreen
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.chat.ChatDetailScreen
import dev.paw.android.presentation.chat.ChatListScreen
import dev.paw.android.presentation.chat.GroupCreateScreen
import dev.paw.android.presentation.chat.NewChatScreen
import dev.paw.android.presentation.search.SearchScreen
import dev.paw.android.presentation.settings.ProfileScreen
import dev.paw.android.presentation.settings.SecurityScreen
import dev.paw.android.presentation.settings.SettingsScreen
import dev.paw.android.presentation.theme.PawBackground
import uniffi.paw_core.AuthStepView

object PawRoutes {
    const val WELCOME = "welcome"
    const val LOGIN_METHOD = "login-method"
    const val PHONE_INPUT = "phone-input"
    const val OTP_VERIFY = "otp-verify"
    const val DEVICE_REGISTER = "device-register"
    const val USERNAME_SETUP = "username-setup"
    const val CHAT_LIST = "chat-list"
    const val CHAT_DETAIL = "chat-detail/{chatId}"
    const val NEW_CHAT = "new-chat"
    const val GROUP_CREATE = "group-create"
    const val SEARCH = "search"
    const val AGENT_HUB = "agent-hub"
    const val AGENT_DETAIL = "agent-detail/{agentId}"
    const val SETTINGS = "settings"
    const val SECURITY = "security"
    const val PROFILE = "profile"

    fun chatDetail(chatId: String) = "chat-detail/$chatId"
    fun agentDetail(agentId: String) = "agent-detail/$agentId"

    private val authScreens = setOf(WELCOME, LOGIN_METHOD, PHONE_INPUT, OTP_VERIFY, DEVICE_REGISTER, USERNAME_SETUP)

    fun isAuthScreen(route: String?): Boolean = route in authScreens
}

private val fadeEnter: EnterTransition = fadeIn(initialAlpha = 0.3f)
private val fadeExit: ExitTransition = fadeOut(targetAlpha = 0f)

@Composable
fun PawNavGraph(viewModel: BootstrapViewModel) {
    val navController = rememberNavController()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val authStep = uiState.preview.auth.step
    val bootstrapReady = uiState.bootstrapReady

    // Show nothing until bootstrap has resolved the initial auth state
    if (!bootstrapReady) {
        Box(modifier = Modifier.fillMaxSize().background(PawBackground))
        return
    }

    // Single centralized navigation watcher — no per-screen LaunchedEffects for auth steps
    LaunchedEffect(authStep) {
        val currentRoute = navController.currentDestination?.route
        val isOnAuthScreen = PawRoutes.isAuthScreen(currentRoute) || currentRoute == null

        when (authStep) {
            AuthStepView.AUTHENTICATED -> {
                if (isOnAuthScreen) {
                    navController.navigate(PawRoutes.CHAT_LIST) {
                        popUpTo(0) { inclusive = true }
                    }
                }
            }
            AuthStepView.AUTH_METHOD_SELECT -> {
                if (!isOnAuthScreen) {
                    navController.navigate(PawRoutes.WELCOME) {
                        popUpTo(0) { inclusive = true }
                    }
                }
            }
            AuthStepView.PHONE_INPUT -> {
                if (isOnAuthScreen) {
                    navController.navigate(PawRoutes.PHONE_INPUT) {
                        popUpTo(PawRoutes.WELCOME)
                    }
                }
            }
            AuthStepView.OTP_VERIFY -> {
                if (isOnAuthScreen) {
                    navController.navigate(PawRoutes.OTP_VERIFY) {
                        popUpTo(PawRoutes.WELCOME)
                    }
                }
            }
            AuthStepView.DEVICE_NAME -> {
                if (isOnAuthScreen) {
                    navController.navigate(PawRoutes.DEVICE_REGISTER) {
                        popUpTo(PawRoutes.WELCOME)
                    }
                }
            }
            AuthStepView.USERNAME_SETUP -> {
                if (isOnAuthScreen) {
                    navController.navigate(PawRoutes.USERNAME_SETUP) {
                        popUpTo(PawRoutes.WELCOME)
                    }
                }
            }
        }
    }

    NavHost(
        navController = navController,
        startDestination = PawRoutes.WELCOME,
        enterTransition = { fadeEnter },
        exitTransition = { fadeExit },
        popEnterTransition = { fadeEnter },
        popExitTransition = { fadeExit },
    ) {
        // ── Auth flow ────────────────────────────────────────────────
        composable(PawRoutes.WELCOME) {
            WelcomeScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.LOGIN_METHOD) {
            LoginMethodScreen(navController = navController)
        }
        composable(PawRoutes.PHONE_INPUT) {
            PhoneInputScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.OTP_VERIFY) {
            OtpVerifyScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.DEVICE_REGISTER) {
            DeviceRegisterScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.USERNAME_SETUP) {
            UsernameSetupScreen(navController = navController, viewModel = viewModel)
        }

        // ── Main app ─────────────────────────────────────────────────
        composable(PawRoutes.CHAT_LIST) {
            ChatListScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.CHAT_DETAIL) { backStackEntry ->
            val chatId = backStackEntry.arguments?.getString("chatId") ?: return@composable
            ChatDetailScreen(chatId = chatId, navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.NEW_CHAT) {
            NewChatScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.GROUP_CREATE) {
            GroupCreateScreen(navController = navController)
        }
        composable(PawRoutes.SEARCH) {
            SearchScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.AGENT_HUB) {
            AgentHubScreen(navController = navController)
        }
        composable(PawRoutes.AGENT_DETAIL) { backStackEntry ->
            val agentId = backStackEntry.arguments?.getString("agentId") ?: return@composable
            AgentDetailScreen(agentId = agentId, navController = navController)
        }
        composable(PawRoutes.SETTINGS) {
            SettingsScreen(navController = navController, viewModel = viewModel)
        }
        composable(PawRoutes.SECURITY) {
            SecurityScreen(navController = navController)
        }
        composable(PawRoutes.PROFILE) {
            ProfileScreen(navController = navController, viewModel = viewModel)
        }
    }
}
