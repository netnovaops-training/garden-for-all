// ScanModel.kt -- what the scan screens know and do: the queue, the beds, the sticky bed and count.
//
// Held by ScanScreen with `remember`. Everything that must survive (the scans and their pictures)
// is in ScanStore on disk; this only mirrors it for the screen and runs the actions.

package org.jasonagibson.junsgarden.garden

import android.content.Context
import android.net.Uri
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.jasonagibson.junsgarden.R
import org.jasonagibson.junsgarden.ui.AppText
import java.io.File

class ScanModel(
    private val context: Context,
    private val camera: GardenCamera,
    private val scope: CoroutineScope,
    login: GardenLogin,
) {
    private val store = ScanStore(context)
    private val api = ScanApi { login.token() }

    /** Newest first. */
    var entries by mutableStateOf(store.all().sortedByDescending { it.createdMs })
        private set
    var beds by mutableStateOf<List<ScanBed>>(emptyList())
        private set

    /** The bed new scans go into until it is changed (kept between runs). */
    var bed by mutableStateOf(store.lastBed()?.let { ScanBed(it.first, it.second) })
        private set
    var qty by mutableIntStateOf(1)
    var owner by mutableStateOf(false)
        private set
    var helperSeen by mutableStateOf<String?>(null)
        private set
    var offline by mutableStateOf(false)
        private set
    var message by mutableStateOf<String?>(null)
    var saving by mutableStateOf(false)
        private set

    fun photo(e: ScanEntry, p: ScanPhoto): File = store.photoFile(e.uid, p.file)

    /** The bed's name now (the website's list), else the name it had when the scan was taken. */
    fun bedName(e: ScanEntry): String =
        beds.firstOrNull { it.id == e.bedId }?.name ?: e.bedName.ifBlank { "?" }

    private fun reload() {
        entries = store.all().sortedByDescending { it.createdMs }
    }

    suspend fun loadBeds() {
        runCatching { api.beds() }
            .onSuccess { list ->
                beds = list
                // A renamed bed keeps its number: show its new name.
                bed = bed?.let { b -> list.firstOrNull { it.id == b.id } ?: b }
            }
            .onFailure { if (beds.isEmpty()) message = AppText.s(R.string.scan_beds_failed) }
    }

    fun chooseBed(b: ScanBed) {
        bed = b
        store.setLastBed(b.id, b.name)
    }

    /** One round with the website: send what is waiting, bring back every scan's status. */
    suspend fun sync() {
        val r = ScanSync.sync(context)
        offline = r.networkProblem
        r.owner?.let { owner = it }
        helperSeen = r.helperSeen
        reload()
    }

    /**
     * Keep a new scan on the phone: the pictures turned upright and shrunk (GardenCamera.jpegBytes),
     * a barcode the phone itself could read, then one folder in the app's own storage. It is sent
     * at once if there is signal, and by the upload worker later if not.
     */
    fun add(kind: String, uris: List<Uri>) {
        val chosen = bed
        val count = qty
        saving = true
        scope.launch {
            val result = runCatching {
                withContext(Dispatchers.IO) {
                    val uid = ScanCore.newUid()
                    val dir = store.newFolder(uid)
                    val sides = if (kind == "plant") listOf("plant") else listOf("front", "back")
                    val photos = uris.mapIndexed { i, uri ->
                        val side = sides.getOrElse(i) { "front" }
                        File(dir, "$side.jpg").writeBytes(camera.jpegBytes(context, uri))
                        ScanPhoto(side, "$side.jpg")
                    }
                    var code = ""
                    for (uri in uris) {
                        code = camera.readBarcode(context, uri).orEmpty()
                        if (code.isNotEmpty()) break
                    }
                    store.save(
                        ScanEntry(
                            uid = uid, kind = kind, bedId = chosen?.id, bedName = chosen?.name.orEmpty(),
                            qty = count, barcode = code, createdMs = System.currentTimeMillis(), photos = photos,
                        )
                    )
                }
            }
            saving = false
            result
                .onSuccess {
                    qty = 1
                    message = AppText.s(R.string.scan_saved, chosen?.name ?: "")
                    reload()
                    ScanUploadWorker.enqueue(context)
                    runCatching { sync() }
                }
                .onFailure { message = AppText.s(R.string.scan_error, it.message ?: "") }
        }
    }

    fun confirm(e: ScanEntry, candidate: ScanCandidate) = act(e) { sid ->
        val c = api.confirm(sid, pick = candidate.id)
        if (c.merged) AppText.s(R.string.scan_added_more, c.name) else AppText.s(R.string.scan_added_ok, c.name)
    }

    fun confirmManual(e: ScanEntry, genus: String, species: String, name: String) = act(e) { sid ->
        AppText.s(R.string.scan_added_ok, api.confirm(sid, manual = Triple(genus, species, name)).name)
    }

    /** "Type it instead" without a scan: the plant is saved at once. */
    fun typeIn(genus: String, species: String, name: String, price: String) {
        val chosen = bed
        val count = qty
        scope.launch {
            runCatching { api.manualAdd(genus, species, name, count, price, chosen?.id) }
                .onSuccess { qty = 1; message = AppText.s(R.string.scan_added_ok, name) }
                .onFailure { message = describe(it) }
        }
    }

    fun changeBed(e: ScanEntry, b: ScanBed) {
        store.update(e.uid) { it.copy(bedId = b.id, bedName = b.name) }
        reload()
        val sid = e.serverId ?: return
        scope.launch { runCatching { api.changeBed(sid, b.id) }.onFailure { message = describe(it) } }
    }

    fun remove(e: ScanEntry) {
        val sid = e.serverId
        scope.launch {
            // Taken off the website first (when it has it), so it is not brought back by the next round.
            if (sid != null) runCatching { api.remove(sid) }.onFailure { message = describe(it); return@launch }
            store.delete(e.uid)
            reload()
        }
    }

    /** Run a call for a scan the website has, show its sentence, then bring the new status back. */
    private fun act(e: ScanEntry, call: suspend (Long) -> String) {
        val sid = e.serverId ?: return
        scope.launch {
            runCatching { call(sid) }
                .onSuccess { message = it }
                .onFailure { message = describe(it) }
            runCatching { sync() }
        }
    }

    private fun describe(t: Throwable): String = when {
        t is ScanHttpException && t.ownerOnly -> AppText.s(R.string.scan_owner_only)
        else -> AppText.s(R.string.scan_error, t.message ?: "")
    }
}
