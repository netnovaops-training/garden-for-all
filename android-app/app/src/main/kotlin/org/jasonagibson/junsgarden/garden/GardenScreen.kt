// GardenScreen.kt -- the main screen after sign-in (or as a guest).
//
// Shows the catalogue or one plant, the camera buttons along the bottom
// (Identify, Tag, Barcode, Receipt -- guests only get Barcode), the
// "Sign in" / "Log out" button, and the sheet that shows what the camera
// found. The `open` panel and `chat` slot are leftovers from the old chat;
// see the README, "Known leftovers".

package org.jasonagibson.junsgarden.garden

import android.content.Context
import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.core.content.FileProvider
import kotlinx.coroutines.launch
import androidx.compose.foundation.isSystemInDarkTheme
import org.jasonagibson.junsgarden.Shared
import org.jasonagibson.junsgarden.ui.AppearanceSheet
import java.io.File

/**
 * The garden: his plants, his camera, and the assistant one tap away.
 *
 * THERE IS NO WEB VIEW HERE ANY MORE. This screen used to be the website in
 * a frame, and the author's instructions ended that: "I want the app I had before
 * just adding the features I wanted. I don't want to go into the website",
 * "I want the plants to be able to be pulled up like it was before as a
 * catalog", and then, plainly, "There's not supposed to be a webview".
 *
 * THE APP HE HAD WAS NATIVE. Its source is gone -- no repository here has it
 * and it is on no disk -- but the built APK still names every screen and
 * every call it made, and the API it called still answers. So the catalogue
 * is rebuilt against that same API rather than reinvented.
 *
 * WHAT THE CAMERA IS FOR, in his words: "take a picture of a plant and
 * describe if it's living or dead or needs nutrients or rusted and identify
 * the plant"; "take pictures of a tag and then import it into the database";
 * "take barcodes and scan them into a plant"; and receipts. Four buttons,
 * one camera, and the answer comes back into this screen rather than into a
 * web page.
 *
 * STILL NO CAMERA PERMISSION. Every one of these hands the job to the
 * phone's own camera app and works on the photograph it returns -- including
 * the barcode, which is decoded from the picture rather than from a live
 * viewfinder. The app never gains the ability to watch.
 */
