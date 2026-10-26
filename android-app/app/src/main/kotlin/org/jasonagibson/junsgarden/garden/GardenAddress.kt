// GardenAddress.kt -- where the garden is, and how the app keeps finding it.
//
// THE ADDRESS. Every call the app makes goes to GardenAddress.base. It starts as the address
// the app was built with (BuildConfig.GARDEN_BASE), so an app installed before this file
// existed behaves exactly as before. Jun's own garden sets it by scanning the SETUP QR the
// wizard shows (/app/setup on the garden): that QR is a link
//     junsgarden://setup?base=https%3A%2F%2Fexample.com%2Fgarden&key=<64 hex>
// which opens the app and saves the address and the garden's public key.
//
// FOLLOWING A MOVE. Once per launch the app asks <base>/app/config.json where the garden is
// now (an old address forwards to the new one). It moves to the new address ONLY when the
// answer is signed with the garden's key (Ed25519, checked here with Google Tink).
// If an old domain ever lapses and a stranger takes it, their answer is not signed with
// Jun's key, so the app stays put. Without a saved key (an app never set up by QR) the app
// never moves on its own.
//
// The signed text is exactly  "junsgarden-config-v1\n<base>\n<issued>"  (see the garden's
// rust/junsgarden/src/address.rs).

package org.jasonagibson.junsgarden.garden

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URI
import java.net.URL
import com.google.crypto.tink.subtle.Ed25519Verify
import java.net.URLDecoder

object GardenAddress {
    private const val PREFS = "junsgarden_address"

    /** The garden's address, no trailing slash: `https://example.com/garden`. */
    @Volatile
    var base: String = org.jasonagibson.junsgarden.BuildConfig.GARDEN_BASE.trimEnd('/')
        private set

    /** The garden's public key (64 hex), from the setup QR; null = never follow a move. */
    @Volatile
    private var key: String? = null

    /** Load the saved address and key. Call once, at start. */
    fun init(context: Context) {
        val p = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        p.getString("base", null)?.let { base = it }
        key = p.getString("key", null)
    }

    private fun save(context: Context, newBase: String, newKey: String?) {
        base = newBase
        if (newKey != null) key = newKey
        // commit(), not apply(): written before this returns, so an app closed right after a
        // move (or a setup scan) still has the new address next time. A tiny, rare write.
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
            .putString("base", newBase)
            .apply { if (newKey != null) putString("key", newKey) }
            .commit()
    }

    /** A setup link the wizard's QR opened. Gives the saved address, or null if it was not one. */
    fun applySetupLink(context: Context, link: String): String? {
        val (newBase, newKey) = parseSetupLink(link) ?: return null
        save(context, newBase, newKey)
        return newBase
    }

    /** Ask the garden where it is now; follow a signed move. True when the address changed. */
    suspend fun refresh(context: Context): Boolean = withContext(Dispatchers.IO) {
        val trusted = key ?: return@withContext false
        val json = runCatching {
            val conn = (URL("$base/app/config.json").openConnection() as HttpURLConnection).apply {
                connectTimeout = 15_000
                readTimeout = 15_000
                instanceFollowRedirects = true
            }
            try {
                if (conn.responseCode != 200) null else JSONObject(conn.inputStream.bufferedReader().use { it.readText() })
            } finally {
                conn.disconnect()
            }
        }.getOrNull() ?: return@withContext false
        val newBase = json.optString("base").trimEnd('/')
        val ok = newBase != base && validBase(newBase) &&
            verify(trusted, configMessage(newBase, json.optLong("issued")), json.optString("sig"))
        if (ok) save(context, newBase, null)
        ok
    }

    /** The exact text the garden signs. */
    fun configMessage(base: String, issued: Long) = "junsgarden-config-v1\n$base\n$issued"

    /** `junsgarden://setup?base=...&key=...` -> (address, key or null); null when not valid. */
    fun parseSetupLink(link: String): Pair<String, String?>? {
        val uri = runCatching { URI(link) }.getOrNull() ?: return null
        if (uri.scheme != "junsgarden" || uri.host != "setup") return null
        val params = (uri.rawQuery ?: "").split('&').mapNotNull {
            val (k, v) = it.split('=', limit = 2).takeIf { p -> p.size == 2 } ?: return@mapNotNull null
            k to URLDecoder.decode(v, "UTF-8")
        }.toMap()
        val b = params["base"]?.trimEnd('/') ?: return null
        if (!validBase(b)) return null
        val k = params["key"]?.lowercase()?.takeIf { it.length == 64 && it.all { c -> c in "0123456789abcdef" } }
        return b to k
    }

    /** An http(s) address with a host and nothing odd in it. */
    fun validBase(b: String): Boolean {
        val u = runCatching { URI(b) }.getOrNull() ?: return false
        return (u.scheme == "https" || u.scheme == "http") && !u.host.isNullOrBlank() &&
            u.rawQuery == null && u.rawFragment == null && b.none { it.isWhitespace() }
    }

    /**
     * Does `sigHex` sign `message` with the Ed25519 public key `keyHex`?
     *
     * Google Tink's Ed25519, not Android's own: Android's crypto refuses Ed25519 keys (tested on
     * an Android 16 emulator, where a correct signature came back "no"), while the desktop Java
     * that runs the unit tests accepts them. Tink behaves the same everywhere.
     */
    fun verify(keyHex: String, message: String, sigHex: String): Boolean = runCatching {
        Ed25519Verify(hex(keyHex)).verify(hex(sigHex), message.toByteArray(Charsets.UTF_8))
        true // verify() returns only when the signature is right; otherwise it throws
    }.getOrDefault(false)

    private fun hex(s: String): ByteArray {
        require(s.length % 2 == 0)
        return ByteArray(s.length / 2) { i -> s.substring(i * 2, i * 2 + 2).toInt(16).toByte() }
    }
}
