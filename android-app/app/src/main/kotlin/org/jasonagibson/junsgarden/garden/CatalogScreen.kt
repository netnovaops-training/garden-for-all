// CatalogScreen.kt -- the list of all the plants (the catalogue).
//
// A search box, a bed filter, a "Theme" button, and plant cards with photos.
// Plants arrive 20 at a time from the website (GardenApi.list); more load
// as you scroll down. Tapping a plant opens PlantDetailScreen.

package org.jasonagibson.junsgarden.garden

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.drop

/**
 * The plant catalogue, native, the way it was before the app became a window
 * onto the website.
 *
 * PAGED AS THE SERVER PAGES IT. The list endpoint hands back twenty at a
 * time with a hasMore flag, and the next page is fetched when the bottom
 * comes into view. Four hundred and thirty-seven plants is too many to hold
 * a scroll position through if they all arrive at once, and far too many to
 * wait for on a phone in a garden.
 *
 * SEARCH IS THE SERVER'S, NOT A FILTER OVER WHAT IS LOADED. Filtering the
 * current page would search twenty plants and look like the other four
 * hundred do not exist -- the failure that is impossible to tell apart from
 * "we don't have that one".
 */
@OptIn(FlowPreview::class)
@Composable
fun CatalogScreen(
    bottomInset: androidx.compose.ui.unit.Dp,
    onAppearance: () -> Unit,
    onOpen: (PlantSummary) -> Unit,
) {
    var search by remember { mutableStateOf("") }
    var bed by remember { mutableStateOf("") }
    var beds by remember { mutableStateOf<List<Bed>>(emptyList()) }

    var plants by remember { mutableStateOf<List<PlantSummary>>(emptyList()) }
    var page by remember { mutableStateOf(1) }
    var hasMore by remember { mutableStateOf(true) }
    var loading by remember { mutableStateOf(true) }
    var failed by remember { mutableStateOf<String?>(null) }
    // Bumped to force a reload after a failure, without touching the query.
    var attempt by remember { mutableStateOf(0) }

    val listState = rememberLazyListState()

    LaunchedEffect(Unit) {
        // Best effort: the beds are a convenience, and a catalogue that works
        // without its filter row beats a screen showing nothing because one
        // secondary call failed.
        beds = runCatching { GardenApi.beds() }.getOrDefault(emptyList())
    }

    // THE SEARCH WAITS FOR HIM TO STOP TYPING. Without the debounce, "hosta"
    // is five queries whose answers can land out of order, so the list
    // settles on whichever request was slowest rather than the last one typed.
    //
    // drop(1): the box's STARTING value is not typing. Counting it bumped
    // `attempt` 350ms after the screen opened, which restarted the first
    // page load mid-flight; on a slow connection the screen then showed
    // "The coroutine scope left the composition" instead of the plants.
    LaunchedEffect(Unit) {
        snapshotFlow { search }
            .drop(1)
            .debounce(350)
            .distinctUntilChanged()
            .collect { attempt++ }
    }

    // First page, and every time the query changes.
    LaunchedEffect(bed, attempt) {
        loading = true
        failed = null
        page = 1
        runCatching { GardenApi.list(1, search, bed) }
            .onSuccess {
                plants = it.plants
                hasMore = it.hasMore
                listState.scrollToItem(0)
            }
            .onFailure {
                // A load cancelled because a newer one replaced it is not a
                // failure to show anybody: let the cancellation through.
                if (it is kotlinx.coroutines.CancellationException) throw it
                failed = it.message ?: "Could not reach the garden."
            }
        loading = false
    }

    // The next page, when the end of the list comes into view.
    LaunchedEffect(listState, plants.size, hasMore) {
        snapshotFlow { listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index }
            .distinctUntilChanged()
            .collect { last ->
                if (last != null && hasMore && !loading &&
                    last >= plants.size - 3 && plants.isNotEmpty()
                ) {
                    loading = true
                    val next = page + 1
                    runCatching { GardenApi.list(next, search, bed) }
                        .onSuccess {
                            plants = plants + it.plants
                            hasMore = it.hasMore
                            page = next
                        }
                        .onFailure {
                            if (it is kotlinx.coroutines.CancellationException) throw it
                            hasMore = false
                        }
                    loading = false
                }
            }
    }

    Column(Modifier.fillMaxSize()) {
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            OutlinedTextField(
                value = search,
                onValueChange = { search = it },
                label = { Text("Search plants") },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
            // Beside the search box rather than behind a menu: the original
            // app kept appearance one tap away, and a theme nobody can find
            // is a theme nobody uses.
            TextButton(onClick = onAppearance) { Text("Theme") }
        }

        if (beds.isNotEmpty()) {
            Row(
                Modifier
                    .horizontalScroll(rememberScrollState())
                    .padding(horizontal = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                FilterChip(
                    selected = bed.isEmpty(),
                    onClick = { bed = "" },
                    label = { Text("All") },
                )
                beds.forEach { b ->
                    FilterChip(
                        selected = bed == b.name,
                        onClick = { bed = if (bed == b.name) "" else b.name },
                        label = { Text(b.name + " (" + b.count + ")") },
                    )
                }
            }
            Spacer(Modifier.height(8.dp))
        }

        when {
            failed != null -> Box(
                Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        failed!!,
                        style = MaterialTheme.typography.bodyMedium,
                        modifier = Modifier.padding(horizontal = 24.dp),
                    )
                    TextButton(onClick = { attempt++ }) { Text("Try again") }
                }
            }

            plants.isEmpty() && loading -> Box(
                Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) { CircularProgressIndicator() }

            plants.isEmpty() -> Box(
                Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    if (search.isBlank()) "No plants here yet."
                    else "Nothing matches that.",
                    style = MaterialTheme.typography.bodyMedium,
                )
            }

            else -> LazyColumn(
                state = listState,
                modifier = Modifier.fillMaxSize(),
                // The camera bar floats over this, so the last plant has to
                // be scrollable clear of it or it can never be tapped.
                contentPadding = PaddingValues(
                    start = 12.dp, end = 12.dp, bottom = bottomInset,
                ),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(plants, key = { it.id }) { p ->
                    PlantRow(p) { onOpen(p) }
                }
                if (hasMore) {
                    item {
                        Box(
                            Modifier.fillMaxWidth().padding(16.dp),
                            contentAlignment = Alignment.Center,
                        ) { CircularProgressIndicator() }
                    }
                }
            }
        }
    }
}

