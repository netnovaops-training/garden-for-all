// PlantDetailScreen.kt -- one plant, with its photo and everything recorded
// about it, grouped into cards the same way the website's catalogue page
// does (the grouping lives in PlantSections.kt). Data: GardenApi.detail.

package org.jasonagibson.junsgarden.garden

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.launch
import org.jasonagibson.junsgarden.R
import org.jasonagibson.junsgarden.ui.AppText

/**
 * One plant, laid out the way catalog.php lays it out.
 *
 * THE AUTHOR, 2026-09-27: "Can you make Jun's app display the plants they do in
 * junsgarden.zapto.org/catalog.php that match with the theme app."
 *
 * TWO REQUIREMENTS IN ONE SENTENCE, PULLING OPPOSITE WAYS. The STRUCTURE
 * comes from the website -- its header, its chips, its five grouped cards,
 * its notes, in its order -- because a record shown two different ways is
 * two different records, and whichever somebody saw last is the one they
 * believe. The COLOUR does not: every surface here is a theme token, so the
 * page follows whichever theme is chosen in the app rather than the
 * website's fixed blue. Copying the blue would have been easier and would
 * have left one screen that quietly ignores the theme picker.
 *
 * SAFETY STAYS AT THE TOP. Toxic-or-edible sits under the photograph, above
 * the cards, where it already was. The website puts it inside Wildlife &
 * Safety near the bottom; on a phone the question is usually being asked
 * with the plant already in somebody's hand or a dog already next to it, so
 * it does not move down to match.
 *
 * NOTHING IS DROPPED. The five cards claim about sixty columns of the 101.
 * Whatever they do not claim is still printed under "More details", because
 * a column nobody has got round to placing should not vanish from the app
 * that exists to show the record.
 */
