// GardenApi.kt -- reads plant information from the website.
//
// Every call goes to GARDEN_BASE/app_api.php (GARDEN_BASE is set in
// app/build.gradle.kts). It only READS: action=list, detail, bed_counts and
// barcode_lookup. No login is needed, because the catalogue is public.

package org.jasonagibson.junsgarden.garden

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

/**
 * The catalogue, read straight from the garden's own API.
 *
 * WHY THIS EXISTS AT ALL. The app used to be a window onto the website, and
 * Jason's answer to that was plain: "I want the app I had before ... I don't
 * want to go into the website." The app he had was native, and its plants
 * came from here.
 *
 * THIS IS THE SAME ENDPOINT THE ORIGINAL APP USED. Its source is gone -- no
 * repository has it and it is on no disk here -- but the built APK still
 * names every call it made, and app_api.php still answers all of them.
 * Verified 2026-09-25: 437 plants across 19 beds, twenty to a page.
 *
 * NO AUTHENTICATION ON THE READ PATHS, deliberately. The catalogue is public
 * on the website and Jason has said the site stays that way. Browsing plants
 * needs no account; only writing does, and nothing here writes.
 *
 * HAND-PARSED WITH org.json RATHER THAN A SERIALISER. The detail record has
 * 101 columns, many of them NULL, several of them numbers stored as strings
 * ("34.99", "2022 / 2023"). A strict deserialiser turns any one of those
 * into a crash on a plant nobody has looked at yet; reading the fields that
 * are wanted and letting the rest lie is the shape that survives real data.
 */
object GardenApi {

    // Set at build time (app/build.gradle.kts): the live site unless a test
    // build asked for the test site.
    private val BASE = org.jasonagibson.junsgarden.BuildConfig.GARDEN_BASE
    private val API = "$BASE/app_api.php"
    private const val TIMEOUT_MS = 20_000

    private fun enc(s: String) = URLEncoder.encode(s, "UTF-8")

    private suspend fun get(query: String): JSONObject = withContext(Dispatchers.IO) {
        val conn = (URL("$API?$query").openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            connectTimeout = TIMEOUT_MS
            readTimeout = TIMEOUT_MS
            setRequestProperty("Accept", "application/json")
        }
        try {
            val code = conn.responseCode
            val body = (if (code in 200..299) conn.inputStream else conn.errorStream)
                ?.bufferedReader()?.use { it.readText() } ?: ""
            if (code !in 200..299) {
                throw GardenException("The garden answered $code.")
            }
            // A body that is not JSON is almost always a proxy error page. Say
            // that, rather than letting a parser exception reach the screen as
            // "Expected literal value at 1".
            if (!body.trimStart().startsWith("{")) {
                throw GardenException("The garden is not reachable right now.")
            }
            JSONObject(body)
        } finally {
            conn.disconnect()
        }
    }

    /** One page of the catalogue. Twenty per page, as the server decides. */
    suspend fun list(page: Int = 1, search: String = "", bed: String = ""): PlantPage {
        val q = buildString {
            append("action=list&page=").append(page)
            if (search.isNotBlank()) append("&search=").append(enc(search.trim()))
            if (bed.isNotBlank()) append("&bed=").append(enc(bed))
        }
        val o = get(q)
        val arr = o.optJSONArray("results") ?: JSONArray()
        val out = ArrayList<PlantSummary>(arr.length())
        for (i in 0 until arr.length()) {
            out.add(PlantSummary.from(arr.getJSONObject(i)))
        }
        return PlantPage(
            page = o.optInt("page", page),
            hasMore = o.optBoolean("hasMore", false),
            plants = out,
        )
    }

    /**
     * Everything recorded about one plant.
     *
     * THE RECORD IS WRAPPED. This endpoint answers `{"plant": {...}}`, not
     * the record itself -- unlike `list`, which answers `results`. Reading
     * the outer object as the record finds exactly one field, called
     * "plant", whose value is the whole thing as JSON, and prints it as a
     * single unreadable blob. That is what shipped in v2.0.
     */
    suspend fun detail(id: Int): PlantDetail {
        val o = get("action=detail&id=$id")
        return PlantDetail.from(o.optJSONObject("plant") ?: o)
    }

    /** The beds, with how many plants are in each. */
    suspend fun beds(): List<Bed> {
        val arr = get("action=bed_counts").optJSONArray("beds") ?: JSONArray()
        val out = ArrayList<Bed>(arr.length())
        for (i in 0 until arr.length()) {
            val b = arr.getJSONObject(i)
            val name = b.optString("name").trim()
            if (name.isNotEmpty()) out.add(Bed(name, b.optInt("count")))
        }
        return out
    }

