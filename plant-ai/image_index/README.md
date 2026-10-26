# Image index: find a plant from a photo

**한국어는 아래에 있어요.**

Give it a photo of a plant; it lists the plants in the garden's own database whose photos look most
like it. It uses only photos that are already there (the plants growing in the garden, and the
wish-list plants the plant AI collected). It runs on this computer's CPU, in Docker, with an open
model that works offline. Nothing leaves the computer. **The photos and the index are the garden
owner's data: they stay on the local disk (`_local-data`, never in git, never in the cloud). Only
this code is in the repo.**

## What it gives you

```python
from image_index import build, search, status
build(photo_dirs, db_path, out_dir, workers)   # new or changed photos only; safe to stop and re-run
search(image_bytes_or_path, k=5)               # best plants first:
# [{plant_id, plant_name, botanical_name, score (0-1), photo, set ('current'|'future'),
#   confidence (0-1, a rough chance, see below)}]
```

* `score` is how alike the best photo of that plant is (cosine similarity, 1 = identical). It is
  **not** a chance of being right: see "How good is it" below. `confidence` is the plant's share
  among all plants (a softmax of the scores, its temperature fitted by `eval --write-temperature`
  on the visit-out test); it is only a guide, and on phone photos it will be too sure.
* `photo` is the matching photo, as a path inside its photo folder.
* `search(..., hints="text")` is the **re-rank with text** hook: pass what was read from a plant
  tag (OCR) or a name; plants whose names share words get up to +0.20 on `score` (rare words weigh
  more than common ones, small spelling mistakes still match). Results then also carry
  `text_match` (0-1).
* One plant can appear twice, once per `set`, when it is both growing and on the wish list.

Command line (inside Docker, see below) and a small service:

```
python -m image_index build --photos current=DIR --photos future=DIR --db EXPORTS [--workers N]
python -m image_index search photo.jpg -k 5 [--hint "Salvia nemorosa"]
python -m image_index status
python -m image_index eval [--write-temperature]
python -m image_index serve --port 8099 --bind 127.0.0.1
```

`serve`: `POST /search?k=5&hints=text` with the picture's raw bytes -> `{"results": [...], "ms": n}`;
`GET /status`. No password, because it only answers this computer: it refuses any bind address that
is not loopback, refuses requests with a non-local `Host` (DNS rebinding) or an `Origin` header
(a web page), and takes pictures up to 25 MB.

## Run it (nothing is installed on Windows)

```
docker build -t gfa-image-index -f plant-ai/image_index/Dockerfile plant-ai    # once; needs internet once
export GFA_DATA="D:/Garden for all/_local-data"      # the folder with in/ and images/ (local data)
sh plant-ai/image_index/index.sh build               # again whenever more photos arrived
sh plant-ai/image_index/index.sh status
sh plant-ai/image_index/index.sh eval                # the accuracy test below
sh plant-ai/image_index/index.sh search some-photo.jpg
PORT=8199 sh plant-ai/image_index/index.sh serve     # http://127.0.0.1:8199  (stop: index.sh stop)
NETWORK=ai-stack_default sh plant-ai/image_index/index.sh serve   # no port at all: http://plant-image-index:8099
```

`build` runs with no network, read-only, and only the index folder writable. `serve` is published
as `127.0.0.1` only. **Host port 8099 is already taken by another container (voice-f5) on the
author's PC**, so use `PORT=` (any free one) or the Docker network name. Inside the container the
program must listen on `0.0.0.0` for the published port to reach it (`IMAGE_INDEX_IN_DOCKER=1`,
set by `index.sh`); the safety is the `127.0.0.1:` in the port mapping.

**How photos are matched to plants** (`catalog.py`): `db_path` is the folder with `current.json`
(the plants table) and `future.json` (the wish list). A current photo is matched by the file names
in the plant's picture columns, or by the plant id in its file name (`Name_218_manual_1779....webp`).
A future photo lives in `images/<future plant id>/<n>.webp`. Photos that match no plant are
listed in the build summary and skipped. Re-running `build` embeds only new or changed files
(size + modified time), forgets files that are gone, and picks up name changes without redoing
anything. Stopping a build loses nothing already saved (it saves every 256 photos).

## How it works, in short

1. Each photo is shrunk to a 336 x 336 square (the middle part) and the model **DINOv2-small**
   (Meta, Apache-2.0, 22 million numbers, `facebook/dinov2-small` pinned to one revision, in the
   image so it works offline) turns it into 384 numbers: its "look".
2. Search: the query photo gets its 384 numbers too; one matrix product against all stored photos
   gives a similarity per photo (about 6 ms for 50,000 photos); each plant scores as its best
   photo; the best plants are returned. No FAISS needed at this size.
3. The index is one SQLite file (`index.db`, vectors stored as 16-bit numbers, about 0.8 KB per
   photo: 50,000 photos is about 40 MB).

## Which model (measured here, 1,775 photos, same test for all)

Top-1 / top-5 over plants, "leave one photo out", current plants as test photos, all other photos
(also the wish-list ones) as the searchable collection:

