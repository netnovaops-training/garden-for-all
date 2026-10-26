// ScanReview.kt -- the parts of the scan screens that sit on top: the bed picker, the match review
// with its one-tap confirm, and "type it instead".

package org.jasonagibson.junsgarden.garden

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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import org.jasonagibson.junsgarden.R

/** Which bed? A big list; the bed used last is the first and biggest choice. */
@Composable
fun BedPicker(beds: List<ScanBed>, current: ScanBed?, onPick: (ScanBed) -> Unit, onClose: () -> Unit) {
    AlertDialog(
        onDismissRequest = onClose,
        title = { Text(stringResource(R.string.scan_bed_pick_title), style = MaterialTheme.typography.headlineSmall) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (current != null) {
                    Button(onClick = { onPick(current) }, Modifier.fillMaxWidth().height(60.dp)) {
                        Text(stringResource(R.string.scan_bed_same, current.name), style = MaterialTheme.typography.titleMedium)
                    }
                }
                if (beds.isEmpty()) {
                    Text(stringResource(R.string.scan_beds_failed), style = MaterialTheme.typography.bodyLarge)
                }
                LazyColumn(Modifier.height(340.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    items(beds, key = { it.id }) { b ->
                        OutlinedButton(onClick = { onPick(b) }, Modifier.fillMaxWidth().height(54.dp)) {
                            Text(b.name, style = MaterialTheme.typography.titleSmall, maxLines = 1)
                        }
                    }
                }
            }
        },
        confirmButton = {},
        dismissButton = { TextButton(onClick = onClose) { Text(stringResource(R.string.scan_cancel)) } },
    )
}

/** Type the plant in: the same boxes as the website's Add plant (genus, species, name, price). */
@Composable
fun TypeItDialog(
    initialName: String = "",
    withPrice: Boolean = true,
    onDismiss: () -> Unit,
    onAdd: (genus: String, species: String, name: String, price: String) -> Unit,
) {
    var genus by remember { mutableStateOf("") }
    var species by remember { mutableStateOf("") }
    var name by remember { mutableStateOf(initialName) }
    var price by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.scan_type_btn)) },
        text = {
            Column(Modifier.verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(genus, { genus = it }, label = { Text(stringResource(R.string.scan_genus)) }, singleLine = true)
                OutlinedTextField(species, { species = it }, label = { Text(stringResource(R.string.scan_species)) }, singleLine = true)
                OutlinedTextField(name, { name = it }, label = { Text(stringResource(R.string.scan_name)) }, singleLine = true)
                if (withPrice) {
                    OutlinedTextField(price, { price = it }, label = { Text(stringResource(R.string.scan_price)) }, singleLine = true)
                }
            }
        },
        confirmButton = {
            Button(
                onClick = { onAdd(genus.trim(), species.trim(), name.trim(), price.trim()) },
                enabled = genus.isNotBlank() && name.isNotBlank(),
            ) { Text(stringResource(R.string.scan_add)) }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(stringResource(R.string.scan_cancel)) } },
    )
}

/**
 * The matches for one scan: the best one first and the likely ones after it, each with how sure the
 * computer is. One tap on "This one" adds the plant (the website checks it is the garden owner
 * tapping). "Same plant" means the plant is already in the garden: its count and bed are updated.
 */
@Composable
fun ScanReview(model: ScanModel, e: ScanEntry, onClose: () -> Unit) {
    var typing by remember { mutableStateOf(false) }
    var pickBed by remember { mutableStateOf(false) }
    val added = e.status == ScanCore.ADDED
    Surface(Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                TextButton(onClick = onClose) { Text("‹  " + stringResource(R.string.scan_title)) }
            }
            Text(stringResource(R.string.scan_review_title), style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.Bold)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                e.photos.forEach { p ->
                    AsyncImage(
                        model = model.photo(e, p), contentDescription = null, contentScale = ContentScale.Crop,
                        modifier = Modifier.size(84.dp).clip(RoundedCornerShape(10.dp)),
                    )
                }
                Text("${model.bedName(e)} × ${e.qty}", Modifier.padding(start = 4.dp), style = MaterialTheme.typography.titleMedium)
            }
            model.message?.let {
                Text(it, Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.secondaryContainer).padding(12.dp))
            }
            Column(Modifier.weight(1f).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                if (e.candidates.isEmpty()) {
                    Text(stringResource(R.string.scan_none_found), style = MaterialTheme.typography.bodyLarge)
                }
                e.candidates.forEachIndexed { i, c ->
                    CandidateCard(c, best = i == 0 && e.status == ScanCore.MATCHED) {
                        // The phone shows the button; the website decides (owner only).
                        if (!added && model.owner) {
                            Button(onClick = { model.confirm(e, c) }, Modifier.fillMaxWidth().height(60.dp)) {
                                Text(
                                    stringResource(if (c.set == "current") R.string.scan_confirm_same else R.string.scan_confirm),
                                    style = MaterialTheme.typography.titleMedium,
                                )
                            }
                        }
                    }
                }
                if (!model.owner && !added) {
                    Text(stringResource(R.string.scan_owner_only), style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                if (!added && model.owner) {
                    OutlinedButton(onClick = { typing = true }, Modifier.fillMaxWidth().height(54.dp)) {
                        Text(stringResource(R.string.scan_none_of_these))
                    }
                }
            }
            if (!added) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilledTonalButton(onClick = { pickBed = true }, Modifier.weight(1f)) { Text(stringResource(R.string.scan_change_bed)) }
                    FilledTonalButton(onClick = { model.remove(e); onClose() }, Modifier.weight(1f)) { Text(stringResource(R.string.scan_remove)) }
                }
            }
        }
        if (typing) {
            TypeItDialog(
                initialName = e.candidates.firstOrNull()?.name.orEmpty(),
                withPrice = false,
                onDismiss = { typing = false },
                onAdd = { g, s, n, _ -> typing = false; model.confirmManual(e, g, s, n) },
            )
        }
        if (pickBed) {
            BedPicker(
                beds = model.beds, current = null,
                onPick = { b -> pickBed = false; model.changeBed(e, b) },
                onClose = { pickBed = false },
            )
        }
    }
}

@Composable
private fun CandidateCard(c: ScanCandidate, best: Boolean, action: @Composable () -> Unit) {
    Column(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(14.dp))
            .background(if (best) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant)
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text(
            stringResource(if (best) R.string.scan_best else R.string.scan_likely),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(c.name, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
        if (c.botanical.isNotBlank()) {
            Text(c.botanical, style = MaterialTheme.typography.bodyMedium, fontStyle = androidx.compose.ui.text.font.FontStyle.Italic)
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            LinearProgressIndicator(progress = { c.confidence.toFloat().coerceIn(0f, 1f) }, Modifier.weight(1f).height(10.dp))
            Text(stringResource(R.string.scan_conf, c.percent), style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
        }
        Text(
            stringResource(
                when (c.source) {
                    "garden" -> R.string.scan_src_garden
                    "wishlist" -> R.string.scan_src_wishlist
                    "image" -> R.string.scan_src_image
                    "web" -> R.string.scan_src_web
                    else -> R.string.scan_src_tag
                }
            ) + if (c.why.isNotBlank()) " · ${c.why}" else "",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(2.dp))
        action()
    }
}
