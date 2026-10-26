// The new app, on a real (emulated) phone, against the lab garden. TEST BUILDS ONLY: matrix.sh
// copies it into a throwaway copy of android-app as src/androidTest. Each test is run on its
// own (am instrument -e class ...#name) at the right moment of matrix.sh's story, with:
//   -e expect  the address the app should have now     -e user / -e pw  the lab's Tester
//   -e vc      the version code the update file offers
package org.jasonagibson.junsgarden

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.jasonagibson.junsgarden.app.Updater
import org.jasonagibson.junsgarden.garden.GardenAddress
import org.jasonagibson.junsgarden.garden.GardenApi
import org.jasonagibson.junsgarden.garden.GardenLogin
import org.jasonagibson.junsgarden.ui.AppText
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.net.HttpURLConnection
import java.net.URL

@RunWith(AndroidJUnit4::class)
class MatrixTest {
    private val args = InstrumentationRegistry.getArguments()
    private val ctx get() = InstrumentationRegistry.getInstrumentation().targetContext
    private fun arg(k: String) = requireNotNull(args.getString(k)) { "missing -e $k" }

    @Before
    fun setUp() {
        AppText.init(ctx)
        GardenAddress.init(ctx)
    }

    /** The address the app uses right now (after a setup link, or a move). */
    @Test
    fun addressIs() = assertEquals(arg("expect"), GardenAddress.base)

    /** Login, the plant list, one plant, the beds, a barcode, a photo and comments. */
    @Test
    fun loginAndBrowse() = runBlocking {
        val r = GardenLogin(ctx).login(arg("user"), arg("pw"))
        assertTrue("login: ${r.exceptionOrNull()?.message}", r.isSuccess)
        val page = GardenApi.list(1)
        assertTrue("plant list is empty", page.plants.isNotEmpty())
        val first = page.plants.first()
        assertEquals(first.id, GardenApi.detail(first.id).id)
        assertTrue("no beds", GardenApi.beds().isNotEmpty())
        GardenApi.barcode("000000000007") // answered (found or not) without an error
        val photo = page.plants.firstNotNullOfOrNull { it.photo }
        assertNotNull("no plant photo on page 1", photo)
        assertEquals("photo $photo", 200, status(photo!!))
        assertEquals(200, status("${GardenAddress.base}/app_api.php?action=thumb&img=plant-0.png&w=200"))
        val comments = URL("${GardenAddress.base}/app_api.php?action=comments&id=2").readText()
        assertTrue("comments: ${comments.take(80)}", comments.trimStart().startsWith("{"))
    }

    /** The signature check works on the phone itself (RFC 8032 test key, the same pinned
     *  signature the JVM unit test and the Rust side use). */
    @Test
    fun signatureCheckWorksOnThePhone() {
        val key = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a" // gitleaks:allow
        val sig = "9932dc7d968058367785afbcff754936a057e9bbd672df8ae39afc7bb1614cac" +
            "2417e53999c1ce944247ee699ac17be6b85282fc856456b3151ad13f003ba40b"
        val msg = GardenAddress.configMessage("https://example.com/garden", 1790000000)
        assertTrue("a good signature was refused", GardenAddress.verify(key, msg, sig))
        assertTrue("a bad signature was accepted", !GardenAddress.verify(key, msg + "x", sig))
    }

    /** The garden moved: the app follows on its next contact (signed config only). */
    @Test
    fun followsMove() = runBlocking {
        assertTrue("refresh did not move", GardenAddress.refresh(ctx))
        assertEquals(arg("expect"), GardenAddress.base)
    }

    /** The update file at the current address offers vc, from that address, with a sha256 that matches. */
    @Test
    fun updateFromCurrentAddress() = runBlocking {
        val up = Updater().check()
        assertNotNull("no update offered", up)
        assertEquals(arg("vc").toInt(), up!!.versionCode)
        assertTrue("apk from ${up.url}", up.url.startsWith(arg("expect")))
        assertEquals(64, up.sha256.length)
        Updater().downloadAndInstall(ctx, up) {} // throws if the sha256 does not match
    }

    private fun status(url: String): Int =
        (URL(url).openConnection() as HttpURLConnection).run {
            try { responseCode } finally { disconnect() }
        }
}
