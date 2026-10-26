# Local plant AI (the desktop helper)

**한국어는 아래에 있어요.**

The plant admin's **Add plant** tab asks for six things (genus, species, name, bed, quantity,
price). This helper fills in the rest, on a Windows PC with an NVIDIA card with **12 GB or more**
of graphics memory (like an RTX 4070 Ti), with two local models in [Ollama](https://ollama.com):

1. **Search** (polite: robots.txt, at least 3 seconds between two requests to one site, every page
   fetched once and cached): the grower's own product page (the **Source** link), and up to three
   trusted species pages (NC State Extension, universities, botanical gardens).
2. **Fill** with `phi4`: every field a page states, with the page lines that say so, normalised to
   the exact forms the garden's database uses (`plantai/norm.py`); where pages disagree, phi4 picks
   one and says why. Values that cannot be made valid are dropped.
3. **Photos**: up to 5, the largest original each page offers, each saved ONCE as a `.webp`
   (500 to 2000 px, quality 90, no metadata). No thumbnails. Logos, banners, blurry pictures and
   near-duplicates are skipped.
4. **Check** with `gemma4:12b`: is every value really in its quote, are the pages about this very
   plant (not a similar cultivar), and does each photo show it as the main subject.

Only values the checker confirmed are written, and only into **empty** fields; the rest is shown
under "Needs review". Before every write the website keeps a copy of the plant
(`plant_ai_backups`). Approving is the garden owner's own button.

## How it talks to the website

```
 laptop or PC with the website                     Windows PC with the graphics card
 ┌──────────────────────────────┐                 ┌───────────────────────────────┐
 │ Add plant  ->  plant_ai_jobs │  <-- asks every │ python -m plantai worker      │
 │ (queue)                      │      minute,    │   phi4 for all queued plants, │
 │ /plant-ai/jobs  (Caddy, its  │      outgoing   │   then gemma4:12b for all     │
 │ usual https address)         │  --> answers    │ Ollama on 127.0.0.1 only      │
 └──────────────────────────────┘                 └───────────────────────────────┘
```

No port is opened anywhere: the helper only makes outgoing requests, through the website's usual
address, with its own key (`Authorization: Worker gw_...`; the website stores only a SHA-256 of
it), and the website accepts them from the home network only. Plants added while the PC is off
wait in the queue. Work runs in two rounds per batch (all phi4 work, then all gemma work), so the
card never swaps models per plant (`OLLAMA_MAX_LOADED_MODELS=1`, `OLLAMA_NUM_PARALLEL=1`).

## Set-up

The Windows setup wizard does it (CUSTOM, "Local plant AI", shown only with a 12 GB card), or the
Windows menu, More, 22 (`windows/Garden-PlantAi.ps1`). **The Linux installer (`garden.sh`) never
installs AI models**: this is the one part the two installers do differently. The helper key is
made on the website (Add plant, Desktop helper, Make a helper key) and saved in
`plant-ai/worker-key.txt`, readable by your Windows user only.

## The same code for a batch

```
python -m plantai check                                  # Ollama and both models ready?
python -m plantai run --in plants.json --work W --out O  # a JSON array of plant rows
python -m plantai report --work W --out O                # rebuild the report files
python -m plantai worker --settings worker.json          # the desktop helper
```

Photo rules can be tuned with the garden's own `calib.json` in the work folder (keys:
`min_frame_share`, `max_distinct_plants`, `blur_min`, `min_px`, `max_px`). Example photos for the
checker are optional: `--reference-photos DIR` (up to 3 of your own good plant photos); none are
built in.

Tests (no network, no Ollama): `python -m pytest plant-ai/tests -q`.

## The scan queue (tags and plants scanned with the phone)

Guide: `START-HERE/27-scan-queue.md`. The same helper (`worker.py`) also takes **scans** from the
website, before it does plant jobs (`worker.run_round`). A scan is a tag (front, back) or a plant
photo with a bed (by number) and a count. For each one (`scans.py`):

```
 tagocr.py    OCR service (ocr/, Docker, 127.0.0.1:8788) -> lines; tagparse.py -> name, Latin name,
              size, grower, barcode. Weak reading -> gemma4:12b reads the picture (llm.ask).
 catalog.py   the garden's plants (GET /plant-ai/catalog): barcode, name, Latin name, fuzzy
 imgindex.py  the plant image index on 127.0.0.1:8099 (optional; a note when it is not running)
 webfind.py   DuckDuckGo + grower/reference pages, with discover.py/pages.py (polite, cached)
 matcher.py   clues combined (1 - product of doubts) -> best match + 2 to 4 likely, each with %
```

The garden lookup comes first; the photo index and the internet run at the same time in threads.
The website decides "matched" (best at 75% or more and 15 points ahead) or "needs you" and keeps
the scan until the garden owner taps one match (`scan_queue.rs`).

The tag reader (`ocr/`): `docker build -t garden-ocr plant-ai/ocr`, then the `docker run` line at
the top of `ocr/Dockerfile` (published on 127.0.0.1 only, read-only, no extra rights). The Windows
wizard does this (`windows/Garden-PlantAi.ps1`). Settings: `PLANTAI_OCR` (address),
`PLANTAI_OCR_ENGINE` (`rapid`, `tesseract` or `both`), `PLANTAI_IMAGE_INDEX` (address).

### How the tag reader was chosen

