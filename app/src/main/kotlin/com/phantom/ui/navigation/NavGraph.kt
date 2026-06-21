package com.phantom.ui.navigation

import androidx.compose.runtime.Composable
import androidx.navigation.NavHostController
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.phantom.ui.screens.AddContactScreen
import com.phantom.ui.screens.ChatListScreen
import com.phantom.ui.screens.ChatScreen
import com.phantom.ui.screens.QrDisplayScreen
import com.phantom.ui.screens.QrScannerScreen
import com.phantom.ui.screens.SettingsScreen

object Routes {
    const val CHAT_LIST = "chat_list"
    const val CHAT = "chat/{contactId}"
    const val SETTINGS = "settings"
    const val ADD_CONTACT = "add_contact"
    const val QR_DISPLAY = "qr_display"
    const val QR_SCANNER = "qr_scanner"

    fun chat(contactId: Long) = "chat/$contactId"
}

@Composable
fun PhantomNavGraph(
    navController: NavHostController = rememberNavController(),
) {
    NavHost(navController = navController, startDestination = Routes.CHAT_LIST) {
        composable(Routes.CHAT_LIST) {
            ChatListScreen(
                onOpenChat = { navController.navigate(Routes.chat(it)) },
                onAddContact = { navController.navigate(Routes.ADD_CONTACT) },
                onSettings = { navController.navigate(Routes.SETTINGS) },
            )
        }
        composable(
            route = Routes.CHAT,
            arguments = listOf(navArgument("contactId") { type = NavType.LongType }),
        ) { backStackEntry ->
            val contactId = backStackEntry.arguments?.getLong("contactId") ?: return@composable
            ChatScreen(
                contactId = contactId,
                onBack = { navController.popBackStack() },
            )
        }
        composable(Routes.SETTINGS) {
            SettingsScreen(
                onBack = { navController.popBackStack() },
                onShowQr = { navController.navigate(Routes.QR_DISPLAY) },
            )
        }
        composable(Routes.ADD_CONTACT) {
            AddContactScreen(
                onBack = { navController.popBackStack() },
                onScan = { navController.navigate(Routes.QR_SCANNER) },
            )
        }
        composable(Routes.QR_DISPLAY) {
            QrDisplayScreen(onBack = { navController.popBackStack() })
        }
        composable(Routes.QR_SCANNER) {
            QrScannerScreen(
                onBack = { navController.popBackStack() },
                onScanned = { content ->
                    navController.previousBackStackEntry
                        ?.savedStateHandle
                        ?.set("scanned_qr", content)
                    navController.popBackStack()
                },
            )
        }
    }
}
