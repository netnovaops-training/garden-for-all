#!/usr/bin/env bash
# The phone app on a real (emulated) Android phone, against the lab garden, through a domain change.
# Run from a PC with the Android SDK (Git Bash on Windows works), the lab reachable by ssh:
#   GARDEN_LAB=user@host OUT=<folder for screenshots> bash docs/rust-pages/app-matrix/matrix.sh
# Prints one PASS/FAIL line per step. Everything it builds is a throwaway copy (debug, with the
# test overlay that allows plain http to 10.0.2.2); the emulator is wiped at the start.
#
# The story, in order (each step is a row of the app test matrix):
#   OLD = app 3.2 as phones have it (git cd4e526; its two fixed update addresses pointed at the
#         lab's old name), NEW = this branch's app (version code 19), both signed with the same key
#   1. garden at its first address: OLD logs in and browses
#   2. the garden moves (lab appserver new): OLD, never updated, still logs in and browses
#   3. OLD's update check finds NEW on the old name, downloads it from the new name, md5 checked
#   4. NEW installed over OLD (= Jun taps Install): it starts on its built-in address and works
#   5. the setup QR (junsgarden://setup link) points NEW at its garden; NEW follows the signed move
#   6. NEW on the new address: logs in and browses; self-update from the new address, sha256 checked
#   7. English and Korean: the sign-in screen, the catalogue and a plant, screenshots of each
set -u
# MATRIX_HERE: this folder, when running from a copy of the script (so edits cannot disturb a run).
here=${MATRIX_HERE:-$(cd "$(dirname "$0")" && pwd)}
repo=$(cd "$here/../../.." && pwd)
LAB=${GARDEN_LAB:?set GARDEN_LAB=user@host}
OUT=${OUT:-$here/out}
SDK=$(cygpath -u "${ANDROID_HOME:-$LOCALAPPDATA/Android/Sdk}" 2>/dev/null || echo "${ANDROID_HOME:?}")
export JAVA_HOME=${JAVA_HOME:-"/c/Program Files/Android/Android Studio/jbr"}
export ANDROID_HOME; ANDROID_HOME=$(cygpath -m "$SDK" 2>/dev/null || echo "$SDK")   # Gradle wants C:/...
ADB="$SDK/platform-tools/adb"
EMU="$SDK/emulator/emulator"
AVD=gardentest
# Its own emulator on a fixed port; every adb call (and ui.py's) goes to it alone.
EPORT=5584
export ANDROID_SERIAL=emulator-$EPORT
PKG=org.jasonagibson.junsgarden
OLDNAME=http://10.0.2.2:18181
NEWNAME=http://10.0.2.2:18182
work=$(mktemp -d)
pass=0; fail=0
ok()  { pass=$((pass+1)); echo "PASS $1"; }
bad() { fail=$((fail+1)); echo "FAIL $1"; }
lab() { ssh -o BatchMode=yes "$LAB" "cd ~/garden-rust-pages && $1"; }
shot() { "$ADB" exec-out screencap -p > "$OUT/$1.png"; }
ui() { PATH="$SDK/platform-tools:$PATH" python "$here/ui.py" "$@"; }
mkdir -p "$OUT/logs"

# Run one instrumented test; PASS when it reports OK.
inst() {   # inst LABEL CLASS#METHOD [-e k v ...]   (full output: $OUT/logs/<method>.log)
    local label=$1 t=$2 log; shift 2
    log="$OUT/logs/${t#*#}-$(date +%H%M%S).log"
    if "$ADB" shell am instrument -w -r -e class "$PKG.$t" "$@" "$PKG.test/androidx.test.runner.AndroidJUnitRunner" \
        > "$log" 2>&1 && grep -q "^OK (1 test)" "$log"; then ok "$label"
    else bad "$label ($(grep -m1 -E 'AssertionError|Exception|Error|INSTRUMENTATION_|Unable' "$log" | tr -d '\r' | cut -c1-160))"; fi
}

cleanup() {
    "$ADB" emu kill >/dev/null 2>&1
    [ -n "${tunnel:-}" ] && kill "$tunnel" 2>/dev/null
    lab "sh lab.sh appserver stop" >/dev/null 2>&1
    rm -rf "$work"
}
trap cleanup EXIT

# --- the lab garden: this branch's code, first address -----------------------------------------
GARDEN_LAB=$LAB sh "$repo/docs/rust-pages/sync.sh" >/dev/null && lab "sh lab.sh build cand" >/dev/null \
    && lab "sh lab.sh appserver old" >/dev/null && ok "lab garden at its first address" || { bad "lab garden"; exit 1; }
