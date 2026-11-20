# Jun's Garden: the Android app

This folder is the source code of the **Jun's Garden** phone app, version
**3.2** (`versionCode` 18). It is written in **Kotlin** with **Jetpack
Compose** (Google's way of drawing Android screens in code).

The app shows the plants from your garden website, and lets people who are
logged in use the camera to identify plants and read plant tags and receipts.

Guide 17 in `START-HERE` walks you through opening this in Android Studio
step by step. This README is the reference you come back to.

---

## What the app does, screen by screen

| Screen | What you see | File |
|---|---|---|
| Splash | A short film of the garden drawing, once each time the app starts. If the film cannot play, the app moves on after at most 12 seconds. | `garden/SplashScreen.kt` |
| Sign in | Username, password, **Sign In**, **Continue as guest**, and a "Forgot password? / First time?" link that opens the website in the browser. | `garden/GardenSignInScreen.kt`, `garden/GardenLogin.kt` |
| Catalogue | All the plants, 20 at a time, with photos and small sun and water symbols. A search box, a row of bed buttons to filter, and a **Theme** button. | `garden/CatalogScreen.kt`, `garden/PlantIcons.kt` |
| Plant page | One plant: photo, name, and everything recorded about it in cards, grouped the same way as the website. | `garden/PlantDetailScreen.kt`, `garden/PlantSections.kt` |
| Camera buttons | Along the bottom: **Identify**, **Tag**, **Barcode**, **Receipt**. Guests only get **Barcode**. The phone's own camera app takes the photo. | `garden/GardenScreen.kt`, `garden/GardenCamera.kt` |
| Many tags at once | From **Tag**, choose "Many at once": photograph a whole tray of tags (front, then back), check the pairs, send them all to be read later. | `garden/BulkTagScreen.kt` |
| Theme | Pick a colour theme (Evergreen, Forest, Ivy and more) and Light, Dark or Match phone. | `ui/AppearanceSheet.kt`, `ui/GardenThemes.kt`, `ui/Theme.kt` |
| Update offer | If the website has a newer version, a small box says "Update available" with **Update** and **Later**. | `app/Updater.kt`, `MainActivity.kt` |

`MainActivity.kt` is the front door: Android starts there, and it decides
which screen to show.

Every Kotlin file starts with a short plain-English comment saying what it
does. Longer comments inside explain *why* things are done a certain way.

---

## Where the app gets its data

**Only from your website, `https://junsgarden.zapto.org`.** Nothing else.

| What | Address | Login needed? |
|---|---|---|
| Plant list, one plant, beds, barcode lookup | `GARDEN_BASE/app_api.php?action=list`, `detail`, `bed_counts`, `barcode_lookup` | No (the catalogue is public) |
| Log in, check login, log out | `GARDEN_BASE/app_api.php?action=login`, `whoami`, `logout` | It *is* the login |
| Plant photos | `GARDEN_BASE/images/...` | No |
| Identify, diagnose, tag, tag queue, receipt | `GARDEN_BASE/app/garden/identify`, `diagnose`, `tag`, `tag-queue`, `receipt` | Yes, the garden login key |
| Is there a newer app? | `https://junsgarden.zapto.org/apps/version.json` | No |

`GARDEN_BASE` is set in `app/build.gradle.kts` (see below). On the website
side, `app_api.php` is answered by `rust/junsgarden/src/pages/app_api.rs` (the
old PHP name is kept so the app does not have to change), and `/app/garden/...`
by `rust/junsgarden/src/pages/garden_camera.rs`. The rule in `AGENTS.md`
applies: you can add fields to these answers, but never rename or remove one,
or the app breaks.

**About the login.** The password is sent once, over HTTPS, in the body of
the request (never in the address). The site answers with a key. The app
keeps only that key, in encrypted storage on the phone, and sends it with
each camera request. Log out tells the site to forget the key.

**No camera permission.** The app never opens the camera itself. It asks the
phone's camera app for a photo, so it does not need the camera permission.

---

## Open it in Android Studio

1. Install Android Studio (guide 17, part A, has the official links).
2. Start Android Studio. Choose **Open** and pick this `android-app` folder
   (the folder that has `settings.gradle.kts` in it).
3. Wait. Android Studio runs a **Gradle sync**: it downloads the build tools
   and libraries the app needs. The first time this can take several minutes.
   You can see progress in the bar at the bottom.
4. If it asks to install a missing SDK platform (Android 36), say yes.

What is checked: this exact folder was built on 2026-10-13 with Gradle 9.5.0,
Android Gradle Plugin 9.3.1, Kotlin 2.2.10 and the JDK that comes with
Android Studio. The build succeeded.

---

## Build a debug APK

An **APK** is the file Android installs. A **debug** APK is the kind you make
while learning and testing.

**In Android Studio:** menu **Build > Generate Bundle(s) / APK(s) > Generate
APK(s)**. (Older versions call it **Build > Build Bundle(s) / APK(s) > Build
APK(s)**.) When it finishes, a message appears with a link to the file.

**From a terminal** (PowerShell, in this folder):

```
.\gradlew.bat assembleDebug
```

The file appears at `app\build\outputs\apk\debug\app-debug.apk`.

If the terminal says it cannot find Java, point it at the Java that comes
with Android Studio first, then run the command again:

```
$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
```

Official pages:
- Build from the command line: <https://developer.android.com/build/building-cmdline>
- Build for release (and the Build menu): <https://developer.android.com/build/build-for-release>

To run it on your phone with a cable, or on a pretend phone (emulator), see
guide 17, part A.

---

## Change the website address the app uses

If you ever move your site to a new address, two places need changing.

**1. `GARDEN_BASE`** in `app/build.gradle.kts`. Find this line:

```kotlin
val gardenBase = (project.findProperty("gardenBase") as String?) ?: "https://junsgarden.zapto.org"
```

Change only the address in quotes, for example to `"https://my-new-name.example"`.
No slash at the end. The catalogue, the login and the camera tools all use
this one setting.

You can also try a different address for one build without editing the file:

```
.\gradlew.bat assembleDebug -PgardenBase=https://test-address.example
```

**2. The updater addresses** in `app/Updater.kt`. Near the bottom there are
two lines, `MANIFEST` and `DEFAULT_APK`, with the address typed in. They do
not follow `GARDEN_BASE` (yet). Change them too, or the app will keep looking
for updates at the old address. (Making them follow `GARDEN_BASE` is a good
small coding exercise.)

Then make a new version (see "Publish a new version" below), so the phones
get the new address.

**Careful:** phones that still have the old version will look for updates at
the *old* address. Publish the new version *before* you turn the old address
off.

---

## Signing and updates (read this before you publish anything)

**Every APK is signed** with a key: a file that proves who made the app.

**The rule:** Android installs an update over the app on a phone **only if**
the new APK has the **same application id** (`org.jasonagibson.junsgarden`,
in `app/build.gradle.kts`) **and is signed with the same key** as the
version already on the phone. If the key is different, Android refuses the
update. Google's page says: "If you sign the new version with a different
certificate, you must assign a different package name to the app—in this
case, the user installs the new version as a completely new app."

That is why the application id and the package name in this folder still
say `jasonagibson`. They are only names. They do not connect to the author's
servers. Changing them would make Android treat it as a different app.

**The signing key is NOT included in this folder**, on purpose. A key in a
project folder can be copied by anyone who gets the folder.

**What is on the phones today.** We checked the APK the website serves now
(`site/apps/junsgarden.apk`, version 3.2, built from this folder on 2026-10-13). It is
signed with the **Android Debug** key from the author's computer, the same key as 3.1, so the
phones take it as a normal update. That is the key Android Studio makes by
itself on every computer (`.android\debug.keystore` in your user folder). It
is not a separate "release" key. Your own Android Studio will make a
*different* debug key, so an APK you build on your computer will **not**
install over the app that is on the phones now, until you pick one of these:

**Choice (a): use the same key as now (only if you already have it).**
If you already have the key file that signs the current app (that computer's
`debug.keystore`) and its passwords, keep the file somewhere safe and the
passwords in your password manager. **Never put the key file in this project
folder, in git, in a chat, in an email, or into any AI.** Then the phones
update as normal. If you do not have it, choose (b): it is the usual way.

**Choice (b): make your own new key (recommended).**
In Android Studio: **Build > Generate Signed Bundle/APK**, choose **APK**,
then under "Key store path" click **Create new**. Pick strong passwords,
save them in your password manager, and keep the `.jks` file safe (not in
this folder). Google suggests a validity of at least 25 years.
After that, everyone who has the app must **uninstall the old app once**
and install your new one. (They may need to log in and pick their theme
again.) From then on, always sign with your new key.

Either way: **if you lose the key, you can never update the app again on
those phones**. Google's page says: "If you lose or misplace your key, you
will not be able to publish updates to your existing app. You cannot
regenerate a previously generated key." Keep a backup copy of the key file
somewhere safe (not online).

Official page: <https://developer.android.com/studio/publish/app-signing>

---

## How the in-app updater works

1. Each time the app starts, it downloads a small file:
   `https://junsgarden.zapto.org/apps/version.json`.
2. That file says which version is on the website. It looks like this:

   ```json
   {
     "versionCode": 18,
     "versionName": "3.2",
     "url": "https://junsgarden.zapto.org/apps/junsgarden.apk",
     "md5": "the fingerprint of the new APK (md5sum)",
     "sha256": "its stronger fingerprint (sha256sum)",
     "notes": "What is new, in one or two sentences."
   }
   ```

3. If `versionCode` there is **bigger** than the app's own `versionCode`, the
   app shows "Update available" with the `notes`.
4. On **Update**, it downloads the APK from `url`, checks the `sha256`
   fingerprint (apps before 3.2 check `md5`; keep both, so every phone can
   update), so a broken download is never installed, and hands it to
   Android. Android always asks the person to confirm. The first time, the
   phone asks to allow "install unknown apps" for Jun's Garden.

Both files live on your website in the `site/apps/` folder:
`site/apps/version.json` and `site/apps/junsgarden.apk`. The website serves
these files as they are, with no rebuild (guide 11 explains the `site/`
folder; `.apk` and `.json` in `apps/` are on the allowed list in
`rust/junsgarden/src/main.rs`).

---

## Publish a new version to your own website

Do this only after you have decided about the key (see "Signing and
updates").

1. **Change the version numbers** in `app/build.gradle.kts`:
   - `versionCode`: add 1 (17 becomes 18). Phones only update when this number goes up.
   - `versionName`: what people see, for example `"3.2"`.
2. **Build a signed APK** with the key the phones already have:
   **Build > Generate Signed Bundle/APK**, choose **APK**, pick your key
   file, enter the passwords, choose the **release** variant.
3. **Test it on your own phone first.** Install it over the old version. If
   Android says the app cannot be installed or "conflicts with an existing
   package", the key is not the same one. Stop and read "Signing and
   updates" again.
4. **Get its two fingerprints.** Android Studio tells you where it saved the
   APK (usually `app\release\app-release.apk`). In PowerShell, in that folder:

   ```
   (Get-FileHash -Algorithm MD5 .\app-release.apk).Hash.ToLower()
   (Get-FileHash -Algorithm SHA256 .\app-release.apk).Hash.ToLower()
   ```

5. **Copy the APK** to your website as `site/apps/junsgarden.apk` (replace
   the old one; keep a copy of the old one in your backups).
6. **Edit `site/apps/version.json`**: the new `versionCode`, `versionName`,
   `md5`, `sha256`, and new `notes`. Save it. Check it is still valid JSON (commas,
   quotes) by opening `https://junsgarden.zapto.org/apps/version.json` in a
   browser.
7. **Check** on a phone with the old version: start the app. It should offer
   the update.

Order matters: put the APK in place **before** you change `version.json`,
so nobody is offered a file that is not there yet.

---

## Known leftovers (safe to leave; good first tidy-ups)

These all build and work. None of them talks to anything except your site.

- **Sharing into the app.** The app still appears in the phone's **Share**
  list for links and text. This was for the old "Ask" chat, which has been
  removed. Today a shared link only opens an empty panel. The pieces: the
  second `<intent-filter>` in `app/src/main/AndroidManifest.xml`,
  `Shared.kt`, and the `open` panel and `chat` / `showAsk` parameters in
  `garden/GardenScreen.kt`.
- **Unused libraries** listed in `app/build.gradle.kts` that no Kotlin file
  in this folder imports now: `navigation-compose`, `datastore-preferences`,
  `kotlinx-serialization-json` (and its plugin), `lifecycle-runtime-ktx`.
  Removing one is a good way to practise: remove it, build, and see.
- **`androidx.security:security-crypto` is deprecated.** It keeps the login
  key encrypted (`GardenLogin.kt`). The build shows yellow "deprecated"
  warnings for it. It still works. Replacing it is a bigger job; not urgent.
- **Portrait only.** The screen does not rotate. This stops a turn of the
  phone from throwing away a photo that is being read. The proper fix is a
  ViewModel (state that survives rotation); see the comment in the manifest.
- The package folders are named `org/jasonagibson/junsgarden`. Leave them:
  they match the application id (see "Signing and updates").

## What was removed (October 2026)

An older household version of this app talked to the author's household server.
That code was dead (no screen used it) and was deleted: the `auth/` and
`chat/` folders, `app/AppApi.kt`, `app/RecipeScreen.kt`,
`app/ToolsScreen.kt`, the AppAuth sign-in library and the Ktor library, the
sign-in redirect entry in the manifest, and an iOS build file for a
different app. The app was built before and after the removal; both builds
succeeded. Nothing in this folder points at the author's household server any
more; the only web address the app uses is your own site.
