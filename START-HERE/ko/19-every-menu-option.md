# 19. 메뉴의 모든 선택, 하나씩 알아보기

> 영어 원문: [../19-every-menu-option.md](../19-every-menu-option.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 모든 메뉴의 모든 선택을 찾아보는 목록이에요: 무엇을 하는지, 언제 쓰는지, 무엇을 바꾸는지, 어떻게 되돌리는지.
- 메뉴 열기: 바탕화면의 "Jun's Garden" 아이콘, 또는 정원 폴더에서 `bash garden.sh`. Windows에서는 `Garden-Menu-Windows.cmd`.
- 모든 선택은 무언가를 바꾸기 전에 화면에서 스스로 설명하고, 명령을 실행하기 전에 보여 줘요.
- 평소에 쓰는 선택: `1` 설치(또는 이어서 하기, 고치기), `2` 시작, `3` 멈추기, `4` 웹사이트 열기, `5` 점검표, `6` 백업, `7` 복원.
- 다른 선택: `8` 설정, `9` 구성 요소, `U` 사용자, `K` API 키, `A` 내 주소, `H` 도움 파일, `M` More(고급), `L` 언어.
- 대부분의 선택은 명령으로도 돼요: `bash garden.sh WORD`, 예를 들면 `bash garden.sh backup`.
- 제거(Uninstall)는 More, `12`예요. 그 안의 선택 `2`는 모든 것을 지우지만, 먼저 마지막 백업을 만들고 `DELETE`를 입력하라고 해요.

---

이 안내서는 정원 메뉴의 모든 선택을 하나씩 보여 줘요: 무엇을 하는지, 언제 쓰는지,
컴퓨터에 무엇을 넣는지, 그리고 어떻게 되돌리는지. 무언가를 찾아볼 때 쓰거나,
나중에 새것을 더할지 정할 때 쓰세요.

**메뉴 여는 방법**

- **Ubuntu:** 바탕화면의 "Jun's Garden" 아이콘을 두 번 클릭하거나, 정원 폴더에서
  터미널을 열고 `bash garden.sh`를 입력하세요.
- **Windows:** 정원 폴더의 `Garden-Menu-Windows.cmd`를 두 번 클릭하세요.
  정원 자체는 항상 Ubuntu에서 실행돼요(다른 컴퓨터, 또는 Windows 안의
  Ubuntu). Windows 메뉴는 각 작업을 그 컴퓨터로 보내고, 결과를 내 화면에 보여 줘요.

**무엇이든 바뀌기 전에, 각 선택이 화면에서 스스로 설명해요.** 파란 상자가
*What*(무엇인지), *Why*(왜 원할 수 있는지), *Read*(더 읽을 곳)를 알려 줘요. 각
명령은 실행되기 전에 화면에 나와요. 화면에 나온 모든 것은 `logs/` 폴더에도 저장돼요.

**대부분의 선택은 명령으로도 할 수 있어요.** 메뉴 대신 `bash garden.sh WORD`를
입력해도 돼요(예: `bash garden.sh backup`). 이 단어들은 각 선택 아래와 끝부분의
표에 적혀 있어요.

아래에서 쓰는 몇 가지 말:

- **.env**는 정원 폴더 안의 비공개 설정 파일이에요. 본인만 읽을 수 있어요.
- **컨테이너(container)**는 자기만의 밀봉된 상자(Docker) 안에서 실행되는 프로그램 하나예요.
- **127.0.0.1 / localhost**는 "이 컴퓨터만"이라는 뜻이에요. 다른 사람은 아무도 닿을 수 없어요.
- **cron**은 Ubuntu의 알람 시계예요: 정해진 시간에 작업을 실행해요.

---

## 메인 메뉴

![메인 메뉴](../images/screen-01-main-menu.png)

`0) Quit`(끝내기)은 메뉴를 닫아요.

### 1) INSTALL  (first time, or carry on, or repair) — 설치(처음, 이어서 하기, 또는 고치기)

![설치: 14단계, 단계마다 보통 걸리는 시간](../images/screen-04-install-start.png)

![5단계: 방화벽이 SSH를 포트 22에서 집에서만 열어 둬요](../images/screen-05-install-firewall.png)

![12단계: 내 식물, 그다음 사진 파일에서 사진](../images/screen-06-install-plants-photos.png)

![마지막 단계, 그다음 ALL DONE](../images/screen-07-install-done.png)

- **무엇을 하나요:** 정원 전체를 14단계로 설정해요. 전에 중간에 멈췄다면, 그 단계부터 이어서 해요. 이미 끝나서 잘 작동하는 단계는 건너뛰어요. 처음에는 이렇게 물어봐요: 이 컴퓨터인지 다른 컴퓨터인지, EASY인지 CUSTOM인지, 웹 이름을 그대로 쓸지 새로 고를지.
- **언제 쓰나요:** 처음에. 설치가 중간에 멈춘 뒤에. 또는 점검표가 고장 났다고 하는 것을 고칠 때.
- **무엇을 설치하거나 바꾸나요:** Ubuntu 업데이트와 git, curl, openssl, rsync, unzip. SSH 서버. 방화벽(ufw): 집 네트워크에서 오는 SSH 말고는 들어오는 것을 모두 막아요. Docker. 작은 컴퓨터에는 4 GB 스왑 파일. `.env` 설정 파일, `local/` 안의 폴더들, git 기록. 정원을 만들고 시작하고(웹사이트는 127.0.0.1:8030에서만), `backups/`에서 식물을, 사진 다운로드 파일(`junsgarden-photos-addon.zip`, 있으면)에서 사진을 불러오고, 새벽 2시 30분의 매일 밤 백업(cron)과 바탕화면 아이콘을 추가해요. EASY는 정원 도구도 켜고, 그다음 비밀번호와 AI 키를 물어봐요.
- **되돌리려면:** More, 12(Uninstall, 제거하기).
- **명령:** `bash garden.sh install`. **안내서:** [`01-ubuntu-laptop.md`](../01-ubuntu-laptop.md)(영어), [`03-run-your-garden.md`](03-run-your-garden.md), 그리고 [`WHAT-GETS-INSTALLED.ko.md`](../../WHAT-GETS-INSTALLED.ko.md)(정원 폴더 맨 위에 있어요).

### 2) Start the garden — 정원 시작하기
- **무엇을 하나요:** 켜져 있는 정원 프로그램을 모두 시작하고, 웹사이트가 대답할 때까지 기다려요(최대 5분).
- **언제 쓰나요:** 메뉴 3 뒤에, 또는 컴퓨터가 꺼져 있었는데 정원이 다시 살아나지 않았을 때.
- **무엇을 설치하거나 바꾸나요:** 없어요. 컨테이너를 시작하기만 해요.
- **되돌리려면:** 메뉴 3.
- **명령:** `bash garden.sh start`

### 3) Stop the garden — 정원 멈추기
- **무엇을 하나요:** 정원의 프로그램을 멈춰요. 아무것도 지워지지 않아요.
- **언제 쓰나요:** 컴퓨터를 옮기기 전에, 또는 웹사이트를 잠시 꺼 두고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** 없어요. 다시 시작할 때까지 웹사이트는 꺼져 있어요.
- **되돌리려면:** 메뉴 2.
- **명령:** `bash garden.sh stop`

### 4) Open the website — 웹사이트 열기
- **무엇을 하나요:** 웹 브라우저에서 `http://localhost:8030`으로 정원을 열어요(Windows에서는 Windows 부분을 보세요).
- **언제 쓰나요:** 이 컴퓨터에서 정원을 보고 싶을 때 언제든지.
- **무엇을 설치하거나 바꾸나요:** 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh open`

### 5) Checklist: is everything OK? — 점검표: 모두 괜찮은가요?

![점검표: PASS, WARN, FAIL과 할 일](../images/screen-19-checklist.png)

- **무엇을 하나요:** 프로그램, 서비스, 포트(문), 방화벽, 폴더, 백업, 네트워크와 웹 이름, 남은 디스크 공간을 확인해요. 줄마다 PASS(통과), WARN(주의), FAIL(실패) 중 하나가 나오고, 그 아래에 고치는 방법이 있어요.
- **언제 쓰나요:** 무언가 이상해 보일 때, 무언가를 바꾼 뒤, 또는 가끔.
- **무엇을 설치하거나 바꾸나요:** `CHECKLIST.txt`를 쓰고 `checklist-history.log`에 한 줄을 더해요. 그 밖에는 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh check`. **안내서:** [`09-troubleshooting.md`](../09-troubleshooting.md)(영어).

### 6) Backups: back up, NAS, Google Drive — 백업: 백업하기, NAS, Google Drive
- **무엇을 하나요:** Backups(백업) 메뉴를 열어요(아래에 따로 부분이 있어요).
- **명령:** `bash garden.sh backups`. **안내서:** [`13-backups-and-folders.md`](13-backups-and-folders.md).

### 7) Restore a backup — 백업 복원하기
- **무엇을 하나요:** 백업을 최신 것부터 보여 줘요. 하나를 고르세요. 데이터베이스에 식물이 있으면, 먼저 지금 있는 것의 안전 백업을 만들어요(라벨 `before-restore`). 그다음 웹사이트, 정원 도구, 위키를 멈추고, 백업을 되돌려 놓고, 모든 것을 다시 시작해요.
- **언제 쓰나요:** 실수한 뒤에, 또는 예전 날로 돌아가고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** 데이터베이스, 사진, 댓글, 그 밖에 저장된 조각들을 백업에 있는 것으로 바꿔요.
- **되돌리려면:** 다시 복원하고 `before-restore` 백업을 고르세요.
- **명령:** `bash garden.sh restore`(또는 `bash garden.sh restore FILE-NAME`). **안내서:** [`13-backups-and-folders.md`](13-backups-and-folders.md).

### 8) Settings: names, emails, passwords, keys, folders — 설정: 이름, 이메일, 비밀번호, 키, 폴더
- **무엇을 하나요:** Settings(설정) 메뉴를 열어요(아래에 따로 부분이 있어요).
- **명령:** `bash garden.sh settings`

### 9) Parts: garden tools, wiki, n8n (on / off) — 구성 요소: 정원 도구, 위키, n8n(켜기 / 끄기)
- **무엇을 하나요:** Parts(구성 요소) 화면을 열어요(아래에 따로 부분이 있어요).
- **명령:** `bash garden.sh parts`

### U) Users and access: add, remove, guests, lock down — 사용자와 접근 권한: 추가, 지우기, 게스트, 잠그기
- **무엇을 하나요:** Users(사용자) 메뉴를 열어요(아래에 따로 부분이 있어요).
- **명령:** `bash garden.sh users`. **안내서:** [`07-sso-gate.md`](../07-sso-gate.md)(영어).

### K) API keys: paste, test or replace (Gemini, Gmail, ...) — API 키: 붙여 넣기, 테스트, 바꾸기(Gemini, Gmail, ...)
- **무엇을 하나요:** Keys(키) 메뉴를 열어요(아래에 따로 부분이 있어요).
- **명령:** `bash garden.sh keys`. **안내서:** [`16-ai-studio.md`](16-ai-studio.md).

### A) My addresses: every web address, and which need signing in — 내 주소: 모든 웹 주소와, 로그인이 필요한 주소

![내 주소](../images/screen-20-addresses.png)

- **무엇을 하나요:** 정원의 모든 주소를 보여 줘요: 인터넷 주소, 집 주소, 그리고 이 컴퓨터에서만 쓰는 주소. 어느 주소가 로그인 페이지를 거쳐야 하는지 알려 줘요.
- **언제 쓰나요:** 주소를 잊었을 때, 또는 정원을 인터넷에 올린 뒤에.
- **무엇을 설치하거나 바꾸나요:** 목록을 `MY-ADDRESSES.txt`에 써요. 그 밖에는 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh addresses`

### H) Help file for someone helping you (no passwords in it) — 도와주는 사람에게 줄 도움 파일(비밀번호 없음)
- **무엇을 하나요:** 도와주는 사람에게 필요한 것을 텍스트 파일 하나로 만들어요: 설치가 어디까지 됐는지, 컴퓨터, Docker, 모든 비밀번호와 키를 가린 설정, 점검표, 그리고 로그의 마지막 줄들.
- **언제 쓰나요:** 막혀서 누군가(또는 Gemini)에게 무슨 일이 있었는지 보여 주고 싶을 때. 먼저 직접 읽어 보세요.
- **무엇을 설치하거나 바꾸나요:** `help/junsgarden-help-DATE.txt`를 써요. 점검표도 실행해서 `CHECKLIST.txt`를 다시 써요.
- **되돌리려면:** 필요 없으면 파일을 지우세요.
- **명령:** `bash garden.sh helpme`. **안내서:** [`09-troubleshooting.md`](../09-troubleshooting.md)(영어).

### M) More (advanced) — 더 보기(고급)
- **무엇을 하나요:** More(더 보기) 메뉴를 열어요(아래에 따로 부분이 있어요).

### L) Language: English / 한국어 / both — 언어: English / 한국어 / 둘 다
- **무엇을 하나요:** 1) English, 2) Korean, 3) Both 중에서 물어봐요. 바뀌는 것은 글자뿐이에요. 입력하는 것(숫자, 글자, y/n)은 그대로예요. 한국어판이 없는 문장은 영어로 나와요.
- **언제 쓰나요:** 언제든지.
- **무엇을 설치하거나 바꾸나요:** 선택을 `.garden-lang`에 저장해요.
- **되돌리려면:** L을 다시 고르고 다른 언어를 고르세요.
- **명령:** `bash garden.sh language`

---

## More (M): 더 보기 메뉴

![More(더 보기) 메뉴](../images/screen-02-more-menu.png)

이것은 Ubuntu의 번호예요. Windows의 More 메뉴는 4 다음부터 순서가 달라요.
끝의 "Windows에만 있는 것" 부분을 보세요. `0) Back`(뒤로)은 메인 메뉴로 돌아가요.

### 1) The front door: HTTPS, your sign-in page, your web name — 정문: HTTPS, 로그인 페이지, 웹 이름
- **무엇을 하나요:** 정원을 다른 기기에 여는 세 가지 방법을 보여 줘요(아래 "정문의 세 가지 선택"을 보세요).
- **언제 쓰나요:** 정원을 집 Wi-Fi에서 쓰고 싶을 때, 연습할 때, 또는 인터넷에 올릴 때.
- **무엇을 설치하거나 바꾸나요:** 고른 것에 따라 달라요: Caddy, 로그인 페이지, 방화벽 문 80과 443, No-IP 업데이트 프로그램.
- **되돌리려면:** More, 14(정문 끄기).
- **명령:** `bash garden.sh public`. **안내서:** [`04-go-public-no-ip-caddy.md`](04-go-public-no-ip-caddy.md), [`04b-take-over-junsgarden-zapto.md`](../04b-take-over-junsgarden-zapto.md)(영어), [`07-sso-gate.md`](../07-sso-gate.md)(영어).

### 2) Network: my IP address and reserving it in the router — 네트워크: 내 IP 주소와 공유기에서 예약하기
- **무엇을 하나요:** 이 컴퓨터의 네트워크 카드, IP 주소, MAC 주소(카드의 고정된 ID), 공유기의 주소를 보여 주고, 공유기에서 IP를 예약하는 단계도 보여 줘요. 예약했다고 대답하면 그 IP를 기억해 두고, 나중에 IP가 바뀌면 점검표가 알려 줘요.
- **언제 쓰나요:** 인터넷에 올리기 전에. 포트 전달(port forwarding)에는 절대 바뀌지 않는 주소가 필요해요.
- **무엇을 설치하거나 바꾸나요:** yes라고 대답할 때만 `.env`에 `LAPTOP_IP`를 저장해요. 예약 자체는 공유기에서 직접 해요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh network`. **안내서:** [`14-reserve-your-ip.md`](14-reserve-your-ip.md).

