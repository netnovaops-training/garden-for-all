# 21. 모든 프로그램: 왜 여기 있고, 어디서 오는지

> 영어 원문: [../21-software-and-sources.md](../21-software-and-sources.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 이 페이지는 정원이 쓸 수 있는 **모든** 프로그램을 보여 줘요. 꼭 필요한 것과 선택인 것
  모두요. 각각 한두 문장으로 설명하고, 만든 곳의 공식 문서 링크도 있어요.
- 모든 프로그램은 **공식** 출처에서 와요: Ubuntu 자체 스토어, 만든 곳의 자체 패키지,
  또는 만든 곳의 공식 Docker 이미지예요.
- 정원이 쓰는 버전도 적혀 있어요. 그래서 알맞은 문서를 읽을 수 있어요.
- HTTPS(자물쇠)는 **Let's Encrypt**에서 와요. 무료이고, 저절로 돼요. 어떻게 작동하는지는
  아래에서 설명해요.
- Ubuntu 자체(안내서 01) 말고는 손으로 설치할 것이 없어요.

---

**버전 읽는 법.** `mariadb:11`은 "가장 새로운 MariaDB 11"이라는 뜻이에요. 메뉴가
프로그램을 업데이트할 때(그 버전의 가장 새로운 이미지를 내려받아요) 같은 버전 안에서만
바뀌어요. 그래서 여기 있는 문서 링크가 계속 맞아요.

## 꼭 필요한 것: 정원 컴퓨터에서(Ubuntu)

| 프로그램 | 버전 | 정원에 필요한 이유 | 공식 문서 |
|---|---|---|---|
| **Ubuntu**(운영체제) | 24.04 LTS | 정원이 돌아가는 운영체제예요. "LTS"는 몇 년 동안 무료 보안 업데이트를 받는다는 뜻이에요 | <https://ubuntu.com/server/docs> |
| **Raspberry Pi OS**(Pi에서 Ubuntu 대신) | 64비트 | Pi 자체의 시스템이에요. Raspberry Pi 5용이에요 | <https://www.raspberrypi.com/documentation/computers/os.html> |
| **apt** 업데이트, `git`, `curl`, `openssl`, `rsync`, `unzip`, `ca-certificates` | Ubuntu 스토어에서 | 기본 도구예요: 업데이트, 변경 기록, 내려받기, 튼튼한 무작위 비밀번호 만들기, 복사하기, 추가 기능 압축 풀기 | <https://ubuntu.com/server/docs/package-management> |
| **OpenSSH server**(원격 접속 서버) | Ubuntu 스토어에서 | Windows PC에서 정원 컴퓨터로 일할 수 있게 해 줘요. 암호화돼요(안내서 20) | <https://www.openssh.com/manual.html> |
| **ufw**(방화벽) | Ubuntu 스토어에서 | 들어오는 것을 모두 막아요. 정원이 허락한 것만 빼고요 | <https://ubuntu.com/server/docs/how-to/security/firewalls/> |
| **Docker Engine**(상자 실행기) | Docker의 자체 Ubuntu 패키지 | 각 프로그램을 자기만의 작은 상자에서 돌려요(안내서 02a) | <https://docs.docker.com/engine/install/ubuntu/> |
| **cron**(예약 실행, Ubuntu에 이미 있어요) | | 밤마다 새벽 2시 30분에 백업을 돌려요 | <https://manpages.ubuntu.com/manpages/noble/man5/crontab.5.html> |

## 꼭 필요한 것: 정원 자체의 상자들(Docker 이미지)

| 프로그램 | 버전 | 이유 | 공식 문서 |
|---|---|---|---|
| **MariaDB**(데이터베이스. **MySQL**과 같은 언어를 써요) | `mariadb:11` | 내 식물, 댓글, 로그인 정보를 담아요. MariaDB는 MySQL을 처음 만든 사람들이 시작했고, 같은 방식으로 작동해요 | <https://mariadb.com/kb/en/documentation/> 그리고 이미지: <https://hub.docker.com/_/mariadb> |
| **Your website**(내 웹사이트. 내 컴퓨터에서 `rust/`로 만들어요) | Rust 1.95, Debian 12 "bookworm" 위에서 | 정원 웹사이트 그 자체예요. 내려받지 않고, 여기 있는 코드로 만들어요 | <https://www.rust-lang.org/learn>, <https://hub.docker.com/_/rust>, <https://hub.docker.com/_/debian> |
| **The toolbox**(도구 상자. 여기서 만들어요) | Alpine Linux 3.20 | 백업, 복원, 비밀번호. 어느 컴퓨터에서나 똑같아요 | <https://alpinelinux.org/about/>, <https://hub.docker.com/_/alpine> |

## 선택 부분(메뉴에서 켜요)

| 프로그램 | 버전 | 이유 | 공식 문서 |
|---|---|---|---|
| **Garden tools**(정원 도구. `/beds`, 여기서 만들어요) | Python 3.12 | 화단 계획, 점검, 진단, 태그 스캔, 지출 목록, 위키 대기열 | <https://docs.python.org/3.12/>, <https://hub.docker.com/_/python> |
| **Wiki.js**(위키) | 2 | 내 정원 백과사전이에요 | <https://docs.requarks.io> |
| 위키의 한 번 쓰는 도우미(여기서 만들어요) | Python 3.12, Alpine | 위키를 설정하고 페이지를 넣어요. 그다음 버려져요 | <https://hub.docker.com/_/python> |
| **n8n**(자동화) | 만든 곳의 최신 이미지 | "이런 일이 생기면, 저렇게 해" 자동화예요 | <https://docs.n8n.io> |
| **LiteLLM**(AI 연결 창구) | `main-stable` | AI를 위한 비공개 주소 하나예요. 한 달 지출 한도가 있어요 | <https://docs.litellm.ai> |
| **Adminer**(데이터베이스 보기) | latest | 한 페이지짜리 데이터베이스 보기 프로그램이에요. 쓰는 동안만 켜져요 | <https://www.adminer.org> |

## 인터넷에 올리기: 정문

| 프로그램 | 버전 | 이유 | 공식 문서 |
|---|---|---|---|
| **Caddy**(정문) | Caddy의 자체 Ubuntu 패키지(버전 2) | 정문이에요: 포트 80과 443에서 방문자를 받고, 자물쇠를 받아 와요(아래) | <https://caddyserver.com/docs/> |
| **Let's Encrypt**(서비스예요. 설치하는 것은 없어요) | | 자물쇠 인증서를 발급하는 무료 비영리 기관이에요 | <https://letsencrypt.org/how-it-works/> |
| **Authelia**(내 로그인 페이지) | `4.39` | 정원 도구 앞에 있는 로그인 페이지예요 | <https://www.authelia.com/integration/proxies/caddy/> |
| **No-IP DUC**(업데이트 프로그램) | noip.com에서 | 내 웹 이름이 계속 우리 집을 가리키게 해요 | <https://www.noip.com/support/knowledgebase/install-linux-3-x-dynamic-update-client-duc> |
| **DuckDNS**(서비스예요. cron 한 줄) | | 하위 도메인(subdomain)을 위한 무료 추가 이름이에요 | <https://www.duckdns.org/install.jsp> |
| **Avahi**(Ubuntu에 이미 있어요) | | 집에 있는 기기들이 `garden-lab.local`을 찾게 해 줘요 | <https://avahi.org> |

### Let's Encrypt가 `http`를 `https`로 바꾸는 방법
1. `https://`로 시작하는 웹 주소에는 **인증서**(certificate)가 필요해요. 인증서는 작은
   파일이에요. "이곳은 정말 junsgarden.zapto.org예요"라고 증명해요. 인증서가 유효하면
   브라우저가 자물쇠를 보여 줘요.
2. **Let's Encrypt**는 이 인증서를 무료로 나눠 줘요. 그 이름이 자기 컴퓨터를 가리킨다고
   증명할 수 있는 사람이면 누구에게나요.
3. 누군가 내 이름에 처음 방문하면 **Caddy가 스스로 인증서를 요청해요**.
   Let's Encrypt는 내 이름으로 접속해 봐서 확인해요(그래서 포트 80과 443이 정원
   컴퓨터까지 닿아야 해요). 올바른 대답이 오면 인증서가 도착해요. 보통 1분 안에요.
