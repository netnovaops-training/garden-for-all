// PlantSections.kt -- decides which plant fields go into which card on the
// plant page, so the app groups them the same way the website does.

package org.jasonagibson.junsgarden.garden

/**
 * The same grouping catalog.php uses, so the app and the website agree.
 *
 * JASON, 2026-09-27: "Can you make Jun's app display the plants they do in
 * junsgarden.zapto.org/catalog.php that match with the theme app."
 *
 * WHY GROUPING IS THE WHOLE POINT. The detail call returns 101 columns. Shown
 * as one flat table that is a wall of text, and Jason said so about an earlier
 * version: "it just looks like a bunch of data". The website already solved
 * this -- Planting Guide, Root System, Growing Conditions, Appearance,
 * Wildlife & Safety -- and the answer is not a new design, it is the design
 * he already has, on the phone.
 *
 * TWO SIDES OF ONE RECORD MUST NOT DISAGREE. If the app decides for itself
 * which fields matter, the website and the phone start showing different
 * plants, and the one somebody trusts is whichever they looked at last. So
 * the sections here mirror catalog.php's, in its order, with its labels.
 *
 * ONLY WHAT EXISTS IS SHOWN. A row with no value is dropped rather than
 * printed blank, and a whole section with nothing in it never appears. Half
 * the catalogue is still being backfilled, and an empty "Root System" heading
 * on four hundred plants is noise that teaches people to scroll past the
 * headings that do have something under them.
 *
 * NOTHING HERE WRITES. This reads the record and decides what to show. In
 * particular `grow_months` and `bloom_months` are only ever DECODED -- see
 * monthRange below.
 */

/** A labelled group of rows, rendered as one card. */
data class PlantSection(
    val title: String,
    val icon: String,
    val rows: List<Pair<String, String>>,
)

private val MONTH_NAMES = listOf(
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
)

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
    if (on.size == 12) return "All year"

    // Contiguous is the ordinary case and reads best as a range.
    val contiguous = on.zipWithNext().all { (a, b) -> b == a + 1 }
    if (contiguous) {
        val first = MONTH_NAMES[on.first()]
        val last = MONTH_NAMES[on.last()]
        return if (first == last) first else "$first–$last"
    }
    // Split seasons do exist -- something that blooms in spring and again in
    // autumn -- and collapsing those to "March–October" would be a lie.
    return on.joinToString(", ") { MONTH_NAMES[it].take(3) }
}

private fun Map<String, String>.take(key: String): String =
    this[key]?.trim().orEmpty()

/** "3 - 4 feet" x "4 feet", or whatever halves exist. */
private fun Map<String, String>.size2d(): String {
    val l = take("plant_length")
    val w = take("plant_width")
    return when {
        l.isNotEmpty() && w.isNotEmpty() -> "$l × $w"
        else -> l.ifEmpty { w }
    }
}

private fun money(raw: String): String {
    if (raw.isBlank()) return ""
    val n = raw.toDoubleOrNull() ?: return raw
    // Whole dollars print as whole dollars. "$25" beats "$25.00" on a phone,
    // and the cents are never meaningful here.
    return if (n % 1.0 == 0.0) "$" + n.toInt() else "$" + String.format("%.2f", n)
}

/** Joined with the website's separator, skipping the parts that are missing. */
private fun Map<String, String>.joined(vararg keys: String): String =
    keys.map { take(it) }.filter { it.isNotEmpty() }.joinToString(" · ")

/**
 * The five cards, in the website's order.
 *
 * `f` is the whole record as sent, lowercased keys. Anything not claimed by a
 * section is still shown by the screen, under "More details" -- a column
 * nobody has thought to place here yet should not vanish because of that.
 */
fun plantSections(f: Map<String, String>): List<PlantSection> {
    val out = ArrayList<PlantSection>(5)

    fun add(title: String, icon: String, rows: List<Pair<String, String>>) {
        val kept = rows.filter { it.second.isNotBlank() }
        if (kept.isNotEmpty()) out.add(PlantSection(title, icon, kept))
    }

    add("Planting Guide", "🌱", listOf(
        "Grow season" to monthRange(f["grow_months"]).ifEmpty { f.take("grow_season") },
        "Bloom season" to monthRange(f["bloom_months"]).ifEmpty { f.take("season") },
        "Spacing" to f.take("spacing"),
        "Planting depth" to f.take("planting_depth"),
        "Mature size" to f.size2d(),
        "Chilling hours" to f.take("chilling_hours").takeIf { it != "0" }.orEmpty(),
        "Life expectancy" to f.take("life_expectancy"),
        "Avg. cost" to money(f.take("average_cost")),
        "Store" to f.take("local_store").ifEmpty { f.take("company") },
    ))

    add("Root System", "🪵", listOf(
        "System" to f.take("root_system"),
        "Morphology" to f.take("root_morphology"),
        "Function" to f.take("root_function"),
    ))

    add("Growing Conditions", "💧", listOf(
        "Sun" to f.take("sun_exposure"),
        "Water" to f.take("water_needs"),
        "Soil type" to f.take("soil_type"),
        "Soil pH" to f.take("soil_ph"),
        "Zones" to f.take("zones"),
        "Native status" to f.take("native_status"),
        "Fertilizer" to f.take("fertilizer"),
    ))

    add("Appearance", "🌸", listOf(
        "Type" to f.take("plant_type"),
        "Leaf" to f.joined("leaf_type", "leaf_shape", "leaf_margin"),
        "Foliage color" to f.take("foliage_color"),
        "Stem color" to f.take("stem_color"),
        "Bloom" to f.joined("bloom_color", "flower_shape", "flower_size"),
        "Fruit" to f.joined("fruit_type", "fruit_color", "fruit_edibility"),
        "Evergreen" to f.take("is_evergreen"),
        "Growth rate" to f.take("growth_rate"),
    ))

    add("Wildlife & Safety", "🐝", listOf(
        "Attracts" to f.take("attracts"),
        "Deer" to f.take("deer_resistance"),
        "Toxicity" to f.take("toxicity"),
        "Edibility" to f.take("fruit_edibility"),
        "Invasive here" to f.take("invasive_local"),
    ))

    return out
}