### 3) Email (Gmail) for password links, with a test email — 비밀번호 링크용 이메일(Gmail), 테스트 메일 포함
- **무엇을 하나요:** Gmail 주소와 Gmail '앱' 비밀번호(16글자, 입력하는 동안 보이지 않아요)를 묻고, 테스트 이메일을 보낼지 물어봐요.
- **언제 쓰나요:** 웹사이트와 로그인 페이지가 "비밀번호를 잊었어요" 이메일을 보낼 수 있게 할 때.
- **무엇을 설치하거나 바꾸나요:** Gmail 설정을 `.env`에 저장해요. 정문이 켜져 있으면 로그인 페이지의 설정을 다시 써요. 프로그램을 다시 시작해요.
- **되돌리려면:** Keys(K), Gmail app password, 그다음 4(Remove, 지우기).
- **명령:** `bash garden.sh email`. **안내서:** [`03-run-your-garden.md`](03-run-your-garden.md)(C 부분).

### 4) Set my password (website + sign-in page) — 내 비밀번호 정하기(웹사이트 + 로그인 페이지)
- **무엇을 하나요:** 로그인 이름을 묻고, 그다음 새 비밀번호를 물어봐요(보이지 않아요). 웹사이트 로그인과 로그인 페이지에 함께 쓰는 비밀번호 하나예요. 알아볼 수 없게 바꾼 해시(hash)로만 저장돼요.
- **언제 쓰나요:** 처음에, 또는 새 비밀번호를 원할 때마다.
- **무엇을 설치하거나 바꾸나요:** 저장된 비밀번호. 로그인 페이지를 다시 시작해서 바로 적용돼요.
- **되돌리려면:** 다시 정하세요.
- **명령:** `bash garden.sh password`(또는 `bash garden.sh password NAME`).

### 5) Install on ANOTHER computer (over SSH) — '다른' 컴퓨터에 설치하기(SSH로)
- **무엇을 하나요:** 다른 컴퓨터의 IP 주소와, 그 컴퓨터에서 쓰는 사용자 이름을 물어봐요. 그 컴퓨터가 SSH 포트(22)에서 대답하는지 확인해요. 이 컴퓨터에 SSH 키가 없으면 만들고, 그 컴퓨터에 추가해요(그 컴퓨터의 비밀번호를 한 번 입력해요). 정원 폴더(`.env`, `local/`, `backups/`, `logs/`, `.git`은 빼고)와 최신 백업을 복사한 다음, 거기서 설치를 실행해요.
- **언제 쓰나요:** 정원을 다른 Ubuntu 노트북이나 Raspberry Pi에 두고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** 여기: `~/.ssh/id_ed25519`(새로 만든 경우)와 `.garden-remote`(마지막 IP와 사용자 이름, 비밀번호는 없어요). 거기: 메뉴 1이 설치하는 모든 것.
- **되돌리려면:** 그 컴퓨터에서 More 12(Uninstall). 그 컴퓨터에 있는 이 컴퓨터의 키는 Users(U), 7로 지우세요.
- **명령:** `bash garden.sh remote`

### 6) Show the programs' recent messages (logs) — 프로그램의 최근 메시지 보기(로그)
- **무엇을 하나요:** 각 정원 프로그램이 출력한 마지막 50줄을 보여 줘요.
- **언제 쓰나요:** 무언가 시작되지 않거나 이상하게 움직일 때.
- **무엇을 설치하거나 바꾸나요:** 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh logs`

### 7) Rebuild after editing Rust files — Rust 파일을 고친 뒤 다시 만들기
- **무엇을 하나요:** `rust/`의 코드로 웹사이트를 다시 만들고 다시 시작해요. 바뀐 파일만 다시 컴파일해요.
- **언제 쓰나요:** 본인이(또는 Gemini가) 웹사이트의 코드나 템플릿을 고친 뒤에.
- **무엇을 설치하거나 바꾸나요:** 내 코드로 만든 새 웹사이트 컨테이너.
- **되돌리려면:** `git checkout -- FILE`로 파일을 되돌린 다음, 다시 빌드하세요.
- **명령:** `bash garden.sh rebuild`. **안내서:** [`11-maintain-your-site.md`](../11-maintain-your-site.md)(영어).

### 8) Look inside the database (Adminer web page) — 데이터베이스 안 보기(Adminer 웹 페이지)
- **무엇을 하나요:** 한 페이지짜리 데이터베이스 보기 프로그램인 Adminer를 `http://localhost:8082`에서 시작해요(이 컴퓨터에서만). 무엇을 입력할지 보여 줘요: System MySQL, Server `db`, Username `dbadmin`, Database `juns_garden_db`, Password = `.env`의 `DB_PASSWORD` 줄.
- **언제 쓰나요:** SQL을 입력하지 않고 표와 줄(행)을 볼 때.
- **무엇을 설치하거나 바꾸나요:** 처음에 Adminer 이미지를 내려받고, `garden-adminer` 컨테이너를 시작해요.
- **되돌리려면:** `bash garden.sh dbweb-stop`(이것에는 메뉴 번호가 없어요).
- **명령:** `bash garden.sh dbweb`. **안내서:** [`12-your-database.md`](../12-your-database.md)(영어).

### 9) Connect a local AI computer — 로컬 AI 컴퓨터 연결하기
- **무엇을 하나요:** Ollama(로컬 AI)가 실행 중인 컴퓨터의 IP 주소를 물어봐요. 대답하는지 확인하고, 모델 목록을 보여 주고, 하나를 고르게 해요. 로컬 AI를 끄려면 주소를 비워 두세요.
- **언제 쓰나요:** 성능 좋은 그래픽 카드가 있고 Ollama를 실행하는 컴퓨터가 따로 있을 때만.
- **무엇을 설치하거나 바꾸나요:** `.env`에 `OLLAMA_URL`과 `OLLAMA_MODEL`을 저장하고 웹사이트를 다시 시작해요. 이 컴퓨터에는 아무것도 설치하지 않아요.
- **되돌리려면:** 다시 고르고 주소를 비워 두세요.
- **명령:** `bash garden.sh ai`. **안내서:** [`15-local-ai.md`](../15-local-ai.md)(영어). (Settings, 10과 같아요.)

### 10) Desktop icon — 바탕화면 아이콘
- **무엇을 하나요:** 바탕화면과 앱 목록에 "Jun's Garden" 아이콘을 놓아요. 이 아이콘은 이 메뉴를 열어요.
- **언제 쓰나요:** 아이콘이 없을 때(보통은 설치가 만들어요).
- **무엇을 설치하거나 바꾸나요:** `~/.local/share/applications/`와 바탕화면(Desktop)에 `junsgarden.desktop`.
- **되돌리려면:** 아이콘을 지우세요. Uninstall(More 12, 선택 1)도 둘 다 지워요.
- **명령:** `bash garden.sh icon`

