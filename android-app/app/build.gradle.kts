plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization)
}

android {
    namespace = "org.jasonagibson.junsgarden"
    compileSdk {
        version = release(36) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        applicationId = "org.jasonagibson.junsgarden"
        minSdk = 34
        targetSdk = 36
        // BUMPED so Android treats an install as an upgrade rather than a
        // no-op, and so the version on screen says which build is running.
        // A same-versionCode reinstall is legal but gives nothing to check
        // against when someone reports "it still looks the same".
        versionCode = 18
        versionName = "3.2"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        // Where the catalogue lives. The shipped app talks to the live site;
        // a build made with  -PgardenBase=https://test.example.duckdns.org
        // talks to the test site instead, so a new version of the SITE can be
        // tried from the app before it goes live. Read by GardenApi.kt.
        val gardenBase = (project.findProperty("gardenBase") as String?) ?: "https://junsgarden.zapto.org"
        buildConfigField("String", "GARDEN_BASE", "\"$gardenBase\"")
    }

    buildTypes {
        release {
            optimization {
                enable = false
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
        // Required for BuildConfig.VERSION_NAME. Off by default in AGP 8+,
        // and the error when it is missing names the symbol, not the flag.
        buildConfig = true
    }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.datastore.preferences)
    implementation(libs.kotlinx.serialization.json)
    // Plant photographs. The original app used Coil for exactly this.
    implementation(libs.coil.compose)
    // Barcode decoding from a PHOTOGRAPH, not a live camera feed. The pure
    // Java core only -- the embedded scanner would need CAMERA permission,
    // and this app deliberately has none.
    implementation(libs.zxing.core)

    // Encrypted storage for the garden login key -- see GardenLogin.kt.
    implementation(libs.androidx.security.crypto)
    // Reads which way up a photo was taken, so it is not sent sideways --
    // see GardenCamera.kt.
    implementation(libs.androidx.exifinterface)

    testImplementation(libs.junit)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
    debugImplementation(libs.androidx.compose.ui.tooling)
}
