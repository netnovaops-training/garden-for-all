// ScanStore.kt -- the scan queue on the phone: one folder per scan, kept until the website has it.
//
// filesDir/scanq/<uid>/item.properties   the scan (ScanCore.toProperties)
// filesDir/scanq/<uid>/front.jpg ...     its pictures
//
// filesDir is the app's private storage: it survives restarts and a missing signal, and is not
// cleared like the cache. Writes go to a temporary file first and are then renamed, so a phone
// that dies half way never leaves a half-written scan.

package org.jasonagibson.junsgarden.garden

import android.content.Context
import java.io.File
import java.util.Properties

class ScanStore(context: Context) {
    private val root = File(context.filesDir, "scanq").apply { mkdirs() }

    /** Every scan on the phone, oldest first. A damaged one is skipped. */
    fun all(): List<ScanEntry> = synchronized(LOCK) {
        (root.listFiles { f -> f.isDirectory } ?: emptyArray())
            .mapNotNull { read(File(it, ITEM)) }
            .sortedBy { it.createdMs }
    }

    fun get(uid: String): ScanEntry? = synchronized(LOCK) { read(File(folder(uid), ITEM)) }

    /** Save a scan (its folder is made if needed). */
    fun save(e: ScanEntry) = synchronized(LOCK) { write(e) }

    /**
     * Change one scan: read the newest copy, apply `change`, save. The upload worker and the
     * screen both change scans, so each change starts from what is on disk, not from an old copy.
     * Gives the saved scan, or null when it is gone.
     */
    fun update(uid: String, change: (ScanEntry) -> ScanEntry): ScanEntry? = synchronized(LOCK) {
        val now = read(File(folder(uid), ITEM)) ?: return null
        change(now).also { write(it) }
    }

    /** Throw a scan and its pictures away. */
    fun delete(uid: String) = synchronized(LOCK) { folder(uid).deleteRecursively(); Unit }

    fun photoFile(uid: String, file: String): File = File(folder(uid), File(file).name)

    /** The folder for a new scan's pictures. */
    fun newFolder(uid: String): File = folder(uid).apply { mkdirs() }

    /** The bed used last, kept so the next scan is one tap: its number and its name then. */
    fun lastBed(): Pair<Long, String>? = synchronized(LOCK) {
        val f = File(root, LAST_BED)
        if (!f.isFile) return null
        val p = Properties().apply { f.inputStream().use { load(it) } }
        val id = p.getProperty("id")?.toLongOrNull() ?: return null
        id to p.getProperty("name", "")
    }

    fun setLastBed(id: Long, name: String) = synchronized(LOCK) {
        val p = Properties().apply { setProperty("id", id.toString()); setProperty("name", name) }
        File(root, LAST_BED).outputStream().use { p.store(it, null) }
    }

    // A uid is made by ScanCore.newUid (letters, digits, "-"); the name is cleaned anyway, so a
    // strange one can never reach outside the scan folder.
    private fun folder(uid: String) = File(root, uid.filter { it.isLetterOrDigit() || it == '-' || it == '_' }.ifEmpty { "x" })

    private fun read(f: File): ScanEntry? = runCatching {
        if (!f.isFile) null else ScanCore.fromProperties(Properties().apply { f.inputStream().use { load(it) } })
    }.getOrNull()

    private fun write(e: ScanEntry) {
        val dir = folder(e.uid).apply { mkdirs() }
        val tmp = File(dir, "$ITEM.tmp")
        tmp.outputStream().use { ScanCore.toProperties(e).store(it, null) }
        val dest = File(dir, ITEM)
        if (!tmp.renameTo(dest)) {
            dest.delete()
            tmp.renameTo(dest)
        }
    }

    private companion object {
        const val ITEM = "item.properties"
        const val LAST_BED = "last-bed.properties"
        val LOCK = Any()
    }
}
