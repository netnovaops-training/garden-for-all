package org.jasonagibson.junsgarden

// Temporary file to prove that the detekt check reports a new finding. Never used; removed again right after.
internal fun detektProbe(text: String): Int {
    try {
        return text.toInt()
    } catch (e: NumberFormatException) {
    }
    return 0
}
