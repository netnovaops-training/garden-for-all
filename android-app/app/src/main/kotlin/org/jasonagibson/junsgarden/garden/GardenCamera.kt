// GardenCamera.kt -- the camera tools: Identify, Tag, Barcode and Receipt.
//
// The phone's own camera app takes the photo; this app never opens the
// camera itself. Barcodes are read on the phone (ZXing library). Identify,
// Tag and Receipt send the photo to GARDEN_BASE/app/garden/<action> with
// the garden login key, and turn the website's answer into plain text.

package org.jasonagibson.junsgarden.garden

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Matrix
import android.net.Uri
import android.util.Base64
import androidx.exifinterface.media.ExifInterface
import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.MultiFormatReader
import com.google.zxing.RGBLuminanceSource
import com.google.zxing.common.HybridBinarizer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.jasonagibson.junsgarden.R
import org.jasonagibson.junsgarden.ui.AppText
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.net.HttpURLConnection
import java.net.URL

/**
 * The camera, doing the four things the author asked it to do.
 *
 * HIS WORDS, 2026-09-25: "The camera supposed to be used on live pictures of
 * plants. There's not supposed to be a webview ... It's supposed to take a
 * picture of a plant and describe if it's living or dead or needs nutrients
 * or rusted and identify the plant ... The other part of it is supposed to
 * take pictures of a tag and then import it into the database ... also
 * supposed to be able to take barcodes and scan them into a plant."
 *
 * NOTHING HERE IS NEW WORK ON THE SERVER. garden-ingest already reads plant
 * tags, identifies plants against the catalogue, diagnoses a sick one and
 * turns a receipt into price history. What was missing was a way for a
 * phone to reach any of it: those endpoints decide who you are from an
 * `X-Auth-Email` header they trust completely, which is only safe because
 * nothing off this machine can set one. So the app talks to /app/garden/…
 * on the GARDEN SITE (since 3.1, 2026-10-07; before that it was the author's
 * household sign-in service), which checks the garden login key and
 * attaches the person's address upstream. The reading itself is done by
 * free cloud AI (garden-ingest's freeai.py), never a home GPU; when every
 * free model is busy a tag is QUEUED and read later, and the answer says so.
 *
 * A LONG READ TIMEOUT, DELIBERATELY. Each of these runs a vision model over
 * a photograph and then a catalogue lookup. The honest number is minutes,
 * and a timeout shorter than the work produces a failure that reads as a
 * broken camera rather than a slow one.
 */
class GardenCamera(private val accessTokenProvider: suspend () -> Result<String>) {

    /** What came back, already turned into something a person can read. */
    data class Reading(
        val headline: String,
        val body: String,
        /** Set when the server queued something for the author to approve. */
        val queued: Int = 0,
    )

    /**
     * What plant is this, and is it well?
     *
     * TWO QUESTIONS, ONE PHOTOGRAPH, because that is how it was asked for.
     * `identify` matches the picture against his own catalogue first, and
     * `diagnose` reads the leaves. They are separate endpoints upstream and
     * are kept separate here, so one failing still answers the other -- a
     * plant that cannot be named can still be visibly rusted, and saying so
     * is more use than saying nothing.
     */
    suspend fun examine(context: Context, photo: Uri): Reading {
        val image = encode(context, photo)

        val named = runCatching { post("identify", JSONObject().put("image", image)) }
        val health = runCatching { post("diagnose", JSONObject().put("image", image)) }

        val name = named.getOrNull()?.let { readIdentity(it) } ?: ""
        val verdict = health.getOrNull()?.let { readDiagnosis(it) } ?: ""

        if (name.isBlank() && verdict.isBlank()) {
            val why = named.exceptionOrNull() ?: health.exceptionOrNull()
            throw GardenException(
                why?.message ?: AppText.s(R.string.photo_not_read_by_garden)
            )
        }

        return Reading(
            headline = name.ifBlank { AppText.s(R.string.couldnt_name) },
            body = verdict.ifBlank { AppText.s(R.string.no_health) },
        )
    }

    /**
     * A nursery tag, photographed, read, and queued for the database.
     *
     * QUEUED, NOT WRITTEN. The server files these as proposals for approval
     * rather than inserting them, which is the author's own arrangement: he
     * reviews everything before it reaches the catalogue. So the honest
     * thing to tell somebody holding the tag is that it is waiting, not
     * that it is in.
     */
    suspend fun readTag(context: Context, photo: Uri): Reading =
        readTag(context, listOf(photo))

