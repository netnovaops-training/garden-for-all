// AppText.kt -- the app's words, in the phone's language, for code that is not a screen.
//
// Screens read their words with stringResource(R.string.x). Code that is not a screen (the
// garden's error messages, the plant page's labels, the updater) uses AppText.s(R.string.x).
// Both read res/values/strings.xml (English) or res/values-ko/strings.xml (Korean), whichever
// Android picks for this app: the phone's language, or the one chosen for this app alone
// under Settings > Apps > Jun's Garden > Language (res/xml/locales_config.xml lists both).

package org.jasonagibson.junsgarden.ui

import android.content.Context
import androidx.annotation.StringRes

object AppText {
    @Volatile
    private var app: Context? = null

    /** Called once at start (MainActivity). The application context follows the app's language. */
    fun init(context: Context) {
        app = context.applicationContext
    }

    /** The line `id` in the app's language, with `args` filled into %1$s, %2$d, ... */
    fun s(@StringRes id: Int, vararg args: Any): String =
        app?.getString(id, *args) ?: ""
}
