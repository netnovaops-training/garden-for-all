// ScanApi.kt -- talks to the website's scan queue (/app/scan/...), signed in with the garden key.
//
// The phone does not talk to the PC. It hands scans to the garden website (which is reachable from
// anywhere); the PC with the graphics card asks the website for them. So a scan taken far from
// home is uploaded as soon as there is signal, and read when the PC next asks (about a minute).

package org.jasonagibson.junsgarden.garden

import android.util.Base64
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/** A bed: its number (never changes) and its name now. */
data class ScanBed(val id: Long, val name: String)

/** The website's list: our scans, whether this person may confirm, and when the PC last asked. */
data class ScanList(val scans: List<RemoteScan>, val owner: Boolean, val helperSeen: String)

/** What a confirm did. */
data class ScanConfirmed(val plantId: Long, val name: String, val bed: String, val merged: Boolean)

/** The website said no. `ownerOnly` is true when only the garden owner may do it. */
class ScanHttpException(val code: Int, message: String, val ownerOnly: Boolean = false) : Exception(message)

class ScanApi(private val token: () -> String?) {
    private val base get() = GardenAddress.base + "/app/scan"

    suspend fun beds(): List<ScanBed> {
        val arr = call("GET", "/beds").optJSONArray("beds") ?: JSONArray()
        return (0 until arr.length()).mapNotNull { i ->
            arr.optJSONObject(i)?.let { o -> o.optLong("id").takeIf { it > 0 }?.let { ScanBed(it, o.optString("name")) } }
        }
    }

    suspend fun list(): ScanList {
        val o = call("GET", "/items")
        val arr = o.optJSONArray("items") ?: JSONArray()
        return ScanList(
            scans = (0 until arr.length()).mapNotNull { i -> arr.optJSONObject(i)?.let(::remote) },
            owner = o.optBoolean("owner", false),
            helperSeen = o.optString("helper_last_seen", ""),
        )
    }

    /** Send one scan with its pictures. The website answers with the same scan if it already has it. */
    suspend fun upload(e: ScanEntry, bytes: (ScanPhoto) -> ByteArray): RemoteScan {
        val photos = JSONArray()
        e.photos.forEach { p ->
            photos.put(
                JSONObject().put("side", p.side)
                    .put("data", Base64.encodeToString(bytes(p), Base64.NO_WRAP))
            )
        }
        val body = JSONObject()
            .put("client_uid", e.uid).put("kind", e.kind).put("qty", e.qty)
            .put("barcode", e.barcode).put("photos", photos)
        e.bedId?.let { body.put("bed_id", it) }
        val o = call("POST", "/items", body)
        return remote(o.optJSONObject("item") ?: throw ScanHttpException(500, "the website sent no scan"))
    }

    /** One tap on a match (`pick` = "c1"...) or a typed plant (`manual`). The garden owner only. */
    suspend fun confirm(serverId: Long, pick: String? = null, manual: Triple<String, String, String>? = null): ScanConfirmed {
        val body = JSONObject()
        if (pick != null) body.put("pick", pick)
        if (manual != null) {
            body.put("manual", JSONObject().put("genus", manual.first).put("species", manual.second).put("name", manual.third))
        }
        val o = call("POST", "/items/$serverId/confirm", body)
        return ScanConfirmed(o.optLong("plant_id"), o.optString("name"), o.optString("bed"), o.optBoolean("merged"))
    }

    suspend fun changeBed(serverId: Long, bedId: Long) {
        call("POST", "/items/$serverId/bed", JSONObject().put("bed_id", bedId))
    }

    suspend fun remove(serverId: Long) {
        call("POST", "/items/$serverId/remove", JSONObject())
    }

    /** "Type it instead": a plant saved at once (genus, species, name, how many, price, bed). */
    suspend fun manualAdd(genus: String, species: String, name: String, qty: Int, price: String, bedId: Long?): Long {
        val body = JSONObject().put("genus", genus).put("species", species).put("name", name).put("qty", qty)
        if (price.isNotBlank()) body.put("price", price)
        bedId?.let { body.put("bed_id", it) }
        return call("POST", "/manual", body).optLong("plant_id")
    }

    // -- plumbing ---------------------------------------------------------------------------

    private fun remote(o: JSONObject): RemoteScan {
        val arr = o.optJSONArray("candidates") ?: JSONArray()
        val cands = (0 until arr.length()).mapNotNull { i ->
            arr.optJSONObject(i)?.let { c ->
                ScanCandidate(
                    id = c.optString("id"), name = c.optString("name"), botanical = c.optString("botanical"),
                    confidence = c.optDouble("confidence", 0.0), source = c.optString("source"),
                    set = c.optString("set"), plantId = if (c.isNull("plant_id")) null else c.optLong("plant_id"),
                    why = c.optString("why"),
                )
            }
        }
        return RemoteScan(
            id = o.optLong("id"), clientUid = o.optString("client_uid"), status = o.optString("status", ScanCore.WAITING),
            step = o.optString("step"), error = o.optString("error"),
            plantId = if (o.isNull("plant_id")) null else o.optLong("plant_id"),
            bedId = if (o.isNull("bed_id")) null else o.optLong("bed_id"),
            bedName = if (o.isNull("bed_name")) "" else o.optString("bed_name"),
            candidates = cands,
        )
    }

    private suspend fun call(method: String, path: String, body: JSONObject? = null): JSONObject =
        withContext(Dispatchers.IO) {
            val key = token() ?: throw ScanHttpException(401, "sign in first")
            val conn = (URL(base + path).openConnection() as HttpURLConnection).apply {
                requestMethod = method
                connectTimeout = 20_000
                // A scan is a few pictures: give a weak signal time before giving up (it is retried).
                readTimeout = 120_000
                setRequestProperty("Authorization", "Bearer $key")
                setRequestProperty("Accept", "application/json")
                if (body != null) {
                    doOutput = true
                    setRequestProperty("Content-Type", "application/json")
                }
            }
            try {
                if (body != null) conn.outputStream.use { it.write(body.toString().toByteArray()) }
                val code = conn.responseCode
                val text = (if (code in 200..299) conn.inputStream else conn.errorStream)
                    ?.bufferedReader()?.use { it.readText() } ?: ""
                val json = runCatching { JSONObject(text) }.getOrNull()
                if (code !in 200..299) {
                    throw ScanHttpException(
                        code,
                        json?.optString("error")?.takeIf { it.isNotBlank() } ?: "HTTP $code",
                        json?.optBoolean("owner_only", false) == true,
                    )
                }
                // A page that is not JSON is almost always a proxy error: say "not reachable".
                json ?: throw ScanHttpException(502, "not reachable")
            } finally {
                conn.disconnect()
            }
        }
}