4. 그때부터 Caddy는 내 사이트를 `https`로 보여 줘요. 그리고 `http://`를 입력한 사람은
   `https://` 주소로 보내요.
5. 인증서는 90일 동안 쓸 수 있어요. **Caddy가 스스로 갱신해요.** 90일이 끝나기 훨씬 전에요.
   나는 아무것도 하지 않아도 돼요.

이것에 대한 Caddy의 페이지: <https://caddyserver.com/docs/automatic-https>.
연습 모드(`.test` 이름)에서는 Caddy가 대신 자기만의 연습용 인증서를 만들어요. 그래서
거기서는 브라우저가 경고를 보여 줘요.

## 백업

| 프로그램 | 버전 | 이유 | 공식 문서 |
|---|---|---|---|
| **cifs-utils**, **smbclient**(NAS 연결 도구) | Ubuntu 스토어에서 | 내 NAS(SMB 3.1.1)를 연결하고 네트워크에서 찾아요 | <https://wiki.samba.org/index.php/Mounting_samba_shares_from_a_unix_client> |
| **rclone**(클라우드 복사) | Ubuntu 스토어에서 | 매주 Google Drive로 복사해요 | <https://rclone.org/drive/> |

## 내 Windows PC에서(쓸 때만)

| 프로그램 | 이유 | 공식 문서 |
|---|---|---|
| **WSL**(Windows 자체의 Ubuntu, Microsoft가 만들었어요) | 이 PC에서 정원을 돌려요 | <https://learn.microsoft.com/windows/wsl/about> |
| **Docker Desktop**(선택, Docker Inc.가 만들었어요. 개인 사용은 무료) | 이 PC에서 정원의 프로그램을 돌리는 쉬운 방법이에요. 버튼으로 보고, 시작하고, 멈춰요(안내서 26). 내가 고를 때만 winget으로 설치해요 | <https://docs.docker.com/desktop/setup/install/windows-install/>, WSL: <https://docs.docker.com/desktop/features/wsl/> |
| **PowerShell**(Windows에 들어 있어요) | Windows 메뉴와 안내 설치 프로그램은 PowerShell 스크립트예요 | <https://learn.microsoft.com/powershell/> |
| **OpenSSH client**(Windows 10과 11에 들어 있어요) | SSH, 그리고 정원 컴퓨터로 파일 복사하기 | <https://learn.microsoft.com/windows-server/administration/openssh/openssh_overview> |
| **Rufus**(USB 만들기) | Ubuntu USB 스틱을 만들어요 | <https://rufus.ie> |
| **Remote Desktop Connection**(원격 데스크톱 연결, Windows에 들어 있어요) | 정원 컴퓨터의 화면을 보여 줘요(안내서 20) | <https://learn.microsoft.com/windows-server/remote/remote-desktop-services/clients/remote-desktop-clients> |
| **WinSCP**(선택) | Linux 컴퓨터와 Windows 사이에서 SSH로, 끌어다 놓기로 파일을 복사해요(안내서 20, 한 단계씩) | 내려받기: <https://winscp.net/eng/download.php>, 안내서: <https://winscp.net/eng/docs/guide_connect> |

