package org.jasonagibson.junsgarden.garden

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The address rules (GardenAddress.kt). The key is the RFC 8032 test key; the Rust side
 * (rust/junsgarden/src/address.rs) signs with the same seed in its own tests, and the
 * signature below is what that key makes for this exact config text.
 */
class GardenAddressTest {
    // The PUBLIC key of RFC 8032's published test vector 1: test data, not a secret.
    private val key = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a" // gitleaks:allow
    private val base = "https://example.com/garden"
    private val sig = "9932dc7d968058367785afbcff754936a057e9bbd672df8ae39afc7bb1614cac" +
        "2417e53999c1ce944247ee699ac17be6b85282fc856456b3151ad13f003ba40b"

    @Test
    fun aSignedMoveVerifies() {
        assertTrue(GardenAddress.verify(key, GardenAddress.configMessage(base, 1790000000), sig))
    }

    @Test
    fun anythingElseDoesNot() {
        // another address, another time, another key, a broken signature
        assertFalse(GardenAddress.verify(key, GardenAddress.configMessage("https://evil.example", 1790000000), sig))
        assertFalse(GardenAddress.verify(key, GardenAddress.configMessage(base, 1790000001), sig))
        assertFalse(GardenAddress.verify("00".repeat(32), GardenAddress.configMessage(base, 1790000000), sig))
        assertFalse(GardenAddress.verify(key, GardenAddress.configMessage(base, 1790000000), "zz"))
    }

    @Test
    fun setupLinksAreRead() {
        val p = GardenAddress.parseSetupLink("junsgarden://setup?base=https%3A%2F%2Fexample.com%2Fgarden%2F&key=$key")
        assertEquals(base to key, p)
        assertEquals("http://192.0.2.7:8030" to null, GardenAddress.parseSetupLink("junsgarden://setup?base=http%3A%2F%2F192.0.2.7%3A8030"))
    }

    @Test
    fun badSetupLinksAreRefused() {
        assertNull(GardenAddress.parseSetupLink("https://example.com/setup?base=x"))
        assertNull(GardenAddress.parseSetupLink("junsgarden://setup?base=javascript%3Aalert(1)"))
        assertNull(GardenAddress.parseSetupLink("junsgarden://setup?base=ftp%3A%2F%2Fexample.com"))
        assertNull(GardenAddress.parseSetupLink("junsgarden://setup"))
        // a key that is not 64 hex is simply not saved
        assertEquals(base to null, GardenAddress.parseSetupLink("junsgarden://setup?base=https%3A%2F%2Fexample.com%2Fgarden&key=abc"))
    }
}