@Composable
fun GardenScreen(
    signedInAs: String,
    camera: GardenCamera,
    // GUEST MODE (guest = true): the catalogue, every plant page and the
    // barcode lookup (the phone reads the stripes itself; the lookup is the
    // public catalogue). Identify, Tag and Receipt need a garden login, so
    // they are not shown. In the corner there is one button: `accountLabel`
    // ("Sign in" for a guest, "Log out" for somebody logged in), which runs
    // `onSignIn`.
    guest: Boolean = false,
    accountLabel: String = "Sign in",
    onSignIn: () -> Unit = {},
    // 3.1: logged-in users get Identify, Tag, Barcode and Receipt (read by
    // free cloud AI through the garden site). "Ask" was the old chat and has
    // been removed; MainActivity always passes showAsk = false, so the
    // corner button is `accountLabel` (Log out) instead.
    showAsk: Boolean = true,
    chat: @Composable () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    var open by remember { mutableStateOf(false) }
    var openPlant by remember { mutableStateOf<PlantSummary?>(null) }
    var showAppearance by remember { mutableStateOf(false) }

    // The bulk tag screen, and the little chooser that offers it.
    //
    // OFFERED FROM "TAG" RATHER THAN AS A FIFTH BUTTON. Somebody reaches for
    // Tag because they are holding tags; whether it is one or thirty is the
    // next thought, not a different intention. A fifth button would also
    // make all five narrower on a phone, which is the opposite of what was
    // asked for.
    var showBulk by remember { mutableStateOf(false) }
    var askHowMany by remember { mutableStateOf(false) }

    // What the camera was asked to do, so the result knows how to be read.
    var job by remember { mutableStateOf<Job?>(null) }
    var working by remember { mutableStateOf(false) }
    var reading by remember { mutableStateOf<GardenCamera.Reading?>(null) }
    var failure by remember { mutableStateOf<String?>(null) }

    // Where the camera app was told to write. Kept because the capture
    // contract reports only success or failure, never the location.
    var target by remember { mutableStateOf<Uri?>(null) }
    // THE FRONT, WHILE THE BACK IS BEING TAKEN.
    //
    // A nursery tag prints the name on the front and everything worth
    // recording on the back, so a tag scan is two photographs. The first is
    // held here until the second is taken or declined; nothing is sent
    // until the pair is settled, because the server reads both sides
    // together and merges them.
    var tagFront by remember { mutableStateOf<Uri?>(null) }
    var askBack by remember { mutableStateOf(false) }

    val shutter = rememberLauncherForActivityResult(
        ActivityResultContracts.TakePicture()
    ) { ok ->
        val shot = target
        val what = job
        // false means he backed out of the camera, which is not a failure
        // worth putting on screen.
        if (!ok || shot == null || what == null) {
            job = null
            return@rememberLauncherForActivityResult
        }
        // A TAG'S FIRST SHOT IS NOT A JOB YET. Hold the front, offer the
        // back, and only then send. Any other action goes straight through.
        if (what == Job.Tag && tagFront == null) {
            tagFront = shot
            askBack = true
            return@rememberLauncherForActivityResult
        }

        val front = tagFront
        val pair = if (what == Job.Tag && front != null) listOf(front, shot)
                   else listOf(shot)
        tagFront = null
        askBack = false

        working = true
        reading = null
        failure = null
        scope.launch {
            runCatching {
                when (what) {
                    Job.Examine -> camera.examine(context, shot)
                    Job.Tag -> camera.readTag(context, pair)
                    Job.Receipt -> camera.readReceipt(context, shot)
                    Job.Barcode -> barcodeReading(camera, context, shot)
                }
            }
                .onSuccess { reading = it }
                .onFailure { failure = it.message ?: "That did not work." }
            working = false
        }
    }

    fun shoot(what: Job) {
        job = what
        val where = newPhotoTarget(context)
        target = where
        shutter.launch(where)
    }

    // A SHARED LINK OPENS THE BUBBLE. The chat starts closed, so a link
    // shared in from YouTube would be answered behind a closed panel.
    LaunchedEffect(Shared.pending) {
        if (Shared.pending != null) open = true
    }

    // One layer at a time, innermost first.
    BackHandler(enabled = open) { open = false }
    BackHandler(enabled = askBack) {
        tagFront = null
        askBack = false
    }
    BackHandler(enabled = !open && (reading != null || failure != null)) {
        reading = null
        failure = null
    }
    BackHandler(
        enabled = !open && reading == null && failure == null && openPlant != null
    ) {
        openPlant = null
    }

    Box(Modifier.fillMaxSize()) {
        val plant = openPlant
        if (showBulk) {
            BulkTagScreen(
                camera = camera,
                bottomInset = BAR_SPACE,
                onBack = { showBulk = false },
            )
        } else if (plant == null) {
            CatalogScreen(
                bottomInset = BAR_SPACE,
                onAppearance = { showAppearance = true },
            ) { openPlant = it }
        } else {
            PlantDetailScreen(
                summary = plant,
                bottomInset = BAR_SPACE,
                onBack = { openPlant = null },
            )
        }

        if (!open && !showBulk) {
            Column(
                // A BACKGROUND, BECAUSE THE BAR FLOATS OVER THE CONTENT.
                //
                // The buttons are pills on nothing, so mid-scroll the text
                // underneath showed through the gaps between them -- the
                // "Cleanup" paragraph reading straight through the bar. The
                // list already reserves BAR_SPACE at its end so the LAST
                // item clears; that was never the problem. Everything
                // passing UNDER the bar on the way past was.
                //
                // Painted with the app's own ground colour rather than a
                // translucent scrim: a scrim over arbitrary content is
                // legible on some plants' photographs and not others, and
                // "sometimes readable" is the failure that is hardest to
                // notice in testing.
                Modifier.align(Alignment.BottomCenter).fillMaxWidth()
                    .background(MaterialTheme.colorScheme.background)
                    .padding(start = 12.dp, end = 12.dp, top = 10.dp, bottom = 16.dp),
            ) {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    if (!guest) {
                        GardenAction("Identify", Modifier.weight(1f)) { shoot(Job.Examine) }
                        GardenAction("Tag", Modifier.weight(1f)) { askHowMany = true }
                    }
                    GardenAction("Barcode", Modifier.weight(1f)) { shoot(Job.Barcode) }
                    if (!guest) {
                        GardenAction("Receipt", Modifier.weight(1f)) { shoot(Job.Receipt) }
                    }
                }
                Spacer(Modifier.height(10.dp))
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                ) {
                    if (guest || !showAsk) {
                        FloatingActionButton(onClick = onSignIn) {
                            Text(
                                accountLabel,
                                style = MaterialTheme.typography.labelLarge,
                                modifier = Modifier.padding(horizontal = 16.dp),
                            )
                        }
                    } else {
                        FloatingActionButton(onClick = { open = true }) {
                            Text("Ask", style = MaterialTheme.typography.labelLarge)
                        }
                    }
                }
            }
        }

        // --- one tag, or a pile of them? ----------------------------------
        if (askHowMany) {
            androidx.compose.material3.AlertDialog(
                onDismissRequest = { askHowMany = false },
                title = { Text("Read a tag") },
                text = {
                    Text(
                        "One at a time reads it straight away. Many at once " +
                            "queues them and reads them afterwards, so you " +
                            "can photograph a whole tray first."
                    )
                },
                confirmButton = {
                    androidx.compose.material3.TextButton(onClick = {
                        askHowMany = false
                        showBulk = true
                    }) { Text("Many at once") }
                },
                dismissButton = {
                    androidx.compose.material3.TextButton(onClick = {
                        askHowMany = false
                        shoot(Job.Tag)
                    }) { Text("One now") }
                },
            )
        }

        // --- now turn the tag over ----------------------------------------
        if (askBack) {
            BackSidePrompt(
                onTakeBack = {
                    val where = newPhotoTarget(context)
                    target = where
                    shutter.launch(where)
                },
                onSkip = {
                    // Sending the front alone is a real choice: a tag stuck
                    // in a pot does not always turn over. It reads what it
                    // can rather than refusing.
                    val front = tagFront
                    tagFront = null
                    askBack = false
                    if (front != null) {
                        working = true
                        reading = null
                        failure = null
                        scope.launch {
                            runCatching { camera.readTag(context, listOf(front)) }
                                .onSuccess { reading = it }
                                .onFailure {
                                    failure = it.message ?: "That did not work."
                                }
                            working = false
                        }
                    }
                },
            )
        }

        // --- what the camera found ----------------------------------------
        if (working || reading != null || failure != null) {
            ResultSheet(
                working = working,
                job = job,
                reading = reading,
                failure = failure,
                onClose = {
                    reading = null
                    failure = null
                    job = null
                },
            )
        }

        if (showAppearance) {
            AppearanceSheet(
                dark = isSystemInDarkTheme(),
                onClose = { showAppearance = false },
            )
        }

        AnimatedVisibility(
            visible = open,
            enter = fadeIn() + slideInVertically { it / 3 },
            exit = fadeOut() + slideOutVertically { it / 3 },
        ) {
            Box(
                Modifier
                    .fillMaxSize()
                    // A scrim, so it reads as something ON TOP of the page
                    // rather than as a new page.
                    .background(Color.Black.copy(alpha = 0.32f)),
            ) {
                Surface(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .fillMaxSize(0.88f),
                    shape = MaterialTheme.shapes.extraLarge,
                    color = MaterialTheme.colorScheme.background,
                    tonalElevation = 3.dp,
                ) {
                    Column(Modifier.fillMaxSize()) {
                        Row(
                            Modifier.fillMaxWidth()
                                .padding(start = 18.dp, top = 8.dp, end = 6.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                signedInAs,
                                style = MaterialTheme.typography.labelMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.weight(1f),
                            )
                            // A TYPED GLYPH, not a vector icon. The icons
                            // library is not a dependency of this project and
                            // pulling it in for one X is a megabyte of APK for
                            // a character the font already has.
                            IconButton(onClick = { open = false }) {
                                Text(
                                    "✕",
                                    style = MaterialTheme.typography.titleMedium,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                        chat()
                    }
                }
            }
        }
    }
}

/** Which question the photograph is being taken to answer. */
private enum class Job { Examine, Tag, Barcode, Receipt }

private fun Job.waitingWords(): String = when (this) {
    Job.Examine -> "Looking at the plant…"
    Job.Tag -> "Reading the tag…"
    Job.Barcode -> "Reading the barcode…"
    Job.Receipt -> "Reading the receipt…"
}

/**
 * A barcode, and then what it belongs to.
 *
 * THE NUMBER ALONE IS NOT AN ANSWER. Nobody wants to be told "9781234567".
 * The barcode is looked up against the catalogue, and what comes back is
 * the plant it is stuck to -- which is the thing that was actually asked.
 */
private suspend fun barcodeReading(
    camera: GardenCamera,
    context: Context,
    shot: Uri,
): GardenCamera.Reading {
    val code = camera.readBarcode(context, shot)
        ?: return GardenCamera.Reading(
            headline = "No barcode in that photo",
            body = "Hold the phone closer so the stripes fill the frame, and " +
                "keep it square on.",
        )

    val found = runCatching { GardenApi.barcode(code) }.getOrNull()
    return if (found == null) {
        GardenCamera.Reading(
            headline = code,
            body = "Nothing in the garden has this barcode yet.",
        )
    } else {
        GardenCamera.Reading(
            headline = found.name,
            body = buildString {
                if (found.botanical.isNotBlank()) {
                    append(found.botanical).append('\n')
                }
                append("Barcode ").append(code)
            },
        )
    }
}

/**
 * The answer, over the catalogue.
 *
 * A SHEET RATHER THAN A SCREEN, for the same reason the chat is a bubble:
 * the plants stay behind it, so reading a tag does not mean losing your
 * place in the list.
 */
@Composable
private fun ResultSheet(
    working: Boolean,
    job: Job?,
    reading: GardenCamera.Reading?,
    failure: String?,
    onClose: () -> Unit,
) {
    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.32f)),
    ) {
        Surface(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .fillMaxSize(0.62f),
            shape = MaterialTheme.shapes.extraLarge,
            color = MaterialTheme.colorScheme.background,
            tonalElevation = 3.dp,
        ) {
            Column(Modifier.fillMaxSize().padding(20.dp)) {
                when {
                    working -> Box(
                        Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center,
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            CircularProgressIndicator()
                            Spacer(Modifier.height(14.dp))
                            Text(
                                job?.waitingWords() ?: "Working…",
                                style = MaterialTheme.typography.bodyMedium,
                            )
                            Spacer(Modifier.height(6.dp))
                            // Says the quiet part out loud, because the
                            // honest wait here is tens of seconds and a
                            // silent spinner reads as a hang.
                            Text(
                                "This can take a minute.",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }

                    failure != null -> {
                        Text(
                            "That did not work",
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Spacer(Modifier.height(8.dp))
                        Text(failure, style = MaterialTheme.typography.bodyMedium)
                        Spacer(Modifier.weight(1f))
                        TextButton(onClick = onClose) { Text("Close") }
                    }

                    reading != null -> {
                        Text(
                            reading.headline,
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.SemiBold,
                        )
                        Spacer(Modifier.height(10.dp))
                        Column(
                            Modifier.weight(1f).verticalScroll(rememberScrollState())
                        ) {
                            Text(
                                reading.body,
                                style = MaterialTheme.typography.bodyMedium,
                            )
                        }
                        TextButton(onClick = onClose) { Text("Close") }
                    }
                }
            }
        }
    }
}

@Composable
private fun GardenAction(label: String, modifier: Modifier, onClick: () -> Unit) {
    FilledTonalButton(
        onClick = onClick,
        modifier = modifier,
        contentPadding = ButtonDefaults.TextButtonContentPadding,
    ) {
        Text(label, style = MaterialTheme.typography.labelMedium, maxLines = 1)
    }
}

/** A file the camera app may write into, and a content:// URI for it. */
private fun newPhotoTarget(ctx: Context): Uri {
    val dir = File(ctx.cacheDir, "images").apply { mkdirs() }
    val file = File(dir, "garden_${System.currentTimeMillis()}.jpg")
    return FileProvider.getUriForFile(ctx, "${ctx.packageName}.fileprovider", file)
}

/** How much room the camera bar and the Ask button need at the bottom. */
private val BAR_SPACE = 132.dp

/**
 * Between the two photographs of a tag.
 *
 * ASKED, NOT ASSUMED. Taking the back automatically would leave somebody
 * holding a phone at a plant wondering why the camera opened twice. Saying
 * what the second shot is FOR is what makes people take it -- and the back
 * is where the spacing, zones, sun, water and grower are printed, which is
 * the information that was missing.
 *
 * SKIPPING IS A REAL CHOICE, not a failure. A tag stuck in a pot does not
 * always turn over, and reading one side beats refusing to read any.
 */
@Composable
private fun BackSidePrompt(onTakeBack: () -> Unit, onSkip: () -> Unit) {
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f))) {
        Surface(
            modifier = Modifier
                .align(Alignment.Center)
                .fillMaxWidth()
                .padding(28.dp),
            shape = MaterialTheme.shapes.extraLarge,
            color = MaterialTheme.colorScheme.background,
            tonalElevation = 4.dp,
        ) {
            Column(Modifier.padding(22.dp)) {
                Text(
                    "Now the back",
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    "The back of the tag usually has the spacing, zones, sun "
                        + "and water. Turn it over and take one more.",
                    style = MaterialTheme.typography.bodyMedium,
                )
                Spacer(Modifier.height(18.dp))
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                ) {
                    TextButton(onClick = onSkip) { Text("Just the front") }
                    Spacer(Modifier.width(8.dp))
                    FilledTonalButton(onClick = onTakeBack) { Text("Take the back") }
                }
            }
        }
    }
}
