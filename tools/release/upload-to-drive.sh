#!/bin/bash
# =============================================================================
#  upload-to-drive.sh: how the three downloads go into the Google Drive folder,
#  and how they are checked there. Kept here so anyone can see exactly what is
#  uploaded; nothing else is.
#
#     GARDEN_DRIVE="myremote:Jun's Garden" bash tools/release/upload-to-drive.sh RELEASE-FOLDER
#
#  GARDEN_DRIVE is an rclone remote and a folder in it (rclone's Google Drive
#  setup: https://rclone.org/drive/). The remote's name, the folder's link and
#  any keys stay on the computer that uploads: none of them are in this package.
#  RELEASE-FOLDER is what build_release.py made (the three zips).
#
#  What it does:
#   1. Uploads the three downloads (only new or changed files are sent:
#      compared by fingerprint, --checksum).
#   2. Checks every uploaded file against the one here (rclone check: Google
#      Drive's own MD5 fingerprint of each file must match).
#   3. Prints the SHA-256 fingerprints for DOWNLOADS-SHA256.txt, so anyone who
#      downloads can check a file is exactly the published one.
#  The PDF guides are not uploaded here: they are two zips attached to a GitHub
#  release (make_pdf_zips.py) that the installer fetches (tools/get-pdfs.*). A Drive folder still holding the old "Setup documentation"
#  PDFs is reported; remove it there (rclone purge moves it to Drive's trash).
#  Sharing the folder ("anyone with the link") is done once, by hand, in Google Drive.
# =============================================================================
set -euo pipefail
DRIVE=${GARDEN_DRIVE:?set GARDEN_DRIVE to your rclone remote and folder, for example myremote:Garden}
REL=${1:?give the release folder that build_release.py made}
RCLONE=${RCLONE:-rclone}
ZIPS="junsgarden-for-jun.zip junsgarden-photos-addon.zip junsgarden-wiki-addon.zip"

for z in $ZIPS; do [ -f "$REL/$z" ] || { echo "missing: $REL/$z"; exit 1; }; done
echo "1. Uploading the downloads to $DRIVE"
for z in $ZIPS; do "$RCLONE" copyto "$REL/$z" "$DRIVE/$z" --checksum --drive-chunk-size 64M; done

echo "2. Checking every file on Drive against the one here"
for z in $ZIPS; do "$RCLONE" check "$REL" "$DRIVE" --include "$z" --one-way; done
if "$RCLONE" lsf "$DRIVE" --dirs-only 2>/dev/null | grep -qx 'Setup documentation/'; then
    echo "   NOTE: $DRIVE still has the old 'Setup documentation' PDF folder. The PDFs now come"
    echo "   with the program. To move it to Drive's trash:  $RCLONE purge \"$DRIVE/Setup documentation\""
fi

echo "3. Fingerprints (SHA-256) for DOWNLOADS-SHA256.txt"
( cd "$REL" && for z in $ZIPS; do printf '%s  %s (%s bytes)\n' "$(sha256sum "$z" | cut -c1-64)" "$z" "$(wc -c < "$z" | tr -d ' ')"; done )
