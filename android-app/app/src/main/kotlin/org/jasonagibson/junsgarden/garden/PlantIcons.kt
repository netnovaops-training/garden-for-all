// PlantIcons.kt -- the small sun and water symbols shown beside a plant.

package org.jasonagibson.junsgarden.garden

/**
 * The little symbols beside a plant: how much sun, how much water.
 *
 * THE ORIGINAL APP HAD THESE and the author noticed the moment they were gone:
 * "I also lost a little emojis. I had like water in the sun and stuff." Its
 * APK still carries the classes -- `ui/common/PlantIcons`, `SunLevel`,
 * `WaterLevel` -- so they were a real part of the screen, not decoration
 * somebody added once.
 *
 * WHY THEY EARN THEIR PLACE. "Full Sun - Partial Shade" and "Average" are
 * four words that have to be read; a sun and two drops are read at a glance
 * while holding the phone in one hand and a plant in the other. The words
 * stay on the detail screen, where somebody is actually reading.
 *
 * MATCHED ON WHAT THE COLUMN REALLY CONTAINS. These fields are free text
 * typed over years -- "Full Sun", "Full Sun - Partial Shade", "Sun to Part
 * Shade", "Average", "Medium", "Moist". So the match is on substrings in a
 * deliberate order, most specific first, and anything unrecognised gets no
 * symbol at all rather than a wrong one. A guessed icon is worse than none:
 * it is wrong silently, and nobody checks the words underneath it.
 */

/** A sun symbol for an exposure, or null when the text is unfamiliar. */
fun sunIcon(exposure: String): String? {
    val s = exposure.lowercase()
    val sun = "sun" in s || "full s" in s
    val part = "part" in s
    val shade = "shade" in s
    return when {
        // Checked before the single-condition cases, because "Full Sun -
        // Partial Shade" contains every one of their keywords.
        sun && (part || shade) -> "\uD83C\uDF24"   // sun behind small cloud
        sun -> "\u2600\uFE0F"                       // full sun
        part && shade -> "\u26C5"                   // sun behind cloud
        shade -> "\u2601\uFE0F"                     // cloud
        else -> null
    }
}

/** Drops for a watering need, or null when the text is unfamiliar. */
fun waterIcon(needs: String): String? {
    val s = needs.lowercase()
    return when {
        "high" in s || "wet" in s || "moist" in s -> "\uD83D\uDCA7\uD83D\uDCA7\uD83D\uDCA7"
        "average" in s || "medium" in s || "moderate" in s -> "\uD83D\uDCA7\uD83D\uDCA7"
        "low" in s || "dry" in s || "drought" in s -> "\uD83D\uDCA7"
        else -> null
    }
}
