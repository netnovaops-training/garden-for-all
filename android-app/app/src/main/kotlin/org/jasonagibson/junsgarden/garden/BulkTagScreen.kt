// BulkTagScreen.kt -- the "Many at once" tag screen (logged in only).
//
// Take photos of many plant tags (front, then back), check how they are
// paired, then send them all at once to the website's queue
// (GARDEN_BASE/app/garden/tag-queue). They are read later, not on the phone.

package org.jasonagibson.junsgarden.garden

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.launch

/**
 * A pile of tag photographs, turned into a queue, in one go.
 *
 * THE AUTHOR, 2026-09-27: "there needs to be a bulk tag adder so that front and
 * back can be added to the picture so that if someone wants to add 20 or 30
 * tags they can at once front and back without taking one picture at a
 * time."
 *
 * THE UNIT OF WORK IS A TAG, NOT A PHOTOGRAPH, and that is the whole design
 * problem. Thirty photographs are fifteen tags, and nothing in the pictures
 * says where one tag ends and the next begins. Guessing wrongly pairs the
 * back of one plant with the front of another and produces a record that
 * looks complete and is wrong about half its fields -- far worse than
 * failing.
 *
 * SO THE PAIRING IS STATED, NOT INFERRED. Two modes, both of which match how
 * somebody actually works through a tray of plants:
 *
 *   FRONT THEN BACK -- photograph each tag twice, in order. The ordinary
 *   case, and the default.
 *
 *   FRONTS ONLY -- every photograph is its own tag with no back. For tags
 *   still stuck in pots, which cannot be turned over.
 *
 * And the pairing is SHOWN before anything is sent: each card says "Tag 3
 * front" or "Tag 3 back", so a mistake is visible while it is still cheap.
 * An odd photograph at the end of Front-then-back is called out rather than
 * silently paired with nothing.
 *
 * NOTHING IS READ HERE. The photographs are queued and the phone is
 * released. Reading thirty tags can take many minutes on the server, and no
 * phone holds a request open that long.
 */
