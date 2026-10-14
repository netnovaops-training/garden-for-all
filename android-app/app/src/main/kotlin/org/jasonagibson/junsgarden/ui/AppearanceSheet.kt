// AppearanceSheet.kt -- the "Theme" panel: pick a colour theme, and pick
// light mode, dark mode, or follow the phone.

package org.jasonagibson.junsgarden.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

/**
 * Pick a theme, and pick light or dark.
 *
 * SHOWS THE COLOUR, NOT JUST THE NAME. "Ivy" and "Evergreen" are both green
 * and nobody can tell which is which from the word, so each row carries two
 * real swatches taken from the theme itself -- its ground and its accent, in
 * the light/dark form currently in use. Choosing is then looking rather than
 * guessing and undoing.
 */
@Composable
fun AppearanceSheet(dark: Boolean, onClose: () -> Unit) {
    val appearance = LocalAppearance.current

    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.32f))
            .clickable(onClick = onClose),
    ) {
        Surface(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .fillMaxSize(0.70f),
            shape = MaterialTheme.shapes.extraLarge,
            color = MaterialTheme.colorScheme.background,
            tonalElevation = 3.dp,
        ) {
            Column(Modifier.fillMaxSize().padding(20.dp)) {
                Text("Appearance", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(14.dp))

                Text(
                    "Light or dark",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(Modifier.height(6.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    ThemeMode.entries.forEach { m ->
                        FilterChip(
                            selected = appearance.mode == m,
                            onClick = { appearance.change(null, m) },
                            label = { Text(m.label) },
                        )
                    }
                }

                Spacer(Modifier.height(18.dp))
                Text(
                    "Theme",
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(Modifier.height(6.dp))

                Column(
                    Modifier.fillMaxWidth().weight(1f),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    GardenTheme.entries.forEach { t ->
                        val preview = schemeFor(t, dark)
                        val chosen = appearance.theme == t
                        Row(
                            Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(12.dp))
                                .background(
                                    if (chosen) MaterialTheme.colorScheme.primaryContainer
                                    else MaterialTheme.colorScheme.surfaceVariant
                                )
                                .clickable { appearance.change(t, null) }
                                .padding(horizontal = 14.dp, vertical = 12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Swatch(preview.background)
                            Spacer(Modifier.size(6.dp))
                            Swatch(preview.primary)
                            Spacer(Modifier.size(14.dp))
                            Text(
                                t.label,
                                style = MaterialTheme.typography.bodyLarge,
                                fontWeight =
                                    if (chosen) FontWeight.SemiBold else FontWeight.Normal,
                                modifier = Modifier.weight(1f),
                            )
                            if (chosen) Text("\u2713")
                        }
                    }
                }

                TextButton(onClick = onClose) { Text("Done") }
            }
        }
    }
}

/** One colour, outlined so a pale swatch is still visible on a pale sheet. */
@Composable
private fun Swatch(colour: Color) {
    Box(
        Modifier
            .size(22.dp)
            .clip(CircleShape)
            .background(colour)
            .border(1.dp, MaterialTheme.colorScheme.outline, CircleShape)
    )
}