    /**
     * A pair of photographs of one tag, waiting to be sent.
     *
     * `back` is optional because a tag in a pot is not always turnable --
     * the same rule the single-tag path already follows.
     */
    data class TagPair(val front: Uri, val back: Uri? = null)

    /** What the queue said. Counts, because a batch has no single answer. */
    data class QueueResult(
        val queued: Int,
        val duplicates: Int,
        val unreadable: Int,
        val waiting: Int,
    )

    /**
     * Many tags at once: photographs in, a receipt back, reading afterwards.
     *
     * THE AUTHOR, 2026-09-27: "there needs to be a bulk tag adder so that front
     * and back can be added to the picture so that if someone wants to add
     * 20 or 30 tags they can at once front and back without taking one
     * picture at a time."
     *
     * WHY THIS IS A DIFFERENT CALL AND NOT A LOOP OVER readTag. One tag
     * takes about twenty-two seconds to read and enrich. Thirty tags is
     * eleven minutes, and nothing about a phone survives an eleven-minute
     * request -- it sleeps, the radio drops between rooms, the screen
     * turns. Every one of those loses the answer AND the record that the
     * photographs were ever sent, which is the worse half. So the server
     * only writes the images down and hands back a receipt; the reading
     * happens afterwards to rows that already exist, and a crash costs a
     * retry rather than an afternoon of photographs.
     *
     * SENT IN SMALL BATCHES, and that is not the same as sending one at a
     * time. Thirty photographs at 1600px is roughly fifteen megabytes of
     * base64 in one request body, which is enough to be refused, to time
     * out on a weak signal, or to be held entirely in memory at both ends.
     * Batching keeps each request ordinary while still costing one round
     * trip per several tags instead of one per tag.
     *
     * PROGRESS IS REPORTED AS IT GOES. A silent thirty-second upload looks
     * identical to a broken one, and the button gets pressed again.
     *
     * A FAILED BATCH DOES NOT LOSE THE ONES ALREADY ACCEPTED. Each batch is
     * its own transaction on the server, so the counts returned here are
     * what genuinely landed. Partial success is reported honestly rather
     * than being rounded up to "done" or down to "failed".
     */
    suspend fun queueTags(
        context: Context,
        pairs: List<TagPair>,
        onProgress: (done: Int, total: Int) -> Unit = { _, _ -> },
    ): QueueResult = withContext(Dispatchers.IO) {
        if (pairs.isEmpty()) return@withContext QueueResult(0, 0, 0, 0)

        var queued = 0
        var duplicates = 0
        var unreadable = 0
        var waiting = 0
        var done = 0

        pairs.chunked(BULK_BATCH).forEach { batch ->
            val arr = org.json.JSONArray()
            batch.forEach { pair ->
                // An unreadable photograph costs ITS OWN tag and not the
                // batch. Counting it here keeps the total honest without
                // throwing away the ones either side of it.
                val front = runCatching { encode(context, pair.front) }.getOrNull()
                if (front == null) {
                    unreadable++
                    return@forEach
                }
                val back = pair.back?.let {
                    runCatching { encode(context, it) }.getOrNull()
                }
                arr.put(
                    JSONObject()
                        .put("front", front)
                        .put("back", back ?: JSONObject.NULL)
                )
            }
            if (arr.length() == 0) {
                done += batch.size
                onProgress(done, pairs.size)
                return@forEach
            }
            val o = post("tag-queue", JSONObject().put("tags", arr))
            queued += o.optInt("queued")
            duplicates += o.optInt("duplicates")
            unreadable += o.optInt("unreadable")
            waiting = o.optInt("waiting", waiting)
            done += batch.size
            onProgress(done, pairs.size)
        }
        QueueResult(queued, duplicates, unreadable, waiting)
    }