@Composable
private fun PlantRow(p: PlantSummary, onClick: () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .clickable(onClick = onClick)
            .padding(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .size(72.dp)
                .clip(RoundedCornerShape(10.dp))
                .background(MaterialTheme.colorScheme.surface),
            contentAlignment = Alignment.Center,
        ) {
            if (p.photo != null) {
                AsyncImage(
                    model = p.photo,
                    contentDescription = p.name,
                    modifier = Modifier.fillMaxSize(),
                    contentScale = ContentScale.Crop,
                )
            } else {
                // A plant with no photograph is common in this catalogue and
                // is not an error, so it gets a quiet placeholder rather than
                // a broken-image mark.
                Text(SEEDLING, style = MaterialTheme.typography.titleLarge)
            }
        }

        Spacer(Modifier.width(12.dp))

        Column(Modifier.weight(1f)) {
            Text(
                p.name,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
            )
            if (p.botanical.isNotBlank()) {
                Text(
                    p.botanical,
                    style = MaterialTheme.typography.bodySmall,
                    fontStyle = FontStyle.Italic,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            // SYMBOLS INSTEAD OF WORDS ON THE LIST. "Full Sun - Partial
            // Shade" is four words to read on a row meant to be scanned, and
            // the words are still there on the detail screen. Jason: "I also
            // lost a little emojis. I had like water in the sun and stuff."
            val line = listOfNotNull(
                p.bed.takeIf { it.isNotBlank() },
                sunIcon(p.sun),
                waterIcon(p.water),
                if (p.qty > 1) MULTIPLY + p.qty else null,
            ).joinToString("  " + MIDDOT + "  ")
            if (line.isNotBlank()) {
                Text(
                    line,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            // SAFETY SITS WITH THE PLANT, not three screens in. Jason asked
            // for toxic and edible at the top on the wiki; the same reason
            // applies harder on a phone held in the garden, where the
            // question is usually being asked about something already in
            // somebody's hand.
            SafetyLine(p.toxicity, p.edibility)
        }
    }
}

@Composable
internal fun SafetyLine(toxicity: String, edibility: String) {
    val toxic = toxicity.isNotBlank() &&
        !toxicity.equals("None", true) &&
        !toxicity.equals("Non-toxic", true)
    val edible = edibility.isNotBlank() && !edibility.equals("None", true)
    if (!toxic && !edible) return
    Row(
        Modifier.padding(top = 3.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        if (toxic) {
            Text(
                WARNING + " " + toxicity,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.error,
                fontWeight = FontWeight.Medium,
            )
        }
        if (edible) {
            Text(
                APPLE + " " + edibility,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

// Built from code points rather than typed inline. Every tool between here
// and the file has an opinion about how to encode an emoji, and one of them
// being wrong turns a warning sign into a question mark on the one row where
// it says a plant is poisonous.
private val SEEDLING = String(Character.toChars(0x1F331))
private val WARNING = String(Character.toChars(0x26A0))
private val APPLE = String(Character.toChars(0x1F34E))
private val MIDDOT = String(Character.toChars(0x00B7))
private val MULTIPLY = String(Character.toChars(0x00D7))