### 11) Look at the files that came from the author (and check them) — 제작자가 보낸 파일 보기(그리고 확인하기)
- **무엇을 하나요:** zip에서 온 각 폴더가 무엇인지, 파일 개수와 크기와 함께 보여 줘요. 모든 파일의 지문을 `FILES-FROM-AUTHOR.txt`와 비교해서 바뀐 파일의 목록을 보여 줄지 물어봐요. Files(파일) 앱에서 폴더를 열지도 물어봐요.
- **언제 쓰나요:** zip에 정확히 무엇이 들어 있었는지 보거나, 그 뒤로 무엇이 바뀌었는지 찾을 때.
- **무엇을 설치하거나 바꾸나요:** 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh files`. **안내서:** [`18-review-the-code.md`](18-review-the-code.md).

### 12) Uninstall (programs only, or everything) — 제거하기(프로그램만, 또는 모두)
- **무엇을 하나요:** 두 가지 단계를 보여 줘요(아래 "제거하기" 부분을 보세요).
- **명령:** `bash garden.sh uninstall`

### 13) Subdomains with DuckDNS (wiki., n8n., ...): add, remove — DuckDNS 하위 도메인(wiki., n8n., ...): 추가, 지우기
- **무엇을 하나요:** Subdomains(하위 도메인) 메뉴를 열어요(아래에 따로 부분이 있어요).
- **명령:** `bash garden.sh subdomains`. **안내서:** [`04-go-public-no-ip-caddy.md`](04-go-public-no-ip-caddy.md)(고급).

### 14) On / off: the whole garden, or only off the internet; check-up, clean-up — 켜기 / 끄기: 정원 전체, 또는 인터넷에서만 내리기; 점검, 정리
- **무엇을 하나요:** 여섯 가지 선택이 있는 화면 하나예요. `1`은 모두 꺼요(`docker compose down`, `-v` 없이: 아무것도 지우지 않아요). 현관문도 멈춰요. `2`는 몇 달 뒤에도 모두 다시 켜고 웹사이트를 확인해요. `4`는 멈춘 프로그램을 다시 시작해요. `5`는 Docker의 남은 것(7일이 넘은 정원 프로그램의 옛 버전, 오래된 빌드 기록)을 물어본 뒤 지워요. Enter는 '아니요'예요. `6`은 6시간마다 점검을 해요.
- **`3`, 인터넷에서만 내리기:** Caddy를 멈추고, 방화벽에서 문 80과 443(그리고 집에서만 쓰는 문)을 닫고, No-IP 업데이트 프로그램과 DuckDNS 업데이트 프로그램을 멈추고, 로그인 페이지를 꺼요. 정원은 집에서 계속 실행돼요.
- **언제 쓰나요:** `1`은 한동안 사이트를 운영하고 싶지 않을 때나 접속이 너무 많을 때. `3`은 인터넷에서는 내리고 집에서는 쓰고 싶을 때, 또는 옮기기 전에. `5`는 디스크가 찰 때.
- **무엇을 설치하거나 바꾸나요:** `1`은 `.garden-off`를 만들어요(그 파일이 있는 동안 밤 백업도 쉬어요). `3`: Caddy와 No-IP 업데이트 프로그램은 꺼져요(지워지지는 않아요). DuckDNS cron 줄은 지워지고, `.env`의 웹 이름이 비워지고 주소가 localhost로 돌아가요. `6`은 cron 한 줄이에요.
- **되돌리려면:** `1`: `2`를 고르세요. `3`: More, 1을 다시 하세요(No-IP 업데이트 프로그램을 다시 켜요). 하위 도메인이 있었다면 Subdomains, 1도 하세요. 완전히 닫고 싶다면 공유기에서 80과 443 전달도 직접 지우세요. `6`: 다시 고르세요.
- **명령:** `bash garden.sh power`, 또는 바로 `bash garden.sh off`, `bash garden.sh on`, `bash garden.sh offline`(`3`), `bash garden.sh checkup`, `bash garden.sh cleanup`. 안내서: [`26-rename-move-and-switch.md`](26-rename-move-and-switch.md).

### 15) AI cost control: LiteLLM gateway with a monthly limit (on / off) — AI 비용 관리: 월 한도가 있는 LiteLLM 게이트웨이(켜기 / 끄기)
- **무엇을 하나요:** LiteLLM을 켜거나, 켜져 있으면 관리해요(아래 "AI 비용 관리(LiteLLM)"를 보세요).
- **명령:** `bash garden.sh llm`. **안내서:** [`16-ai-studio.md`](16-ai-studio.md).

### 16) Website name swap: use another web name than junsgarden.zapto.org — 웹사이트 이름 바꾸기: junsgarden.zapto.org 말고 다른 웹 이름 쓰기

![웹사이트 이름 바꾸기](../images/screen-18-website-name.png)

- **무엇을 하나요:** 파일 12개에 적힌 웹 이름(공유 버튼, 개인정보 페이지, 휴대폰 앱의 업데이트 확인, Android 앱의 코드)을 입력한 이름으로 바꿔요. 먼저 파일을 보여 주고 물어봐요.
- **언제 쓰나요:** 다른 이름으로 정원을 운영하고 싶을 때(그 이름도 '내' No-IP 계정에 있어야 해요).
- **무엇을 설치하거나 바꾸나요:** 그 파일들(예전 파일의 사본은 `logs/web-name-change_DATE/`로 가요). 설치되어 있고 예전 이름으로 온라인이면: `.env`와 Caddy도 따라 바뀌어요. 웹사이트를 다시 빌드할지 묻고, 바뀐 내용을 내 git 기록에 저장해요.
- **되돌리려면:** 예전 이름으로 다시 실행하거나, `logs/web-name-change_DATE/`에서 예전 파일을 다시 복사하세요. 앱이 있는 휴대폰은 앱을 다시 빌드할 때까지 예전 이름에서 업데이트를 확인해요.
- **명령:** `bash garden.sh webname`. **안내서:** [`04-go-public-no-ip-caddy.md`](04-go-public-no-ip-caddy.md).

> **왜 파일 탐색기가 아니라 메뉴로 하나요?**
> 웹 이름은 파일 12개에 적혀 있어요(목록은 `garden.sh`의 `WEBNAME_FILES`예요: 웹사이트
> 코드, 개인정보 페이지, 휴대폰 앱의 업데이트 파일, Android 앱, `docker-compose.yml` 등).
> 그리고 내 설정(`.env`)과 정문의 설정(Caddy와 로그인 페이지)에도 있어요. 손으로 바꾸면
> 그것을 하나하나 다 찾아야 해요. 하나라도 빠뜨리면 공유 버튼, 휴대폰 앱의 업데이트, 또는
> 정문이 아무 말 없이 예전 이름을 계속 써요. 메뉴는 모두 함께 바꿔요: 새 이름이 웹 이름
> 모양인지 확인하고, 바꾸기 전에 파일 목록을 보여 주고, 파일마다 사본을 두고, `.env`와
> 정문이 예전 이름을 쓰고 있으면 그것도 바꾸고, 웹사이트를 다시 빌드할지 물어요(빌드하면
> 웹사이트가 대답할 때까지 기다려요). 되돌릴 수 있어요: 예전 이름으로 다시 실행하거나,
> `logs/web-name-change_DATE/`에서 파일을 다시 복사하세요.

---

### 17) Reach the garden from anywhere, privately (Tailscale): on / off — 어디서나 나만 정원에 닿기(Tailscale): 켜기 / 끄기
- **무엇을 하나요:** Tailscale 자체의 설치 프로그램으로 Tailscale을 설치하고, 로그인 링크를 보여 줘요(내 Tailscale 계정으로 로그인하세요). 그다음 SSH와 Remote Desktop(원격 데스크톱)이 Tailscale로만 들어오게 해요(방화벽 줄 하나, `ufw allow in on tailscale0`). 쓸 이름도 알려 줘요.
- **언제 쓰나요:** 어디서나 정원 컴퓨터에 닿고 싶을 때. 내 기기만 연결할 수 있어요. 선택이지만, 꼭 권해요(안내서 [`20-ssh-and-remote-desktop.md`](20-ssh-and-remote-desktop.md)의 "집 밖에서").
- **무엇을 설치하거나 바꾸나요:** `tailscale` 프로그램(Tailscale 자체의 패키지 저장소에서 와요)과 방화벽 줄 하나. 공유기에는 포트를 하나도 열지 않아요.
- **되돌리려면:** 같은 항목을 다시 고르세요(연결을 끊고 방화벽 줄을 지워요). 프로그램까지 지우려면 `sudo apt remove tailscale`.
- **명령:** `bash garden.sh tailscale`

### 18) Rename or move a file or folder (every place that uses it follows) — 파일이나 폴더 이름 바꾸기 또는 옮기기(그것을 쓰는 모든 곳이 따라와요)
- **무엇을 하나요:** 네 가지 선택. `1`은 웹사이트의 파일이나 폴더(사진 폴더, 페이지, 데이터 파일)의 이름을 바꾸고, 그 이름을 쓰는 모든 곳을 바꿔요: 코드, 페이지, 사진 목록, 데이터베이스 줄. `2`는 큰 폴더 하나를 다른 곳이나 다른 디스크로 옮겨요(예전 설정 8과 같아요). `3`은 정원 폴더 전체를 옮겨요. `4`는 지금까지 바꾸거나 옮긴 것을 보여 줘요. 먼저 모든 곳을 보여 주고, 백업을 권하고, 내가 `y`를 입력하기 전에는 아무것도 바꾸지 않아요.
- **언제 쓰나요:** 웹사이트에 내 이름을 붙이고 싶을 때, 또는 정원을 더 큰 디스크로 옮기고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** 그 이름, 그것을 쓰는 모든 곳, 사진 폴더 이름을 바꾸면 `.env`의 `IMAGE_FOLDERS`, git 기록 한 줄, `logs/renames.tsv`의 한 줄. 바꾸는 것의 사본은 먼저 `logs/rename_DATE/`에 남겨요.
- **되돌리려면:** 다시 골라서 원래 이름으로 바꾸세요.
- **알아 두기:** 안내서에는 계속 정원이 처음 가진 이름이 나와요.
- **명령:** `bash garden.sh rename`. 안내서: [`26-rename-move-and-switch.md`](26-rename-move-and-switch.md).

## Backups (6): 백업 메뉴

![백업 메뉴](../images/screen-13-backups.png)

이 화면 맨 위에는 백업이 있는 곳, 최신 백업, NAS와 Google Drive 폴더, 그리고
마지막으로 만든 사본이 보여요. `0) Back`(뒤로)은 메인 메뉴로 돌아가요.
여기 있는 모든 것의 안내서: [`13-backups-and-folders.md`](13-backups-and-folders.md).

### 1) Back up now (and copy to the NAS / Drive) — 지금 백업하기(그리고 NAS / Drive로 복사)
- **무엇을 하나요:** 데이터베이스, 사진, 데이터, AI 캐시(그리고 위키, n8n, 로그인 페이지가 켜져 있으면 그것들도)를 `backups/` 안의 `.tar` 파일 하나에 저장해요. 그다음 NAS로 복사하고, 마지막 Drive 사본이 6일 이상 됐으면 Google Drive로도 복사해요.
- **언제 쓰나요:** 큰 변경을 하기 전에. (백업은 저절로도 만들어져요: 정원이 바쁠 때는 더 자주, 적어도 일주일에 한 번. 7번.)
- **무엇을 설치하거나 바꾸나요:** `backups/`에 새 파일 하나. 직접 만든 백업은 절대 저절로 지워지지 않아요. 자동 백업은 솎아 내요(오늘 것은 최신 3개, 그다음 일주일은 하루에 하나). 백업마다 사진이 전부 들어 있어서(약 800 MB), 자동 백업에 많아야 약 8 GB가 필요해요.
- **되돌리려면:** 되돌릴 것이 없어요. 원하지 않으면 파일을 지우세요.
- **명령:** `bash garden.sh backup`(또는 `bash garden.sh backup LABEL`).

### 2) Restore a backup — 백업 복원하기
- **무엇을 하나요:** 메인 메뉴 7과 같아요.
- **명령:** `bash garden.sh restore`

### 3) Check the newest backup (fingerprints) — 최신 백업 확인하기(지문)
- **무엇을 하나요:** 최신 백업 안의 모든 조각을 지문과 비교해서, 손상되지 않았는지 알려 줘요.
- **언제 쓰나요:** 가끔, 또는 백업에 기대기 전에.
- **무엇을 설치하거나 바꾸나요:** 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.
- **명령:** `bash garden.sh verify`(또는 `bash garden.sh verify FILE-NAME`).

### 4) Connect your NAS (network drive) — NAS 연결하기(네트워크 드라이브)
- **무엇을 하나요:** 집 네트워크에서 NAS를 찾아볼 수 있고, NAS의 사용자 이름과 비밀번호(보이지 않아요)를 묻고, 공유 폴더 목록을 보여 줘서 하나를 고르게 하고, 네트워크 장소 1로 연결해요. 그다음부터는 모든 백업이 거기에도 복사돼요(최신 14개를 보관해요). 지금 바로 복사할지, 그리고 정원 폴더 '전체'를 매주 복사할지 물어봐요.
- **언제 쓰나요:** NAS가 있고, 집에 두 번째 사본을 두고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** `cifs-utils`와 `smbclient`를 설치해요. NAS 로그인 정보를 `/etc/garden-nas.credentials`에 저장해요(관리자만 읽을 수 있어요). `/etc/fstab`에 한 줄을 추가해요(예전 사본: `/etc/fstab.bak-garden`). NAS는 `/mnt/garden-nas`에 나타나요. `.env`에 `BACKUP_NAS_DIR`(그리고 경우에 따라 `NAS_FULL_COPY`)를 저장해요.
- **되돌리려면:** Backups, 8.
- **명령:** `bash garden.sh nas`

### 5) Connect Google Drive — Google Drive 연결하기
- **무엇을 하나요:** 나만의 무료 Google "client ID"를 만드는 과정을 안내하고(추천해요: rclone의 공용 ID는 2026년 중에 없어져요), 그다음 브라우저에서 Google 로그인을 해요. Drive 폴더 `JunsGarden-backups`를 만들어요. 일주일에 한 번 최신 백업이 거기로 가요(최신 4개를 보관해요). 지금 바로 복사할지 물어봐요(약 800 MB).
- **언제 쓰나요:** 집 밖에 사본을 두고 싶을 때(불, 홍수, 도난).
- **무엇을 설치하거나 바꾸나요:** `rclone`을 설치해요. 설정과 Google 권한은 `~/.config/rclone/rclone.conf`에 들어가요. rclone은 자기가 직접 만든 파일만 볼 수 있어요. `.env`에 `BACKUP_CLOUD`를 저장해요.
- **되돌리려면:** Backups, 8. Google 권한까지 지우려면: `rclone config delete garden-gdrive`.
- **명령:** `bash garden.sh cloud`

### 6) Copy the newest backup to the NAS / Drive now — 지금 최신 백업을 NAS / Drive로 복사하기
- **무엇을 하나요:** 최신 백업을 지금 바로 NAS와 Google Drive로 복사해요(이번 주에 이미 Drive 사본이 있어도 Drive로 복사해요).
- **언제 쓰나요:** NAS나 Drive를 연결한 뒤, 또는 복사가 실패한 뒤에.
- **무엇을 설치하거나 바꾸나요:** 파일 하나를 복사해요. 한도(NAS는 14개, Drive는 4개)를 넘는 예전 사본은 거기서 지워져요.
- **되돌리려면:** 되돌릴 것이 없어요.

### 7) Backup schedule: automatic (busy, medium, quiet) or every night — 백업 일정: 자동(바쁨, 보통, 조용함) 또는 매일 밤
- **무엇을 하나요:** cron에 있는 백업 줄을 보여 주고, (자동이면) 지금 단계를 보여 줘요: 바쁨(지난 백업 뒤로 수정 4개 이상: 지금 백업), 보통(1~3개: 6시간 안에 또는 밤 9시부터), 조용함(없음: 적어도 일주일에 한 번). 다음 백업이 언제인지와 `backups/backup.log`의 최근 5번도 보여 줘요. 그다음 `1`은 자동 일정을 그대로 두거나 켜고, `2`는 매일 밤 새벽 2시 30분에 백업 하나로 바꿔요.
- **언제 쓰나요:** 백업이 예약되어 있는지, 일정이 무엇을 정했는지 볼 때, 또는 두 방식 중에서 고를 때. 수정을 세는 방법은 안내서 13에 있어요.
- **무엇을 설치하거나 바꾸나요:** `1`이나 `2`를 고르면: `.env`의 `BACKUP_SCHEDULE`과 내 cron의 백업 줄(`crontab -l`). 아니면 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.

### 8) Stop copying to the NAS or Google Drive — NAS나 Google Drive로 복사 멈추기

![8: 번호가 붙은 네트워크 장소 목록, 그다음 어느 곳을 멈출지](../images/screen-22-nas-stop.png)

- **무엇을 하나요:** 모든 백업을 받는 네트워크 장소를 번호와 주소와 함께 보여 주고, "Number of the network place to stop (0 = none)"(멈출 네트워크 장소의 번호, 0 = 없음)을 물어봐요. 번호를 입력하면 그곳을 멈추고, `0`이면 모두 그대로 둬요. 그다음 Google Drive가 연결되어 있으면 "Stop copying to Google Drive?"(Google Drive로 복사를 멈출까요?)를 물어봐요(기본은 No). NAS, PC, Drive에 이미 있는 파일은 남아요.
- **언제 쓰나요:** 그 장소 중 하나를 더 이상 쓰지 않을 때, 또는 13으로 다른 곳을 더하기 전에 자리를 비울 때.
- **무엇을 설치하거나 바꾸나요:** 네트워크 장소: 연결을 끊고, `/etc/fstab`에서 그 줄을 지우고, 로그인 파일(`/etc/garden-nas.credentials`, 또는 `-2` / `-3`)을 지우고, `.env`의 `BACKUP_NAS_DIR`와 `BACKUP_NAS_HOST` 줄을 비워요(장소 2와 3은 `_2` / `_3`). Drive: `BACKUP_CLOUD`를 비워요. rclone의 Google 권한은 `rclone config delete garden-gdrive`를 실행할 때까지 남아요.
- **되돌리려면:** Backups 4(장소 1), 13(장소 2와 3), 또는 5(Drive)를 다시 하세요.

### 9) Change to a different NAS (moved, new model or maker) — 다른 NAS로 바꾸기(옮겼거나, 새 모델이나 다른 회사 제품)
- **무엇을 하나요:** 예전 NAS(네트워크 장소 1)의 연결을 끊고(그 안의 파일은 남아요), 새 NAS로 "Connect your NAS"(NAS 연결하기)를 실행한 다음, 예전 NAS에서 최신 정원 백업 3개를 새 NAS로 복사할지 물어봐요. (장소 2와 3: 8로 멈추고 13으로 다시 더하세요.)
- **언제 쓰나요:** NAS를 옮겼거나 새로 샀을 때.
- **무엇을 설치하거나 바꾸나요:** 새 NAS에 대해 Backups 4와 같아요.
- **되돌리려면:** 9를 다시 고르고 예전 NAS를 연결하세요.

### 10) Copy the WHOLE garden folder to the NAS now — 지금 정원 폴더 '전체'를 NAS로 복사하기
- **무엇을 하나요:** 정원 폴더 전체(코드, `.env`를 포함한 설정, 사진, 백업. 실행 중인 데이터베이스 파일과 로그는 빼고)를 `.tar` 하나로 만들어 NAS(네트워크 장소 1)의 `whole-garden-folder/`에 둬요. 최신 2개를 보관해요.
- **언제 쓰나요:** 한곳에 완전한 사본을 두고 싶을 때. NAS를 비공개로 두세요: `.env`가 안에 들어 있어요.
- **무엇을 설치하거나 바꾸나요:** NAS에 파일 하나.
- **되돌리려면:** NAS에서 지우세요.

### 11) Load the photos download (now, or again) — 사진 파일 불러오기(지금, 또는 다시)
- **무엇을 하나요:** `junsgarden-photos-addon.zip`을 찾아서(정원 폴더, Home, Downloads, Desktop, 또는 한 단계 위 폴더. 한국어 Ubuntu에서는 `~/다운로드`와 `~/바탕화면`에서도), 지문을 확인하고, 사진 폴더에 압축을 풀어요. 이미 있는 사진은 그대로 둬요.
- **언제 쓰나요:** 사진을 내려받기 전에 설치를 했다면, 또는 사진을 다시 받고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** `local/images/`의 사진. `unzip`이 없으면 설치해요.
- **되돌리려면:** 쉽게 되돌릴 방법은 없어요(사진은 평범한 파일이에요).
- **명령:** `bash garden.sh photos`

### 12) Keep a copy of the downloads (the zips) on the NAS — 내려받은 파일(zip)의 사본을 NAS에 두기
- **무엇을 하나요:** `junsgarden-for-jun.zip`, 사진 추가 파일, 위키 추가 파일(찾은 것만)을 NAS의 `downloads` 폴더로 복사해요. 새 것이나 바뀐 것만 복사해요.
- **언제 쓰나요:** Google Drive에서 다시 내려받을 일이 없게 하고 싶을 때. 먼저 NAS를 연결하세요(Backups 4).
- **무엇을 설치하거나 바꾸나요:** NAS의 파일들.
- **되돌리려면:** NAS에서 지우세요.

### 13) Add another network place (a second NAS, or a PC's shared folder) — 네트워크 장소 하나 더하기(두 번째 NAS, 또는 PC의 공유 폴더)

![13: 장소 2 더하기, 192.168.1.30에 있는 PC의 공유 폴더 GardenCopies](../images/screen-21-nas-add.png)

- **무엇을 하나요:** 이미 있는 장소를 보여 주고, 새 곳이 몇 번이 되는지 알려 준 다음, 4와 같은 질문을 해요(찾기 또는 주소, 사용자 이름, 비밀번호, 공유 폴더). 네트워크 장소는 모두 3곳까지예요. 장소 1은 4에서 연결한 곳이고 바뀌지 않아요. 그다음부터는 매일 밤 백업이 장소마다 가고, 장소마다 최신 14개를 보관해요. 3곳이 모두 쓰이고 있으면 그렇다고 알려 주고 아무것도 바꾸지 않아요.
- **언제 쓰나요:** 집 안의 다른 기기에 사본을 하나 더 두고 싶을 때: 두 번째 NAS, 또는 Windows PC가 공유하는 폴더(그 PC는 새벽 2시 30분에 켜져 있어야 해요).
- **무엇을 설치하거나 바꾸나요:** 4와 같고, 장소 번호가 붙어요: 로그인 파일 `/etc/garden-nas-2.credentials`(또는 `-3`), `/etc/fstab`의 한 줄, `/mnt/garden-nas-2`(또는 `-3`)의 폴더, `.env`의 `BACKUP_NAS_DIR_2` / `BACKUP_NAS_HOST_2`(또는 `_3`). 정원 폴더 '전체'의 매주 사본은 장소 1에만 가요.
- **되돌리려면:** Backups, 8, 그리고 그 장소의 번호를 입력하세요.

---

## Settings (8): 설정 메뉴

![설정 메뉴(비밀번호는 절대 보이지 않아요)](../images/screen-10-settings.png)

모든 설정은 `.env`에 저장돼요. 이 화면이 대신 바꿔 줘요. 비밀번호는 절대 보이지
않아요. `0) Back`(뒤로)은 메인 메뉴로 돌아가요.

### 1) Your login name and email — 로그인 이름과 이메일
- **무엇을 하나요:** 이름(영문 글자만)과 이메일을 물어봐요.
- **언제 쓰나요:** 이름이나 이메일이 바뀌었을 때.
- **무엇을 설치하거나 바꾸나요:** `.env`의 `GARDEN_USERS`와 `GARDEN_UPLOADER_NAMES`, 그다음 다시 시작해요. 조심하세요: '본인'만 편집하는 사람으로 저장해요. 그래서 Users 2로 추가한 다른 편집자는 그 목록에서 빠져요. 이름을 바꿨다면, 그 이름의 비밀번호를 정하세요(Settings 5).
- **되돌리려면:** 예전 값으로 1을 다시 고르세요(그리고 다른 사람들을 Users 2로 다시 추가하세요).

### 2) Time zone — 시간대
- **무엇을 하나요:** 시간대를 물어봐요(예: America/New_York, Asia/Seoul). 설치할 때는 그 컴퓨터 자신의 시간대로 시작해요.
- **언제 쓰나요:** 사이트의 시간이 틀리거나, 이사했을 때.
- **무엇을 설치하거나 바꾸나요:** `.env`의 `TZ`, 그다음 다시 시작해요.
- **되돌리려면:** 2를 다시 고르세요.

### 3) Email (Gmail) — 이메일(Gmail)
- **무엇을 하나요:** More, 3과 같아요.

### 4) API keys (Gemini, OpenRouter, Perenual, Gmail, No-IP): paste, test, replace — API 키(Gemini, OpenRouter, Perenual, Gmail, No-IP): 붙여 넣기, 테스트, 바꾸기
- **무엇을 하나요:** 메인 메뉴 K와 같아요.

### 5) Your password (website + sign-in page) — 내 비밀번호(웹사이트 + 로그인 페이지)
- **무엇을 하나요:** More, 4와 같아요.

### 6) Who may use the garden tools — 정원 도구를 쓸 수 있는 사람
- **무엇을 하나요:** 정원 도구(/beds)에 들어올 수 있는 이메일을, 쉼표로 구분해서 물어봐요.
- **언제 쓰나요:** 다른 사람이 정원 도구를 쓰게 하거나, 못 쓰게 할 때.
- **무엇을 설치하거나 바꾸나요:** `.env`의 `BEDS_ALLOWED_EMAILS`, 그다음 다시 시작해요.
- **되돌리려면:** 6을 다시 고르세요.

### 7) Web name (front door) — 웹 이름(정문)
- **무엇을 하나요:** 이름만이 아니라 정문 단계 전체를 열어요(More, 1과 같아요).
- **되돌리려면:** More, 14. (파일에 적힌 이름을 바꾸려면 More, 16을 쓰세요.)

### 8) Rename or move a file or folder — 파일이나 폴더 이름 바꾸기 또는 옮기기
- **무엇을 하나요:** 더 보기 18과 같아요(네 가지 선택). `2`를 고르면 정원 폴더 8개(웹사이트 파일, 사진, 데이터, 캐시, 데이터베이스, n8n, 로그인 페이지, 백업)를 보여 줘요. 하나를 고르고 새 위치를 입력하세요. 정원 폴더 안이나 다른 Linux 디스크도 돼요. 정원을 멈추고, 폴더를 옮기고, `.env`를 업데이트하고, 다시 시작해요.
- **언제 쓰나요:** 폴더를 다른 곳에 두고 싶을 때, 또는 웹사이트의 파일이나 폴더에 내 이름을 붙이고 싶을 때(`1`).
- **무엇을 설치하거나 바꾸나요:** 폴더의 위치와 `.env`의 그 줄. 백업 폴더를 옮기면 매일 밤 백업의 cron 줄도 다시 써요.
- **되돌리려면:** 8을 다시 고르고 원래 자리로 옮기세요.
- **안내서:** [`13-backups-and-folders.md`](13-backups-and-folders.md), [`26-rename-move-and-switch.md`](26-rename-move-and-switch.md).

> **왜 파일 탐색기가 아니라 메뉴로 하나요?**
> 정원은 폴더마다 `.env`에 있는 한 줄로 그 폴더를 찾아요(예: `IMAGES_DIR=./local/images`).
> Docker는 그 줄을 읽어서 폴더를 프로그램에 연결하고, 백업 프로그램도 같은 방법으로 백업
> 폴더를 찾아요. 손으로 폴더 이름을 바꾸거나 옮기면, 그 줄은 여전히 예전 자리를 가리켜요:
> 웹사이트가 사진 없이 시작하거나, 백업이 갈 곳을 잃어요. 메뉴는 모든 부분을 함께 해요:
> 정원을 멈추고, 이미 쓰이는 새 이름은 받아들이지 않고, 폴더를 옮기고(데이터베이스와
> 로그인 페이지 폴더는 시스템 것이라 관리자 권한으로), 새 자리를 `.env`에 적고, 정원을
> 다시 시작하고, 웹사이트가 대답할 때까지 기다려요. 옮기기가 실패하면 아무것도 바뀌지
> 않아요. 되돌리려면 8을 다시 고르고 폴더를 원래 자리로 옮기세요.

### 9) Renew the internal database passwords — 내부 데이터베이스 비밀번호 새로 만들기
- **무엇을 하나요:** 데이터베이스 자체 계정에 쓸 길고 무작위인 새 비밀번호를 만들고, 데이터베이스 안에서 바꾸고, `.env`에 저장하고, 다시 시작해요. 직접 입력할 일은 없어요.
- **언제 쓰나요:** 1년에 한 번, 또는 `.env`를 누군가에게 보여 준 적이 있다면.
- **무엇을 설치하거나 바꾸나요:** `.env`의 `DB_PASSWORD`와 `DB_ROOT_PASSWORD`.
- **되돌리려면:** 되돌릴 것이 없어요(예전 비밀번호는 없어졌고, 그것이 필요한 것도 없어요).

### 10) Local AI computer — 로컬 AI 컴퓨터
- **무엇을 하나요:** More, 9와 같아요.

---

## Parts (9): 구성 요소 화면

![구성 요소 화면: 숫자로 켜고 끄고, 그다음 A](../images/screen-09-parts.png)

구성 요소의 번호를 입력하면 켜짐(ON)과 꺼짐(OFF)이 바뀌어요. 그다음 **A**를
입력하면 적용해요. `0) Back without changing`(바꾸지 않고 뒤로)은 모든 것을 그대로 둬요.
구성 요소를 끄면(OFF) 멈추지만 데이터는 남아요. 그래서 다시 켜면(ON) 돌아와요.

### 1) Garden tools (/beds): planner, checkup, diagnose, tag scan, spend — 정원 도구(/beds): 계획, 점검, 진단, 이름표 스캔, 지출
- **무엇을 하나요:** 작은 Python 프로그램이에요: 화단 계획, 식물 점검, 진단, 이름표 스캔, 지출 목록, 위키 대기열. 휴대폰 앱을 위해 이름표 사진도 읽어요.
- **언제 쓰나요:** EASY가 켜요. 인터넷에서는 내 로그인 페이지에서 로그인한 뒤에만 열려요.
- **무엇을 설치하거나 바꾸나요:** `junsgarden-ingest` 컨테이너를 127.0.0.1:8025에 만들고 시작해요.
- **되돌리려면:** Parts, 1(OFF), 그다음 A.
- 읽어 보기: `beds/README.md`.

### 2) Wiki: your garden encyclopedia (Wiki.js) — 위키: 나의 정원 백과사전(Wiki.js)
- **무엇을 하나요:** `http://localhost:3000`에 있는 나만의 정원 위키예요. 처음에는 도우미가 '나'를 관리자(admin)로 만들고(위키 관리자 비밀번호를 물어봐요), `junsgarden-wiki-addon.zip`을 찾으면 정원 페이지를 넣어요(약 30분). 그 zip이 없으면 위키는 빈 채로 시작해요.
- **언제 쓰나요:** 백과사전을 원할 때. 페이지는 나중에 `bash garden.sh wiki`로 넣을 수 있어요.
- **무엇을 설치하거나 바꾸나요:** Wiki.js를 내려받고 시작해요(페이지는 정원의 데이터베이스에 보관돼요).
- **되돌리려면:** Parts, 2(OFF), 그다음 A.

