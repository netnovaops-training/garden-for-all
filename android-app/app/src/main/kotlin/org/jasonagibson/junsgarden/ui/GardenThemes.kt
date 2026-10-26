// GardenThemes.kt -- the list of colour themes (Evergreen, Forest, Ivy and
// more) and where the chosen one is saved on the phone.

package org.jasonagibson.junsgarden.ui

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color
import org.jasonagibson.junsgarden.R

/**
 * The themes the original app had, put back.
 *
 * THE AUTHOR, 2026-09-25: "I like how the PHP page displayed the results and had
 * themes ... On the original app."
 *
 * THE NAMES ARE RECOVERED, NOT INVENTED. The old APK still carries its
 * palette names in the dex -- Evergreen, Forest, Ivy, Lavender, Sunflower,
 * Sunrise, Sunset -- alongside `GardenThemesKt` and the `AppearanceSheet`
 * that chose between them. The colours themselves are gone with the source,
 * so they are rebuilt here; the names are his.
 *
 * NOT TO BE CONFUSED WITH THE BED NAMES. The same dex also holds "Zen
 * Garden", "Korea Garden", "China Town Garden", "Uncle Dennis Garden" and
 * the rest. Those are flower beds in his actual garden, and putting one in
 * a theme menu would have been a confident, plausible mistake.
 *
 * EVERY THEME HAS BOTH A LIGHT AND A DARK FORM. A theme that only works in
 * one of them turns the light/dark switch into a trap, where picking a
 * favourite colour silently costs somebody a readable screen at night.
 */
enum class GardenTheme(@StringRes val label: Int) {
    Evergreen(R.string.theme_evergreen),
    Forest(R.string.theme_forest),
    Ivy(R.string.theme_ivy),
    Lavender(R.string.theme_lavender),
    Sunflower(R.string.theme_sunflower),
    Sunrise(R.string.theme_sunrise),
    Sunset(R.string.theme_sunset),
    ;

    companion object {
        val DEFAULT = Evergreen

        fun from(name: String?): GardenTheme =
            entries.firstOrNull { it.name.equals(name, ignoreCase = true) }
                ?: DEFAULT
    }
}

/** Light or dark, or whatever the phone is set to. */
enum class ThemeMode(@StringRes val label: Int) {
    System(R.string.mode_system),
    Light(R.string.mode_light),
    Dark(R.string.mode_dark),
    ;

    companion object {
        fun from(name: String?): ThemeMode =
            entries.firstOrNull { it.name.equals(name, ignoreCase = true) }
                ?: System
    }
}

/**
 * The choice, remembered.
 *
 * SharedPreferences RATHER THAN DataStore, deliberately, even though
 * DataStore is already a dependency. The theme has to be known BEFORE the
 * first frame is drawn; an asynchronous read means the app paints the
 * default theme and then visibly repaints in the chosen one every single
 * launch. This read is synchronous and the file is two strings.
 */
object Appearance {
    private const val FILE = "appearance"
    private const val KEY_THEME = "theme"
    private const val KEY_MODE = "mode"

    fun theme(ctx: Context): GardenTheme =
        GardenTheme.from(prefs(ctx).getString(KEY_THEME, null))

    fun mode(ctx: Context): ThemeMode =
        ThemeMode.from(prefs(ctx).getString(KEY_MODE, null))

    fun setTheme(ctx: Context, t: GardenTheme) {
        prefs(ctx).edit().putString(KEY_THEME, t.name).apply()
    }

    fun setMode(ctx: Context, m: ThemeMode) {
        prefs(ctx).edit().putString(KEY_MODE, m.name).apply()
    }

    private fun prefs(ctx: Context) =
        ctx.getSharedPreferences(FILE, Context.MODE_PRIVATE)
}

/**
 * One theme's two colour schemes.
 *
 * Built from four colours rather than a full Material palette: a ground, a
 * raised surface, an accent and the ink that goes on the accent. Everything
 * else is derived, so a new theme is four decisions instead of twenty, and
 * the relationships that make text readable cannot drift between themes.
 */
private fun scheme(
    dark: Boolean,
    ground: Color,
    raised: Color,
    accent: Color,
    onAccent: Color,
    ink: Color,
    accentSoft: Color,
    onAccentSoft: Color,
): ColorScheme {
    val base = if (dark) darkColorScheme() else lightColorScheme()
    return base.copy(
        background = ground,
        onBackground = ink,
        surface = if (dark) raised else ground,
        onSurface = ink,
        surfaceVariant = raised,
        onSurfaceVariant = ink,
        primary = accent,
        onPrimary = onAccent,
        primaryContainer = accentSoft,
        onPrimaryContainer = onAccentSoft,
        secondary = accent,
        onSecondary = onAccent,
        outline = if (dark) Color(0xFF8A8A8A) else Color(0xFF7A7A7A),
        // Warning red is NOT themed. It marks a plant as toxic, and a colour
        // that shifts with somebody's taste is a colour that stops meaning
        // "careful" -- a lavender warning on a lavender theme is invisible
        // exactly when it matters.
        error = if (dark) Color(0xFFFFB3B8) else Color(0xFF9B1C2B),
        onError = if (dark) Color(0xFF5A1420) else Color(0xFFFFFFFF),
    )
}