    /**
     * A nursery tag, front and back.
     *
     * THE BACK IS WHERE THE FACTS ARE. Jun photographed a tag and got a
     * name and almost nothing else, because a nursery tag prints the name
     * and a picture on the front and the spacing, zones, sun, water, bloom
     * months and grower on the back. The server reads each side on its own
     * and merges them, keeping any disagreement rather than resolving it
     * quietly.
     *
     * The front goes first and settles the name when both sides print one.
     * One photograph still works: the back is an offer, not a requirement,
     * because a tag in a pot is not always turnable.
     */
    suspend fun readTag(context: Context, photos: List<Uri>): Reading {
        val shots = org.json.JSONArray()
        photos.forEach { shots.put(encode(context, it)) }
        val o = post(
            "tag",
            JSONObject().put("images", shots).put("enrich", true),
        )
        val note = o.optString("note").trim()
        // EVERY FREE AI WAS BUSY: the tag is safe in the queue and will be
        // read on its own (garden-ingest retries every 15 minutes). Saying
        // "couldn't read that tag" here would make him photograph it again.
        if (o.optBoolean("queued")) {
            val waiting = o.optInt("waiting", 1)
            return Reading(
                headline = AppText.s(R.string.saved_for_later),
                body = note.ifBlank { AppText.s(R.string.ai_busy) } +
                    if (waiting > 1) "\n\n" + AppText.s(R.string.tags_in_line, waiting) else "",
            )
        }
        val read = o.optJSONObject("read")
        val name = read?.optString("plant_name").orEmpty().ifBlank {
            read?.optString("botanical_name").orEmpty()
        }
        val queued = o.optJSONArray("proposal_ids")?.length() ?: 0

        if (name.isBlank()) {
            return Reading(
                headline = AppText.s(R.string.couldnt_read_tag),
                body = note.ifBlank { AppText.s(R.string.tag_retry) },
            )
        }

        val lines = buildString {
            read?.keys()?.forEach { k ->
                val v = read.optString(k).trim()
                if (v.isNotBlank() && k != "plant_name") {
                    append(k.replace('_', ' ').replaceFirstChar { it.uppercase() })
                    append(": ").append(v).append('\n')
                }
            }
            val sides = o.optInt("sides_read", 1)
            if (sides > 1) {
                append('\n')
                append(AppText.s(R.string.read_both_sides))
            }
            // WHAT HAPPENED TO THE PLANT, not just to the fields.
            // Before this the answer never said whether anything had
            // been added -- and nothing had been, because the tag
            // flow created no row at all.
            val serverNote = o.optString("note").trim()
            if (serverNote.isNotBlank()) {
                append('\n')
                append(serverNote)
            }
            // The photographs are kept, so a value that looks wrong in
            // six months can be checked against the tag it came from.
            val album = o.optJSONArray("album")
            val kept = (0 until (album?.length() ?: 0)).count {
                album?.optJSONObject(it)?.has("file") == true
            }
            if (kept > 0) {
                append('\n')
                append(
                    AppText.s(if (kept == 1) R.string.photo_saved_one else R.string.photo_saved_both)
                )
            }
            val clashes = o.optJSONObject("conflicts")
            if (clashes != null && clashes.length() > 0) {
                append('\n').append('\n')
                val fields = clashes.keys().asSequence().joinToString(", ") { it.replace('_', ' ') }
                append(AppText.s(R.string.sides_disagree, fields))
            }
            if (queued > 0) {
                append('\n')
                append(AppText.s(R.string.fields_waiting, queued))
            }
        }.trim()

        return Reading(headline = name, body = lines, queued = queued)
    }

    /** A receipt, photographed, turned into what each plant cost. */
    suspend fun readReceipt(context: Context, photo: Uri): Reading {
        val o = post("receipt", JSONObject().put("image", encode(context, photo)))
        val lines = o.optJSONArray("lines")
        val kept = o.optInt("kept", 0)

        // THE SERVER'S OWN SENTENCE WHEN IT HAS ONE. It knows whether it
        // found nothing, matched everything, or matched some -- and saying
        // that in its words beats reconstructing it badly from the counts.
        if (lines == null || lines.length() == 0) {
            return Reading(
                headline = AppText.s(R.string.nothing_recorded),
                body = o.optString("message").trim().ifBlank { AppText.s(R.string.receipt_retry) },
            )
        }

        val body = buildString {
            for (i in 0 until lines.length()) {
                val row = lines.optJSONObject(i) ?: continue
                val name = row.optString("plant_name").trim()
                    .ifBlank { row.optString("name").trim() }
                val price = row.optString("price").trim()
                    .ifBlank { row.optString("each").trim() }
                append("• ").append(name)
                if (price.isNotBlank()) append("  —  $").append(price)
                append('\n')
            }
            val spent = o.optString("spent").trim()
            if (spent.isNotBlank()) append('\n').append(AppText.s(R.string.spent, spent))
            val msg = o.optString("message").trim()
            if (msg.isNotBlank()) append("\n\n").append(msg)
        }.trim()

        val store = o.optString("store").trim()
        val date = o.optString("date").trim()
        return Reading(
            headline = listOf(store, date).filter { it.isNotBlank() }
                .joinToString("  ·  ")
                .ifBlank { AppText.s(R.string.receipt_recorded) },
            body = body,
            queued = kept,
        )
    }