### 3) n8n automations (advanced) — n8n 자동화(고급)
- **무엇을 하나요:** "이런 일이 생기면, 저렇게 하기"를 끌어다 놓기로 만드는 도구예요(예: 흙이 마르면 이메일 보내기). `http://localhost:5678`에 있어요.
- **언제 쓰나요:** 나중에, 재미있는 프로젝트를 할 때.
- **무엇을 설치하거나 바꾸나요:** n8n을 내려받고 컨테이너를 시작해요.
- **되돌리려면:** Parts, 3(OFF), 그다음 A.
- **안내서:** [`START-HERE/n8n/`](../n8n/)(영어).

### 4) Sign-in page (Authelia); set up by the front door step — 로그인 페이지(Authelia). 정문 단계에서 설정해요
- **무엇을 하나요:** 정원 도구 앞에 있는 내 로그인 페이지예요. 정문 단계(More, 1)를 하기 전에는 여기서 켤 수 없어요.
- **언제 쓰나요:** 보통 여기서는 쓰지 않아요. 정문이 켜고 꺼요.
- **무엇을 설치하거나 바꾸나요:** 127.0.0.1:9091의 `garden-auth` 컨테이너.
- **되돌리려면:** More, 14가 정문과 함께 꺼요. (정원이 온라인일 때 여기서 OFF로 바꾸면, 정원 도구가 인터넷에서 열리지 않아요.)