@Composable
fun BulkTagScreen(
    camera: GardenCamera,
    bottomInset: androidx.compose.ui.unit.Dp,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    val picked = remember { mutableStateListOf<Uri>() }
    var pairMode by remember { mutableStateOf(PairMode.FrontThenBack) }
    var busy by remember { mutableStateOf(false) }
    var done by remember { mutableIntStateOf(0) }
    var result by remember { mutableStateOf<GardenCamera.QueueResult?>(null) }
    var failure by remember { mutableStateOf<String?>(null) }

    // 100 is the server's own ceiling, so the picker refuses past it rather
    // than letting somebody choose 120 and be told afterwards.
    val picker = rememberLauncherForActivityResult(
        ActivityResultContracts.PickMultipleVisualMedia(MAX_PHOTOS)
    ) { uris ->
        if (uris.isNotEmpty()) {
            // ADDED, NOT REPLACED. Picking again is usually "and these too",
            // and losing the first forty to a second trip to the gallery is
            // the kind of thing nobody forgives.
            picked.addAll(uris.filter { it !in picked })
            result = null
            failure = null
        }
    }

    val pairs = remember(picked.toList(), pairMode) { pairUp(picked.toList(), pairMode) }
    val oddOne = pairMode == PairMode.FrontThenBack && picked.size % 2 == 1

    Column(Modifier.fillMaxSize()) {
        Row(
            Modifier.fillMaxWidth().padding(start = 4.dp, end = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            TextButton(onClick = onBack) { Text("‹  Garden") }
            Spacer(Modifier.weight(1f))
            if (picked.isNotEmpty() && !busy) {
                TextButton(onClick = { picked.clear(); result = null }) {
                    Text("Clear")
                }
            }
        }

        Column(
            Modifier.fillMaxWidth().padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                "Many tags at once",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
            )
            Text(
                "Photograph every tag first, then pick them all here. " +
                    "They are queued and read afterwards, so you can put " +
                    "the phone down.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ModeChip(
                    "Front then back",
                    pairMode == PairMode.FrontThenBack,
                    enabled = !busy,
                ) { pairMode = PairMode.FrontThenBack }
                ModeChip(
                    "Fronts only",
                    pairMode == PairMode.FrontsOnly,
                    enabled = !busy,
                ) { pairMode = PairMode.FrontsOnly }
            }

            OutlinedButton(
                onClick = {
                    picker.launch(
                        PickVisualMediaRequest(
                            ActivityResultContracts.PickVisualMedia.ImageOnly
                        )
                    )
                },
                enabled = !busy,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(if (picked.isEmpty()) "Pick photos" else "Add more photos")
            }

            if (picked.isNotEmpty()) {
                Text(
                    "${picked.size} photos · ${pairs.size} tags",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                )
            }
            if (oddOne) {
                // SAID PLAINLY, NOT FIXED QUIETLY. An odd photograph in
                // front-then-back mode means either one was missed or the
                // last tag genuinely has no back. Both are fine; guessing
                // which is not.
                Text(
                    "There is an odd photo at the end — it will be sent as a " +
                        "front with no back.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }

            result?.let { r ->
                Column(
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(12.dp))
                        .background(MaterialTheme.colorScheme.primaryContainer)
                        .padding(12.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                ) {
                    Text(
                        "${r.queued} tags queued",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                    // Every number that is not zero gets said. A duplicate
                    // is recorded rather than refused, and somebody should
                    // be able to see that it happened.
                    if (r.duplicates > 0) Text(
                        "${r.duplicates} were already sent before",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                    if (r.unreadable > 0) Text(
                        "${r.unreadable} could not be read as photographs",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                    Text(
                        "${r.waiting} waiting to be read. They appear in the " +
                            "catalogue for approval as each one finishes.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onPrimaryContainer,
                    )
                }
            }
            failure?.let {
                Text(
                    it,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.error,
                )
            }

            if (busy) {
                Column {
                    LinearProgressIndicator(
                        progress = {
                            if (pairs.isEmpty()) 0f
                            else done.toFloat() / pairs.size.toFloat()
                        },
                        modifier = Modifier.fillMaxWidth(),
                    )
                    Text(
                        "Sending $done of ${pairs.size}…",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(top = 4.dp),
                    )
                }
            }

            Button(
                onClick = {
                    busy = true; done = 0; failure = null; result = null
                    scope.launch {
                        runCatching {
                            camera.queueTags(context, pairs) { d, _ -> done = d }
                        }.onSuccess {
                            result = it
                            picked.clear()
                        }.onFailure {
                            failure = it.message ?: "Could not send those tags."
                        }
                        busy = false
                    }
                },
                enabled = !busy && pairs.isNotEmpty(),
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(
                    if (pairs.isEmpty()) "Nothing to send"
                    else "Queue ${pairs.size} tags"
                )
            }
        }

        Spacer(Modifier.height(10.dp))

        // WHAT IS PAIRED WITH WHAT, before it is sent. The thumbnails are
        // the only chance to notice that photograph 14 was a blurry retake
        // and everything after it is paired one out.
        LazyVerticalGrid(
            columns = GridCells.Adaptive(96.dp),
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(
                start = 16.dp, end = 16.dp, bottom = bottomInset,
            ),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            items(picked.toList()) { uri ->
                val i = picked.indexOf(uri)
                val label = labelFor(i, pairMode)
                Column {
                    Box {
                        AsyncImage(
                            model = uri,
                            contentDescription = label,
                            contentScale = ContentScale.Crop,
                            modifier = Modifier
                                .fillMaxWidth()
                                .aspectRatio(1f)
                                .clip(RoundedCornerShape(10.dp))
                                .background(MaterialTheme.colorScheme.surfaceVariant)
                                .border(
                                    1.dp,
                                    MaterialTheme.colorScheme.outline,
                                    RoundedCornerShape(10.dp),
                                )
                                .clickable(enabled = !busy) { picked.remove(uri) },
                        )
                    }
                    Text(
                        label,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(top = 2.dp),
                    )
                }
            }
        }
    }
}

private const val MAX_PHOTOS = 100

enum class PairMode { FrontThenBack, FrontsOnly }

/** "Tag 3 front" / "Tag 3 back", so a mis-pairing is visible before sending. */
private fun labelFor(index: Int, mode: PairMode): String = when (mode) {
    PairMode.FrontsOnly -> "Tag ${index + 1}"
    PairMode.FrontThenBack ->
        "Tag ${index / 2 + 1} " + if (index % 2 == 0) "front" else "back"
}

/**
 * Photographs into tags.
 *
 * The odd trailing photograph in FrontThenBack becomes a front with no
 * back rather than being dropped. Dropping it would lose a plant silently,
 * and the screen has already said out loud that this is what will happen.
 */
fun pairUp(photos: List<Uri>, mode: PairMode): List<GardenCamera.TagPair> =
    when (mode) {
        PairMode.FrontsOnly -> photos.map { GardenCamera.TagPair(it, null) }
        PairMode.FrontThenBack -> photos.chunked(2).map { pair ->
            GardenCamera.TagPair(pair[0], pair.getOrNull(1))
        }
    }

@Composable
private fun ModeChip(
    label: String,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Row(
        Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(
                if (selected) MaterialTheme.colorScheme.primary
                else MaterialTheme.colorScheme.surfaceVariant
            )
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 8.dp),
    ) {
        Text(
            label,
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.SemiBold,
            color = if (selected) MaterialTheme.colorScheme.onPrimary
            else MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