    /**
     * A barcode, read from a photograph rather than a live viewfinder.
     *
     * NO CAMERA PERMISSION, ON PURPOSE. A live scanner runs the camera
     * inside this app and needs the permission to match; handing the job to
     * the camera app and decoding the picture that comes back needs none.
     * The manifest says this app asks for no camera access and that stays
     * true. It costs a shutter press.
     */
    suspend fun readBarcode(context: Context, photo: Uri): String? =
        withContext(Dispatchers.IO) {
            val bmp = decodeBitmap(context, photo) ?: return@withContext null
            val pixels = IntArray(bmp.width * bmp.height)
            bmp.getPixels(pixels, 0, bmp.width, 0, 0, bmp.width, bmp.height)
            val source = RGBLuminanceSource(bmp.width, bmp.height, pixels)
            bmp.recycle()
            val reader = MultiFormatReader().apply {
                // TRY HARDER: a tag photographed at arm's length in a garden
                // is not a scanner's idea of a barcode. The extra passes cost
                // milliseconds and are the difference between reading it and
                // telling him to try again.
                setHints(mapOf(DecodeHintType.TRY_HARDER to true))
            }
            runCatching {
                reader.decode(BinaryBitmap(HybridBinarizer(source))).text
            }.getOrNull()
        }

    // -- plumbing -------------------------------------------------------------

    private suspend fun post(action: String, body: JSONObject): JSONObject =
        withContext(Dispatchers.IO) {
            val token = accessTokenProvider().getOrElse {
                throw GardenException(AppText.s(R.string.sign_in_again_camera))
            }
            val conn = (URL("$BASE/$action").openConnection() as HttpURLConnection).apply {
                requestMethod = "POST"
                doOutput = true
                connectTimeout = 20_000
                readTimeout = 300_000
                setRequestProperty("Content-Type", "application/json")
                setRequestProperty("Authorization", "Bearer $token")
            }
            try {
                conn.outputStream.use { it.write(body.toString().toByteArray()) }
                val code = conn.responseCode
                val text = (if (code in 200..299) conn.inputStream else conn.errorStream)
                    ?.bufferedReader()?.use { it.readText() } ?: ""
                when {
                    code == 401 || code == 403 ->
                        throw GardenException(AppText.s(R.string.sign_in_again_camera))
                    code !in 200..299 ->
                        // The server's own sentence when it sent one -- e.g.
                        // "The free AI readers are all busy ... try again in
                        // about 12 minutes" (503) -- rather than a bare code.
                        throw GardenException(
                            runCatching { JSONObject(text).optString("error") }.getOrNull()
                                ?.takeIf { it.isNotBlank() }
                                ?: text.takeIf { it.isNotBlank() && it.length < 200 }
                                ?: AppText.s(R.string.garden_answered, code)
                        )
                    !text.trimStart().startsWith("{") ->
                        throw GardenException(AppText.s(R.string.unreachable_now))
                    else -> JSONObject(text)
                }
            } finally {
                conn.disconnect()
            }
        }

    /**
     * The name, preferring the catalogue's answer to the world's.
     *
     * `candidates` are rows from HIS garden and `verdict` is the one the
     * server settled on. A plant he already owns should come back as the
     * plant he already owns, named the way he named it -- not as a species
     * page from the internet.
     */
    private fun readIdentity(o: JSONObject): String {
        val verdict = o.optJSONObject("verdict")
        val best = verdict ?: o.optJSONArray("candidates")?.optJSONObject(0)
        if (best != null) {
            val n = best.optString("plant_name").trim()
            if (n.isNotBlank()) {
                val bot = best.optString("botanical_name").trim()
                val mine = if (o.optBoolean("in_catalogue")) AppText.s(R.string.in_your_garden) else ""
                return n + (if (bot.isNotBlank()) "  ($bot)" else "") + mine
            }
        }
        // Nothing matched the catalogue: fall back to what the vision pass
        // said it saw, which is a description rather than a determination.
        val seen = o.optJSONObject("seen")?.optJSONArray("names")?.optString(0)?.trim()
        return seen.orEmpty()
    }