### A) Apply the changes — 바뀐 것 적용하기
- **무엇을 하나요:** 새것은 만들고, 켜진(ON) 것은 시작하고, 꺼진(OFF) 것의 컨테이너는 멈추고 지우고(데이터는 남아요), 위키가 켜져 있으면 위키 설정을 실행해요.

---

## Users and access (U): 사용자와 접근 권한

![사용자와 접근 권한](../images/screen-11-users.png)

본인이 관리자예요. `0) Back`(뒤로)은 메인 메뉴로 돌아가요. 안내서: [`07-sso-gate.md`](../07-sso-gate.md)(영어).

### 1) Who can sign in right now — 지금 로그인할 수 있는 사람
- **무엇을 하나요:** 로그인할 수 있는 모든 사람과, 정원 도구에 들어올 수 있는 이메일을 보여 줘요.
- **언제 쓰나요:** 누가 접근할 수 있는지 확인할 때.
- **무엇을 설치하거나 바꾸나요:** 없어요.
- **되돌리려면:** 되돌릴 것이 없어요.

### 2) Add a person who may EDIT (website + garden tools) — '편집'할 수 있는 사람 추가(웹사이트 + 정원 도구)
- **무엇을 하나요:** 그 사람의 이름과 이메일을 묻고, 추가한 다음, 그 사람의 첫 비밀번호를 입력하라고 해요(직접 만나서 알려 주세요. 나중에 그 사람이 바꿀 수 있어요).
- **언제 쓰나요:** 누군가 정원 편집을 도와줄 때.
- **무엇을 설치하거나 바꾸나요:** `.env`의 `GARDEN_USERS`, `GARDEN_UPLOADER_NAMES`, `BEDS_ALLOWED_EMAILS`에 추가해요. 웹사이트와 정원 도구를 다시 시작해요.
- **되돌리려면:** Users, 4.

### 3) Add a GUEST (an account on the sign-in page only; sees nothing private) — 게스트 추가(로그인 페이지 계정만, 비공개 내용은 보지 못해요)
- **무엇을 하나요:** 이름과 이메일(선택)을 묻고, 로그인 페이지에만 계정을 만들어요. 게스트는 편집하거나 정원 도구를 열 수 없어요.
- **언제 쓰나요:** 로그인만 하면 되는 손님에게.
- **무엇을 설치하거나 바꾸나요:** 로그인 페이지의 계정 하나.
- **되돌리려면:** Users, 4.

### 4) Remove a person (takes away every login they have) — 사람 지우기(그 사람의 모든 로그인을 없애요)
- **무엇을 하나요:** 그 사람을 모든 목록에서 빼고, 모든 로그인(웹사이트와 로그인 페이지)을 지워요. 본인은 지우지 않아요.
- **언제 쓰나요:** 누군가 더 이상 접근하면 안 될 때.
- **무엇을 설치하거나 바꾸나요:** `.env`의 목록과 그 사람의 계정. 그 사람의 SSH 키는 Users 7에서도 확인하세요.
- **되돌리려면:** 다시 추가하세요(Users 2 또는 3).

### 5) Change someone's password (yours too) — 비밀번호 바꾸기(본인 것도)
- **무엇을 하나요:** 이름을 묻고, 그다음 새 비밀번호를 물어봐요(보이지 않아요). More, 4와 같아요.

### 6) Sign everybody out now — 지금 모두 로그아웃시키기
- **무엇을 하나요:** 웹사이트와 로그인 페이지에서 모든 사람을 로그아웃시켜요.
- **언제 쓰나요:** 휴대폰이나 컴퓨터를 잃어버렸을 때, 또는 누가 로그인해 있는지 확실하지 않을 때.
- **무엇을 설치하거나 바꾸나요:** `.env`에 새 무작위 로그인 비밀 값. 모두 다시 로그인해야 해요.
- **되돌리려면:** 되돌릴 것이 없어요. 다시 로그인하면 돼요.

### 7) SSH keys: which computers can open this computer's terminal — SSH 키: 이 컴퓨터의 터미널을 열 수 있는 컴퓨터
- **무엇을 하나요:** `~/.ssh/authorized_keys`에 있는 키를 보여 줘요(컴퓨터마다 하나, user@computer 같은 이름표가 붙어 있어요). 번호로 하나를 지울 수 있어요.
- **언제 쓰나요:** 모르는 컴퓨터나 더 이상 쓰지 않는 컴퓨터를 지울 때.
- **무엇을 설치하거나 바꾸나요:** 그 줄을 지워요. 예전 목록의 사본은 `~/.ssh/authorized_keys.bak`에 남겨요.
- **되돌리려면:** `.bak` 파일을 `authorized_keys` 위에 다시 복사하세요.

### 8) LOCK DOWN: only me, new passwords and secrets, everyone signed out — 잠그기(LOCK DOWN): 나만, 새 비밀번호와 비밀 값, 모두 로그아웃
- **무엇을 하나요:** 본인 것 말고 모든 로그인을 지우고(게스트도), 정원 도구에는 본인의 이메일만 들어오게 하고, 모두 로그아웃시키고, 새 로그인 비밀 값과 새 데이터베이스 비밀번호를 만들고(이 단계는 먼저 물어봐요), '새' 비밀번호를 물어보고, SSH 키를 보여 줘요. 맨 처음에 "Lock down now?"(지금 잠글까요?)라고 물어봐요(기본은 No).
- **언제 쓰나요:** 누가 들어왔다고 생각될 때, 또는 접근 권한을 처음부터 새로 하고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** `.env`의 사용자 목록과 비밀 값, 그리고 본인 것 말고 모든 계정. 그다음 직접 하세요: Ubuntu 비밀번호(`passwd`), 공유기의 관리자 비밀번호, 예전 Gmail 앱 비밀번호, No-IP 비밀번호.
- **되돌리려면:** 되돌릴 수 없어요. Users 2나 3으로 사람들을 다시 추가하세요.
- **명령:** `bash garden.sh lockdown`

