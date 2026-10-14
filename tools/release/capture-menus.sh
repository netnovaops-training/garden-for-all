#!/bin/bash
# shellcheck disable=SC2016,SC2012  # each check runs in its own bash -c on purpose; ls only shows a folder
# =============================================================================
#  capture-menus.sh: the real output of the newest menus, for the guides'
#  pictures (make_menu_pictures.py draws them). Run it in an installed garden
#  on a TEST computer: it renames the gallery folder and back once (a website
#  rebuild, a few minutes) to show a real rename.
#
#     bash tools/release/capture-menus.sh OUTPUT-FOLDER
#
#  This computer's name and address are replaced by the samples the guides use
#  (garden-lab, 192.168.1.50); make_menu_pictures.py replaces any email address.
# =============================================================================
set -u
OUT=${1:?give a folder for the captured text}
cd "$(dirname "$0")/../.." || exit 1
mkdir -p "$OUT"
H=$(hostname); IP=$(hostname -I | awk '{print $1}')
clean() { sed -e "s/$H/garden-lab/g" -e "s/$IP/192.168.1.50/g" -e 's/\r//g'; }
for _ in $(seq 1 60); do curl -fs -o /dev/null http://localhost:8030/healthz && break; sleep 3; done

printf 'M\n0\n0\n' | bash garden.sh 2>&1 | clean | awk '/Jun.s Garden  -  More/{f=1; print prev} f{print} f && /^ *0\) Back/{exit} {prev=$0}' > "$OUT/more.txt"
printf '8\n0\n0\n' | bash garden.sh 2>&1 | clean | awk '/== Settings/{f=1} f{print} f && /Every setting is saved/{exit}' > "$OUT/settings.txt"
printf '0\n' | bash garden.sh rename 2>&1 | clean > "$OUT/rename-menu.txt"
# The preview, answered "n": nothing changes.
printf '1\nlocal/images/Gallery\nPhotos\nn\n' | bash garden.sh rename 2>&1 | clean | awk '/Which file or folder/{f=1} f{print}' > "$OUT/rename-preview.txt"
# A real rename, then back again.
printf '1\nlocal/images/Gallery\nPhotos\ny\nn\ny\n' | bash garden.sh rename 2>&1 | clean > "$OUT/rename-full.txt"
printf '1\nlocal/images/Photos\nGallery\ny\nn\ny\n' | bash garden.sh rename >/dev/null 2>&1
printf '0\n' | bash garden.sh power 2>&1 | clean > "$OUT/switch.txt"
# The clean-up with one pretend old version of the garden's website image (answered "n"), removed afterwards.
docker pull -q hello-world >/dev/null; docker tag hello-world junsgarden/rs:old-version
printf 'n\n' | bash garden.sh cleanup 2>&1 | clean > "$OUT/cleanup.txt"
docker rmi junsgarden/rs:old-version hello-world >/dev/null 2>&1
ls local/images | tr '\n' ' '; echo
wc -l "$OUT"/*.txt
