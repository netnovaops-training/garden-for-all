// ScanScreen.kt -- "Scan plants": photograph a tag (front and back) or a plant, pick the bed, and go
// straight on to the next one. Big buttons, one thing per screen.
//
// The scans are kept on the phone (ScanStore) and sent to the garden website (ScanSync); the
// computer at home reads them (tag text, the garden's plants, the internet) and sends back a best
// match and a few likely ones with how sure it is. The status of every scan is shown here, and a
// tap on one that has matches opens ScanReview.kt for the one-tap confirm.
//
// Still no camera permission: each photo is taken by the phone's own camera app (TakePicture),
// exactly as for Identify and Tag.

package org.jasonagibson.junsgarden.garden

import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import org.jasonagibson.junsgarden.R

/** What the next photo from the camera app is for. */
private enum class Shot { TagFront, TagBack, Plant }

@Composable
fun ScanScreen(camera: GardenCamera, bottomInset: Dp, onBack: () -> Unit) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val model = remember { ScanModel(context, camera, scope, GardenLogin(context)) }

    var shot by remember { mutableStateOf<Shot?>(null) }
    var target by remember { mutableStateOf<Uri?>(null) }
    var front by remember { mutableStateOf<Uri?>(null) }
    var askBack by remember { mutableStateOf(false) }
    var pickBed by remember { mutableStateOf(false) }
    var afterBed by remember { mutableStateOf<(() -> Unit)?>(null) }
    var typing by remember { mutableStateOf(false) }
    var reviewUid by remember { mutableStateOf<String?>(null) }

    val shutter = rememberLauncherForActivityResult(ActivityResultContracts.TakePicture()) { ok ->
        val what = shot
        val uri = target
        shot = null
        // false = backed out of the camera app: nothing to say.
        if (!ok || what == null || uri == null) return@rememberLauncherForActivityResult
        when (what) {
            Shot.TagFront -> { front = uri; askBack = true }
            Shot.TagBack -> { model.add("tag", listOfNotNull(front, uri)); front = null }
            Shot.Plant -> model.add("plant", listOf(uri))
        }
    }

    fun shoot(what: Shot) {
        shot = what
        val where = newPhotoTarget(context)
        target = where
        shutter.launch(where)
    }

    /** A scan needs a bed: ask once (it is then kept), and carry on with what was wanted. */
    fun withBed(go: () -> Unit) {
        if (model.bed != null) go() else { afterBed = go; pickBed = true }
    }

    // The website's list now and then while this screen is open (often while anything waits).
    LaunchedEffect(Unit) {
        model.loadBeds()
        while (isActive) {
            runCatching { model.sync() }
            delay(if (ScanCore.busy(model.entries)) 8_000 else 30_000)
        }
    }

    BackHandler(enabled = askBack) { askBack = false; front = null }
    BackHandler(enabled = reviewUid != null) { reviewUid = null }

    val review = model.entries.firstOrNull { it.uid == reviewUid }
    val summary = ScanCore.summary(model.entries)

    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize(),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(start = 16.dp, end = 16.dp, bottom = bottomInset),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    TextButton(onClick = onBack) { Text(stringResource(R.string.back_garden)) }
                }
                Text(
                    stringResource(R.string.scan_title),
                    style = MaterialTheme.typography.headlineMedium,
                    fontWeight = FontWeight.Bold,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    stringResource(R.string.scan_intro),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            item {
                model.message?.let {
                    Text(
                        it,
                        Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp))
                            .background(MaterialTheme.colorScheme.secondaryContainer)
                            .clickable { model.message = null }.padding(14.dp),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                    )
                }
            }
            item {
                // THE BED IS PICKED FIRST AND STAYS PICKED: the next scan is one tap, and the
                // chip says plainly where it will go.
                OutlinedButton(
                    onClick = { afterBed = null; pickBed = true },
                    Modifier.fillMaxWidth().height(60.dp),
                ) {
                    Text(
                        model.bed?.let { stringResource(R.string.scan_bed_label, it.name) }
                            ?: stringResource(R.string.scan_bed_none),
                        style = MaterialTheme.typography.titleMedium,
                    )
                }
            }
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    FilledTonalButton(
                        onClick = { if (model.qty > 1) model.qty -= 1 },
                        Modifier.size(56.dp),
                    ) { Text("−", style = MaterialTheme.typography.headlineSmall) }
                    Text(
                        stringResource(R.string.scan_qty, model.qty),
                        Modifier.weight(1f),
                        style = MaterialTheme.typography.titleLarge,
                        textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                    )
                    FilledTonalButton(
                        onClick = { if (model.qty < 99) model.qty += 1 },
                        Modifier.size(56.dp),
                    ) { Text("+", style = MaterialTheme.typography.headlineSmall) }
                }
            }
            item {
                Button(
                    onClick = { withBed { shoot(Shot.TagFront) } },
                    enabled = !model.saving,
                    modifier = Modifier.fillMaxWidth().height(76.dp),
                ) {
                    Text(
                        if (model.saving) stringResource(R.string.scan_saving) else stringResource(R.string.scan_tag_btn),
                        style = MaterialTheme.typography.titleLarge,
                    )
                }
            }
            item {
                FilledTonalButton(
                    onClick = { withBed { shoot(Shot.Plant) } },
                    enabled = !model.saving,
                    modifier = Modifier.fillMaxWidth().height(64.dp),
                ) { Text(stringResource(R.string.scan_plant_btn), style = MaterialTheme.typography.titleMedium) }
                TextButton(onClick = { withBed { typing = true } }, Modifier.fillMaxWidth()) {
                    Text(stringResource(R.string.scan_type_btn), style = MaterialTheme.typography.titleSmall)
                }
            }
            item {
                if (model.entries.isEmpty()) {
                    Text(stringResource(R.string.scan_empty), style = MaterialTheme.typography.bodyLarge)
                } else {
                    Text(
                        stringResource(R.string.scan_summary, summary.onPhone, summary.atPc, summary.needYou, summary.added),
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
                if (model.offline) {
                    Text(
                        stringResource(R.string.scan_offline),
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.error,
                    )
                } else if (summary.atPc > 0) {
                    val seen = model.helperSeen
                    Text(
                        if (seen.isNullOrBlank()) stringResource(R.string.scan_pc_never) else stringResource(R.string.scan_pc_seen, seen),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
            items(model.entries, key = { it.uid }) { e ->
                ScanCard(model, e, onOpen = { reviewUid = e.uid })
            }
        }

        if (askBack) {
            BackSidePrompt(
                onTakeBack = { askBack = false; shoot(Shot.TagBack) },
                onSkip = {
                    askBack = false
                    front?.let { model.add("tag", listOf(it)) }
                    front = null
                },
            )
        }
        if (pickBed) {
            BedPicker(
                beds = model.beds,
                current = model.bed,
                onPick = { b ->
                    model.chooseBed(b)
                    pickBed = false
                    afterBed?.invoke()
                    afterBed = null
                },
                onClose = { pickBed = false; afterBed = null },
            )
        }
        if (typing) {
            TypeItDialog(
                onDismiss = { typing = false },
                onAdd = { g, s, n, p -> typing = false; model.typeIn(g, s, n, p) },
            )
        }
        if (review != null) {
            ScanReview(model, review, onClose = { reviewUid = null })
        }
    }
}

/** One scan in the list: its picture, where it goes, how it is doing. */
@Composable
private fun ScanCard(model: ScanModel, e: ScanEntry, onOpen: () -> Unit) {
    val open = e.candidates.isNotEmpty() || e.status == ScanCore.NEEDS_YOU || e.status == ScanCore.MATCHED
    Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(14.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .clickable(enabled = open, onClick = onOpen).padding(10.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        e.photos.firstOrNull()?.let { p ->
            AsyncImage(
                model = model.photo(e, p),
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.size(76.dp).clip(RoundedCornerShape(10.dp)),
            )
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            val best = e.candidates.firstOrNull()
            Text(
                if (best != null && e.status != ScanCore.SAVED) best.name
                else stringResource(if (e.kind == "plant") R.string.scan_word_plant else R.string.scan_word_tag),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                maxLines = 2,
            )
            Text(
                "${model.bedName(e)} × ${e.qty}",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            StatusChip(e)
        }
    }
}

@Composable
private fun StatusChip(e: ScanEntry) {
    val (words, color) = when (e.status) {
        ScanCore.WAITING -> stringResource(R.string.scan_status_waiting) to MaterialTheme.colorScheme.surface
        ScanCore.READING -> stringResource(R.string.scan_status_reading) to MaterialTheme.colorScheme.secondaryContainer
        ScanCore.MATCHED -> stringResource(R.string.scan_status_matched) to MaterialTheme.colorScheme.primaryContainer
        ScanCore.NEEDS_YOU -> stringResource(R.string.scan_status_needs_you) to MaterialTheme.colorScheme.tertiaryContainer
        ScanCore.ADDED -> stringResource(R.string.scan_status_added) to MaterialTheme.colorScheme.primaryContainer
        ScanCore.FAILED -> stringResource(R.string.scan_status_failed) to MaterialTheme.colorScheme.errorContainer
        else -> stringResource(R.string.scan_status_saved) to MaterialTheme.colorScheme.surface
    }
    Text(
        words,
        Modifier.clip(RoundedCornerShape(999.dp)).background(color).padding(horizontal = 12.dp, vertical = 4.dp),
        style = MaterialTheme.typography.labelLarge,
        color = Color.Unspecified,
    )
}