fun schemeFor(theme: GardenTheme, dark: Boolean): ColorScheme = when (theme) {
    GardenTheme.Evergreen -> if (dark) scheme(
        true, Color(0xFF101A14), Color(0xFF1B2B22), Color(0xFF7FD1A0),
        Color(0xFF07281A), Color(0xFFE3EFE7), Color(0xFF2E4A3A), Color(0xFFD6F0E1),
    ) else scheme(
        false, Color(0xFFF2F7F3), Color(0xFFE2EDE6), Color(0xFF2E7D32),
        Color(0xFFFFFFFF), Color(0xFF16221B), Color(0xFFCDE6D5), Color(0xFF10301D),
    )

    GardenTheme.Forest -> if (dark) scheme(
        true, Color(0xFF12170F), Color(0xFF1F2718), Color(0xFFA8C686),
        Color(0xFF1B2411), Color(0xFFE8EDE0), Color(0xFF3A4A2B), Color(0xFFE2EFD2),
    ) else scheme(
        false, Color(0xFFF4F6EE), Color(0xFFE6EBD9), Color(0xFF4A6B2A),
        Color(0xFFFFFFFF), Color(0xFF1C2413), Color(0xFFD6E4BE), Color(0xFF1E2C10),
    )

    GardenTheme.Ivy -> if (dark) scheme(
        true, Color(0xFF0E1A18), Color(0xFF172A27), Color(0xFF6FD3C4),
        Color(0xFF052723), Color(0xFFDFEFEC), Color(0xFF265049), Color(0xFFD2F2EC),
    ) else scheme(
        false, Color(0xFFF0F7F5), Color(0xFFDDEDE9), Color(0xFF1B6F63),
        Color(0xFFFFFFFF), Color(0xFF12241F), Color(0xFFC6E7E0), Color(0xFF082E28),
    )

    // The theme the app has been wearing since it was rebuilt, kept so a
    // switch away from it can be switched back.
    GardenTheme.Lavender -> if (dark) scheme(
        true, Color(0xFF141018), Color(0xFF2B2437), Color(0xFFC9B8EC),
        Color(0xFF2A1F45), Color(0xFFE9E4F2), Color(0xFF3E3260), Color(0xFFEBE2FF),
    ) else scheme(
        false, Color(0xFFF4F1FA), Color(0xFFE8E2F4), Color(0xFF5B4B8A),
        Color(0xFFFFFFFF), Color(0xFF1C1725), Color(0xFFE2D9F6), Color(0xFF1E1633),
    )

    GardenTheme.Sunflower -> if (dark) scheme(
        true, Color(0xFF191505), Color(0xFF2A2410), Color(0xFFF2CE5B),
        Color(0xFF322700), Color(0xFFF1EADA), Color(0xFF4A3D10), Color(0xFFFFEFC0),
    ) else scheme(
        false, Color(0xFFFCF8EC), Color(0xFFF4EBCF), Color(0xFF8A6A08),
        Color(0xFFFFFFFF), Color(0xFF241D07), Color(0xFFF7E4AE), Color(0xFF2E2305),
    )

    GardenTheme.Sunrise -> if (dark) scheme(
        true, Color(0xFF1A1211), Color(0xFF2C1E1C), Color(0xFFFFB59B),
        Color(0xFF3A1B10), Color(0xFFF3E7E3), Color(0xFF50302A), Color(0xFFFFDCCF),
    ) else scheme(
        false, Color(0xFFFFF4EF), Color(0xFFFAE3D8), Color(0xFFB4502A),
        Color(0xFFFFFFFF), Color(0xFF2A1A14), Color(0xFFFFD8C6), Color(0xFF3A1A0C),
    )

    GardenTheme.Sunset -> if (dark) scheme(
        true, Color(0xFF17111A), Color(0xFF291D2E), Color(0xFFE79BC0),
        Color(0xFF3B1228), Color(0xFFEFE4F0), Color(0xFF4E2440), Color(0xFFFFD6E8),
    ) else scheme(
        false, Color(0xFFFCF2F6), Color(0xFFF3DEE8), Color(0xFF9A3B69),
        Color(0xFFFFFFFF), Color(0xFF261520), Color(0xFFF7CFE2), Color(0xFF3A0F28),
    )
}

/**
 * The live choice plus the way to change it, handed down the tree.
 *
 * `change` takes nulls so a caller can set one without knowing the other --
 * the sheet has two independent controls, and making each supply both would
 * mean each could clobber the other's value.
 */
fun Appearance.State(
    theme: GardenTheme,
    mode: ThemeMode,
    change: (GardenTheme?, ThemeMode?) -> Unit,
) = AppearanceState(theme, mode, change)

data class AppearanceState(
    val theme: GardenTheme,
    val mode: ThemeMode,
    val change: (GardenTheme?, ThemeMode?) -> Unit,
)
