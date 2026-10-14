// GardenSignInScreen.kt -- the first screen: log in, or "Continue as guest".
//
// Guests can look at every plant and use Barcode. Logging in (GardenLogin)
// is needed for Identify, Tag and Receipt. "Forgot password" opens the
// website's own page in the browser.

package org.jasonagibson.junsgarden.garden

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import org.jasonagibson.junsgarden.BuildConfig

/**
 * The first screen: log in as Jun or the author, or carry on as a guest.
 *
 * MODELLED ON THE ORIGINAL "Jun & Kona's Garden" APP (1.0, June 2026), whose
 * sign-in screen this is: a seedling, the name, Username, Password with a
 * show/hide switch, Sign In, and "Continue as guest". Guests see everything a
 * visitor to the website sees; logging in is only for changing things.
 *
 * Glyphs rather than icons: the icon library is not a dependency of this
 * project, and a megabyte of APK for three symbols is not worth it.
 */
@Composable
fun GardenSignInScreen(
    login: GardenLogin,
    onLoggedIn: (String) -> Unit,
    onGuest: () -> Unit,
) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var show by remember { mutableStateOf(false) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }

    fun submit() {
        if (busy || username.isBlank() || password.isEmpty()) return
        busy = true
        error = null
        scope.launch {
            login.login(username, password).fold(
                onSuccess = { name -> password = ""; onLoggedIn(name) },
                onFailure = { e -> error = e.message ?: "Login failed." },
            )
            busy = false
        }
    }

    Column(
        modifier = Modifier.fillMaxSize().padding(horizontal = 28.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("🌱", fontSize = 56.sp)
        Spacer(Modifier.height(8.dp))
        Text(
            "Jun & Kona's Garden",
            style = MaterialTheme.typography.headlineSmall,
            textAlign = TextAlign.Center,
        )
        Text(
            "Sign in to continue",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.primary,
        )
        Spacer(Modifier.height(28.dp))

        OutlinedTextField(
            value = username,
            onValueChange = { username = it; error = null },
            label = { Text("Username") },
            leadingIcon = { Text("👤") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            keyboardOptions = KeyboardOptions(
                capitalization = KeyboardCapitalization.Words,
                autoCorrectEnabled = false,
            ),
        )
        Spacer(Modifier.height(12.dp))
        OutlinedTextField(
            value = password,
            onValueChange = { password = it; error = null },
            label = { Text("Password") },
            leadingIcon = { Text("🔒") },
            trailingIcon = {
                TextButton(onClick = { show = !show }) { Text(if (show) "Hide" else "Show") }
            },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            visualTransformation = if (show) VisualTransformation.None else PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, autoCorrectEnabled = false),
        )
        error?.let {
            Spacer(Modifier.height(10.dp))
            Text(
                it,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.error,
                textAlign = TextAlign.Center,
            )
        }
        Spacer(Modifier.height(20.dp))
        Button(
            onClick = { submit() },
            enabled = !busy && username.isNotBlank() && password.isNotEmpty(),
            modifier = Modifier.fillMaxWidth().height(48.dp),
        ) { Text(if (busy) "Signing in…" else "Sign In") }

        Spacer(Modifier.height(16.dp))
        TextButton(onClick = onGuest) { Text("Continue as guest") }
        Text(
            "Guests can browse the whole catalog.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.primary,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(8.dp))
        // Forgot it, or never had one: the site emails a link to the address
        // on file. Done in the browser so the app never handles that link.
        TextButton(onClick = {
            context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(GardenLogin.forgotPasswordUrl)))
        }) { Text("Forgot password? / First time?") }
        Text(
            "v${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
