// SplashScreen.kt -- the short opening film (res/raw/splash_juns_garden.mp4),
// shown once each time the app starts. If the film cannot play, the app
// moves on anyway after at most 12 seconds, so nobody is ever locked out.

package org.jasonagibson.junsgarden.garden

import android.net.Uri
import android.widget.VideoView
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import org.jasonagibson.junsgarden.R

/**
 * The longest the splash is ever allowed to hold the app.
 *
 * A BACKSTOP, NOT THE LENGTH OF THE FILM. Normally the video ends and the
 * app moves on. This exists for the cases where it never does -- a codec the
 * device will not decode, a player that reports neither completion nor
 * error -- because the one thing a splash screen must never be is a way to
 * be locked out of the app.
 */
private const val SPLASH_CEILING_MILLIS = 12_000L

/**
 * The backdrop of Jason's drawing, sampled from its top edge.
 *
 * The video is 16:9 and a phone is not, so there is ground either side of
 * it. Painting that ground the drawing's own sky green means it reads as
 * one picture rather than as a letterboxed rectangle in a black field.
 *
 * Sampled from the TOP edge specifically: the bottom of the frame is nearly
 * black, because roses and soil run right to it, so an average of all four
 * corners would produce a muddy colour that matches nothing.
 */
private val DRAWING_GROUND = Color(0xFFD7E9B9)

/**
 * Jun's garden, animated, while the app starts.
 *
 * THE STILL IS SHOWN UNDERNEATH THE VIDEO, always. A VideoView paints
 * nothing at all until the first frame is decoded, which on a cold start is
 * long enough to read as a flash of empty colour. Laying the poster frame
 * behind it means the drawing is on screen from the first moment and the
 * video simply starts moving over the top of it. It is also what is left if
 * the video cannot be played at all.
 *
 * SKIPPABLE, AND SKIPPABLE ANYWHERE. Jason asked for a skip, so there is a
 * button -- but the whole screen is a target too, because the only thing
 * worse than a splash is one you have to aim at to dismiss.
 *
 * ONCE PER LAUNCH, not once per screen. The caller holds the flag, so
 * signing in or returning from the camera does not play it again.
 */
@Composable
fun SplashScreen(onDone: () -> Unit) {
    // Guards against onDone arriving twice -- the video completing at the
    // same moment he taps Skip would otherwise advance two screens.
    var finished by remember { mutableStateOf(false) }
    val done = {
        if (!finished) {
            finished = true
            onDone()
        }
    }

    LaunchedEffect(Unit) {
        delay(SPLASH_CEILING_MILLIS)
        done()
    }

    // No ripple: this is a picture being tapped away, and a spreading grey
    // circle over Jun's face looks like a fault.
    val noIndication = remember { MutableInteractionSource() }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(DRAWING_GROUND)
            .clickable(
                interactionSource = noIndication,
                indication = null,
                onClick = done,
            ),
        contentAlignment = Alignment.Center,
    ) {
        Image(
            painter = painterResource(R.drawable.splash_juns_garden),
            contentDescription = "Jun in his garden, holding a sign that " +
                "reads Jun's Garden",
            modifier = Modifier.fillMaxWidth(),
            // FIT, NOT CROP. Cropping a 16:9 drawing to a tall screen cuts
            // Jun out of his own splash -- he is dead centre in the frame.
            contentScale = ContentScale.Fit,
        )

        AndroidView(
            modifier = Modifier.fillMaxWidth(),
            factory = { context ->
                VideoView(context).apply {
                    setVideoURI(
                        Uri.parse(
                            "android.resource://${context.packageName}/"
                                + R.raw.splash_juns_garden
                        )
                    )
                    setOnPreparedListener { player ->
                        // Silent. The app may be opened anywhere, and a
                        // splash that makes noise in a quiet room is a
                        // reason to uninstall it.
                        player.setVolume(0f, 0f)
                        start()
                    }
                    setOnCompletionListener { done() }
                    // A device that cannot decode this keeps the still and
                    // moves on, rather than showing the system's "Can't play
                    // this video" dialog over the top of the splash.
                    setOnErrorListener { _, _, _ ->
                        done()
                        true
                    }
                }
            },
            onRelease = { it.stopPlayback() },
        )

        TextButton(
            onClick = done,
            modifier = Modifier
                .align(Alignment.TopEnd)
                // The button, unlike the picture, has to clear the clock.
                .windowInsetsPadding(WindowInsets.systemBars)
                .padding(8.dp),
        ) {
            Text(
                "Skip",
                style = MaterialTheme.typography.labelLarge,
                // Dark on the light green ground, not the theme's onSurface,
                // which in dark mode would be near-white on near-white.
                color = Color(0xFF1B3A1B),
            )
        }
    }
}
