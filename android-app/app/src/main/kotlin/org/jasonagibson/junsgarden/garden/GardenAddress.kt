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
// answer is signed with the garden's key (Ed25519, checked here with Android's own crypto).
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
import java.net.URLDecoder
import java.security.KeyFactory
import java.security.Signature
import java.security.spec.X509EncodedKeySpec

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
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
            .putString("base", newBase)
            .apply { if (newKey != null) putString("key", newKey) }
            .apply()
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

    /** Does `sigHex` sign `message` with the Ed25519 public key `keyHex`? */
    fun verify(keyHex: String, message: String, sigHex: String): Boolean = runCatching {
        // A raw Ed25519 key wrapped in the standard X.509 header, which Java's KeyFactory reads.
        val spki = hex("302a300506032b6570032100") + hex(keyHex)
        val pub = KeyFactory.getInstance("Ed25519").generatePublic(X509EncodedKeySpec(spki))
        Signature.getInstance("Ed25519").run {
            initVerify(pub)
            update(message.toByteArray(Charsets.UTF_8))
            verify(hex(sigHex))
        }
    }.getOrDefault(false)

    private fun hex(s: String): ByteArray {
        require(s.length % 2 == 0)
        return ByteArray(s.length / 2) { i -> s.substring(i * 2, i * 2 + 2).toInt(16).toByte() }
    }
}