@Composable
fun PlantDetailScreen(
    summary: PlantSummary,
    bottomInset: androidx.compose.ui.unit.Dp,
    onBack: () -> Unit,
) {
    var detail by remember(summary.id) { mutableStateOf<PlantDetail?>(null) }
    var failed by remember(summary.id) { mutableStateOf<String?>(null) }

    LaunchedEffect(summary.id) {
        failed = null
        runCatching { GardenApi.detail(summary.id) }
            .onSuccess { detail = it }
            .onFailure { failed = it.message ?: AppText.s(R.string.could_not_load_plant) }
    }

    // Lowercased once, because every lookup below is by column name and the
    // server has not always been consistent about case.
    val byKey = remember(detail) {
        detail?.fields?.associate { it.first.lowercase() to it.second }.orEmpty()
    }

    Column(Modifier.fillMaxSize()) {
        Row(
            Modifier.fillMaxWidth().padding(start = 4.dp, end = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            TextButton(onClick = onBack) { Text(stringResource(R.string.back_catalog)) }
        }

        LazyColumn(
            Modifier.fillMaxSize(),
            contentPadding = PaddingValues(
                start = 16.dp, end = 16.dp, bottom = bottomInset,
            ),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            // ------------------------------------------------- the header
            item {
                Column(
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(16.dp))
                        .background(MaterialTheme.colorScheme.primary)
                        .padding(14.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                ) {
                    Text(
                        summary.name,
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onPrimary,
                    )
                    if (summary.botanical.isNotBlank()) {
                        Text(
                            summary.botanical,
                            style = MaterialTheme.typography.titleMedium,
                            fontStyle = FontStyle.Italic,
                            color = MaterialTheme.colorScheme.onPrimary,
                        )
                    }
                    val by = byKey["company"].orEmpty()
                    if (by.isNotBlank()) {
                        Text(
                            stringResource(R.string.by_company, by),
                            style = MaterialTheme.typography.titleSmall,
                            fontWeight = FontWeight.SemiBold,
                            color = MaterialTheme.colorScheme.onPrimary,
                        )
                    }
                    val taxon = taxonomyLine(byKey)
                    if (taxon.isNotBlank()) {
                        Text(
                            "🌿 $taxon",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onPrimary,
                        )
                    }
                }
            }

            // ------------------------------------------------- the photos
            item {
                val shots = detail?.photos?.takeIf { it.isNotEmpty() }
                    ?: listOfNotNull(summary.photo)
                if (shots.isNotEmpty()) {
                    val pager = rememberPagerState(pageCount = { shots.size })
                    val scope = rememberCoroutineScope()
                    Column {
                        HorizontalPager(
                            state = pager,
                            modifier = Modifier.fillMaxWidth().aspectRatio(4f / 3f),
                            pageSpacing = 8.dp,
                        ) { page ->
                            AsyncImage(
                                model = shots[page],
                                contentDescription = summary.name,
                                modifier = Modifier
                                    .fillMaxSize()
                                    .clip(RoundedCornerShape(16.dp))
                                    .background(MaterialTheme.colorScheme.surfaceVariant),
                                contentScale = ContentScale.Crop,
                            )
                        }
                        // THUMBNAILS, LIKE THE WEBSITE. The swipe still
                        // works; this adds what a swipe cannot say -- how
                        // many there are and what is in them. Tapping jumps,
                        // because reaching the last of seven photographs by
                        // swiping is six gestures.
                        if (shots.size > 1) {
                            Row(
                                Modifier
                                    .fillMaxWidth()
                                    .horizontalScroll(rememberScrollState())
                                    .padding(top = 8.dp),
                                horizontalArrangement = Arrangement.spacedBy(6.dp),
                            ) {
                                shots.forEachIndexed { i, url ->
                                    val active = i == pager.currentPage
                                    AsyncImage(
                                        model = url,
                                        contentDescription = null,
                                        contentScale = ContentScale.Crop,
                                        modifier = Modifier
                                            .size(54.dp)
                                            .clip(RoundedCornerShape(8.dp))
                                            .background(
                                                MaterialTheme.colorScheme.surfaceVariant
                                            )
                                            .border(
                                                width = if (active) 2.dp else 0.dp,
                                                color = if (active)
                                                    MaterialTheme.colorScheme.primary
                                                else
                                                    MaterialTheme.colorScheme.surfaceVariant,
                                                shape = RoundedCornerShape(8.dp),
                                            )
                                            .clickable {
                                                scope.launch {
                                                    pager.animateScrollToPage(i)
                                                }
                                            },
                                    )
                                }
                            }
                        }
                    }
                }
            }

            // -------------------------------------------- safety + chips
            item {
                SafetyLine(summary.toxicity, summary.edibility)
                Spacer(Modifier.height(8.dp))
                val chips = plantChips(byKey).ifEmpty {
                    // Before the detail call lands the list row already knows
                    // enough for a few chips, so the space does not sit empty
                    // and then jump when the record arrives.
                    listOfNotNull(
                        summary.type.takeIf { it.isNotBlank() }?.let { "🌿" to it },
                        summary.zones.takeIf { it.isNotBlank() }?.let { "🌡️" to it },
                        summary.sun.takeIf { it.isNotBlank() }?.let { "☀️" to it },
                        summary.water.takeIf { it.isNotBlank() }?.let { "💧" to it },
                    )
                }
                ChipFlow(chips)
            }

            // ----------------------------------------------- the seasons
            val grow = monthRange(byKey["grow_months"])
            val bloom = monthRange(byKey["bloom_months"])
            if (grow.isNotBlank() || bloom.isNotBlank()) {
                item {
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        if (grow.isNotBlank()) Banner("🌱", stringResource(R.string.plant_grow), grow)
                        if (bloom.isNotBlank()) Banner("🌸", stringResource(R.string.bloom), bloom)
                    }
                }
            }

            when {
                failed != null -> item {
                    Text(failed!!, style = MaterialTheme.typography.bodyMedium)
                }

                detail == null -> item {
                    Box(
                        Modifier.fillMaxWidth().padding(24.dp),
                        contentAlignment = Alignment.Center,
                    ) { CircularProgressIndicator() }
                }

                else -> {
                    val notes = plantNotes(byKey)
                    if (notes.isNotEmpty()) {
                        item { SectionCard("📝", stringResource(R.string.notes)) { NotesBody(notes) } }
                    }

                    plantSections(byKey).forEach { section ->
                        item {
                            SectionCard(section.icon, section.title) {
                                section.rows.forEach { (k, v) -> FieldRow(k, v) }
                            }
                        }
                    }

                    val rest = plantLeftovers(detail!!.fields)
                    if (rest.isNotEmpty()) {
                        item {
                            SectionCard("📋", stringResource(R.string.more_details)) {
                                rest.forEach { (k, v) -> FieldRow(prettify(k), v) }
                            }
                        }
                    }
                }
            }
        }
    }
}

