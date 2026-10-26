// ScanUploadWorker.kt -- sends the phone's waiting scans when there is signal, even if the app is
// closed. Android's WorkManager runs it once the phone is online; if the website cannot be reached
// it asks to be run again later (a little more patiently each time). The pictures are kept on the
// phone until the website has them, so nothing is lost on the way.

package org.jasonagibson.junsgarden.garden

import android.content.Context
import androidx.work.BackoffPolicy
import androidx.work.CoroutineWorker
import androidx.work.Constraints
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import java.util.concurrent.TimeUnit

class ScanUploadWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val r = ScanSync.sync(applicationContext)
        // Something still on the phone and the website not reachable: try again later.
        return if (r.stillOnPhone > 0 && r.networkProblem) Result.retry() else Result.success()
    }

    companion object {
        private const val NAME = "scan-upload"

        /** Ask Android to send the waiting scans as soon as the phone is online. Safe to call often. */
        fun enqueue(context: Context) {
            val request = OneTimeWorkRequestBuilder<ScanUploadWorker>()
                .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
                .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
                .build()
            // APPEND_OR_REPLACE: a run already in progress finishes, then this one picks up scans
            // added meanwhile.
            WorkManager.getInstance(context).enqueueUniqueWork(NAME, ExistingWorkPolicy.APPEND_OR_REPLACE, request)
        }
    }
}
