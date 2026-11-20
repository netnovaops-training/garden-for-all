// PlantMonths.kt -- turns the twelve-character month columns (grow_months,
// bloom_months) into words such as "May–September". Used by PlantSections.kt
// and PlantDetailScreen.kt.
//
// Moved out of PlantSections.kt on 2026-11-20 without changing a line of it:
// detekt TooManyFunctions (code-scanning alert 432) counted 11 functions there,
// at its limit of 11. Verified on the NUC: the old and the new code give the
// same answers for the same month strings, and detekt no longer reports the file.

package org.jasonagibson.junsgarden.garden

import org.jasonagibson.junsgarden.R
import org.jasonagibson.junsgarden.ui.AppText
import java.time.Month
import java.time.format.TextStyle
import java.util.Locale

/** Month `i` (0 = January) in the app's language: "May" or "5월"; short: "May" or "5월". */
private fun monthName(i: Int, short: Boolean = false): String =
    Month.of(i + 1).getDisplayName(if (short) TextStyle.SHORT else TextStyle.FULL, Locale.getDefault())

/**
 * "....MJJAS..." -> "May–September".
 *
 * THE ENCODING IS TWELVE CHARACTERS, ONE PER MONTH, a dot for off. It is
 * also the column with a history of being corrupted by app-layer bugs --
 * there are one-off repair scripts in the old project because of it. So this
 * only ever READS it. Nothing in the app writes a month bitmap, and if this
 * cannot make sense of a value it returns the raw string rather than
 * guessing, because a wrong month range looks exactly as confident as a
 * right one.
 */
fun monthRange(raw: String?): String {
    val s = raw?.trim().orEmpty()
    if (s.isEmpty()) return ""
    if (s.length != 12) return s          // not the bitmap; show it as-is

    val on = s.indices.filter { s[it] != '.' && !s[it].isWhitespace() }
    if (on.isEmpty()) return ""
    if (on.size == 12) return AppText.s(R.string.all_year)

    // Contiguous is the ordinary case and reads best as a range.
    val contiguous = on.zipWithNext().all { (a, b) -> b == a + 1 }
    if (contiguous) {
        val first = monthName(on.first())
        val last = monthName(on.last())
        return if (first == last) first else "$first–$last"
    }
    // Split seasons do exist -- something that blooms in spring and again in
    // autumn -- and collapsing those to "March–October" would be a lie.
    return on.joinToString(", ") { monthName(it, short = true) }
}
