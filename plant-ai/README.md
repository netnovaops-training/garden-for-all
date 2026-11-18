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
