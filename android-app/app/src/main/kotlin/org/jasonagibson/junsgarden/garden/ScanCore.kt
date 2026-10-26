// ScanCore.kt -- the scan queue's rules, with no Android in them (so they run as plain unit tests).
//
// A SCAN is one tag (front and back) or one plant photo, with a bed and a count. It lives in a
// folder on the phone until the website has it, and the website's answer (waiting, reading,
// matched, needs you, added, failed) is copied back here so the screen can show it.
// ScanStore.kt keeps the folders, ScanApi.kt talks to the website, ScanSync.kt joins them.

package org.jasonagibson.junsgarden.garden

import java.util.Properties
import kotlin.math.min
import kotlin.random.Random

/** One picture of a scan, kept on the phone as a file in the scan's folder. */
data class ScanPhoto(val side: String, val file: String)

/** One possible plant for a scan, as the PC found it. */
data class ScanCandidate(
    val id: String,
    val name: String,
    val botanical: String,
    /** 0.0 to 1.0. */
    val confidence: Double,
    /** garden, wishlist, image, web or tag. */
    val source: String,
    /** "current" when it is a plant already in the garden (its count is added to). */
    val set: String,
    val plantId: Long?,
    val why: String,
) {
    val percent: Int get() = Math.round(confidence * 100).toInt().coerceIn(0, 100)
}

/**
 * A scan on the phone.
 *
 * THE BED IS KEPT BY NUMBER (`bedId`). The name is only what it was called when the picture was
 * taken, for showing when the website cannot be asked; the screen prefers the bed's name now.
 */
data class ScanEntry(
    val uid: String,
    /** "tag" or "plant". */
    val kind: String,
    val bedId: Long?,
    val bedName: String,
    val qty: Int,
    val barcode: String,
    val createdMs: Long,
    val photos: List<ScanPhoto>,
    /** The website's number for it, once it has it. */
    val serverId: Long? = null,
    /** saved (on the phone only), waiting, reading, matched, needs_you, added, failed. */
    val status: String = ScanCore.SAVED,
    val step: String = "",
    val error: String = "",
    val plantId: Long? = null,
    val candidates: List<ScanCandidate> = emptyList(),
    val attempts: Int = 0,
    val nextTryMs: Long = 0,
)

/** What the website says about one of our scans (parsed from its JSON by ScanApi). */
data class RemoteScan(
    val id: Long,
    val clientUid: String,
    val status: String,
    val step: String,
    val error: String,
    val plantId: Long?,
    val bedId: Long?,
    val bedName: String,
    val candidates: List<ScanCandidate>,
)

object ScanCore {
    const val SAVED = "saved"
    const val WAITING = "waiting"
    const val READING = "reading"
    const val MATCHED = "matched"
    const val NEEDS_YOU = "needs_you"
    const val ADDED = "added"
    const val FAILED = "failed"

    /** A scan's id, made on the phone, so sending it twice never makes two scans. */
    fun newUid(random: Random = Random.Default): String {
        val hex = ByteArray(12).also { random.nextBytes(it) }.joinToString("") { "%02x".format(it) }
        return "ph-$hex"
    }

    /** Wait before the next try after `attempts` failures: 15 s, 30 s, 1 min ... at most 30 min. */
    fun backoffMs(attempts: Int): Long =
        min(30 * 60_000L, 15_000L shl min(attempts.coerceAtLeast(0), 7))

    /** Scans still only on the phone and due for a try, oldest first. */
    fun due(entries: List<ScanEntry>, nowMs: Long): List<ScanEntry> =
        entries.filter { it.status == SAVED && it.serverId == null && it.nextTryMs <= nowMs }
            .sortedBy { it.createdMs }

    /** After a failed upload: count it and say when to try again. */
    fun afterFailure(e: ScanEntry, nowMs: Long, why: String): ScanEntry =
        e.copy(attempts = e.attempts + 1, nextTryMs = nowMs + backoffMs(e.attempts), error = why)

    /** After the website took it. */
    fun afterUpload(e: ScanEntry, remote: RemoteScan): ScanEntry =
        applyRemote(e.copy(serverId = remote.id, attempts = 0, nextTryMs = 0), remote)

    /** Copy the website's view of a scan onto ours. */
    fun applyRemote(e: ScanEntry, r: RemoteScan): ScanEntry = e.copy(
        serverId = r.id,
        status = r.status,
        step = r.step,
        error = r.error,
        plantId = r.plantId,
        candidates = r.candidates,
        bedId = r.bedId ?: e.bedId,
        bedName = r.bedName.ifBlank { e.bedName },
    )

