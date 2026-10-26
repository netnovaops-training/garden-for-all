// ScanSync.kt -- one round between the phone's queue and the website: send what is waiting on the
// phone, then bring back the status of every scan. Used by the screen (every few seconds while it
// is open) and by ScanUploadWorker (when the app is closed or there was no signal).

package org.jasonagibson.junsgarden.garden

import android.content.Context

/** What a round did. `networkProblem` is true when the website could not be reached at all. */
data class SyncResult(val uploaded: Int, val stillOnPhone: Int, val networkProblem: Boolean, val owner: Boolean?, val helperSeen: String)

object ScanSync {
    /**
     * Upload the due scans (oldest first, each on its own: one failing never stops the rest), then
     * read the website's list and copy it back. A scan that cannot be sent keeps its place and is
     * tried again later, a little less often each time (ScanCore.backoffMs); that is the "no
     * signal" case, and nothing is lost: the pictures stay in the phone's own storage.
     */
    suspend fun sync(context: Context, now: Long = System.currentTimeMillis()): SyncResult {
        val store = ScanStore(context)
        val login = GardenLogin(context)
        val api = ScanApi { login.token() }
        if (login.token() == null) return SyncResult(0, store.all().count { it.status == ScanCore.SAVED }, false, null, "")
        var uploaded = 0
        var network = false
        for (e in ScanCore.due(store.all(), now)) {
            try {
                val remote = api.upload(e) { p -> store.photoFile(e.uid, p.file).readBytes() }
                store.update(e.uid) { ScanCore.afterUpload(it, remote) }
                uploaded++
            } catch (x: ScanHttpException) {
                // The website answered "no" (a bad picture, a bed that is gone, signed out): keep it
                // for the person to see, and try later.
                store.update(e.uid) { ScanCore.afterFailure(it, now, x.message ?: "") }
                if (x.code == 401 || x.code == 502) network = true
            } catch (x: java.io.IOException) {
                store.update(e.uid) { ScanCore.afterFailure(it, now, "no signal") }
                network = true
            }
        }
        var owner: Boolean? = null
        var seen = ""
        try {
            val list = api.list()
            owner = list.owner
            seen = list.helperSeen
            val (keep, gone) = ScanCore.merge(store.all(), list.scans)
            keep.forEach { e -> store.update(e.uid) { cur -> mergeKeepingLocal(cur, e) } }
            gone.forEach { store.delete(it.uid) }
        } catch (x: ScanHttpException) {
            if (x.code == 401 || x.code == 502) network = true
        } catch (x: java.io.IOException) {
            network = true
        }
        return SyncResult(uploaded, store.all().count { it.status == ScanCore.SAVED }, network, owner, seen)
    }

    /** The website's view, but the retry bookkeeping the phone keeps stays as it is on disk now. */
    private fun mergeKeepingLocal(current: ScanEntry, merged: ScanEntry) =
        merged.copy(attempts = current.attempts, nextTryMs = current.nextTryMs)
}