No real tag photos exist yet, so `python -m plantai synth --tags DIR --count 40 --seed 3` makes made-up
tags (3 layouts, 9 font families, tilt, perspective, blur, shadow, noise, JPEG; front and back; real
EAN-13 barcodes) and `python -m plantai ocr-eval --tags DIR` scores each engine on them.
Fresh 40 tags (parsing rules were tuned on other seeds), the share of tags read right:

| engine | name | Latin name | size | grower | barcode | all five |
|---|---|---|---|---|---|---|
| Tesseract | 47.5% | 62.5% | 80% | 82.5% | 100% | 22.5% |
| RapidOCR (PaddleOCR models, ONNX) | 92.5% | 100% | 100% | 100% | 100% | **92.5%** |
| both merged | 85% | 100% | 97.5% | 100% | 100% | 82.5% |

(name, Latin name, grower: at least 85% alike; size and barcode exact. Barcodes are decoded with
zbar, so they are exact in every row.) On 20 "hard" tags RapidOCR got all five on 70%. Seconds per
tag on the processor (front and back): Tesseract 1.6, RapidOCR 4.7. These are made-up tags: they show the
pipeline works, not the accuracy on Jun's tags. `gemma4:12b` as a reader was NOT measured (the
graphics card was busy with another run and no model was pulled); it is only asked for weak readings.

Lab test of the whole queue (a throw-away website and database in Docker, a fake phone, the real
helper code): `python lab/scan-queue/e2e.py`.

Not tested yet, because it needs a real phone, real tags or the graphics card:
- the Android screens on a phone (they compile, lint and unit-test, but nobody has tapped them);
  the phone camera app (TakePicture), the phone's own barcode read (ZXing), WorkManager sending
  with the app closed and on a weak signal;
- real nursery tags (glare, curved plastic, faded print, tags in pots, other languages);
- `gemma4:12b` as the second reader (not pulled; the card was busy) and the real image index
  (another branch; only its answer shape is tested);
- the Windows wizard's new tag-reader step on a real Windows install (the script parses; Docker
  Desktop on the wizard's PC was not exercised);
- a very long queue (hundreds of scans) and two phones scanning at once.

---

## 한국어

식물 관리 화면의 **식물 추가** 탭은 여섯 가지만 물어봐요 (속, 종, 이름, 화단, 수량, 가격). 나머지는 이
도우미가 채워요. 그래픽 메모리 **12GB 이상**의 NVIDIA 카드(예: RTX 4070 Ti)가 있는 Windows PC에서,
[Ollama](https://ollama.com)의 로컬 모델 두 개로요.

1. **찾기** (예의 바르게: robots.txt를 지키고, 한 사이트에 요청 사이 최소 3초, 페이지마다 한 번만 받아
   보관): 재배 회사의 제품 페이지(**출처** 링크)와 믿을 만한 종 페이지 최대 세 개.
2. `phi4`로 **채우기**: 페이지에 적힌 칸마다 그 근거가 되는 줄과 함께, 정원 데이터베이스가 쓰는 형식
   그대로 맞춰요. 맞출 수 없는 값은 버려요.
3. **사진**: 최대 5장, 페이지에 있는 가장 큰 원본을 `.webp` 파일 하나로 (500~2000px). 작은 사본은 만들지
   않아요.
4. `gemma4:12b`로 **확인하기**: 값이 정말 그 인용에 있는지, 페이지가 바로 이 식물인지, 사진이 이 식물을
   주인공으로 보여 주는지.

확인된 값만, **빈** 칸에만 써요. 나머지는 '검토 필요'에 보여요. 쓰기 전마다 웹사이트가 식물의 사본을
남겨요. 승인은 정원 주인만 누를 수 있어요.

어디에도 새로 여는 포트가 없어요: 도우미는 웹사이트의 평소 주소로 밖으로만 요청하고, 자기 키를 쓰고,
웹사이트는 집 네트워크에서 온 요청만 받아요. PC가 꺼져 있을 때 추가한 식물은 대기열에서 기다려요.

설정: Windows 설치 마법사(CUSTOM, '로컬 식물 AI', 12GB 카드가 있을 때만 보여요) 또는 Windows 메뉴
More, 22. **Linux 설치 프로그램(garden.sh)은 AI 모델을 절대 설치하지 않아요.** 도우미 키는 웹사이트에서
만들어요 (식물 추가, 데스크톱 도우미, 도우미 키 만들기).

**스캔 대기열** (폰으로 찍은 이름표와 식물): 안내서 `START-HERE/ko/27-scan-queue.md`. 같은 도우미가
웹사이트의 **스캔**도 가져와요 (식물 작업보다 먼저). 스캔마다 `tagocr.py`가 이름표 글자를 읽고(Docker의
`ocr/`, 이 PC의 127.0.0.1:8788, 약한 읽기는 gemma4:12b가 사진으로 읽어요), `catalog.py`가 내 정원에서,
`imgindex.py`가 사진 색인(있으면)에서, `webfind.py`가 인터넷(DuckDuckGo, 재배 회사 페이지)에서 찾고,
`matcher.py`가 가장 가까운 것과 가능성 있는 것 2~4개를 확신도와 함께 만들어요. 정원 주인이 한 번 누를 때까지는
아무것도 추가되지 않아요. 이름표 읽기 프로그램은 가짜 이름표 40장으로 재서 골랐어요 (RapidOCR: 다섯 칸 모두
맞은 것 92.5%, Tesseract 22.5%). 진짜 이름표로는 더 낮을 수 있어요.