| model | size px | top-1 | top-5 | images/sec on the CPU* |
|---|---|---|---|---|
| **DINOv2-small, class token** | 224 | 0.566 | 0.842 | 17 to 28 |
| **DINOv2-small, class token (chosen)** | 336 | 0.597 | 0.863 | 11 |
| DINOv2-small, class + mean of patches | 224 | 0.545 | 0.824 | 8 to 22 |
| DINOv2-base | 224 | 0.549 | 0.843 | 8 |
| SigLIP-base | 224 | 0.550 | 0.832 | 5 to 8 |
| CLIP ViT-B/32 | 224 | 0.390 | 0.695 | 13 |

DINOv2-small and the bigger models are equal within noise; the small one is the smallest and
fastest, so it is the pick. Joining DINOv2 + SigLIP gave +2 points for double the time: not worth
it. *The PC was busy with a live LLM run and another container (speeds jumped between 5 and 28
images/sec for the same job); treat speeds as rough.

## How good is it (honest)

`python -m image_index eval` on the current index (471 current plants and 201 wish-list plants have
photos; 2,114 photos):

| test | test photos | top-1 | top-5 | top-1 right genus | top-1 right species |
|---|---|---|---|---|---|
| leave one photo out (a near-identical photo of the same plant may remain) | 951 | 0.60 | 0.86 | 0.87 | 0.73 |
| leave one visit out (no photo of that plant taken within 30 minutes) | 138 | 0.23 | 0.57 | 0.67 | 0.54 |

The second test is the fair one, but it is small (only 21 plants have photos from two different
times). "Wrong" top-1 is very often a sibling: another cultivar of the same species.

**Important: none of this is a phone photo in a bed.** The garden's current photos are mostly
clean, centred photos from plant sellers and the web, not photos taken in the garden. A degraded
copy test (random crop, turn, colour, blur, JPEG) scored the same as the clean one, so the model
shrugs off those changes, but a real photo has clutter, other plants, shade and a tiny subject, and
that cannot be measured without real photos.

**What to expect from a phone photo of one plant in a bed** (an estimate, not a measurement):
the right plant first about 1 time in 5 (0.15 to 0.30), in the first five about 1 time in 2
(0.35 to 0.55), the right genus first about half the time. It is a good "shortlist" and a bad
"answer". It can only find plants whose photos are in the index (now about 670 of the roughly
7,000 plants in the database; the wish list grows as the plant AI collects photos, so run `build`
again). A plant tag text hint helps a lot: with the right name in the hint, top-1 went from 0.51
to 0.72 in the test (111 plants, hint = the botanical name with a letter missing); a wrong hint
(another plant's name) did not push the right plant out of the top five (0.77 with and without
it). To measure the real number, have the
scan queue keep which suggestion the person confirmed; after about 50 scans the true top-1 / top-5
is known and `confidence` can be fitted on it.

## Files

`catalog.py` which photo is which plant, `decode.py` open and shrink a photo, `embed.py` the model,
`models.py` the model list (pinned), `store.py` the SQLite index, `indexer.py` build,
`finder.py` search and the text hook, `evaluate.py` the accuracy test, `server.py` the local
service, `cli.py`, `Dockerfile`, `requirements.txt` (exact versions and checksums, rebuild steps
in `requirements.in`), `index.sh`. Tests (no model needed): `python -m pytest plant-ai/tests/test_image_index.py -q`.

---

## 한국어

사진 한 장을 주면, 정원 데이터베이스에 이미 있는 사진 중 가장 비슷한 식물을 찾아 줘요. 이미 있는 사진만
써요 (정원에 심은 식물, 그리고 식물 AI가 모은 위시리스트 식물). 이 컴퓨터의 CPU에서, Docker 안에서,
오프라인으로 돌아가는 공개 모델로 실행하고, 아무것도 컴퓨터 밖으로 나가지 않아요. **사진과 색인은 정원
주인의 자료예요: 로컬 디스크(`_local-data`)에만 있고, git이나 클라우드에는 절대 올리지 않아요. 저장소에는
코드만 있어요.**

* 사용법과 실행 명령은 위 영어 부분과 같아요 (`index.sh build`, `status`, `eval`, `search`, `serve`).
* `search`는 `score`(0~1, 얼마나 닮았는지)와 `confidence`(대략적인 가능성)를 주고, 식물 태그에서 읽은
  글자를 `hints`로 주면 이름이 맞는 식물의 점수를 올려 줘요.
* 정확도(솔직하게): 같은 식물의 다른 사진을 찾기 어렵게 한 공정한 시험에서 1순위 약 23%, 5순위 안에
  약 57%. 정원의 현재 사진은 대부분 판매처/웹 사진이라, **정원에서 폰으로 찍은 사진은 이 시험으로 잴 수
  없어요.** 예상은 1순위 15~30%, 5순위 안 35~55%: "정답"이 아니라 "후보 목록"으로 쓰세요.
* 서비스는 이 컴퓨터(127.0.0.1)에서만 답하고 비밀번호가 없어요. 이 PC에서는 8099 포트를 다른 컨테이너가
  이미 쓰고 있어서 `PORT=8199` 같은 다른 포트나 Docker 네트워크 이름을 써요.