/** A titled card, coloured by the chosen theme rather than the website's blue. */
@Composable
private fun SectionCard(
    icon: String,
    title: String,
    body: @Composable () -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant),
    ) {
        Row(
            Modifier
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.primary)
                .padding(horizontal = 12.dp, vertical = 9.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("$icon  ", style = MaterialTheme.typography.titleSmall)
            Text(
                title,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onPrimary,
            )
        }
        Column(
            Modifier.padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(7.dp),
        ) { body() }
    }
}

@Composable
private fun NotesBody(notes: List<Pair<String, String>>) {
    notes.forEachIndexed { i, (heading, text) ->
        if (i > 0) Spacer(Modifier.height(2.dp))
        Text(
            heading,
            style = MaterialTheme.typography.titleSmall,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.primary,
        )
        Text(text, style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun Banner(icon: String, label: String, value: String) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(MaterialTheme.colorScheme.primaryContainer)
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text("$icon  ")
        Text(
            label,
            style = MaterialTheme.typography.titleSmall,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onPrimaryContainer,
        )
        Spacer(Modifier.width(8.dp))
        Text(
            value,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onPrimaryContainer,
        )
    }
}

/**
 * The chip row, packed as tightly as the width allows.
 *
 * THE AUTHOR, 2026-09-27: "The icons can be more densly packed."
 *
 * TWO PER LINE WAS THE BUG. Chipping them in fixed pairs meant "Tree" and
 * "Moderate" -- eleven characters between them -- took a whole line and left
 * half of it empty, while "Planted 2021, 2023" had to share with "Qty 3".
 * The width of a chip has nothing to do with its neighbour, so a fixed count
 * per row is always wrong in one direction or the other.
 *
 * FlowRow measures each one and wraps when it genuinely runs out of room.
 * It is still experimental in this Compose version, which costs one opt-in
 * here and nothing at the call site.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ChipFlow(chips: List<Pair<String, String>>) {
    if (chips.isEmpty()) return
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        chips.forEach { (icon, label) -> Chip(icon, label) }
    }
}

@Composable
private fun Chip(icon: String, label: String) {
    Row(
        Modifier
            .clip(RoundedCornerShape(999.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(horizontal = 9.dp, vertical = 5.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text("$icon ", style = MaterialTheme.typography.labelMedium)
        Text(
            label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.primary,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

@Composable
private fun FieldRow(label: String, value: String) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.Top) {
        Text(
            label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.fillMaxWidth(0.42f),
        )
        Text(
            value,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

/**
 * A column name, as a person would say it.
 *
 * `soil_ph` is not a label, it is a database identifier, and showing it as
 * one is how a page ends up looking like a table dump instead of a record
 * about a plant.
 */
private fun prettify(key: String): String = key
    .replace('_', ' ')
    .replaceFirstChar { it.uppercase() }
    .replace(" ph", " pH")
    .replace("Qty", "How many")
