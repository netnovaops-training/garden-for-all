# Jun's Garden

A complete, self-hosted garden website, the kind you run on your own computer at
home: a plant catalogue with photos, a blog, garden tools, an optional wiki, and
an Android app. Plus an installer that sets all of it up, step by step, on an old
laptop, a Raspberry Pi or a Windows PC, explaining every step as it goes.

This is the full code of [junsgarden.zapto.org](https://junsgarden.zapto.org),
handed over to its owner, Jun, to run on his own computer. **한국어 요약은 아래에 있어요.**

Its public home: <https://github.com/netnovaops-training/garden-for-all>.

## The story
- **PHP to Rust, faithfully.** The site began as a PHP website. In October 2026
  it was converted, page by page, to Rust (axum, askama, sqlx): the same look,
  the same addresses, the same features, checked against the original screens.
  It went live on Rust on 2026-10-06. The conversion notes are in
  [`docs/rust-port/`](docs/rust-port/) and [`docs/streamlit-port/`](docs/streamlit-port/).
- **A package for one person.** Then everything that belonged to the old server
  was taken out (its logins, keys and household links) and the whole site was
  wrapped in an installer, guides in English and Korean, and a security review,
  so Jun can own and run it himself. [`CHANGES.md`](CHANGES.md) lists every change
  and why; [`JOURNAL.md`](JOURNAL.md) tells the story in plain words; the git
  history has every step. **The one-page summary: [`HANDOVER-SUMMARY.md`](HANDOVER-SUMMARY.md).**

## What is in here

| Folder or file | What it is |
|---|---|
| [`garden.sh`](garden.sh) | The installer and menu for Ubuntu: 14 resumable steps, Easy or Custom, each explained before it runs |
| [`windows/`](windows/) | A guided installer (with Next/Back buttons) and a menu for Windows |
| [`rust/`](rust/) | The website (Rust). Built on your own computer |
| [`beds/`](beds/) | Garden tools: bed planner, checkup, tag scan (Python) |
| [`android-app/`](android-app/) | The Android app (Kotlin) |
| [`docker/`](docker/), [`docker-compose.yml`](docker-compose.yml) | Each program in its own container; only the front door faces the internet |
| [`START-HERE/`](START-HERE/) | Guides 00 to 26, each with a quick version first; Korean in [`START-HERE/ko/`](START-HERE/ko/) |
| [`Korean guides (PDF)/`](Korean%20guides%20(PDF)/) | 한국어 안내서 PDF: every Korean guide as a PDF, short and full, to read or print |
| [`tools/release/`](tools/release/) | How the downloads, PDFs and the web handbook are built (`build_release.py`), and the checks run before each release |

The garden's own data (plants database, photos, wiki pages) is **not** in this
repository; it is a separate private download for the owner. Without it, the
installer sets up an empty garden of your own.

One song is left out of this public copy too: the music behind the
"End of Garden. Boogie?" button (`site/Boogie.mp3`), because its rights are not
known. The button still works, silently; put your own `site/Boogie.mp3` there
if you like.

## Try it
On Ubuntu 24.04 (tested on ordinary 64-bit PCs and laptops, and Windows' own Ubuntu; also tested on a Raspberry Pi 5 with 64-bit Raspberry Pi OS, where it builds and runs on ARM; use a fan, it gets hot while building):

```
bash garden.sh
```

then choose `1` (install). On Windows, double-click `Start-Here-Windows.cmd`.
Start with [`00-READ-ME-FIRST.html`](00-READ-ME-FIRST.html) and
[`START-HERE/00b-before-you-start.md`](START-HERE/00b-before-you-start.md).

## Security, in short
- No passwords, keys or accounts are in this code or its history. Every install
  makes its own secrets, and nobody has a built-in login.
- Only Caddy (the front door, ports 80 and 443, HTTPS from Let's Encrypt) is
  reachable from the internet; every other program listens on the computer itself.
- The code was reviewed before publishing; the results and the fixes are in
  [`CHANGES.md`](CHANGES.md), Part 4.
- Check it yourself, or with any AI: [`START-HERE/18-review-the-code.md`](START-HERE/18-review-the-code.md).
  `sh tools/check-secrets.sh` runs gitleaks and semgrep.

## License
MIT: free to use, change and share. See [`LICENSE`](LICENSE).

---

## 한국어 요약
준의 정원(Jun's Garden)은 집에 있는 컴퓨터에서 직접 운영하는 정원 웹사이트예요: 식물 목록과 사진,
블로그, 정원 도구, 선택으로 위키, 그리고 Android 앱. 오래된 노트북, Raspberry Pi, Windows PC에
한 단계씩 설치해 주고, 단계마다 무엇을 왜 하는지 설명하는 설치 프로그램도 있어요.

- 처음에는 PHP 사이트였고, 2026년 10월에 Rust로 한 페이지씩 똑같이 옮겼어요(같은 모양, 같은 주소, 같은 기능).
- 그다음 예전 서버의 로그인, 키, 링크를 모두 빼고, 준이 직접 가지고 운영할 수 있게 설치 프로그램과
  영어·한국어 안내서를 만들었어요.
- 이 코드나 기록에는 비밀번호, 키, 계정이 하나도 없어요. 설치할 때마다 새 비밀 정보를 만들어요.
- 시작: [`00-READ-ME-FIRST-KO.html`](00-READ-ME-FIRST-KO.html), 안내서: [`START-HERE/ko/`](START-HERE/ko/). 한 페이지 요약: [`HANDOVER-SUMMARY.md`](HANDOVER-SUMMARY.md).
- MIT 라이선스: 자유롭게 쓰고, 바꾸고, 나눌 수 있어요.
