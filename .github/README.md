# Jun's Garden

A self-hosted garden website you run on your own computer at home: a plant
catalogue with photos, a blog, garden tools, an optional wiki and an Android
app, plus an installer that sets it all up step by step and explains every step.
**한국어 안내는 아래에 있어요.**

This page is here so that **if you need help while the rest of the files are
still coming in, you already have a guide.**

## Start here

1. **Read the first guide:** [START-HERE/00-READ-ME-FIRST.md](../START-HERE/00-READ-ME-FIRST.md)
   (한국어: [START-HERE/ko/00-READ-ME-FIRST.md](../START-HERE/ko/00-READ-ME-FIRST.md)).
2. **Windows:** double-click `Start-Here-Windows.cmd`. **Ubuntu:** run `bash Start-Here-Ubuntu.sh`.
   The first start downloads the program from this page's repository and checks
   every file before it is used.
3. Follow the setup wizard (Windows) or the menu (Ubuntu). Every step is
   explained before it runs, and each page has a **Help** button.

## If something does not work

- **The first start says it cannot download:** the program is fetched from
  GitHub, so check that the computer is online and run the start file again. It
  writes nothing if a file does not match, so running it again is safe.
- **You are stuck on a step:** open [guide 9, Troubleshooting](../START-HERE/09-troubleshooting.md)
  or [guide 25, Common questions](../START-HERE/25-common-questions.md)
  (한국어: [9](../START-HERE/ko/09-troubleshooting.md), [25](../START-HERE/ko/25-common-questions.md)).
- **You want to read a guide on paper or a phone:** every guide is a PDF, an optional download the installer fetches while it works
  (about 27 MB in English, 25 MB in Korean, checked by fingerprint; menu item `P` gets them later). The same guides
  are on the website at `/help`.
- **Once the garden is running,** the same guides are on your own site at
  `/help` (after you sign in), with a short overview first and the full guide
  one click away.

## Where things are

| What | Where |
|---|---|
| Every guide, English and Korean | [`START-HERE/`](../START-HERE/) |
| Every guide as a PDF (optional download) | [`PDF-GUIDES-SHA256.txt`](../PDF-GUIDES-SHA256.txt), [`tools/get-pdfs.py`](../tools/get-pdfs.py) |
| The installer and menu (Ubuntu) | [`garden.sh`](../garden.sh) |
| The installer and menu (Windows) | [`windows/`](../windows/) |
| The website (Rust) | [`rust/`](../rust/) |
| Every change, and why | [`CHANGES.md`](../CHANGES.md) |

The garden's own data (plants database, photos, wiki pages) is not in this
repository. It comes in the owner's separate, much smaller download.

## 한국어

집에 있는 컴퓨터에서 직접 운영하는 정원 웹사이트예요. 나머지 파일이 아직 내려받아지는 동안에도
도움이 필요하면 바로 볼 수 있는 안내가 여기 있어요.

1. 먼저 읽기: [START-HERE/ko/00-READ-ME-FIRST.md](../START-HERE/ko/00-READ-ME-FIRST.md)
2. Windows는 `Start-Here-Windows.cmd`를 두 번 클릭하고, Ubuntu는 `bash Start-Here-Ubuntu.sh`를 실행하세요.
   처음 시작할 때 프로그램을 이 저장소에서 내려받고 모든 파일을 확인한 뒤에 사용해요.
3. 설정 마법사(Windows)나 메뉴(Ubuntu)를 따라가세요. 모든 단계는 실행 전에 설명해 주고,
   페이지마다 **도움말** 버튼이 있어요.

**잘 안 될 때:** 내려받기가 안 되면 컴퓨터가 인터넷에 연결되어 있는지 확인하고 시작 파일을 다시
실행하세요(파일이 맞지 않으면 아무것도 쓰지 않으니 다시 실행해도 안전해요). 막힌 단계는
[문제 해결](../START-HERE/ko/09-troubleshooting.md)과 [자주 묻는 질문](../START-HERE/ko/25-common-questions.md)을
보세요. 모든 안내서는 PDF로도 있어요(설치하는 동안 받는 선택 사항, 영어 약 27 MB, 한국어 약 25 MB, 나중에는 메뉴 `P`).
정원이 실행되면 같은 안내서를 내 사이트의 `/help`에서(로그인 후) 볼 수 있어요.