---

## API keys (K): API 키

![키 메뉴](../images/screen-12-keys.png)

화면에 각 키가 **set**(설정됨) 또는 **not set**(설정 안 됨)과 함께 나와요(마지막으로
바꾼 날짜도). 키는 화면에 절대 나오지 않아요. 키는 `.env`에 저장되고, 날짜는(키는
절대 아니에요) `logs/key-changes.log`에 적혀요. `0) Back`(뒤로)은 메인 메뉴로 돌아가요.
안내서: [`08-gemini-setup.md`](../08-gemini-setup.md)(영어), [`16-ai-studio.md`](16-ai-studio.md).

### 1) Google Gemini key (AI Studio) — Google Gemini 키(AI Studio)
- **무엇을 하나요:** 웹사이트의 AI 상자와 정원 도구(이름표 읽기, 식물 알아보기, 진단)가 써요. aistudio.google.com에서 무료로 받아요.

### 2) OpenRouter key (a backup when Gemini is busy) — OpenRouter 키(Gemini가 바쁠 때 쓰는 예비 키)
- **무엇을 하나요:** Gemini가 바쁠 때 AI 상자와 정원 도구가 써요. 정원은 이름이 ":free"로 끝나는 모델만 요청해요.

### 3) Perenual plant-data key — Perenual 식물 정보 키
- **무엇을 하나요:** 식물 관리(plant admin)와 정원 도구의 찾아보기에 쓰는 식물 정보예요. perenual.com에 무료 요금제가 있어요.

### 4) Gmail app password (for sending email) — Gmail 앱 비밀번호(이메일 보내기용)
- **무엇을 하나요:** 비밀번호 재설정 이메일과 로그인 페이지의 이메일에 써요. 여기서 "Paste a new key"(새 키 붙여 넣기)를 고르면 이메일 단계(More, 3)가 열려요.

### 5) Wiki API key (your own wiki) — 위키 API 키(본인의 위키)
- **무엇을 하나요:** 정원 도구의 위키 대기열이 초안을 내 위키에 올릴 수 있게 해요. 내 위키 안에서 만들어요(Administration > API Access).

### N) No-IP DDNS key (for your web name) — No-IP DDNS 키(웹 이름용)
- **무엇을 하나요:** No-IP 업데이트 프로그램 설정을 실행해요: DDNS 키의 사용자 이름과 비밀번호를 물어봐요(보이지 않아요).
- **무엇을 설치하거나 바꾸나요:** No-IP 업데이트 프로그램이 아직 설치되지 않았으면, noip.com에서 내려받아 설치해요. 키는 `/etc/default/noip-duc`에 저장돼요(관리자만 읽을 수 있어요).
- **되돌리려면:** More, 14가 업데이트 프로그램을 멈춰요. Uninstall(More 12)이 프로그램과 키를 지워요.

### 키를 고른 뒤(1~5): 키마다 있는 작은 메뉴
- **1) Paste a new key (or replace the old one)**(새 키 붙여 넣기, 또는 예전 키 바꾸기): 키를 받는 방법을 보여 주고, 붙여 넣게 해요(아무것도 보이지 않아요). 저장하고, 그 키를 쓰는 프로그램을 다시 시작하고, 키를 테스트해요. 되돌리려면: 4(Remove), 또는 예전 키를 붙여 넣으세요.
- **2) Test the key that is saved now**(지금 저장된 키 테스트하기): 그 키의 서비스에 키가 작동하는지 물어봐요. 아무것도 바꾸지 않아요.
- **3) How to get a key, step by step**(키를 받는 방법, 단계별로): 공식 단계를 보여 줘요. 아무것도 바꾸지 않아요.
- **4) Remove this key**(이 키 지우기): `.env`에서 키를 비우고, 그 키를 쓰는 프로그램을 다시 시작해요. 서비스 쪽에서도 지우세요(화면이 어디서 하는지 보여 줘요). 되돌리려면: 1(다시 붙여 넣기).
- **0) Back**(뒤로)

---

## 정문의 세 가지 선택

![정문의 세 가지 선택](../images/screen-14-front-door.png)

More, 1에서 나와요(또는 Settings, 7, 또는 `bash garden.sh public`).
`0) Back`(뒤로)은 아무것도 바꾸지 않아요. 안내서: [`04-go-public-no-ip-caddy.md`](04-go-public-no-ip-caddy.md).

### 1) NOT ON THE INTERNET YET (your home Wi-Fi only, no www) — 아직 인터넷에 없음(집 Wi-Fi에서만, www 없음)
- **무엇을 하나요:** 집 Wi-Fi에 있는 휴대폰과 컴퓨터가 `http://THIS-COMPUTER.local`이나 이 컴퓨터의 IP 주소로 정원을 열어요. 집 밖에서는 아무도 닿을 수 없어요. 정원 도구(/beds)는 여기서는 닫혀 있어요.
- **언제 쓰나요:** No-IP 이름이 준비되기 전에, 집에서 정원을 쓸 때.
- **무엇을 설치하거나 바꾸나요:** Caddy를 (공식 출처에서) 시스템 서비스로 설치해요. `/etc/caddy/Caddyfile`을 써요(예전 것은 `Caddyfile.bak`으로 남겨요). 방화벽: 집 네트워크에서만 포트 80. 공유기는 건드리지 않아요.
- **되돌리려면:** More, 14.

### 2) PRACTICE — 연습
- **무엇을 하나요:** 모든 것(HTTPS, 내 로그인 페이지, 관문)이 이 노트북 안에서만, `.test`로 끝나는 지어낸 이름과 연습용 인증서로 작동해요. Firefox가 인증서에 대해 경고해요: 예상한 일이에요. 마지막에 로그인 비밀번호를 물어봐요.
- **언제 쓰나요:** 진짜를 하기 전에 먼저 안전하게 해 볼 때.
- **무엇을 설치하거나 바꾸나요:** Caddy를 설치하고 로그인 페이지(Authelia)를 시작해요. Caddyfile을 써요. 방화벽에서 80과 443을 열어요. 연습용 이름을 `/etc/hosts`에 추가해요(이 노트북에서만).
- **되돌리려면:** More, 14. (Uninstall은 `/etc/hosts`의 `garden.test` 줄도 지워요.)

### 3) THE REAL WEB NAME (junsgarden.zapto.org, or your own) — 진짜 웹 이름(junsgarden.zapto.org, 또는 내 이름)
- **무엇을 하나요:** 웹 이름을 묻고(반드시 '내' No-IP 계정에 있어야 해요), 위키가 켜져 있으면 위키 웹 이름도 물어봐요(선택이에요). HTTPS, `https://NAME/auth`의 로그인 페이지, 관문, No-IP 업데이트 프로그램을 설정해요. 이 컴퓨터의 네트워크 정보를 보여 주고, 공유기에서 TCP 80과 443'만' 이 컴퓨터로 전달하라고 알려 줘요. 마지막에 로그인 비밀번호를 물어봐요.
- **언제 쓰나요:** 인터넷에 올릴 준비가 됐을 때(먼저 안내서 04의 A 부분을 하세요).
- **무엇을 설치하거나 바꾸나요:** Caddy, 로그인 페이지, `/etc/caddy/Caddyfile`, 방화벽 문 80과 443, 내 DDNS 키가 든 No-IP 업데이트 프로그램(`/etc/default/noip-duc`). `.env`: 웹 이름과 주소. 공유기 단계는 직접 해요.
- **되돌리려면:** More, 14, 그다음 공유기에서 80과 443 전달을 지우세요.
- **안내서:** [`04b-take-over-junsgarden-zapto.md`](../04b-take-over-junsgarden-zapto.md)(영어), [`07-sso-gate.md`](../07-sso-gate.md)(영어).

---

## Subdomains (More 13; Windows: More 14): 하위 도메인

![하위 도메인](../images/screen-15-subdomains.png)

무료 DuckDNS 이름은 하위 이름(wiki.NAME.duckdns.org, ...)을 가질 수 있어요. 모두
같은 두 문(80과 443)으로 들어오고, Caddy가 각각에 자물쇠를 붙여 줘요. 공유기에서
새로 여는 것은 없어요. `0) Back`(뒤로)은 More로 돌아가요.
안내서: [`04-go-public-no-ip-caddy.md`](04-go-public-no-ip-caddy.md)(고급).

### 1) Set up DuckDNS (your name, token and the 5-minute updater) — DuckDNS 설정하기(이름, 토큰, 5분마다 도는 업데이트 프로그램)
- **무엇을 하나요:** duckdns.org에서 할 단계를 보여 주고, DuckDNS 이름과 토큰(보이지 않아요)을 묻고, 테스트해요.
- **언제 쓰나요:** 하위 도메인을 추가하기 전에.
- **무엇을 설치하거나 바꾸나요:** `~/.config/garden-duckdns/token`의 토큰(본인만 읽을 수 있어요), `.env`의 `DUCKDNS_DOMAIN`, 작은 업데이트 스크립트, 그리고 그것을 5분마다 실행하는 cron 줄.
- **되돌리려면:** More, 14가 cron 줄을 지워요. Uninstall은 폴더까지 지워요.

### 2) Add a subdomain — 하위 도메인 추가하기
- **무엇을 하나요:** 이름(wiki = 포트 3000, n8n = 5678, status = Uptime Kuma용 3001, home = Home Assistant용 8123, 또는 아무 이름과 그 포트)과, 로그인 페이지 뒤에 둘지를 물어봐요. 그다음 Caddy와 로그인 페이지의 설정을 다시 쓰고 다시 불러와요.
- **언제 쓰나요:** 프로그램 하나를 더, 자기 주소로 인터넷에 올릴 때. 먼저 DuckDNS(1)와 정문(More 1, 선택 2 또는 3)이 필요해요.
- **무엇을 설치하거나 바꾸나요:** `.env`의 `SUBDOMAINS`(wiki라면 `WIKI_SITE_URL`도), `/etc/caddy/Caddyfile`, 로그인 페이지의 규칙. 위키는 설정이 끝날 때까지 빠져 있어요.
- **되돌리려면:** Subdomains, 3.
- **안내서:** [`projects/P01-uptime-kuma.md`](../projects/P01-uptime-kuma.md)(영어), [`projects/P03-home-assistant.md`](../projects/P03-home-assistant.md)(영어).

### 3) Remove a subdomain — 하위 도메인 지우기
- **무엇을 하나요:** 이름을 묻고, 목록에서 빼고, 설정을 다시 쓰고, 다시 불러와요.
- **되돌리려면:** Subdomains, 2.

### 4) Write the settings again (Caddy + sign-in page) and reload — 설정을 다시 쓰고(Caddy + 로그인 페이지) 다시 불러오기
- **무엇을 하나요:** `.env`를 바탕으로 Caddy와 로그인 페이지의 설정을 다시 쓰고, Caddy를 다시 불러와요. Caddy의 확인에 통과하지 못하면, 예전 설정으로 되돌려요.
- **언제 쓰나요:** 무언가를 바꾼 뒤, 또는 무언가 최신 상태가 아닌 것 같을 때.
- **무엇을 설치하거나 바꾸나요:** `/etc/caddy/Caddyfile`(예전 것은 `Caddyfile.bak`으로 남겨요).
- **되돌리려면:** 되돌릴 것이 없어요.

---

## AI cost control (LiteLLM) (More 15; Windows: More 16): AI 비용 관리

![켜기 전의 LiteLLM](../images/screen-16-litellm.png)

LiteLLM은 이 컴퓨터에 있는 비공개 AI 주소 하나예요(`http://localhost:4000`). '내'
프로젝트(n8n, 코딩 도구, 스크립트)를 위한 것이에요. 내 키를 가지고 있고, 월 한도에
닿으면 멈춰요. 정원 웹사이트와 정원 도구는 계속 자기 키를 직접 써요. 무료 키만
쓴다면 필요 없어요. 안내서: [`16-ai-studio.md`](16-ai-studio.md).

