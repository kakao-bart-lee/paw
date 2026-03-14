package dev.paw.android.presentation.navigation

import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavHostController
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
}

private val fadeEnter: EnterTransition = fadeIn(
    initialAlpha = 0.3f,
)
private val fadeExit: ExitTransition = fadeOut(
    targetAlpha = 0f,
)

@Composable
fun PawNavGraph(viewModel: BootstrapViewModel) {
    val navController = rememberNavController()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val authStep = uiState.preview.auth.step

    // Navigate based on auth state changes
    LaunchedEffect(authStep) {
        val currentRoute = navController.currentDestination?.route
        val isOnAuthScreen = currentRoute == null ||
            currentRoute == PawRoutes.WELCOME ||
            currentRoute == PawRoutes.LOGIN_METHOD ||
            currentRoute == PawRoutes.PHONE_INPUT ||
            currentRoute == PawRoutes.OTP_VERIFY ||
            currentRoute == PawRoutes.DEVICE_REGISTER ||
            currentRoute == PawRoutes.USERNAME_SETUP

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
            else -> { /* auth step navigation handled by user actions */ }
        }
    }

    val startDestination = if (authStep == AuthStepView.AUTHENTICATED) {
        PawRoutes.CHAT_LIST
    } else {
        PawRoutes.WELCOME
    }

    NavHost(
        navController = navController,
        startDestination = startDestination,
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
            ChatDetailScreen(
                chatId = chatId,
                navController = navController,
                viewModel = viewModel,
            )
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
