// Shared.kt -- holds text or a link that another app "shared" into this app.
//
// LEFTOVER. It was made for the old "Ask" chat, which has been removed.
// Today a shared link only opens an empty panel in GardenScreen. It is
// harmless; see the README, "Known leftovers", if you want to tidy it away.

package org.jasonagibson.junsgarden

import android.content.Intent
import androidx.compose.runtime.mutableStateOf

/**
 * What another app shared into this one.
 *
 * THE AUTHOR, 2026-09-24: "if I wanted to send a URL video from YouTube straight
 * to the app ... so that I don't have to copy and paste."
 *
 * HELD HERE RATHER THAN PASSED DOWN, and the reason is `launchMode`. The
 * activity is singleTask, so a share into an already-running app does NOT
 * go through onCreate -- it arrives at onNewIntent, with the composition
 * already built and no constructor to pass anything to. A single observable
 * slot is read by whichever screen is on top, whenever it turns up.
 *
 * CONSUMED, NOT JUST READ. `take()` clears the slot, so a shared link is
 * sent once. Left in place it would be re-sent on every recomposition --
 * every rotation, every keyboard open -- and the same video would be
 * imported a dozen times.
 */
object Shared {
    private val slot = mutableStateOf<String?>(null)

    /** The pending shared text, if any. Reading it in a composable makes
     *  that composable recompose when something arrives. */
    val pending get() = slot.value

    fun offer(intent: Intent?) {
        if (intent == null) return
        if (intent.action != Intent.ACTION_SEND) return
        if (intent.type?.startsWith("text/") != true) return
        val text = intent.getStringExtra(Intent.EXTRA_TEXT)?.trim()
        if (text.isNullOrEmpty()) return

        // YouTube shares "Title\nhttps://youtu.be/..." on some versions and
        // the bare URL on others. Both are kept whole: the assistant works
        // out what to do from the content, and the title is useful context
        // rather than noise to be stripped.
        slot.value = text

        // The subject is sometimes the video's title when the body is not.
        val subject = intent.getStringExtra(Intent.EXTRA_SUBJECT)?.trim()
        if (!subject.isNullOrEmpty() && !text.contains(subject)) {
            slot.value = "$subject\n$text"
        }
    }

    /** Take it once. Returns null when there is nothing waiting. */
    fun take(): String? {
        val v = slot.value
        slot.value = null
        return v
    }
}