### 꺼져 있을 때: "Switch LiteLLM on?"(LiteLLM을 켤까요?)
- **무엇을 하나요:** 월 한도를 미국 달러로 묻고(0 = 무료 모델만), 모델 두 개 `gemini-flash`와 `free-openrouter`로 LiteLLM을 시작해요.
- **무엇을 설치하거나 바꾸나요:** LiteLLM 이미지를 내려받아요. `local/litellm/config.yaml`을 써요(키는 안 들어 있어요). `.env`에 `LITELLM_BUDGET`과 `LITELLM_PORT`(4000, 또는 다음 빈 포트). `garden-llm` 컨테이너를 시작해요(이 컴퓨터에서만).
- **되돌리려면:** LiteLLM 메뉴, 7.

### 켜져 있을 때: LiteLLM 메뉴(`0) Back`은 More로 돌아가요)
바꿀 때마다 `local/litellm/config.yaml`을 다시 쓰고 LiteLLM을 다시 시작해요.

### 1) Add a Gemini model (uses your Gemini key, menu K) — Gemini 모델 추가하기(Gemini 키를 써요, 메뉴 K)
- **무엇을 하나요:** Gemini 모델 ID와, 내 프로젝트가 쓸 짧은 이름을 물어봐요. **되돌리려면:** LiteLLM, 4.

### 2) Add an OpenRouter model (uses your OpenRouter key; ":free" models cost nothing) — OpenRouter 모델 추가하기(OpenRouter 키를 써요. ":free" 모델은 무료예요)
- **무엇을 하나요:** OpenRouter 모델 ID와 짧은 이름을 물어봐요. **되돌리려면:** LiteLLM, 4.

### 3) Add a local model from Ollama (your own AI computer, guide 15) — Ollama의 로컬 모델 추가하기(내 AI 컴퓨터, 안내서 15)
- **무엇을 하나요:** Ollama 컴퓨터의 주소, 모델 이름, 짧은 이름을 물어봐요. **되돌리려면:** LiteLLM, 4.

### 4) Remove a model — 모델 지우기
- **무엇을 하나요:** 짧은 이름을 묻고 그 모델을 지워요. **되돌리려면:** 다시 추가하세요(1, 2 또는 3).

### 5) Change the monthly spending limit — 월 지출 한도 바꾸기
- **무엇을 하나요:** 새 한도를 미국 달러로 물어봐요. `.env`와 설정 파일에 저장돼요. **되돌리려면:** 5를 다시 고르세요.

### 6) Show the gateway key (to paste into one of your projects) — 게이트웨이 키 보기(내 프로젝트에 붙여 넣기용)
- **무엇을 하나요:** `LITELLM_MASTER_KEY`를 화면에 보여 줘요. 비밀번호처럼 다루세요. 아무것도 바꾸지 않아요.

### 7) Switch LiteLLM off — LiteLLM 끄기
- **무엇을 하나요:** 먼저 물어봐요(기본은 No). 구성 요소를 끄고 컨테이너를 지워요. 설정은 남아요. **되돌리려면:** More 15를 다시 고르세요.

---

## Uninstall (More 12): 제거하기

![제거하기: 두 단계](../images/screen-17-uninstall.png)

`0) Back`(뒤로)은 아무것도 바꾸지 않아요.

### 1) Remove the PROGRAMS, keep my garden — 프로그램만 지우고 정원은 남기기
- **무엇을 하나요:** 먼저 물어봐요(기본은 No). 설치가 Ubuntu에 더한 것을 지우지만, 정원은 지우지 않아요.
- **무엇을 설치하거나 바꾸나요:** 정원의 컨테이너와 여기서 만든 이미지를 멈추고 지워요. 매일 밤 백업 줄, 바탕화면 아이콘, 모든 네트워크 장소의 연결(거기 있는 파일은 남아요), Caddy, No-IP 업데이트 프로그램과 그 키, DuckDNS 업데이트 프로그램을 지워요. 방화벽 문 80과 443을 닫아요. `/etc/hosts`의 `garden.test` 줄을 지워요. 남기는 것: 이 폴더, `local/`(사진과 데이터베이스), 백업. 지우지 않는 것(본인이 정해요): Docker, SSH 서버, git, 4 GB 스왑 파일(화면이 방법을 보여 줘요).
- **되돌리려면:** 메인 메뉴 1(INSTALL)을 다시 하세요.

### 2) Remove EVERYTHING (start completely fresh) — '모두' 지우기(완전히 새로 시작)
- **무엇을 하나요:** 먼저 마지막 백업을 만들고(NAS가 연결되어 있으면 거기로도 복사해요), 어느 폴더로 옮길지 물어봐요(비워 두면 = Home 폴더). 그다음 `DELETE`를 입력해야 해요. 그러면 선택 1의 모든 것을 하고, 거기에 더해 이 컴퓨터의 사진, 데이터베이스, 위키, 설정(`.env`), 로그, 백업까지 지워요.
- **무엇을 설치하거나 바꾸나요:** `local/`, `backups/`, `logs/`, `help/`, `.env`, 기억해 둔 선택과 점검표 파일을 지워요. rclone에서 Google Drive 연결을 지워요. 정원 자신이 남긴 Docker 저장 공간(볼륨)을 지워요(다른 프로그램의 것은 그대로 둬요). 프로그램 파일은 폴더에 남아요.
- **되돌리려면:** 마지막 백업을 `backups/`에 넣고, 메인 메뉴 1(INSTALL)을 하세요: 그 백업으로 정원을 불러와요.

---

## Windows에만 있는 것

Windows 메인 메뉴는 Ubuntu와 **같은 번호와 글자**를 쓰고, **T**가 더 있어요.
대부분은 정원 컴퓨터에서 `bash garden.sh WORD`를 실행하고(SSH로, 또는 Windows
안의 Ubuntu인 WSL 안에서) 이 창에 보여 주기만 해요. 이것들은 Windows에서 다르게
작동해요:

### 1) INSTALL  (first time, or carry on, or repair) — 설치(처음, 이어서 하기, 또는 고치기)
- **시작하기 전에:** 정원 폴더가 OneDrive 안에 있으면, `Start-Here-Windows.cmd`가 먼저 폴더를 OneDrive 밖으로(`C:\Users\<you>\junsgarden-for-jun`, 또는 내가 고른 곳으로) 옮기자고 해요. 모든 파일이 도착했는지 확인하고, 새 위치에서 설치 프로그램을 열어요. 예전 폴더는 절대 대신 지우지 않아요. Windows 메뉴의 INSTALL도 이것을 알려 줘요. 안내서: [`00b-before-you-start.md`](00b-before-you-start.md), "바탕 화면이 OneDrive 안에 있다면".
- **무엇을 하나요:** 아직 고르지 않았다면 정원이 어디서 실행될지 물어봐요(T를 보세요). 다른 컴퓨터: SSH 포트를 확인하고, 이 PC의 SSH 키를 만들어 거기에 추가해요(그 컴퓨터의 비밀번호를 한 번 입력해요). 이 PC: WSL을 켜고(그다음 Windows를 한 번 다시 시작해요), `JunsGarden`이라는 이름으로 Ubuntu 24.04를 설치하고, Ubuntu 사용자를 만들어요. Docker Desktop을 고르면 필요할 때 Docker Desktop을 설치하고(winget) WSL integration 스위치를 기다려요(안내서 26). 그다음 물어봐요: 웹 이름을 그대로 쓸지, 새로 고를지. 정원 폴더와 최신 백업을 복사하고, 사진 zip과 위키 zip을 정원 폴더, 바탕화면(OneDrive 바탕화면도), 또는 Downloads에서 바로 그쪽으로 보내고(PC에 사본을 하나 더 만들지 않아요), 거기서 `garden.sh install`을 실행해요.
- **무엇을 설치하거나 바꾸나요:** 이 PC에: `.ssh\id_ed25519`(새로 만든 경우), `.garden-target-windows.txt`. WSL을 쓰면 Windows 안의 Ubuntu, 그리고(yes라고 하면) 작업 스케줄러(Task Scheduler) 작업 "Jun's Garden keep running". 이 작업은 내가 로그인해 있는 동안 Ubuntu가 계속 켜져 있게 해요. 정원 컴퓨터에: 메뉴 1이 설치하는 모든 것.
- **되돌리려면:** 정원 컴퓨터에서 More 12(Uninstall). Windows 안의 Ubuntu와 작업 스케줄러 작업은 메뉴가 지우지 않아요. Windows에서 직접 지우세요.

### 4) Open the website — 웹사이트 열기
- **무엇을 하나요:** WSL이면: `http://localhost:8030`을 열어요. 다른 컴퓨터면: 안전한 터널 '그 자체'인 새 창을 열고(이 PC의 포트 18030에서 정원 컴퓨터의 웹사이트로), 그다음 `http://localhost:18030`을 열어요. 터널을 멈추려면 그 창을 닫으세요.
- **무엇을 설치하거나 바꾸나요:** 없어요.

### L) Language on the garden computer: English / Korean / both — 정원 컴퓨터의 언어: English / 한국어 / 둘 다
- **무엇을 하나요:** 정원 컴퓨터에서 언어 선택을 실행해요(Ubuntu의 L과 같아요).

### T) Choose the garden computer — 정원 컴퓨터 고르기
- **무엇을 하나요:** 1) 집에 있는 다른 컴퓨터(그 컴퓨터의 로컬 IP와, 거기서 쓰는 사용자 이름을 물어봐요), 또는 2) WSL 안의, 이 Windows PC. 1은 `F`를 입력하면 SSH가 열린 집의 컴퓨터를 찾아 줘요. 2는 그다음 물어요: Docker Desktop(쉬운 방법) 또는 Ubuntu 안의 Docker.
- **무엇을 설치하거나 바꾸나요:** 선택을 `.garden-target-windows.txt`에 저장해요(IP와 사용자 이름만, 비밀번호는 절대 저장하지 않아요).
- **되돌리려면:** T를 다시 고르세요.

### Windows의 More 메뉴: Ubuntu와 번호가 다른 것

| Windows More | 무엇인가요 | Ubuntu More |
|---|---|---|
| 1 to 4 | Ubuntu와 같아요 | 1 to 4 |
| 5 | Open a terminal on the garden computer(정원 컴퓨터의 터미널 열기) | (Ubuntu의 5는 "Install on ANOTHER computer"('다른' 컴퓨터에 설치하기)예요) |
| 6 to 9 | Ubuntu와 같아요 | 6 to 9 |
| 10 | Copy the newest backup to THIS PC(최신 백업을 이 PC로 복사하기) | (Ubuntu의 10은 "Desktop icon"(바탕화면 아이콘)이에요) |
| 11, 12 | Ubuntu와 같아요 | 11, 12 |
| 13 | Weekly automatic copy of the backup to THIS PC (on / off)(매주 이 PC로 백업 자동 복사, 켜기 / 끄기) | (Windows에만 있어요) |
| 14 | Subdomains with DuckDNS(DuckDNS 하위 도메인) | 13 |
| 15 | On / off: the whole garden, or only off the internet; check-up, clean-up(켜기 / 끄기, 점검, 정리) | 14 |
| 16 | AI cost control: LiteLLM(AI 비용 관리: LiteLLM) | 15 |
| 17 | Website name swap(웹사이트 이름 바꾸기) | 16 |
| 18 | Desktop shortcuts: Garden terminal (SSH) and Garden screen (Remote Desktop)(바탕화면 바로 가기: Garden terminal(SSH)과 Garden screen(원격 데스크톱)) | (Windows에만 있어요) |
| 19 | Reach the garden from anywhere, privately (Tailscale): on / off(어디서나 나만 정원에 닿기(Tailscale): 켜기 / 끄기) | 17 |
| 20 | Rename or move a file or folder(파일이나 폴더 이름 바꾸기 또는 옮기기) | 18 |
| 21 | Find the garden computer again(정원 컴퓨터 다시 찾기) | (Windows만) |

`garden.sh`가 보여 주는 메시지는 Ubuntu 번호를 써요. Windows에서는 Ubuntu 13~16은
1을 더하고(Ubuntu의 "menu M, then 13"은 Windows의 More 14예요), Ubuntu 17과 18은 2를
더하세요(Tailscale은 Windows More 19, 이름 바꾸기는 Windows More 20이에요).