pw=$(lab "sed -n 's/^TEST_PW=//p' test-login.env")
ssh -o BatchMode=yes -N -L 18181:127.0.0.1:18181 -L 18182:127.0.0.1:18182 "$LAB" & tunnel=$!
sleep 3
pubkey=$(curl -s http://127.0.0.1:18181/app/config.json | sed -n 's/.*"key":"\([0-9a-f]\{64\}\)".*/\1/p')
[ ${#pubkey} = 64 ] && ok "garden's public key read from /app/config.json" || bad "public key"

# --- the two apps, built as throwaway test copies ----------------------------------------------
build() {   # build DIR GIT-REF|work TESTFILE   ("work" = this working copy, uncommitted changes too)
    mkdir -p "$1" || return 1
    if [ "$2" = work ]; then tar -C "$repo" -c --exclude=build --exclude=.gradle --exclude=local.properties android-app | tar -x -C "$1"
    else git -C "$repo" archive "$2" android-app | tar -x -C "$1"; fi || return 1
    cp -r "$here/overlay/src/debug" "$1/android-app/app/src/"
    mkdir -p "$1/android-app/app/src/androidTest/kotlin/org/jasonagibson/junsgarden"
    cp "$here/$3" "$1/android-app/app/src/androidTest/kotlin/org/jasonagibson/junsgarden/"
}
build "$work/old" cd4e526 OldMatrixTest.kt && python "$here/patch.py" \
    "$work/old/android-app/app/src/main/kotlin/org/jasonagibson/junsgarden/app/Updater.kt" \
    '"https://junsgarden.zapto.org/apps/version.json"' "\"$OLDNAME/apps/version.json\"" \
    '"https://junsgarden.zapto.org/apps/junsgarden.apk"' "\"$OLDNAME/apps/junsgarden.apk\"" \
    && (cd "$work/old/android-app" && ./gradlew -q assembleDebug assembleDebugAndroidTest -PgardenBase=$OLDNAME >"$OUT/logs/build-old.log" 2>&1) \
    && ok "old app (3.2, cd4e526) built, update addresses pointed at the lab's old name" || bad "old app build"
build "$work/new" work MatrixTest.kt && python "$here/patch.py" "$work/new/android-app/app/build.gradle.kts" \
    'versionCode = 18' 'versionCode = 19' \
    && (cd "$work/new/android-app" && ./gradlew -q assembleDebug assembleDebugAndroidTest -PgardenBase=$OLDNAME >"$OUT/logs/build-new.log" 2>&1) \
    && ok "new app built (version code 19)" || bad "new app build"
apk() { echo "$work/$1/android-app/app/build/outputs/apk/$2"; }

# --- a fresh emulator ----------------------------------------------------------------------------
if ! "$EMU" -list-avds | grep -qx "$AVD"; then
    echo no | "$SDK/cmdline-tools/latest/bin/avdmanager.bat" -s create avd -n "$AVD" \
        -k "system-images;android-36;google_apis;x86_64" -d pixel_7 >/dev/null 2>&1
fi
"$ADB" emu kill >/dev/null 2>&1   # a test emulator left from a stopped run
"$EMU" -avd "$AVD" -port $EPORT -no-window -no-audio -no-boot-anim -no-snapshot -wipe-data -gpu swiftshader_indirect >/dev/null 2>&1 &
"$ADB" wait-for-device
for _ in $(seq 120); do [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ] && break; sleep 2; done
[ "$("$ADB" shell getprop sys.boot_completed | tr -d '\r')" = 1 ] && ok "emulator booted (Android $("$ADB" shell getprop ro.build.version.release | tr -d '\r'))" || { bad "emulator"; exit 1; }
"$ADB" shell settings put global window_animation_scale 0; "$ADB" shell settings put global transition_animation_scale 0

# --- 1. the old app at the first address ---------------------------------------------------------
"$ADB" install -r "$(apk old debug/app-debug.apk)" >/dev/null && "$ADB" install -r "$(apk old androidTest/debug/app-debug-androidTest.apk)" >/dev/null
inst "1 old app, first address: login, list, plant, beds" OldMatrixTest#loginAndBrowse -e user Tester -e pw "$pw"

# --- 2. the garden moves --------------------------------------------------------------------------
lab "sh lab.sh appserver new" >/dev/null && ok "garden moved to the new address (old name answers phones, forwards pages)" || bad "move"
[ "$(curl -s -o /dev/null -w '%{redirect_url}' http://127.0.0.1:18181/catalog.php)" = "$NEWNAME/catalog.php" ] \
    && ok "old name forwards a page to the new name" || bad "old name forwards pages"
inst "2 old app NOT updated, after the move: login, list, plant, beds" OldMatrixTest#loginAndBrowse -e user Tester -e pw "$pw"

# --- 3. the old app finds the new one ---------------------------------------------------------------
newapk=$(apk new debug/app-debug.apk)
sha=$(sha256sum "$newapk" | cut -c1-64); md5=$(md5sum "$newapk" | cut -c1-32)
scp -q "$newapk" "$LAB:garden-rust-pages/app-www/apps/junsgarden.apk"
printf '{"versionCode":19,"versionName":"3.3-test","url":"%s/apps/junsgarden.apk","md5":"%s","sha256":"%s","notes":"test"}\n' \
    "$NEWNAME" "$md5" "$sha" > "$work/version.json"
scp -q "$work/version.json" "$LAB:garden-rust-pages/app-www/apps/version.json"
"$ADB" logcat -c
inst "3 old app's update check finds the new app; downloads it from the new name; md5 matches" \
    OldMatrixTest#updateToNewApp -e vc 19 -e apk "$NEWNAME/apps/junsgarden.apk"
# Handed to Android's installer? (Android's own log of the start; the prompt itself closes when
# the test's process ends, so the log is the proof, not a screenshot.)
"$ADB" logcat -d | grep -q "vnd.android.package-archive" \
    && ok "the APK was handed to Android's installer" || bad "handed to the installer"
# The emulator sometimes drops off adb for a while right after the installer opens. Wait until
# the phone answers again (and say so) instead of failing every later step on "device offline".
ready() {
    for _ in $(seq 90); do
        [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ] && return 0
        sleep 2
    done
    return 1
}
"$ADB" get-state >/dev/null 2>&1 || echo "     (the emulator dropped off adb after the installer opened; waiting for it)"
ready || { bad "emulator came back"; exit 1; }

# --- 4. installed over the old one ------------------------------------------------------------------
"$ADB" shell input keyevent KEYCODE_BACK
"$ADB" install -r "$newapk" >/dev/null && "$ADB" install -r "$(apk new androidTest/debug/app-debug-androidTest.apk)" >/dev/null \
    && ok "new app installed over the old one (same key: login and data kept)" || bad "update install"
inst "4a new app, never set up: built-in address" MatrixTest#addressIs -e expect "$OLDNAME"
inst "4b new app on the old name after the move: login, list, plant, beds, barcode, photo, thumb, comments" \
    MatrixTest#loginAndBrowse -e user Tester -e pw "$pw"

# --- 5. the setup QR, then the signed move --------------------------------------------------------------
"$ADB" shell am start -W -a android.intent.action.VIEW \
    -d "'junsgarden://setup?base=http%3A%2F%2F10.0.2.2%3A18181&key=$pubkey'" "$PKG" >/dev/null
sleep 4; shot "05-setup-link-toast"
inst "5a setup link saved the garden's address and key" MatrixTest#addressIs -e expect "$OLDNAME"
inst "5b app follows the signed move on its next contact" MatrixTest#followsMove -e expect "$NEWNAME"
"$ADB" shell am force-stop "$PKG"

# --- 6. on the new address ----------------------------------------------------------------------------
inst "6a new app on the new address: login, list, plant, beds, barcode, photo, thumb, comments" \
    MatrixTest#loginAndBrowse -e user Tester -e pw "$pw"
printf '{"versionCode":20,"versionName":"3.4-test","url":"%s/apps/junsgarden.apk","md5":"%s","sha256":"%s","notes":"test"}\n' \
    "$NEWNAME" "$md5" "$sha" > "$work/version.json"
scp -q "$work/version.json" "$LAB:garden-rust-pages/app-www/apps/version.json"
inst "6b self-update from the new address, sha256 checked" MatrixTest#updateFromCurrentAddress -e expect "$NEWNAME" -e vc 20
"$ADB" shell input keyevent KEYCODE_BACK

# --- 7. English and Korean ------------------------------------------------------------------------------
# No update on offer for the pictures (its dialog would cover the screens).
printf '{"versionCode":1,"versionName":"0","url":"%s/apps/junsgarden.apk","md5":"","sha256":"","notes":""}\n' \
    "$NEWNAME" > "$work/version.json"
scp -q "$work/version.json" "$LAB:garden-rust-pages/app-www/apps/version.json"
for lang in en-US ko-KR; do
    l=${lang%%-*}
    # Signed out, first start; then the app's own language (a clear would reset it, so it comes after).
    "$ADB" shell am force-stop "$PKG"; "$ADB" shell pm clear "$PKG" >/dev/null
    "$ADB" shell cmd locale set-app-locales "$PKG" --user 0 --locales "$lang" >/dev/null 2>&1
    "$ADB" shell am start -W -n "$PKG/.MainActivity" >/dev/null
    sleep 2; ui tap "Skip" "건너뛰기" || true
    sleep 2; ui tap "Later" "나중에" || true
    if [ "$l" = en ]; then words=("Sign In" "Continue as guest"); else words=("로그인" "손님으로 계속하기"); fi
    ui wait "${words[@]}" && ok "sign-in screen in $l" || bad "sign-in screen in $l"
    shot "07-$l-1-sign-in"
    ui tap "${words[1]}" && ui wait "Search plants" "식물 찾기" && ok "catalogue in $l" || bad "catalogue in $l"
    sleep 3; shot "07-$l-2-catalogue"
    ui tapfirstcard && sleep 4 && ui has "Catalog" "목록" && ok "plant page in $l" || bad "plant page in $l"
    shot "07-$l-3-plant"
done

echo "$pass passed, $fail failed (screenshots: $OUT)"
[ "$fail" = 0 ]