    /**
     * Bring our scans up to date with the website's list. A scan the website no longer has
     * (somebody removed it there) is returned in `gone` so its folder can be deleted; scans that
     * are still only on the phone are left alone.
     */
    fun merge(local: List<ScanEntry>, remote: List<RemoteScan>): Pair<List<ScanEntry>, List<ScanEntry>> {
        val byId = remote.associateBy { it.id }
        val keep = ArrayList<ScanEntry>()
        val gone = ArrayList<ScanEntry>()
        for (e in local) {
            val sid = e.serverId
            when {
                sid == null -> keep.add(e)
                byId[sid] != null -> keep.add(applyRemote(e, byId.getValue(sid)))
                // Added scans drop off the website's list of 100; keep their last known state.
                e.status == ADDED -> keep.add(e)
                else -> gone.add(e)
            }
        }
        return keep to gone
    }

    /** The counts for the line at the top of the screen. */
    data class Summary(val onPhone: Int, val atPc: Int, val needYou: Int, val added: Int, val failed: Int)

    fun summary(entries: List<ScanEntry>): Summary = Summary(
        onPhone = entries.count { it.status == SAVED },
        atPc = entries.count { it.status == WAITING || it.status == READING },
        needYou = entries.count { it.status == MATCHED || it.status == NEEDS_YOU },
        added = entries.count { it.status == ADDED },
        failed = entries.count { it.status == FAILED },
    )

    /** Does this scan still wait for something (so the screen keeps asking the website)? */
    fun busy(entries: List<ScanEntry>): Boolean =
        entries.any { it.status == SAVED || it.status == WAITING || it.status == READING }

    // ---- saving to a file -------------------------------------------------------------------

    fun toProperties(e: ScanEntry): Properties = Properties().apply {
        setProperty("uid", e.uid)
        setProperty("kind", e.kind)
        e.bedId?.let { setProperty("bedId", it.toString()) }
        setProperty("bedName", e.bedName)
        setProperty("qty", e.qty.toString())
        setProperty("barcode", e.barcode)
        setProperty("createdMs", e.createdMs.toString())
        setProperty("photos", e.photos.joinToString(";") { "${it.side}|${it.file}" })
        e.serverId?.let { setProperty("serverId", it.toString()) }
        setProperty("status", e.status)
        setProperty("step", e.step)
        setProperty("error", e.error)
        e.plantId?.let { setProperty("plantId", it.toString()) }
        setProperty("attempts", e.attempts.toString())
        setProperty("nextTryMs", e.nextTryMs.toString())
        setProperty("cands", e.candidates.size.toString())
        e.candidates.forEachIndexed { i, c ->
            setProperty("c$i.id", c.id)
            setProperty("c$i.name", c.name)
            setProperty("c$i.botanical", c.botanical)
            setProperty("c$i.confidence", c.confidence.toString())
            setProperty("c$i.source", c.source)
            setProperty("c$i.set", c.set)
            c.plantId?.let { setProperty("c$i.plantId", it.toString()) }
            setProperty("c$i.why", c.why)
        }
    }

    /** The scan saved by [toProperties]; null when the file is damaged (it is then skipped). */
    fun fromProperties(p: Properties): ScanEntry? {
        val uid = p.getProperty("uid")?.takeIf { it.isNotBlank() } ?: return null
        val photos = p.getProperty("photos", "").split(";").filter { it.contains("|") }
            .map { ScanPhoto(it.substringBefore("|"), it.substringAfter("|")) }
        val n = p.getProperty("cands", "0").toIntOrNull() ?: 0
        val cands = (0 until n).map { i ->
            ScanCandidate(
                id = p.getProperty("c$i.id", "c${i + 1}"),
                name = p.getProperty("c$i.name", ""),
                botanical = p.getProperty("c$i.botanical", ""),
                confidence = p.getProperty("c$i.confidence")?.toDoubleOrNull() ?: 0.0,
                source = p.getProperty("c$i.source", ""),
                set = p.getProperty("c$i.set", ""),
                plantId = p.getProperty("c$i.plantId")?.toLongOrNull(),
                why = p.getProperty("c$i.why", ""),
            )
        }
        return ScanEntry(
            uid = uid,
            kind = p.getProperty("kind", "tag"),
            bedId = p.getProperty("bedId")?.toLongOrNull(),
            bedName = p.getProperty("bedName", ""),
            qty = p.getProperty("qty")?.toIntOrNull() ?: 1,
            barcode = p.getProperty("barcode", ""),
            createdMs = p.getProperty("createdMs")?.toLongOrNull() ?: 0L,
            photos = photos,
            serverId = p.getProperty("serverId")?.toLongOrNull(),
            status = p.getProperty("status", SAVED),
            step = p.getProperty("step", ""),
            error = p.getProperty("error", ""),
            plantId = p.getProperty("plantId")?.toLongOrNull(),
            candidates = cands,
            attempts = p.getProperty("attempts")?.toIntOrNull() ?: 0,
            nextTryMs = p.getProperty("nextTryMs")?.toLongOrNull() ?: 0L,
        )
    }
}