/**
 * The prose blocks, in reading order, with the website's headings.
 *
 * These are paragraphs somebody wrote, and the reason they are separated from
 * the tables at all: on the old layout they were buried among ninety columns
 * of one-word values and nobody read them.
 */
fun plantNotes(f: Map<String, String>): List<Pair<String, String>> = listOf(
    "Care" to f.take("care_notes"),
    "Maintenance" to f.take("maintenance_notes"),
    "Cleanup" to f.take("cleanup_notes"),
    "Propagation" to f.take("propagation_method").ifEmpty { f.take("cuttings") },
    "Companion plants" to f.take("companion_plants"),
    "Pests & disease" to f.take("disease_susceptibility"),
    "Aromatics" to f.take("aromatics"),
    "More" to f.take("extra_details"),
).filter { it.second.isNotBlank() }

/**
 * The quick chips under the photograph: the handful of facts worth seeing
 * before reading anything.
 */
fun plantChips(f: Map<String, String>): List<Pair<String, String>> {
    val chips = ArrayList<Pair<String, String>>(10)
    fun put(icon: String, v: String) {
        if (v.isNotBlank()) chips.add(icon to v)
    }
    val qty = f.take("qty").ifEmpty { f.take("quantity") }
    put("🪴", if (qty.isNotBlank() && qty != "0") "Qty $qty" else "")
    put("📅", f.take("year_planted").let { if (it.isBlank()) "" else "Planted $it" })
    put("🌿", f.take("plant_type"))
    put("📈", f.take("growth_rate"))
    put("🌡️", f.take("zones"))
    put("☀️", f.take("sun_exposure"))
    put("💧", f.take("water_needs"))
    put("🌍", f.take("native_status"))
    put("🛒", f.take("local_store").ifEmpty { f.take("company") })
    return chips
}

/**
 * The taxonomy line: Family › Genus › species.
 *
 * Empty when none of the three is known, which is most of the wishlist until
 * the backfill reaches it -- and an arrow with nothing either side of it is
 * worse than no line at all.
 */
fun taxonomyLine(f: Map<String, String>): String =
    listOf("family", "genus", "species")
        .map { f.take(it) }
        .filter { it.isNotEmpty() }
        .joinToString(" › ")

/**
 * Everything a section did not claim, so no column silently disappears.
 * Ordering is the server's, which is the catalogue's own column order.
 */
private val PLACED = setOf(
    "grow_months", "bloom_months", "grow_season", "season", "spacing",
    "planting_depth", "plant_length", "plant_width", "chilling_hours",
    "life_expectancy", "average_cost", "local_store", "company",
    "root_system", "root_morphology", "root_function", "sun_exposure",
    "water_needs", "soil_type", "soil_ph", "zones", "native_status",
    "fertilizer", "plant_type", "leaf_type", "leaf_shape", "leaf_margin",
    "foliage_color", "stem_color", "bloom_color", "flower_shape",
    "flower_size", "fruit_type", "fruit_color", "fruit_edibility",
    "is_evergreen", "growth_rate", "attracts", "deer_resistance",
    "toxicity", "invasive_local", "care_notes", "maintenance_notes",
    "cleanup_notes", "propagation_method", "cuttings", "companion_plants",
    "disease_susceptibility", "aromatics", "extra_details", "family",
    "genus", "species", "qty", "quantity", "year_planted",
    // Housekeeping the record carries and nobody reads on a phone.
    "id", "plant_name", "botanical_name", "images", "image", "picture",
    "photo_position", "photo_flip", "barcode_image", "image_search_term",
    "image_verified", "image_approved", "approved_jason", "approved_jun",
    "needs_review", "admin_todo", "tag_done", "fav_count", "favorited_by",
    "is_favorited", "is_liked", "like_count", "new_name", "new_total",
    "split_amt", "died", "died_year", "year_added", "size", "size_spacing",
    "spacing_min_in", "spacing_max_in", "plant_length_min_in",
    "plant_length_max_in", "plant_width_min_in", "plant_width_max_in",
    "month_start", "month_end", "planting_month_start", "planting_month_end",
    "barcode", "flower_bed", "garden_beds",
)

fun plantLeftovers(fields: List<Pair<String, String>>): List<Pair<String, String>> =
    fields.filter { (k, v) -> k.lowercase() !in PLACED && v.isNotBlank() }
