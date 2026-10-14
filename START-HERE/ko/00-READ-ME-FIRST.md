# Jun's Garden: 한국어 안내서

> 영어 전체 목록: [../00-READ-ME-FIRST.md](../00-READ-ME-FIRST.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 이 안내서들은 메뉴가 무엇을 설치했는지, 정원을 어떻게 돌보는지, 다음에 무엇을 만들지 설명해요. 설치 자체는 메뉴가 해요(한 폴더 위의 `00-READ-ME-FIRST-KO.html`).
- 규칙 1: 메뉴로 바꾸세요. 설정(메뉴 `8`), 사용자(메뉴 `U`), 구성 요소(메뉴 `9`), 정문(메뉴 `M`, `1`). 큰 변경 전에는 백업하세요(메뉴 `6`).
- 규칙 2: 비밀번호와 키는 `.env`에만 있어요. 채팅, 스크린샷, 이메일, AI 채팅에 절대 넣지 마세요.
- 규칙 3: 물 주기 프로젝트에는 저전압 부품(5 V 또는 12 V)만 쓰세요. 벽 콘센트에 꽂는 것은 절대 자르거나 연결하지 마세요.
- 프로젝트는 쉬운 것부터 어려운 것 순서예요. 프로젝트 1(Uptime Kuma, 20분)부터 시작하세요.
- AI는 가끔 자신 있는 목소리로 틀려요. 명령을 실행하기 전에 안내서와 비교해 보세요.
- 막히면 점검표(메뉴 `5`)를 실행하세요. 메뉴 `H`가 비밀번호 없는 도움 파일을 만들어요.

---

이 폴더에는 안내서 몇 개의 한국어 번역이 있어요. 설치 자체는 메뉴가 해요
(정원 폴더의 [00-READ-ME-FIRST-KO.html](../../00-READ-ME-FIRST-KO.html)을 보세요).
메뉴도 한국어로 볼 수 있어요: 메뉴에서 `L`을 입력하고 `2`(한국어) 또는 `3`(둘 다)을 고르세요.
입력하는 것(숫자, 글자, y/n, 명령)은 그대로예요.

번역되지 않은 안내서는 영어로 읽어요. 명령, 파일 이름, 웹 주소, 메뉴 번호는 영어
안내서와 한국어 안내서에서 똑같아요.

여기 있는 모든 것은 한 번에 작은 한 단계씩 하도록 만들었어요. 어느 단계에서든 멈췄다가
나중에 다시 와도 돼요.

---

## 세 가지 규칙

1. **메뉴로 바꾸세요.** 설정(메뉴 `8`), 사용자(메뉴 `U`), 구성 요소(메뉴 `9`), 정문
   (메뉴 `M`, `1`)을 쓰면 모든 것이 서로 맞게 유지돼요. 큰 변경 전에는 백업하세요(메뉴 `6`).
2. **비밀은 비밀로.** 비밀번호, 앱 비밀번호, No-IP 키, API 키는 `.env`에만 있어요
   (메뉴가 보이지 않게 넣어 줘요). 채팅, 스크린샷, 이메일, 남에게 보내는 파일에 절대 넣지
   마세요. AI 채팅도 마찬가지예요.
3. **안전한 전기만.** 물 주기 프로젝트에는 키트에 든 저전압 부품(5 V 또는 12 V)을 쓰고,
   벽 콘센트에 꽂는 것은 절대 자르거나 연결하지 마세요. 중요한 곳에는 프로젝트에
   "STOP SIGN"(정지 표시)이 있어요.

---

## 갖고 있는 것

| 부분 | 무엇인가요 |
|---|---|
| 웹사이트 | 식물 목록, 갤러리, 블로그, 데이터 페이지, 방명록, 식물 관리 화면이에요. Rust로 썼고, 한 줄 한 줄 설명이 달려 있어요 |
| 정원 도구(`/beds`) | 화단 계획, 점검, 진단, 이름표 스캔, 지출, 위키 대기열(선택 구성 요소) |
| 위키(Wiki) | 나만의 정원 백과사전(선택 구성 요소) |
| `garden.sh` | 모든 것을 설치하고 돌봐 주는 메뉴 |
| `START-HERE/` | 이 안내서들 |
| `lab/` | 한 가지 일만 하는 연습용 스크립트. 배우기 위한 거예요(`lab/README.md`) |
| `docker/` | 프로그램마다 어떻게 만들어지는지, 그리고 Uptime Kuma, Home Assistant, ESPHome, Apache용으로 미리 만들어 둔 파일 |
| `AGENTS.md`, `GEMINI.md` | AI 도우미가 읽는 규칙이에요. 그래서 AI가 안전하게 도와줘요 |
| `WHAT-GETS-INSTALLED.md`, `WHY-THESE-PROGRAMS.md` | 모든 프로그램, 어디서 오는지, 왜 골랐는지(한국어판: [WHAT-GETS-INSTALLED.ko.md](../../WHAT-GETS-INSTALLED.ko.md), [WHY-THESE-PROGRAMS.ko.md](../../WHY-THESE-PROGRAMS.ko.md)) |

## 안내서 찾기
안내서가 많아서, 안내서 웹 페이지(handbook)에는 페이지마다 맨 위에 찾기 칸이 있어요.
`00-WELCOME.html`을 여세요(정원 폴더 안. Windows에서는 설치가 끝나면 열려요).

1. 맨 위의 찾기 칸을 클릭하세요(또는 `/`를 누르세요). 아무것도 입력하지 않으면 자주
   찾는 주제가 보여요: 하나를 클릭하세요. 예: **공유기**, **백업**, **이름 바꾸기**.
2. 또는 아무 낱말이나 입력하세요. 예: `password`, `NAS`, `비밀번호`. 여러 낱말을 쓰면
   그 낱말이 모두 있는 곳을 찾아요: `backup Google Drive`.
3. 칸 옆에서 언어를 고르세요: **English + 한국어**, **English**, **한국어**. 고른 것을 기억해요.
4. 결과를 클릭하면: 안내서가 바로 그 부분에서 열려요. `Enter`는 첫 결과를 열고,
   `Esc`는 칸을 비워요.

여기 있는 모든 안내서를 두 언어로 찾고, 다른 안내서(n8n과 프로젝트 안내서, 설치되는 것,
요약, 일지)도 찾아요. 이 컴퓨터에서 인터넷 없이 돌아가고, 입력한 것은 어디로도 보내지 않아요.

## 정원 돌보기

| 안내서 | 내용 |
|---|---|
| [23 전체 설정](23-full-setup.md) | 지금 정원이 돌아가는 방식 그대로, 전부 순서대로: 나에게 맞는 단계(쉬움, 추천, 전체)를 고르고, 한 부분씩 |
| [11 웹사이트가 돌아가는 방식](11-maintain-your-site.md) | 파일마다 어디에 있는지, 안전하게 바꾸는 방법 |
| [12 내 데이터베이스](12-your-database.md) | 표, 안을 들여다보는 방법, 안전한 습관 |
| [26 이름 바꾸기, 옮기기, 끄기, 정리하기](26-rename-move-and-switch.md) | 폴더와 페이지에 내 이름, 더 큰 디스크로 옮기기, 켜기 / 끄기 스위치, Windows의 Docker |
| [13 백업과 폴더 이름](13-backups-and-folders.md) | 백업 이름, 복원, 폴더 옮기기와 이름 바꾸기 |
| [14 IP 예약하고 포트 두 개 열기](14-reserve-your-ip.md) | 공유기(router)에서 할 일 |
| [04 인터넷에 올리기](04-go-public-no-ip-caddy.md) | No-IP 계정, 정문, 먼저 연습하기 |
| [04b junsgarden.zapto.org 옮기기](04b-take-over-junsgarden-zapto.md) | 이미 내 것인 것, 그리고 확인할 것 |
| [07 내 로그인 페이지](07-sso-gate.md) | 로그인 페이지와 관문이 어떻게 작동하는지, 게스트, 2단계 인증 |
| [09 문제 해결](09-troubleshooting.md) | 증상별 해결 방법 |
| [25 자주 묻는 질문](25-common-questions.md) | 메뉴 키와 함께 짧은 답: 다시 시작, 비밀번호, 친구, 되돌리기, 사진, 업데이트, 옮기기 |

## 배우기와 AI

| 안내서 | 내용 |
|---|---|
| [16 Google AI Studio와 Gemini](16-ai-studio.md) | 출처가 있는 웹 검색, 코딩 도움, 내 컴퓨터 알기, 무료 키 |
| [08 Gemini 설정](08-gemini-setup.md) | 노트북의 Gemini CLI |
| [05 Gemini 질문](05-gemini-prompts.md) | 복사해서 붙여 넣는 시작 질문 |
| [15 로컬 AI](15-local-ai.md) | 집의 성능 좋은 컴퓨터에서 돌리는 AI(선택, 고급) |
| [17 Android 앱, 그리고 AI로 코딩하기](17-android-app-and-ai-coding.md) | Android Studio, 휴대폰의 앱, Android Studio 안의 Gemini, 로컬 AI, 코드를 안전하게 바꾸는 방법 |
| [00b 시작하기 전에](00b-before-you-start.md) | 필요한 모든 것(꼭 필요한 것과 선택)과 부분마다 걸리는 시간. 이것부터 읽으세요 |
| [20 SSH와 원격 데스크톱](20-ssh-and-remote-desktop.md) | Windows PC에서 정원 컴퓨터로 일하기, 키, 파일 복사, 그 컴퓨터 화면 보기 |
| [22 Bash, 한 단계씩](../22-bash-walkthrough.md)(영어) | 터미널로 하는 방법이에요. 더 어렵지만 힘들지 않아요: 터미널, 프롬프트, 첫 명령, 정원 프로그램 실행을 처음으로 함께 따라가 봐요 |
| [21 모든 프로그램과 출처](21-software-and-sources.md) | 프로그램마다(꼭 필요한 것과 선택) 버전, 이유, 만든 곳의 공식 문서. Let's Encrypt가 자물쇠를 주는 방법 |
| [19 모든 메뉴 선택, 설명](19-every-menu-option.md) | 모든 메뉴의 선택마다: 무엇을 하는지, 언제 쓰는지, 무엇을 바꾸는지, 되돌리는 방법 |
| [18 코드를 직접 검토하기](18-review-the-code.md) | 파일 지도, 바뀐 것이 없는지 확인, git 기록, 비밀 정보 검사기, Gemini나 친구에게 검토 부탁하기 |

영어 안내서 [24 Tools for learning and programming](../24-optional-tools.md)(아주 선택)에는 배우기와 프로그래밍에 쓸 수 있는 선택 도구들이 있어요.

## 프로젝트(사다리처럼, 쉬운 것부터 어려운 것까지)

| 단계 | 안내서 | 갖게 되는 것 | 시간 | 난이도 |
|---|---|---|---|---|
| 1 | [프로젝트 1: Uptime Kuma](../projects/P01-uptime-kuma.md)(영어) | 내 사이트가 켜져 있는지 알려 주는 대시보드 | 20분 | 쉬움 |
| 2 | [프로젝트 2: 백업](../projects/P02-backups.md)(영어) | 노트북 밖에 두는 백업 사본 | 30분 | 쉬움 |
| 3 | [프로젝트 3: Home Assistant](../projects/P03-home-assistant.md)(영어) | 나만의 스마트홈 허브 | 45분 | 보통 |
| 4 | [프로젝트 4: Govee 조명](../projects/P04-govee-lights.md)(영어) | 해 질 녘과 내가 정한 규칙을 따르는 조명 | 30분 | 3단계가 되면 쉬움 |
| 5 | [프로젝트 5: 날씨와 비](../projects/P05-weather-and-rain.md)(영어) | 비와 서리 알림, 그리고 새 날씨 측정값 | 30분 | 보통 |
| 6 | [프로젝트 6: 흙 센서](../projects/P06-soil-sensor.md)(영어) | 화단이 마르면 알려 주는 센서 | 2시간 | 보통 |
| 7 | [프로젝트 7: 자동 물 주기](../projects/P07-automatic-watering.md)(영어) | 스스로, 안전하게 물을 주는 밸브나 펌프 | 3시간 | 보통에서 어려움 |
| 8 | [프로젝트 8: 웹사이트에 측정값 올리기](../projects/P08-readings-on-your-website.md)(영어) | 내 데이터(Data) 페이지에 나오는 센서 숫자 | 3시간 | 어려움 |
| 9 | [프로젝트 9: 더 많은 아이디어](../projects/P09-more-ideas.md)(영어) | 다음 프로젝트를 고를 수 있는 목록 | | 골라요 |

n8n 안내서(이메일과 자동화)는 프로젝트 5 다음에 하면 잘 맞아요. 먼저 n8n을
켜세요(메뉴 `9`):
[n8n 1: 첫 이메일](../n8n/N1-install-and-first-email.md)(영어),
[n8n 2: 흙이 마르면 오는 이메일](../n8n/N2-dry-soil-email.md)(영어),
[n8n 3: 주간 정원 보고서](../n8n/N3-weekly-garden-report.md)(영어).

노트북을 처음부터 준비한다면: [01 노트북에 Ubuntu 설치하기](01-ubuntu-laptop.md)와
[02 Docker](02-docker.md)가 설치의 첫 단계들이 하는 일을 설명해요.
Docker가 무엇인지 궁금한데 기술적인 부분은 빼고 싶나요? [02a Docker는 무엇인가요?](02a-what-is-docker.md)(그림 두 장, 쉬운 말).
[10 계획](10-the-plan.md)은 확인 지점이 있는 꼼꼼한 주별 순서예요.

이것도 쓸모 있어요: [살 것 목록](06-shopping-list.md)과 [내가 해낸 것 체크리스트](../MY-WINS.md)(영어).

## 안내서마다 같은 짜임

모든 안내서는 같은 부분이 같은 순서로 있어요. 그래서 지금 어디쯤인지 늘 알 수
있어요.

1. **무엇을 만드나요**(What you will build)
2. **시간과 비용**(Time and cost)
3. **먼저 필요한 것**(What you need first)
4. **단계**(Steps: 번호가 있고, 단계마다 무엇이 보여야 하는지 적혀 있어요)
5. **잘 됐는지 아는 법**(How you know it worked)
6. **다르게 되면**(If something goes differently)
7. **나만의 것으로 만들기**(Make it yours: 해 볼 수 있는 선택 변경)

## Gemini를 도우미로 쓰기

- **먼저 배경을 알려 주세요.** [Gemini 질문](05-gemini-prompts.md)을 열고,
  "context block"(배경 설명)을 복사해서 새 채팅의 첫 메시지로 붙여 넣으세요.
- **오류를 정확히 붙여 넣으세요.** 터미널의 빨간 글자를 한 글자도 바꾸지 말고 그대로 복사하세요.
- **설명해 달라고 하세요.** "실행하기 전에 이 명령을 쉬운 말로 설명해 줘"는 언제나
  좋은 질문이에요.

AI는 가끔 자신 있는 목소리로 틀려요. **명령을 실행하기 전에 안내서와 비교해 보세요.**
둘이 다르면 먼저 안내서를 믿고, 그다음 Gemini에게 왜 다른지 물어보세요. 비밀번호나
키는 어떤 AI 채팅에도 절대 붙여 넣지 마세요. 실수로 붙여 넣었다면 바꾸세요(설정,
메뉴 `8`).

---

## 한국어로 번역된 것

| 안내서 | 내용 |
|---|---|
| [23 전체 설정](23-full-setup.md) | 지금 정원이 돌아가는 방식 그대로, 전부 순서대로: 나에게 맞는 단계(쉬움, 추천, 전체)를 고르고, 한 부분씩 |
| [03 집에서 정원 쓰기](03-run-your-garden.md) | 비밀번호, 이메일, AI 상자, 첫 편집, 평소 메뉴 |
| [04 인터넷에 올리기](04-go-public-no-ip-caddy.md) | No-IP 계정, 정문(Caddy), 로그인 페이지(Authelia), 먼저 연습하기, DuckDNS 하위 도메인 |
| [13 백업과 폴더 이름](13-backups-and-folders.md) | 백업 이름, 복원, NAS와 Google Drive, 폴더 옮기기와 이름 바꾸기 |
| [14 IP 예약하고 포트 두 개 열기](14-reserve-your-ip.md) | 공유기(router)에서 할 일 |
| [16 Google AI Studio와 Gemini](16-ai-studio.md) | 출처가 있는 웹 검색, 코딩 도움, 내 컴퓨터 알기, 무료 키, LiteLLM |
| [WHAT-GETS-INSTALLED.ko.md](../../WHAT-GETS-INSTALLED.ko.md) | 설치되는 모든 프로그램, 출처, 이유 |
| [WHY-THESE-PROGRAMS.ko.md](../../WHY-THESE-PROGRAMS.ko.md) | 왜 이 프로그램들인지(질문과 답) |

## 짧은 버전은 한국어, 전체 안내서는 영어

| 안내서(짧은 버전은 한국어) | 내용 |
|---|---|
| [01 Ubuntu on the laptop](01-ubuntu-laptop.md) | 노트북에 Ubuntu 설치하기 |
| [02 Docker](02-docker.md) | 설치의 첫 단계가 하는 일 |
| [02a Docker는 무엇인가요?](02a-what-is-docker.md) | 그림 두 장과 쉬운 말로: 왜 작은 상자를 쓰는지(참고로만) |
| [04b Moving junsgarden.zapto.org](04b-take-over-junsgarden-zapto.md) | 예전 서버에서 이름 옮기기 |
| [05 Gemini prompts](05-gemini-prompts.md) | 복사해서 쓰는 Gemini 질문 |
| [06 Shopping list](06-shopping-list.md) | 살 것 목록 |
| [07 Your sign-in page](07-sso-gate.md) | 로그인 페이지와 관문, 게스트, 2단계 인증 |
| [08 Gemini setup](08-gemini-setup.md) | 노트북의 Gemini CLI |
| [09 Troubleshooting](09-troubleshooting.md) | 증상별 해결 방법 |
| [10 The plan](10-the-plan.md) | 주별 순서와 확인 지점 |
| [11 How your website works](11-maintain-your-site.md) | 파일 위치, 안전하게 바꾸는 방법 |
| [12 Your database](12-your-database.md) | 표, 안을 보는 방법, 안전한 습관 |
| [15 Local AI](15-local-ai.md) | 집의 성능 좋은 컴퓨터에서 돌리는 AI(선택, 고급) |
| [17 Android app and AI coding](17-android-app-and-ai-coding.md) | Android Studio, 휴대폰 앱, AI로 코딩하기 |
| [00b 시작하기 전에](00b-before-you-start.md) | 필요한 모든 것(꼭 필요한 것과 선택)과 걸리는 시간. 먼저 읽으세요 |
| [20 SSH와 원격 데스크톱](20-ssh-and-remote-desktop.md) | Windows PC에서 정원 컴퓨터로 일하기, 키, 파일 복사, 화면 보기 |
| [22 Bash, 한 단계씩](../22-bash-walkthrough.md)(영어) | 터미널로 하는 방법, 조금 더 어렵지만 힘들지 않아요: 처음 쓰는 사람을 위한 bash 입문서(영어로만 있어요) |
| [21 모든 프로그램과 출처](21-software-and-sources.md) | 프로그램마다 버전, 이유, 만든 곳의 공식 문서. Let's Encrypt가 자물쇠를 주는 방법 |
| [19 모든 메뉴 선택, 설명](19-every-menu-option.md) | 모든 메뉴의 선택마다: 무엇을 하는지, 언제 쓰는지, 무엇을 바꾸는지, 되돌리는 방법 |
| [18 코드를 직접 검토하기](18-review-the-code.md) | 파일 지도, 바뀐 것이 없는지 확인, git 기록, 비밀 정보 검사기, Gemini나 친구에게 검토 부탁하기 |
| [Projects](../projects/P01-uptime-kuma.md) | 프로젝트 1~9(Uptime Kuma, 백업, Home Assistant, Govee 조명, 날씨, 흙 센서, 물 주기 ...) |
| [n8n guides](../n8n/N1-install-and-first-email.md) | n8n으로 이메일과 자동화 |
| [MY-WINS.md](../MY-WINS.md) | 내가 해낸 것 체크리스트 |

## 막히면

1. 점검표를 실행하세요(메뉴 `5`). 무엇부터 고쳐야 하는지 알려 줘요.
2. 안내서의 "다르게 되면"(If something goes differently) 부분을 읽으세요.
3. 메뉴 `H`가 도움 파일(비밀번호 없음)을 만들어요. "I am stuck"(막혔어요) 질문과 함께
   Gemini에 붙여 넣거나, 믿을 수 있는 사람에게 보여 주세요.
4. 20분이 지나도 막혀 있나요? 멈추고, 쉬고, 해 본 것을 적어 두세요. 새로운 눈으로 보면
   대부분의 문제가 풀려요.

막히는 것은 정상이에요. 전문가들도 매일 막혀요. 그것을 푸는 것이 진짜 실력이에요.

## 한 컴퓨터의 세 가지 이름(다들 자주 헷갈려요)

| 이름 | 무엇인가요 | 예 | 보는 방법(그 컴퓨터의 터미널에서) |
|---|---|---|---|
| **User name**(사용자 이름) | 그 컴퓨터에서 쓰는 내 이름: 로그인하는 계정이에요. 사람마다 따로 있어요. SSH가 물어봐요: `ssh USERNAME@ADDRESS`. | `jun` | `whoami` |
| **Host name**(호스트 이름, 기기 이름) | 컴퓨터 자체의 이름이에요. 네트워크에서 사람이 알아보기 위한 것이고, 로그인이 아니에요. | `garden-lab` | `hostname` |
| **IP address**(IP 주소) | 집 네트워크에서 컴퓨터의 번호예요. 공유기가 정해 줘요. 바뀌지 않게 예약하세요(안내서 14). 프로그램은 이것으로 컴퓨터를 찾아요. | `192.168.1.50` | `hostname -I`(첫 번째 숫자) |

그래서 다른 컴퓨터에서 정원 노트북에 닿으려면 `ssh jun@192.168.1.50`이라고 쳐요:
**사용자 이름** `@` **IP 주소**. 호스트 이름(`garden-lab`)은 알아보는 데만 도움이 돼요.

공식 초보자 페이지(영어): [기기 이름(호스트 이름) 바꾸기](https://help.gnome.org/gnome-help/about-hostname.html),
[내 IP 주소 찾기](https://help.gnome.org/gnome-help/net-findip.html),
[새 사용자 계정 추가하기](https://help.gnome.org/gnome-help/user-add.html),
[Ubuntu의 사용자 관리](https://documentation.ubuntu.com/server/how-to/security/user-management/).

## 자주 보는 말

| 말 | 쉬운 뜻 |
|---|---|
| Terminal(터미널) | 명령을 입력하는 창 |
| Command(명령) | 터미널에 입력하고 Enter를 누르는 한 줄 |
| `sudo` | "관리자로 실행하기". 내 비밀번호를 물어봐요 |
| Container(컨테이너) | 프로그램 하나와 그것에 필요한 모든 것이 든 작고 밀봉된 상자 |
| Docker | 컨테이너를 만들고 실행하는 도구 |
| Compose | 함께 시작할 컨테이너 목록이 적힌 파일(`docker-compose.yml`) |
| Image(이미지) | 컨테이너를 만드는 레시피 |
| Port(포트) | 컴퓨터의 번호 붙은 문. `8030`은 웹사이트가 쓰는 문이에요 |
| localhost | "이 컴퓨터". 이것으로 시작하는 주소는 나만 볼 수 있어요 |
| `.env` | 비공개 설정이 든 파일. 메뉴가 대신 바꿔 줘요 |
| Reverse proxy(리버스 프록시) | 방문자를 알맞은 컨테이너로 보내는 정문 프로그램(Caddy) |
| SSO / sign-in page(로그인 페이지) | 여러 도구 앞에 있는 로그인 페이지 하나(내 것은 Authelia) |
| DDNS | 웹 이름이 집의 인터넷 주소를 계속 가리키게 해 주는 서비스 |
| HTTPS | 브라우저의 자물쇠. 연결이 비공개라는 뜻이에요 |

즐겁게 하세요. 진짜로 쓰이는 것을 만들고 있어요.
