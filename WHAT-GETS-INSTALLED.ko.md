# 내 컴퓨터에 설치되는 것

> 영어 원문: [WHAT-GETS-INSTALLED.md](WHAT-GETS-INSTALLED.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 정원 설치 프로그램이 설치할 수 있는 모든 프로그램, 그것이 있는 이유, 그리고 공식 페이지예요. 그 밖에는 아무것도 설치하지 않아요.
- 세 묶음이에요: **꼭 필요한 것**(설치가 해요), **선택**(설치가 예/아니요로 물어요), **고급 선택**(나중에, 메뉴 `M`에서).
- 꼭 필요한 것에는 Ubuntu 업데이트, SSH 서버, ufw 방화벽, Docker, MariaDB 데이터베이스, 내 웹사이트, 자동 백업이 있어요.
- 다른 사람이 내 컴퓨터에 들어오게 하는 것은 하나도 없어요. 설치는 적혀 있는 공식 출처에서 프로그램을 내려받기만 해요.
- 인터넷에서 들어올 수 있는 포트는 80과 443(정문 단계 뒤의 Caddy)뿐이에요. SSH(22)는 집 네트워크에서만 돼요. 나머지는 이 컴퓨터 안에서만 돼요.
- 점검표(메뉴 `5`)가 실행할 때마다 이 모든 것을 확인해요.
- 지우려면: 메뉴 `M`, 그다음 `12`(Uninstall, 제거). 사진, 데이터베이스, 백업은 남겨 둬요.

---

정원 설치 프로그램이 설치할 수 있는 프로그램은 모두 이 페이지에 있어요: 무엇인지, 왜
있는지, 그리고 더 읽을 수 있는 공식 페이지. 그 밖에는 아무것도 설치하지 않아요. 설치
프로그램은 실행하기 전에 각 프로그램과 명령을 화면에도 보여 주고, 화면에 나온 모든
내용을 `logs/`에 저장해요.

세 그룹이 있어요:
- **꼭 필요한 것.** 이것이 없으면 웹사이트가 실행되지 않아요. 설치가 알아서 해요.
- **선택.** 추천하는 것이에요. 설치가 하나씩 yes 또는 no로 물어봐요.
- **고급 선택.** 나중에 메뉴에서 해요(`M` = More, 더 보기).

다른 사람에게 내 컴퓨터에 들어오는 권한을 주는 것은 하나도 없어요. 모든 비밀번호는 내
컴퓨터에서 만들거나 내가 직접 입력해요. 설치 프로그램은 아무 데도 아무것도 보내지 않아요:
아래에 적힌 공식 장소에서 프로그램을 다운로드하기만 해요.

---

## 꼭 필요한 것

| 무엇 | 무엇인가요 | 왜 | 실행되는 곳 | 공식 페이지 |
|---|---|---|---|---|
| Ubuntu 업데이트 | `apt`(Ubuntu의 앱 스토어)를 통한 Ubuntu 자체 업데이트 | 보안 수정이 가장 먼저예요 | Ubuntu | <https://ubuntu.com/server/docs/how-to/software/package-management/> |
| git, curl, openssl, rsync | 작은 기본 도구 | git은 변경 기록을 보관해요. curl은 웹 페이지를 가져와요. openssl은 무작위 비밀번호를 만들어요. rsync는 다른 프로그램이 쓰는 복사 도구예요(직접 실행할 일은 없어요). | Ubuntu | <https://git-scm.com/doc>, <https://curl.se/docs/> |
| OpenSSH 서버 | 집에 있는 다른 컴퓨터에서 이 컴퓨터의 터미널을 쓸 수 있게 해 줘요 | 노트북 뚜껑을 닫은 채로 관리할 수 있어요 | Ubuntu, 포트 22, 집 네트워크에서만 | <https://ubuntu.com/server/docs/openssh-server> |
| ufw 방화벽 | 어떤 연결이 들어와도 되는지 정해요 | 집에서 오는 SSH와 나중에 80, 443 말고는 모두 막아요 | Ubuntu | <https://ubuntu.com/server/docs/how-to/security/firewalls/> |
| Docker Engine과 Compose | 각 프로그램을 자기만의 밀봉된 상자("컨테이너")에서 실행해요 | 프로그램이 Ubuntu나 서로를 망가뜨릴 수 없어요. 같은 설정이 어느 컴퓨터에서나 작동해요. | Ubuntu | <https://docs.docker.com/engine/install/ubuntu/> |
| MariaDB 11 | 데이터베이스예요. MySQL과 같은 언어를 써요. | 식물, 댓글, 로그인, 위키 페이지를 보관해요 | Docker, 안쪽에서만(포트 없음) | <https://mariadb.com/kb/en/documentation/> |
| Jun's Garden 웹사이트 | Rust로 만든 내 사이트. `rust/`로 내 컴퓨터에서 만들어요 | 이것이 내 정원이에요: 식물 목록, 갤러리, 블로그, 데이터, 방명록, 식물 관리, 휴대폰 앱의 서버 | Docker, `127.0.0.1:8030` | <https://www.rust-lang.org/learn>, `docs/rust-port/CODE-TOUR.md` |
| 도구 상자(Toolbox) | 데이터베이스와 압축 도구가 들어 있는 아주 작은 Alpine Linux 컨테이너 | 백업, 복원, 비밀번호 정하기가 Ubuntu와 Windows에서 똑같이 작동해요 | Docker, 작업하는 동안에만 | <https://alpinelinux.org/about/> |
| 백업 일정 | 내 cron(Ubuntu의 알람 시계)에 있는 한 줄 | 한 시간마다 바뀐 것을 세어요: 바쁠 때는 한 시간마다, 수정이 조금 있으면 6시간 안에, 조용하면 적어도 일주일에 한 번 전체 백업. 약 8 GB로 솎아 내요(원하면 매일 밤 새벽 2시 30분에 하나) | Ubuntu | <https://manpages.ubuntu.com/manpages/noble/man5/crontab.5.html> |
| 스왑 파일(작은 노트북만) | 추가 메모리로 쓰는 디스크 공간 | 웹사이트를 만들려면 메모리가 약 3 GB 필요해요 | Ubuntu | <https://manpages.ubuntu.com/manpages/noble/man8/swapon.8.html> |

## 선택(설치가 물어봐요)

| 무엇 | 무엇인가요 | 왜 | 실행되는 곳 | 공식 페이지 |
|---|---|---|---|---|
| 정원 도구(`/beds`) | 작은 Python 프로그램: 화단 계획, 점검, 진단, 이름표 스캔, 지출, 위키 대기열 | 원래 운영 중인 정원의 일부였어요. 휴대폰 앱의 카메라 도구도 이것을 써요. | Docker, `127.0.0.1:8025` | `beds/README.md`, <https://docs.python.org/3/> |
| 위키(고르지 않으면 꺼져 있어요) | Wiki.js 2, 현대적인 위키. 정원 글은 따로 내려받는 `junsgarden-wiki-addon.zip`에 있어요. 없으면 위키는 빈 채로 시작해요 | 내가 편집하는 나의 백과사전. 원할 때만: 프로그램을 하나 더 내려받고, 페이지를 넣는 데 약 30분 걸려요 | Docker, `127.0.0.1:3000` | <https://docs.requarks.io>, <https://js.wiki> |
| 이메일(Gmail) | 내 Gmail이 사이트의 이메일(비밀번호 링크)을 보내요 | 집 인터넷에서 바로 보낸 이메일은 스팸함으로 가요 | 설정만, 설치하는 것 없음 | <https://support.google.com/accounts/answer/185833> |
| 바탕화면 아이콘 | 앱 목록과 바탕화면의 "Jun's Garden" | 한 번 클릭으로 메뉴를 열어요 | Ubuntu | <https://specifications.freedesktop.org/desktop-entry-spec/latest/> |
| Docker Desktop(Windows만, 고를 때만) | Docker Inc.의 Windows 앱: 정원의 프로그램을 돌리고 버튼(시작, 멈춤, 메시지)과 함께 보여 줘요. winget으로 설치해요. 개인 사용은 무료 | Windows PC에서 정원을 다루는 쉬운 방법. 다른 방법인 Ubuntu 안의 Docker는 Windows에 더 설치할 것이 없어요 | Windows | <https://docs.docker.com/desktop/setup/install/windows-install/>, 안내서 26 |

## 고급 선택(나중에, 메뉴 `M`에서)

| 무엇 | 무엇인가요 | 왜 | 실행되는 곳 | 공식 페이지 |
|---|---|---|---|---|
| Caddy | 정문(front door) 역할을 하는 웹 서버. HTTPS가 자동이에요 | 예전 사이트와 같은 관문이 있는 문 하나. 자물쇠 인증서를 스스로 받아 와요. | Ubuntu, 포트 **80과 443**(공유기가 전달하는 유일한 두 포트) | <https://caddyserver.com/docs/> |
| Authelia | `/auth`에 있는 내 로그인 페이지 | 정원 도구는 로그인한 뒤에만 열려요. 비밀번호를 여러 번 틀리면 그 이름이 잠시 잠겨요. | Docker, `127.0.0.1:9091` | <https://www.authelia.com/overview/prologue/introduction/> |
| No-IP 업데이트 프로그램(DUC) | 몇 분마다 No-IP에 집 인터넷 주소를 알려 줘요 | `junsgarden.zapto.org`가 계속 우리 집을 가리키게 해요 | Ubuntu | <https://www.noip.com/support/knowledgebase/install-linux-3-x-dynamic-update-client-duc> |
| 공유기(router) 설정 | 공유기에서 직접 해요: 노트북 주소 예약, 80과 443 전달 | 인터넷 방문자가 Caddy에, 그리고 Caddy에만 닿을 수 있어요 | 내 공유기 | `START-HERE/14-reserve-your-ip.md`(한국어: [START-HERE/ko/14-reserve-your-ip.md](START-HERE/ko/14-reserve-your-ip.md)) |
| n8n | 끌어다 놓기로 만드는 자동화("흙이 마르면 이메일 보내기") | 준비가 되면 재미있고 강력해요 | Docker, `127.0.0.1:5678` | <https://docs.n8n.io> |
| Adminer | 데이터베이스를 보는 한 페이지짜리 웹 프로그램 | SQL을 입력하지 않고 표 안을 볼 수 있어요 | Docker, `127.0.0.1:8082`, 쓰는 동안에만 | <https://www.adminer.org> |
| LiteLLM(메뉴 `M`, `15`) | 월 지출 한도가 있는, AI를 위한 개인 주소 하나 | 유료 AI 키가 내가 정한 것보다 더 쓰지 못하게 | Docker, `127.0.0.1:4000` | <https://docs.litellm.ai> |
| DuckDNS 업데이트 프로그램(메뉴 `M`, `13`) | 하위 도메인을 위해 내 집 주소를 DuckDNS에 알려 주는 cron 한 줄 | 로그인 페이지 뒤의 무료 추가 이름(wiki., n8n., ...) | Ubuntu(cron) | <https://www.duckdns.org/install.jsp> |
| Tailscale(메뉴 `M`, `17`) | Tailscale 자체 프로그램: 내 기기만 들어오는 개인 네트워크 | 공유기에 포트를 열지 않고 어디서나 정원 컴퓨터에 닿기 | Ubuntu | <https://tailscale.com/kb/1017/install> |
| 로컬 AI(Ollama) | 집에 있는 성능 좋은 컴퓨터(이 노트북 말고)에서 돌리는 AI 모델 | 사이트의 AI 상자를 위한 비공개 무료 AI | 그 다른 컴퓨터 | <https://github.com/ollama/ollama>, `START-HERE/15-local-ai.md` |
| cifs-utils(NAS) | 네트워크 드라이브(SMB)를 연결하는 Ubuntu 프로그램. Ubuntu 스토어에서 받아요 | 모든 백업의 두 번째 사본을 NAS에 둬요 | Ubuntu(메뉴 6, 그다음 4) | <https://manpages.ubuntu.com/manpages/noble/man8/mount.cifs.8.html> |
| rclone(Google Drive) | 파일을 클라우드 저장소로 복사하는 잘 알려진 무료 프로그램. Ubuntu 스토어에서 받아요 | 일주일에 한 번 백업 사본을 집 밖에 둬요(3-2-1) | Ubuntu(메뉴 6, 그다음 5) | <https://rclone.org/drive/> |
| Android Studio(본인 선택, 설치 프로그램이 설치하지 않아요) | Android 앱을 만드는 Google 프로그램 | `android-app/`에 있는 정원 휴대폰 앱을 작업할 때 | 내 데스크톱 | <https://developer.android.com/studio>, `START-HERE/17-android-app-and-ai-coding.md` |

위키의 한 번만 쓰는 설정 도우미는 공식 **python:3.12-alpine** 이미지
(<https://hub.docker.com/_/python>) 위에 `docker/wiki/`로 여기서 만들어지고,
실행이 끝나면 버려져요.

---

## 문(포트): 어디에서 무엇에 닿을 수 있는지

| 포트 | 프로그램 | 누가 닿을 수 있나요 |
|---|---|---|
| 22 | SSH | 집 네트워크 안의 컴퓨터만(방화벽 규칙) |
| 80, 443 | Caddy(정문 단계 뒤) | 인터넷. 공유기가 전달하는 포트는 이 두 개뿐이에요. |
| 8030, 8025, 3000, 9091, 5678, 8082, 4000 | 웹사이트, 정원 도구, 위키, 로그인 페이지, n8n, Adminer, LiteLLM | 이 컴퓨터만(`127.0.0.1`). Caddy가 방문을 이쪽으로 넘겨줘요. |
| 3306 | 데이터베이스 | Docker 밖에서는 아무도 못 닿아요 |

점검표(메뉴 `5`)는 실행할 때마다 이것을 모두 확인해요.

## 무엇이든 직접 확인하는 방법

| 보고 싶은 것 | 정원 폴더의 터미널에 이것을 입력하세요 |
|---|---|
| 지금 실행 중인 프로그램 | `docker ps` |
| 프로그램이 해 온 말(로그) | `docker logs --tail 50 garden-site` (또는 garden-db, junsgarden-ingest, garden-wiki, garden-auth) |
| 모든 설정(비밀번호 없이) | 메뉴 `8` 설정(Settings) |
| 각 프로그램이 시작되는 방법 | `cat docker-compose.yml` (모든 줄에 설명이 있어요) |
| 각 프로그램이 만들어지는 방법 | `cat rust/Dockerfile`, `cat docker/tools/Dockerfile`, `cat beds/Dockerfile` |
| 정문 설정 | `cat /etc/caddy/Caddyfile` |
| 설치 프로그램이 언제 무엇을 했는지 | `logs/` 안의 파일들, 그리고 `logs/install-progress.txt` |
| 방화벽 규칙 | `sudo ufw status verbose` |
| 백업 일정 줄 | `crontab -l` |

## 지우기

메뉴 `M`, 그다음 `12`(Uninstall, 제거하기)는 정원의 프로그램, 백업 일정, 아이콘을
멈추고 지우고, Caddy와 No-IP 업데이트 프로그램도 지워요. 사진, 데이터베이스, 백업은
남겨 둬요. Docker를 지우는 명령은 화면에 보여 주기만 해요. 지울지는 본인이 정해요.