    /**
     * What plant is this barcode stuck to?
     *
     * RETURNS NULL FOR "NOT FOUND", which the server signals with
     * `found: false` rather than a 404. A barcode nobody has registered yet
     * is an ordinary answer, not a failure -- Jun scans a nursery label
     * before the plant is in the catalogue all the time.
     *
     * The record only carries the name, so the botanical name is fetched
     * separately and only when there is a real id to fetch it with.
     */
    suspend fun barcode(code: String): PlantSummary? {
        val o = get("action=barcode_lookup&code=" + enc(code))
        if (!o.optBoolean("found")) return null
        val id = o.optInt("id")
        val name = o.str("plant_name").ifBlank { "(unnamed)" }
        // A future plant is not in the catalogue yet, so asking for its
        // detail would 404. The name is all there is, and it is enough.
        if (id <= 0 || o.optBoolean("is_future")) {
            return PlantSummary(
                id = id, name = name, botanical = "", bed = "", sun = "",
                water = "", type = "", zones = "", toxicity = "",
                edibility = "", qty = 0, photo = null,
            )
        }
        return runCatching { detail(id) }
            .map { d ->
                PlantSummary(
                    id = id, name = d.name, botanical = d.botanical,
                    bed = "", sun = "", water = "", type = "", zones = "",
                    toxicity = d.toxicity, edibility = d.edibility,
                    qty = 0, photo = d.photo,
                )
            }
            .getOrElse {
                PlantSummary(
                    id = id, name = name, botanical = "", bed = "", sun = "",
                    water = "", type = "", zones = "", toxicity = "",
                    edibility = "", qty = 0, photo = null,
                )
            }
    }

    /**
     * Turn whatever the `image` column holds into something loadable.
     *
     * The database stores a site-relative path ("/images/Database/x.webp").
     * Some rows are blank and a few already hold a full URL, so all three
     * cases are handled here rather than at each call site.
     */
    fun photoUrl(raw: String?): String? {
        val s = (raw ?: "").trim()
        return when {
            s.isEmpty() -> null
            s.startsWith("http://") || s.startsWith("https://") -> s
            s.startsWith("/") -> BASE + s
            else -> "$BASE/$s"
        }
    }
}

class GardenException(message: String) : Exception(message)

data class Bed(val name: String, val count: Int)

data class PlantPage(
    val page: Int,
    val hasMore: Boolean,
    val plants: List<PlantSummary>,
)

/** A plant as the catalogue list shows it. */
data class PlantSummary(
    val id: Int,
    val name: String,
    val botanical: String,
    val bed: String,
    val sun: String,
    val water: String,
    val type: String,
    val zones: String,
    val toxicity: String,
    val edibility: String,
    val qty: Int,
    val photo: String?,
) {
    companion object {
        fun from(o: JSONObject) = PlantSummary(
            id = o.optInt("id"),
            name = o.str("plant_name").ifBlank { "(unnamed)" },
            botanical = o.str("botanical_name"),
            bed = o.str("flower_bed"),
            sun = o.str("sun_exposure"),
            water = o.str("water_needs"),
            type = o.str("plant_type"),
            zones = o.str("zones"),
            toxicity = o.str("toxicity"),
            edibility = o.str("fruit_edibility"),
            qty = o.optInt("qty"),
            photo = GardenApi.photoUrl(o.str("image")),
        )
    }
}

/**
 * The full record.
 *
 * `fields` keeps everything the server sent, in the order it sent it, so the
 * detail screen can show columns nobody has thought to name here yet. The
 * named properties are only the ones the layout treats specially.
 */
data class PlantDetail(
    val id: Int,
    val name: String,
    val botanical: String,
    val photo: String?,
    /** Every photograph of this plant. Often several; sometimes none. */
    val photos: List<String>,
    val toxicity: String,
    val edibility: String,
    val fields: List<Pair<String, String>>,
) {
    companion object {
        // Shown at the top in their own right, so they are not repeated in
        // the long list underneath.
        private val PROMOTED = setOf(
            "id", "plant_name", "botanical_name", "image", "picture",
            "toxicity", "fruit_edibility",
            // Shown as the gallery at the top. Left in the table it would
            // print as a wall of file paths.
            "images", "photo_position", "photo_flip", "barcode_image",
            "image_search_term", "image_verified", "image_approved",
        )

        fun from(o: JSONObject): PlantDetail {
            // EVERY PHOTOGRAPH. Jason: "Some of these had more than one
            // photo" -- and they do, up to seven. The record hands them
            // back in `images`; showing only the first threw the rest away.
            val shots = ArrayList<String>()
            o.optJSONArray("images")?.let { arr ->
                for (i in 0 until arr.length()) {
                    GardenApi.photoUrl(arr.optString(i))?.let { shots.add(it) }
                }
            }
            val rest = ArrayList<Pair<String, String>>()
            for (key in o.keys()) {
                if (key in PROMOTED) continue
                val v = o.str(key)
                // An empty column is not a fact about the plant, it is a gap
                // in the record. Printing "Season: —" 40 times is the data
                // dump he already asked me to stop making.
                if (v.isNotBlank() && v != "0") rest.add(key to v)
            }
            rest.sortBy { it.first }
            return PlantDetail(
                id = o.optInt("id"),
                name = o.str("plant_name").ifBlank { "(unnamed)" },
                botanical = o.str("botanical_name"),
                photo = GardenApi.photoUrl(
                    o.str("image").ifBlank { o.str("picture") }
                ) ?: shots.firstOrNull(),
                photos = shots,
                toxicity = o.str("toxicity"),
                edibility = o.str("fruit_edibility"),
                fields = rest,
            )
        }
    }
}

/**
 * A string field, with JSON null read as empty.
 *
 * `optString` returns the four characters "null" for a JSON null, which is
 * how a plant with no season ends up displaying the word null on screen.
 */
private fun JSONObject.str(key: String): String {
    if (isNull(key)) return ""
    return optString(key, "").trim()
}