    /**
     * The health reading, as its three written sections.
     *
     * THE SECTIONS ARE THE POINT, and they are kept in the server's order:
     * what is visible, what that is consistent with, and what would confirm
     * it. That shape exists so a reading cannot become a diagnosis by
     * accident -- "yellowing between the veins" is an observation, and the
     * thing it might mean is kept separate from it on purpose. Flattening
     * them into one paragraph would undo that.
     */
    private fun readDiagnosis(o: JSONObject): String {
        val sections = listOf(
            "looked_at" to AppText.s(R.string.seen_visible),
            "consistent" to AppText.s(R.string.seen_consistent),
            "confirm" to AppText.s(R.string.seen_confirm),
        )
        val text = buildString {
            sections.forEach { (key, heading) ->
                val v = o.optString(key).trim()
                if (v.isNotBlank()) {
                    if (isNotEmpty()) append("\n\n")
                    append(heading).append('\n').append(v)
                }
            }
        }
        return text.ifBlank { o.optString("note").trim() }
    }

    private fun decodeBitmap(context: Context, uri: Uri): Bitmap? {
        val resolver = context.contentResolver
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        resolver.openInputStream(uri)?.use {
            BitmapFactory.decodeStream(it, null, bounds)
        }
        var sample = 1
        while (bounds.outWidth / sample > MAX_EDGE * 2 ||
            bounds.outHeight / sample > MAX_EDGE * 2
        ) {
            sample *= 2
        }
        return resolver.openInputStream(uri)?.use { stream ->
            BitmapFactory.decodeStream(
                stream, null,
                BitmapFactory.Options().apply {
                    inSampleSize = sample
                    // ZXing reads pixels out as ARGB, and a hardware bitmap
                    // cannot be read at all -- getPixels throws on one.
                    inPreferredConfig = Bitmap.Config.ARGB_8888
                },
            )
        }
    }

    /**
     * The photograph, shrunk, turned upright, as base64 JPEG.
     *
     * ROTATION MATTERS MORE HERE THAN ANYWHERE. A phone writes the picture
     * in sensor orientation and records the rotation in EXIF; a vision
     * model reading a sideways plant tag reads a sideways plant tag.
     */
    private fun encode(context: Context, uri: Uri): String {
        val decoded = decodeBitmap(context, uri)
            ?: throw GardenException(AppText.s(R.string.photo_unreadable))

        val rotation = context.contentResolver.openInputStream(uri)?.use { stream ->
            when (
                ExifInterface(stream).getAttributeInt(
                    ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL
                )
            ) {
                ExifInterface.ORIENTATION_ROTATE_90 -> 90f
                ExifInterface.ORIENTATION_ROTATE_180 -> 180f
                ExifInterface.ORIENTATION_ROTATE_270 -> 270f
                else -> 0f
            }
        } ?: 0f

        val scale = MAX_EDGE.toFloat() / maxOf(decoded.width, decoded.height).toFloat()
        val matrix = Matrix().apply {
            if (scale < 1f) postScale(scale, scale)
            if (rotation != 0f) postRotate(rotation)
        }
        val finished =
            if (scale < 1f || rotation != 0f) {
                Bitmap.createBitmap(
                    decoded, 0, 0, decoded.width, decoded.height, matrix, true
                )
            } else {
                decoded
            }

        val out = ByteArrayOutputStream()
        finished.compress(Bitmap.CompressFormat.JPEG, JPEG_QUALITY, out)
        if (finished !== decoded) finished.recycle()
        decoded.recycle()
        return Base64.encodeToString(out.toByteArray(), Base64.NO_WRAP)
    }

    private companion object {
        val BASE get() = GardenAddress.base + "/app/garden"
        /** Plenty to read a tag by; far below what a camera produces. */
        const val MAX_EDGE = 1600

        /**
         * Tags per request. Five pairs is at most ten photographs, roughly
         * four megabytes of base64 -- an ordinary request body. Thirty tags
         * in one go would be fifteen, which is the size that gets refused
         * by a proxy or dropped on a weak signal halfway through.
         */
        const val BULK_BATCH = 5
        const val JPEG_QUALITY = 85
    }
}
