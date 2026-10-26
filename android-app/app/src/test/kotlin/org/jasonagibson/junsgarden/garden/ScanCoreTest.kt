package org.jasonagibson.junsgarden.garden

import java.io.StringReader
import java.io.StringWriter
import java.util.Properties
import kotlin.random.Random
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** The scan queue's rules (ScanCore.kt): saving, retrying, copying the website's answer back. */
class ScanCoreTest {
    private fun scan(uid: String = "ph-1", created: Long = 1000, bed: Long? = 7, status: String = ScanCore.SAVED) = ScanEntry(
        uid = uid, kind = "tag", bedId = bed, bedName = "Zen Garden", qty = 2, barcode = "0417776317063",
        createdMs = created, photos = listOf(ScanPhoto("front", "front.jpg"), ScanPhoto("back", "back.jpg")), status = status,
    )

    private fun remote(id: Long, uid: String, status: String, bed: Long? = 7, bedName: String = "Zen Garden", cands: List<ScanCandidate> = emptyList()) =
        RemoteScan(id, uid, status, "", "", null, bed, bedName, cands)

    @Test
    fun aScanSurvivesBeingWrittenToAFileAndReadBack() {
        val cand = ScanCandidate("c1", "May Night Salvia", "Salvia nemorosa 'May Night'", 0.93, "garden", "current", 12, "barcode matches")
        val e = scan().copy(serverId = 55, status = ScanCore.MATCHED, plantId = null, candidates = listOf(cand), attempts = 2, nextTryMs = 99, error = "slow: try again")
        val text = StringWriter().also { ScanCore.toProperties(e).store(it, null) }.toString()
        val back = ScanCore.fromProperties(Properties().apply { load(StringReader(text)) })
        assertEquals(e, back)
    }

    @Test
    fun aDamagedFileIsSkippedNotACrash() {
        assertNull(ScanCore.fromProperties(Properties()))
        val p = Properties().apply { setProperty("uid", "ph-9"); setProperty("qty", "many"); setProperty("cands", "x") }
        val e = ScanCore.fromProperties(p)!!
        assertEquals(1, e.qty)
        assertTrue(e.candidates.isEmpty())
    }

    @Test
    fun theBedIsKeptByNumberWhateverItsNameBecomes() {
        val e = scan(bed = 7)
        // The website now calls bed 7 something else: the scan follows the number.
        val (kept, _) = ScanCore.merge(listOf(e.copy(serverId = 5)), listOf(remote(5, "ph-1", ScanCore.WAITING, bed = 7, bedName = "Zen Garden (renamed)")))
        assertEquals(7L, kept[0].bedId)
        assertEquals("Zen Garden (renamed)", kept[0].bedName)
    }

    @Test
    fun uidsAreUniqueAndLookRight() {
        val a = ScanCore.newUid(Random(1))
        val b = ScanCore.newUid(Random(2))
        assertNotEquals(a, b)
        assertTrue(Regex("ph-[0-9a-f]{24}").matches(a))
        assertTrue(a.length in 8..64)          // what the website accepts
    }

    @Test
    fun noSignalMeansWaitingLongerAndLonger() {
        assertEquals(15_000L, ScanCore.backoffMs(0))
        assertEquals(30_000L, ScanCore.backoffMs(1))
        assertEquals(30 * 60_000L, ScanCore.backoffMs(50))
        val e = ScanCore.afterFailure(scan(), nowMs = 10_000, why = "no signal")
        assertEquals(1, e.attempts)
        assertEquals(25_000L, e.nextTryMs)
        assertTrue(ScanCore.due(listOf(e), 20_000).isEmpty())
        assertEquals(1, ScanCore.due(listOf(e), 25_000).size)
    }

    @Test
    fun onlyScansNotYetUploadedAreDueOldestFirst() {
        val list = listOf(
            scan("ph-3", created = 300),
            scan("ph-1", created = 100),
            scan("ph-2", created = 200).copy(serverId = 9, status = ScanCore.WAITING),
        )
        assertEquals(listOf("ph-1", "ph-3"), ScanCore.due(list, 1_000_000).map { it.uid })
    }

    @Test
    fun theWebsitesAnswerIsCopiedBack() {
        val cand = ScanCandidate("c1", "Blue Angel Hosta", "Hosta 'Blue Angel'", 0.8, "image", "", null, "")
        val up = ScanCore.afterUpload(scan(), remote(5, "ph-1", ScanCore.WAITING))
        assertEquals(5L, up.serverId)
        assertEquals(ScanCore.WAITING, up.status)
        val (kept, gone) = ScanCore.merge(listOf(up), listOf(remote(5, "ph-1", ScanCore.MATCHED, cands = listOf(cand))))
        assertTrue(gone.isEmpty())
        assertEquals(ScanCore.MATCHED, kept[0].status)
        assertEquals(80, kept[0].candidates[0].percent)
    }

    @Test
    fun aScanRemovedOnTheWebsiteGoesButOneStillOnThePhoneStays() {
        val onPhone = scan("ph-1")
        val sent = scan("ph-2").copy(serverId = 8, status = ScanCore.WAITING)
        val added = scan("ph-3").copy(serverId = 9, status = ScanCore.ADDED)
        val (kept, gone) = ScanCore.merge(listOf(onPhone, sent, added), emptyList())
        assertEquals(listOf("ph-1", "ph-3"), kept.map { it.uid })
        assertEquals(listOf("ph-2"), gone.map { it.uid })
    }

    @Test
    fun theTopLineCountsWhereEachScanIs() {
        val s = ScanCore.summary(
            listOf(
                scan("a"), scan("b"),
                scan("c", status = ScanCore.WAITING), scan("d", status = ScanCore.READING),
                scan("e", status = ScanCore.MATCHED), scan("f", status = ScanCore.NEEDS_YOU),
                scan("g", status = ScanCore.ADDED), scan("h", status = ScanCore.FAILED),
            )
        )
        assertEquals(ScanCore.Summary(onPhone = 2, atPc = 2, needYou = 2, added = 1, failed = 1), s)
        assertTrue(ScanCore.busy(listOf(scan("c", status = ScanCore.WAITING))))
        assertFalse(ScanCore.busy(listOf(scan("e", status = ScanCore.MATCHED), scan("g", status = ScanCore.ADDED))))
    }

    @Test
    fun confidenceShowsAsAWholePercent() {
        assertEquals(93, ScanCandidate("c1", "x", "", 0.926, "tag", "", null, "").percent)
        assertEquals(100, ScanCandidate("c1", "x", "", 1.4, "tag", "", null, "").percent)
        assertEquals(0, ScanCandidate("c1", "x", "", -0.2, "tag", "", null, "").percent)
    }
}
