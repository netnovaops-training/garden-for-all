# Jun's Garden: the handover, in one page

The details of every change are in [`CHANGES.md`](CHANGES.md) (what, why, and the
effect on the program), the story in plain words is in [`JOURNAL.md`](JOURNAL.md),
and every step is in the git history. This page is the short version.

## The scope

**The goal:** hand the garden website [junsgarden.zapto.org](https://junsgarden.zapto.org)
to its owner, Jun, so he can run it on his own computer, by himself: fully
transparent, free, public, and with no way in for anyone but him.

**In scope:**

- The live website (Rust), the garden tools (`/beds`), the optional wiki, and the
  Android app's source.
- An installer that sets everything up and explains every step: `garden.sh` on
  Ubuntu (14 resumable steps, EASY or CUSTOM, every option in a menu), and the
  Garden Bridge on Windows (a guided installer with Next and Back).
- Guides in English and Korean, each with a quick version, as PDFs and as a web
  handbook that opens from the disk, and a welcome page with every guide.
- Backups, going online with his own web name, a sign-in page, remote access,
  and the tests that prove each part works.

**Out of scope, on purpose:**

- The old PHP site, the old household server and its services, and every login,
  key and account of the previous owner.
- The garden's own data (plants database, photos, wiki pages): a separate,
  private download for Jun, not in this repository.
- Planned for later: a tool to rename the website's folders and everything that
  points at them.

## Enhancements

- **The install does the work.** Every program is installed by the install itself,
  with each step explained (what, why, the official page) and its commands shown
  before they run, a timer on each step, and a full log. It can stop and carry on
  where it stopped.
- **Three levels to choose from**, so Jun can plan his time: Easy (beer, about 1-2
  hours), Recommended (wine, about 2-4 hours over several days) and Full
  (champagne, about 3-5 hours), each with its chance of a frustrating moment
  (3, 5 and 8 out of 10). Guide 23 is the full setup, part by part.
- **Windows:** the Garden Bridge installer; it offers to move the garden folder
  out of OneDrive; a welcome page opens when the install ends.
- **Ownership:** no built-in account; Jun's own keys, his own No-IP name, his own
  Google client ID; the Website name swap changes the web name in every file at once.
- **Email for "forgot password":** through Gmail with an app password, with the two
  Google pages shown where it asks, and a second "are you sure?" before skipping.
- **Backups, 3-2-1:** every night, to up to three network places (NAS, a second NAS,
  a PC's shared folder), Google Drive, and a Windows folder Jun picks (a USB drive,
  Google Drive for desktop, a NAS).
- **Remote access:** SSH with keys, WinSCP for files between Linux and Windows,
  Termius for a terminal on the phone, and Tailscale (menu M, 17) to reach the
  computer from anywhere with no port opened.
- **Guides:** 28 in English and 26 in Korean, plus 12 project guides; a beginner's
  bash walkthrough; 41 pictures taken from the real program, with sample names and
  rings on what to click; PDFs with a colour per step; a web handbook with a
  step > guide > topic tree for when Jun is stuck.
- **Korean** on screen too: the menu speaks English, Korean or both (every one of
  its sentences is translated).

## Fixes

Testing every option on Ubuntu, on Windows (the Garden Bridge, downloading from
Google Drive as Jun will) and on a Raspberry Pi 5 found **40 problems**, all fixed
and each tested again (`CHANGES.md`, Parts 2 and 5). A few examples:

- The firewall step could have locked out SSH on a computer whose SSH uses
  another port; it now keeps the real port open, from home only.
- Once Docker was installed, "home Wi-Fi only" let only Docker's own network in.
- "Remove everything" left the programs behind; it now removes all it lists.
- Yes/no checks could answer a wrong "no" when a program's output was long.
- A Korean Ubuntu's folders (다운로드, 바탕화면) were not searched for the downloads.
- Moving the garden folder out of OneDrive would have broken the scheduled backup copy.

## Security

A separate review read every file and the whole history before publishing. The key
answer: **someone who reads this code, even knowing Jun's name and email, cannot
sign in as Jun or anyone else.** No passwords, keys or accounts are in the code;
every install makes its own secrets; only the front door (ports 80 and 443) faces
the internet. Ten improvements came from the review (`CHANGES.md`, Part 4).

## Checked before release

- Website: 64 of 64 tests pass. Shell scripts: shellcheck clean (18 files).
- The menu's Korean: every sentence translated. Korean guides: 0 differences from
  the English in links, commands and pictures.
- 219 outside links checked, 0 broken. Secret scan (gitleaks, semgrep): clean.
- Install tested: Ubuntu (Windows' own Ubuntu), the Windows path from Google Drive,
  and a Raspberry Pi 5 (arm64; the build-and-start part, 8 minutes, 0 fails).

---

## 한국어 요약

준의 정원 웹사이트를 주인인 준에게 넘겨서, 준이 자기 컴퓨터에서 혼자 운영할 수 있게 한
프로젝트예요. 투명하고, 무료이고, 공개되어 있고, 준 말고는 아무도 들어올 수 없어요.

- **범위:** 웹사이트(Rust), 정원 도구, 선택으로 위키, Android 앱 소스, 설치 프로그램
  (Ubuntu의 `garden.sh`, Windows의 Garden Bridge), 영어·한국어 안내서(PDF와 웹 안내서),
  백업, 인터넷에 올리기, 원격 접속, 그리고 모든 테스트. 예전 서버와 그 계정·키, 그리고
  정원의 데이터(따로 내려받는 파일)는 포함하지 않아요.
- **좋아진 것:** 설치가 모든 일을 하고 단계마다 설명해요. 세 단계(맥주·와인·샴페인)로
  시간을 계획할 수 있어요. 이메일 비밀번호 찾기, 3-2-1 백업, Tailscale로 어디서든 나만 접속.
- **고친 것:** 테스트에서 찾은 문제 40개를 모두 고치고 다시 확인했어요(`CHANGES.md`).
- **보안:** 이 코드를 읽어도, 준의 이름과 이메일을 알아도, 준으로 로그인할 수 없어요.