## 내 휴대폰이나 태블릿에서(선택)

| 프로그램 | 이유 | 공식 페이지 |
|---|---|---|
| **Termius**(무료 요금제) | 휴대폰 속 Linux 터미널이에요: SSH로 정원 컴퓨터에 들어가서 메뉴를 돌려요(안내서 20) | <https://termius.com>; iPhone: <https://apps.apple.com/app/termius-ssh-shell-console-terminal/id549039908>; Android: <https://play.google.com/store/apps/details?id=com.server.auditor.ssh.client> |
| **Windows App**(Microsoft가 만들었어요, 무료) | 휴대폰에서 Remote Desktop으로 정원 컴퓨터의 바탕 화면 전체를 보고 써요(안내서 20) | <https://learn.microsoft.com/windows-app/get-started-connect-devices-desktops-apps>; iPhone: <https://apps.apple.com/us/app/windows-app-mobile/id714464092>; Android: <https://play.google.com/store/apps/details?id=com.microsoft.rdc.androidx> |

## 집 밖에서(선택)

| 프로그램 | 이유 | 공식 페이지 |
|---|---|---|
| **Tailscale**(개인용은 무료) | 내 기기들만 있는 비공개 네트워크예요. 그래서 공유기에 포트를 하나도 열지 않고도 SSH, WinSCP, Remote Desktop을 어디서나 쓸 수 있어요(안내서 20) | <https://tailscale.com/download>, 설치: <https://tailscale.com/kb/1017/install>, 방화벽과 함께: <https://tailscale.com/kb/1077/secure-server-ubuntu>, 가격: <https://tailscale.com/pricing> |

## 내 계정과 키(서비스예요. 설치하는 것은 없어요)

| 서비스 | 이유 | 공식 페이지 |
|---|---|---|
| **Google AI Studio**(Gemini) | AI 상자들(내 무료 키) | <https://ai.google.dev/gemini-api/docs> |
| **OpenRouter** | Gemini가 바쁠 때 쓰는 예비(내 키) | <https://openrouter.ai/docs> |
| **Perenual** | 식물 정보(내 키) | <https://perenual.com/docs/api> |
| **Gmail app password**(Gmail 앱 비밀번호) | 비밀번호 재설정 이메일을 보내요 | <https://support.google.com/accounts/answer/185833> |
| **Google Cloud client ID**(rclone용) | Drive 백업을 위한 나만의 로그인이에요 | <https://rclone.org/drive/#making-your-own-client-id> |

## 나중 프로젝트를 위한 추가 프로그램(만들 때만)

| 프로그램 | 버전 | 이유 | 공식 문서 |
|---|---|---|---|
| **Home Assistant**(스마트홈) | `stable` | 스마트홈 대시보드(프로젝트) | <https://www.home-assistant.io/installation/> |
| **Mosquitto**(메시지 전달) | 2 | 센서와 Home Assistant 사이의 메시지 | <https://mosquitto.org/documentation/> |
| **ESPHome**(센서 보드) | 최신 | 작은 센서 보드를 프로그래밍해요 | <https://esphome.io> |
| **Uptime Kuma**(상태 확인) | 1 | 내 사이트를 지켜보는 상태 페이지예요 | <https://github.com/louislam/uptime-kuma/wiki> |
| **Ollama**(그래픽 카드가 있는 데스크톱에서) | 최신 | 내 컴퓨터 안의 AI(안내서 15) | <https://ollama.com> |
| **Android Studio**(앱 만들기) | 최신 | 휴대폰 앱 만들기(안내서 17) | <https://developer.android.com/studio> |
