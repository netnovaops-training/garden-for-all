// MainActivity.kt -- the front door of the app.
//
// Android starts here when you tap the Jun's Garden icon. It shows the
// splash film once, then the sign-in screen, then the garden. Once per
// launch it also asks the website whether a newer version of the app exists
// (see app/Updater.kt) and, if so, offers it in a small dialog.
// Screens it opens: SplashScreen, GardenSignInScreen, GardenScreen.

package org.jasonagibson.junsgarden

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.launch
import org.jasonagibson.junsgarden.app.Updater
import org.jasonagibson.junsgarden.garden.GardenCamera
import org.jasonagibson.junsgarden.garden.GardenException
import org.jasonagibson.junsgarden.garden.GardenLogin
import org.jasonagibson.junsgarden.garden.GardenScreen
import org.jasonagibson.junsgarden.garden.GardenSignInScreen
import org.jasonagibson.junsgarden.garden.SplashScreen
import org.jasonagibson.junsgarden.ui.JunsGardenTheme

/**
 * The app: a splash, then the sign-in screen, then the garden.
 *
 * WHO CAN DO WHAT (3.0, 2026-10-06). Exactly the website's rule: anybody
 * can browse as a guest; Jun and Jason log in with their garden passwords
 * (see GardenLogin) to change things. Until 2.9 this app signed in through
 * Jason's household sign-in service instead; that, the trial codes and the
 * household-AI tools (Ask, Identify, Tag, Receipt) are not used any more,
 * because the garden is Jun's now and must not depend on Jason's servers.
 * Their code (the old auth/ and chat/ folders) was removed in October 2026.
 * Identify, Tag and Receipt came back in 3.1, now through the garden site.
 */
class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // A share that LAUNCHED the app arrives here...
        Shared.offer(intent)

        setContent {
            // SHOWN ONCE PER LAUNCH. Held above the theme so nothing inside
            // can reset it; onCreate is the only thing that runs again, and
            // that only happens on a genuine cold start.
            var showSplash by rememberSaveable { mutableStateOf(true) }
            JunsGardenTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    // Explicit rather than inherited: Surface defaults to the
                    // `surface` colour, which is a shade lighter than the
                    // ground and would leave the app looking white again.
                    color = MaterialTheme.colorScheme.background,
                ) {
                    // The drawing, once per cold start, edge to edge.
                    if (showSplash) {
                        SplashScreen(onDone = { showSplash = false })
                        return@Surface
                    }
                    // THE STATUS BAR IS NOT FREE SPACE. targetSdk 36 draws
                    // every app edge to edge; without this the top of the
                    // screen sits under the clock and its taps are lost.
                    // systemBars, not safeDrawing: the keyboard is handled
                    // by the screens that have text boxes.
                    Box(modifier = Modifier.windowInsetsPadding(WindowInsets.systemBars)) {
                        GardenApp()
                    }
                }
            }
        }
    }

    /**
     * ...and a share into an ALREADY-RUNNING app arrives here instead.
     * launchMode is singleTask, so onCreate is not called again.
     */
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        Shared.offer(intent)
    }
}

/** Who is using the app right now. */
private sealed interface Who {
    /** A saved login is being checked with the site. */
    data object Checking : Who
    /** Nobody yet: show the sign-in screen. */
    data object Out : Who
    data object Guest : Who
    data class In(val name: String) : Who
}

@Composable
private fun GardenApp() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val login = remember { GardenLogin(context) }
    var who by remember { mutableStateOf<Who>(if (login.token() != null) Who.Checking else Who.Out) }

    // Two cameras. This first one is for guests.
    // Guests: Barcode only (decoded on the phone, looked up in the public
    // catalogue), so this camera never has a key to send.
    val camera = remember {
        GardenCamera { Result.failure(GardenException("Log in to use the camera tools.")) }
    }
    val loggedInCamera = remember {
        GardenCamera {
            login.token()?.let { Result.success(it) }
                ?: Result.failure(GardenException("Log in again to use the camera."))
        }
    }

    // The update offer: a few-hundred-byte manifest checked once per launch,
    // so the APK is only downloaded when there is genuinely a newer one.
    val updater = remember { Updater() }
    var update by remember { mutableStateOf<Updater.Available?>(null) }
    var progress by remember { mutableStateOf(-1) }
    var updateError by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        if (who is Who.Checking) {
            // false = the site says the key is no good (changed password, or
            // logged out elsewhere). null = no signal: carry on as the saved
            // name rather than logging somebody out over a weak connection.
            who = when (login.check()) {
                false -> Who.Out
                else -> Who.In(login.savedName() ?: "")
            }
        }
        update = runCatching { updater.check() }.getOrNull()
    }

    when (val w = who) {
        is Who.Checking -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        is Who.Out -> GardenSignInScreen(
            login = login,
            onLoggedIn = { who = Who.In(it) },
            onGuest = { who = Who.Guest },
        )
        is Who.Guest -> GardenScreen(
            signedInAs = "Guest",
            camera = camera,
            guest = true,
            accountLabel = "Sign in",
            onSignIn = { who = Who.Out },
        ) {}
        is Who.In -> GardenScreen(
            signedInAs = w.name,
            // Logged in: the camera tools, sent to the garden site with the
            // garden login key (GardenLogin), read there by free cloud AI.
            camera = loggedInCamera,
            guest = false,
            showAsk = false,
            accountLabel = "Log out ${w.name}",
            onSignIn = {
                scope.launch {
                    login.logout()
                    who = Who.Out
                }
            },
        ) {}
    }

    update?.let { up ->
        AlertDialog(
            onDismissRequest = { if (progress < 0) update = null },
            title = { Text("Update available: ${up.versionName}") },
            text = {
                Text(
                    when {
                        updateError != null -> updateError!!
                        progress in 0..99 -> "Downloading $progress%"
                        else -> up.notes.ifBlank { "A newer version of the garden app is ready." }
                    }
                )
            },
            confirmButton = {
                if (progress < 0) TextButton(onClick = {
                    progress = 0
                    updateError = null
                    scope.launch {
                        runCatching { updater.downloadAndInstall(context, up) { p -> progress = p } }
                            .onFailure { e ->
                                progress = -1
                                updateError = e.message ?: "The update could not be installed."
                            }
                    }
                }) { Text("Update") }
            },
            dismissButton = {
                if (progress < 0) TextButton(onClick = { update = null }) { Text("Later") }
            },
        )
    }
}
