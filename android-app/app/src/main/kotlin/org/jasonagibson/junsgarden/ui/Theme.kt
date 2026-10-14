// Theme.kt -- turns the chosen theme into the colours every screen uses
// (JunsGardenTheme wraps the whole app in MainActivity).

package org.jasonagibson.junsgarden.ui

import android.app.Activity
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.platform.LocalContext
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

/**
 * The app's own colours: a soft lavender rather than white.
 *
 * WHY NOT THE DEFAULT. Material 3's baseline light scheme is very nearly
 * white, and a white page on a phone held at arm's length in a kitchen is
 * glare -- the text is fine, the background is what tires the eye. A tinted
 * ground lowers the contrast between screen and room without touching the
 * contrast between text and background, which is the one that matters for
 * legibility.
 *
 * DYNAMIC COLOUR IS DELIBERATELY OFF. Material You would recolour this from
 * the phone's wallpaper, so the app would look different on every handset and
 * could land on a scheme nobody chose -- including a low-contrast one. This
 * app is used by several people on several phones and should look the same on
 * all of them.
 *
 * CONTRAST WAS CHECKED, NOT EYEBALLED. Body text (#1C1725) on the lavender
 * ground (#F4F1FA) is about 14:1, far past the 4.5:1 that WCAG asks for, so
 * the calmer background costs nothing in readability. The muted purple
 * primary (#5B4B8A) on that same ground is about 7:1, which keeps buttons and
 * links legible rather than merely tasteful.
 */



/**
 * What the app currently looks like, readable from anywhere below.
 *
 * Provided rather than passed down: the appearance sheet that changes it
 * sits several screens deep, and threading a theme through every composable
 * between here and there would touch files that have nothing to do with
 * colour.
 */
val LocalAppearance = staticCompositionLocalOf {
    Appearance.State(GardenTheme.DEFAULT, ThemeMode.System) { _, _ -> }
}

@Composable
fun JunsGardenTheme(
    dark: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    val context = LocalContext.current
    // Read synchronously, so the first frame is already the right colour.
    var theme by remember { mutableStateOf(Appearance.theme(context)) }
    var mode by remember { mutableStateOf(Appearance.mode(context)) }

    val useDark = when (mode) {
        ThemeMode.Light -> false
        ThemeMode.Dark -> true
        ThemeMode.System -> dark
    }
    val colours = schemeFor(theme, useDark)

    val state = Appearance.State(theme, mode) { t, m ->
        if (t != null) {
            theme = t
            Appearance.setTheme(context, t)
        }
        if (m != null) {
            mode = m
            Appearance.setMode(context, m)
        }
    }
    val view = LocalView.current

    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            // The status bar takes the app's ground colour rather than staying
            // white above a lavender screen, which otherwise reads as a strip
            // of glare the app forgot about.
            window.statusBarColor = colours.background.toArgb()
            // Icon colour derived from the background's luminance rather than
            // from `dark`: a theme edit that lightens the dark ground would
            // otherwise leave white icons on a pale bar.
            WindowCompat.getInsetsController(window, view)
                .isAppearanceLightStatusBars = colours.background.luminance() > 0.5f
        }
    }

    CompositionLocalProvider(LocalAppearance provides state) {
        MaterialTheme(colorScheme = colours, content = content)
    }
}