### More 5) Open a terminal on the garden computer — 정원 컴퓨터의 터미널 열기
- **무엇을 하나요:** 정원 컴퓨터의 터미널이 있는 새 창을 열어요(SSH, 또는 정원 폴더에 있는 WSL Ubuntu).
- **무엇을 설치하거나 바꾸나요:** 없어요.
- **되돌리려면:** 창을 닫으세요.

### More 8) Look inside the database (Adminer, on the garden computer: http://localhost:8082 there) — 데이터베이스 안 보기(Adminer, 정원 컴퓨터에서: 그 컴퓨터의 http://localhost:8082)
- **무엇을 하나요:** 정원 컴퓨터에서 `garden.sh dbweb`을 실행해요(Ubuntu More 8과 같아요). Adminer는 정원 컴퓨터 자신에서만 대답해요(127.0.0.1:8082).
- **되돌리려면:** 정원 컴퓨터에서 `bash garden.sh dbweb-stop`(More 5가 그 컴퓨터의 터미널을 열어요).

### More 10) Copy the newest backup to THIS PC — 최신 백업을 이 PC로 복사하기
- **무엇을 하나요:** 정원 컴퓨터에서 '새' 백업을 만들고, 그다음 최신 백업을 이 PC의 정원 폴더 안 `backups` 폴더로 복사해요.
- **무엇을 설치하거나 바꾸나요:** 거기에 새 백업 하나, 여기에 파일 하나.
- **되돌리려면:** 필요 없으면 이 PC에서 파일을 지우세요.

### More 13) Weekly automatic copy of the backup to THIS PC (on / off) — 매주 이 PC로 백업 자동 복사(켜기 / 끄기)

![More 13: 사본을 둘 곳, 그다음 매주 작업](../images/windows-13-backup-copy.png)

- **무엇을 하나요:** 먼저 사본을 어디에 둘지 물어봐요: `1` 창에서 폴더 고르기(이 PC, USB 드라이브, Google Drive for desktop, 또는 창 왼쪽 "Network"(네트워크) 아래의 NAS), `2` `C:\Backups\weekly` 같은 폴더나 `\\192.168.1.20\backups` 같은 네트워크 경로를 직접 입력하기, `3` 그대로 두기(처음에는 정원 폴더 안 `backups\from-garden-computer\`). 고른 곳 안에 `junsgarden-backups` 폴더를 만들고, 거기에 쓸 수 있는지 확인하고, 기억해 두고, OneDrive 안이면 경고해요. 그다음 작업 스케줄러 작업 "Jun's Garden - copy backup to this PC"를 추가해요. 매주 일요일 11:00에 실행돼요(놓쳤다면 PC가 켜지자마자). 이 작업은 최신 백업을 그 폴더로 복사하고(새로 만들지는 않아요), 최신 8개를 보관해요. 지금 첫 복사를 할지 물어봐요.
- **언제 쓰나요:** 기억하지 않아도 다른 컴퓨터에 두 번째 사본을 두고 싶을 때. 그 시간에 정원 컴퓨터가 켜져 있어야 해요. NAS라면: 먼저 파일 탐색기에서 NAS를 한 번 열고 "Remember my credentials"(내 자격 증명 기억)에 체크해서 로그인하세요. 그렇지 않으면 메뉴가 거기에 쓸 수 없어요.
- **무엇을 설치하거나 바꾸나요:** 그 작업. `.garden-target-windows.txt`에 고른 곳. 고른 폴더의 사본들. `logs\backup-copy.log`의 기록.
- **되돌리려면:** More 13을 다시 고르세요: "Change where the copies go?"(사본을 둘 곳을 바꿀까요?)를 묻고, `n`이면 "Remove the weekly copy?"(매주 복사를 지울까요?)를 물어요. 이미 만든 사본은 남아요.
- **안내서:** [`13-backups-and-folders.md`](13-backups-and-folders.md), "Windows PC에 사본 두기".

### More 17) Website name swap: use another web name than junsgarden.zapto.org — 웹사이트 이름 바꾸기: junsgarden.zapto.org 말고 다른 웹 이름 쓰기
- **무엇을 하나요:** 정원 컴퓨터에서 이름 바꾸기를 실행하고(Ubuntu More 16), 그다음 이 PC에 있는 파일 사본도 같은 이름으로 바꿔요. 그래야 나중에 이 PC에서 INSTALL을 해도 예전 이름이 다시 복사되지 않아요.
- **무엇을 설치하거나 바꾸나요:** 두 컴퓨터의 같은 파일들. 이 PC의 예전 사본은 `logs\web-name-change-windows_DATE\`로 가요.
- **되돌리려면:** 예전 이름으로 다시 실행하세요.

### More 18) Desktop shortcuts: Garden terminal (SSH) and Garden screen (Remote Desktop) — 바탕화면 바로 가기: Garden terminal(SSH)과 Garden screen(원격 데스크톱)
- **무엇을 하나요:** Windows 바탕화면에 바로 가기 두 개를 만들어요: **Garden terminal**은 정원 컴퓨터의 터미널을 열어요(SSH, 암호화돼요. Windows 안의 Ubuntu를 쓰면 그것을 열어요). **Garden screen**은 따로 있는 정원 컴퓨터로 원격 데스크톱 연결(Remote Desktop Connection)을 열어요.
- **언제 쓰나요:** 내 PC에서 정원 컴퓨터로 일하고 싶을 때, 또는 다른 방에서 그 컴퓨터의 화면을 보고 싶을 때.
- **무엇을 설치하거나 바꾸나요:** 바탕화면에 파일 두 개. 정원 컴퓨터에서는 아무것도 바꾸지 않아요. Garden screen을 쓰려면 먼저 정원 컴퓨터의 Settings(설정)에서 Remote Desktop(원격 데스크톱)을 켜세요(안내서 20).
- **되돌리려면:** 바로 가기 두 개를 지우세요.
- **안내서:** [20-ssh-and-remote-desktop.md](20-ssh-and-remote-desktop.md)

### More 20) Rename or move a file or folder — 파일이나 폴더 이름 바꾸기 또는 옮기기
- **무엇을 하나요:** 정원 컴퓨터에서 이름 바꾸기 도구를 실행하고(Ubuntu 더 보기 18), 이 PC에 있는 정원 폴더 사본도 똑같이 맞춰요: 바뀐 파일을 여기로 복사하고 같은 사이트 파일의 이름을 바꿔요. 그래서 나중에 이 PC에서 INSTALL을 해도 옛 이름이 돌아오지 않아요.
- **무엇을 설치하거나 바꾸나요:** 두 컴퓨터의 같은 파일. 이 PC의 옛 파일은 `logs\rename-windows_DATE\`에 남아요.
- **되돌리려면:** 이름을 바꿔서 다시 실행하세요.

### More 21) Find the garden computer again (if its address changed) — 정원 컴퓨터 다시 찾기(주소가 바뀌었으면)
- **무엇을 하나요:** 정원 컴퓨터가 저장한 주소에서 아직 대답하는지 확인해요. 아니면 SSH 문(포트 22)이 열린 집의 컴퓨터를 살펴서, 같은 네트워크 카드(MAC 주소)나 이름을 가진 것을 찾아 새 주소를 저장해요. 못 찾으면 목록에서 다시 고르게 해요. 메뉴는 모든 작업 전과 매주 백업 복사 전에 이것을 스스로 해요.
- **언제 쓰나요:** 공유기가 주소를 예약하지 못할 때, 또는 공유기를 바꾼 뒤.
- **무엇을 설치하거나 바꾸나요:** `.garden-target-windows.txt`의 저장된 주소(그리고 네트워크 카드 ID와 이름)만. 포트 22를 두드려 보기만 하고, 로그인하지 않아요.
- **안내서:** [`14-reserve-your-ip.md`](14-reserve-your-ip.md), "공유기가 주소를 예약하지 못하면".

---

## 모든 명령 단어

정원 폴더에서 `bash garden.sh WORD`를 입력하세요. 이 목록에 없는 단어를 쓰면
`garden.sh` 맨 위에 있는 짧은 도움말이 나와요. 명령을 입력하는 것이 처음인가요?
터미널을 한 단계씩 처음부터 안내하는 bash 입문서가 영어로 있어요:
[`22-bash-walkthrough.md`](../22-bash-walkthrough.md)(영어).

| 단어 | 같은 메뉴 | 단어 | 같은 메뉴 |
|---|---|---|---|
| `install` | 메뉴 1 | `start` / `stop` | 메뉴 2 / 3 |
| `open` | 메뉴 4 | `check` | 메뉴 5 |
| `backups` | 메뉴 6 | `restore [FILE]` | 메뉴 7 |
| `settings` | 메뉴 8 | `parts` | 메뉴 9 |
| `users` | U | `keys` | K |
| `addresses` | A | `helpme` | H |
| `language` | L | `lockdown` | U, 8 |
| `backup [LABEL]` | Backups 1 | `verify [FILE]` | Backups 3 |
| `nas` | Backups 4 | `cloud` | Backups 5 |
| `photos` | Backups 11 | `public` | More 1 |
| `network` | More 2 | `email` | More 3 |
| `password [NAME]` | More 4 | `remote` | More 5 |
| `logs` | More 6 | `rebuild` | More 7 |
| `dbweb` | More 8 | `ai` | More 9 |
| `icon` | More 10 | `files` | More 11 |
| `uninstall` | More 12 | `subdomains` | More 13 |
| `offline` | More 14, 3 | `llm` | More 15 |
| `webname` | More 16 | `tailscale` | More 17 |
| `rename` | More 18 | `power` | More 14 |
| `off` / `on` | More 14, 1 / 2 | `checkup` / `cleanup` | More 14, 4 / 5 |
| `renames` | More 18, 4 | | |

메뉴 번호가 없는 명령:

- `bash garden.sh wiki`: 위키 설정을 다시 실행해요(위키 추가 zip이 있으면 정원 페이지를 넣어요).
- `bash garden.sh dbweb-stop`: Adminer를 멈추고 지워요.
- `bash garden.sh redo-renames`: 옛 백업을 손으로 복원한 뒤, 그 이름 바꾸기를 다시 해요.
- `bash garden.sh db`: 데이터베이스 명령줄이에요(SQL을 입력해요. 나가려면 `exit`). 안내서: [`12-your-database.md`](../12-your-database.md)(영어).
- `bash garden.sh test`: 웹사이트의 Rust 테스트를 실행해요(처음에는 느려요).

---

## 나중에 새것 더하기

정원은 이것들이 하나도 없어도 작동해요. 각각은 원할 때 더하는 추가 기능이고,
각각 다시 끌 수도 있어요:

| 추가 기능 | 켜기 | 끄기 |
|---|---|---|
| 정원 도구, 위키, n8n | Parts (9) | Parts (9), 같은 번호, 그다음 A |
| 위키 페이지(`junsgarden-wiki-addon.zip`) | 위키 ON, 그다음 `bash garden.sh wiki` | Parts에서 위키 OFF |
| 사진(`junsgarden-photos-addon.zip`) | Backups 11 | (평범한 파일이에요. 실행되는 것이 없어요) |
| 집 Wi-Fi, 연습, 또는 온라인(정문) | More 1 | More 14, 3 |
| 하위 도메인(DuckDNS) | More 13(Windows 14) | Subdomains 3. More 14, 3은 업데이트 프로그램을 멈춰요 |
| LiteLLM(AI 지출 한도) | More 15(Windows 16) | LiteLLM 메뉴, 7 |
| 이메일(Gmail) | More 3 | K, Gmail app password, 4 |
| NAS 사본 | Backups 4(두 번째, 세 번째 장소: Backups 13) | Backups 8 |
| Google Drive 사본 | Backups 5 | Backups 8 |
| 로컬 AI 컴퓨터(Ollama) | More 9 | More 9, 주소 비우기 |
| Windows PC로 매주 복사 | Windows More 13 | Windows More 13 다시 |

내 프로그램을 새로 더하기 전에: **하위 도메인**으로 인터넷에 올리세요(More 13).
그러면 같은 문 80과 443으로 들어오고, 내 로그인 페이지 뒤에 둘 수 있어요. 그래서
공유기에서 새 포트를 열 일이 절대 없어요.
