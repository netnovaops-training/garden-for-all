// The OLD app (3.2 as shipped, git cd4e526: no address following, md5 updates), on a real
// (emulated) phone, against the NEW server. TEST BUILDS ONLY: matrix.sh puts it into a throwaway
// copy of that version's android-app. Proves an app that is never updated keeps working.
//   -e user / -e pw  the lab's Tester     -e vc  the version code the update file offers
//   -e apk           where the update file says the new app is
package org.jasonagibson.junsgarden

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import kotlinx.coroutines.runBlocking
import org.jasonagibson.junsgarden.app.Updater
import org.jasonagibson.junsgarden.garden.GardenApi
import org.jasonagibson.junsgarden.garden.GardenLogin
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class OldMatrixTest {
    private val args = InstrumentationRegistry.getArguments()
    private val ctx get() = InstrumentationRegistry.getInstrumentation().targetContext
    private fun arg(k: String) = requireNotNull(args.getString(k)) { "missing -e $k" }

    @Test
    fun loginAndBrowse() = runBlocking {
        val r = GardenLogin(ctx).login(arg("user"), arg("pw"))
        assertTrue("login: ${r.exceptionOrNull()?.message}", r.isSuccess)
        val page = GardenApi.list(1)
        assertTrue("plant list is empty", page.plants.isNotEmpty())
        assertEquals(page.plants.first().id, GardenApi.detail(page.plants.first().id).id)
        assertTrue("no beds", GardenApi.beds().isNotEmpty())
        assertTrue("whoami after login", GardenLogin(ctx).check() == true)
    }

    /** The old app's update check (its fixed address = the old name) finds the new app. */
    @Test
    fun updateToNewApp() = runBlocking {
        val up = Updater().check()
        assertNotNull("no update offered", up)
        assertEquals(arg("vc").toInt(), up!!.versionCode)
        assertEquals(arg("apk"), up.url)
        Updater().downloadAndInstall(ctx, up) {} // throws if the md5 does not match
    }
}
