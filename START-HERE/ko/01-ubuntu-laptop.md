# 01. 예전 Windows 7 노트북을 Ubuntu 컴퓨터로 바꾸기(USB 메모리로)

> 영어 원문: [../01-ubuntu-laptop.md](../01-ubuntu-laptop.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 정원이 대신 해 주지 않는 단 한 가지예요: 컴퓨터에 Ubuntu를 설치하는 일. 약 2~3시간, 무료예요.
- 먼저 노트북의 파일을 다른 곳에 복사하세요: **이 일은 Windows를 지워요**.
- Windows 컴퓨터에서 Ubuntu 24.04 LTS(`.iso`)와 **Rufus**(rufus.ie)를 내려받으세요. Rufus가 16 GB 이상의 USB 메모리에 Ubuntu를 넣어요(B 부분).
- 그 USB 메모리로 노트북을 시작하고(부팅 키를 톡톡 누르세요, C 부분), **Erase disk and install Ubuntu**(디스크를 지우고 Ubuntu 설치)를 고르세요.
- **사용자 이름**(예: `jun`), **컴퓨터 이름**(예: `garden-lab`), **비밀번호**를 정하세요. 적어 두세요. 그 비밀번호는 무언가를 바꿀 때(`sudo`) Ubuntu가 묻는 비밀번호이기도 해요. 따로 "root password"(루트 비밀번호)는 없어요.
- 노트북의 **IP 주소**를 적어 두세요(Settings(설정), Wi-Fi, 내 네트워크 옆의 톱니바퀴).
- 노트북을 바꾸고 싶지 않나요? **Raspberry Pi 5** 키트도 돼요(안내서 06).

---

## 무엇을 만드나요
예전에 Windows 7이 돌던 노트북이 이제 **Ubuntu Desktop 24.04 LTS**(무료 Linux)로
돌아가요. 다른 모든 안내서에서 쓰는 내 홈랩(home-lab) 컴퓨터가 돼요.

**아니면 대신 Raspberry Pi 5.** 이런 일을 하려고 만든 작은 새 컴퓨터예요. 예전 노트북을
바꿀 필요가 없어요. 메모리 8 GB인 Pi 5, 공식 전원 어댑터, 그리고 128 GB 이상의 저장 카드나
SSD가 있으면 돼요(안내서 06, 쇼핑 목록). Pi 자체의 Imager 프로그램이 약 30분 만에
Raspberry Pi OS(64-bit)를 넣어 줘요: <https://www.raspberrypi.com/software/>.
다음: Google Drive 폴더에서 `junsgarden-for-jun.zip`을 받아 압축을 풀고, 그 안의
`00-WELCOME.html`을 연 다음, `Start-Here-Ubuntu.sh`를 오른쪽 클릭하고 **Run as a Program**을 고르세요
(첫 시작이에요. `00-READ-ME-FIRST-KO.html`은 그때 프로그램과 함께 와요).

> **Pi에 대해 알아 두면 좋은 것(실제 Raspberry Pi 5, 4 GB에서 2026-10-13에 재었어요).**
> - **돼요.** 설치 중 정원을 만들고 시작하는 부분이 **8분** 만에 끝났고, 점검표는
>   **FAIL 0개**, 웹사이트에 식물 485개가 모두 보였어요. 시스템 업데이트부터 하는
>   전체 설치는 더 걸려요: 최대 한 시간으로 잡으세요. 노트북이나 PC는 같은 일을 더 빨리 해요.
> - **만드는 동안 뜨거워져요.** 팬 없이 **84.5 °C**까지 올라갔고, 식히려고 스스로
>   느려졌어요(Pi 5는 80~85 °C 사이에서 이렇게 해요). 다 만든 뒤에는 60 °C로 식었어요.
>   고장이 아니라 느려질 뿐이에요. Raspberry Pi의 공식 **Active Cooler**(액티브 쿨러)나
>   팬이 달린 **Case**(케이스)를 쓰세요(안내서 06).
>   Raspberry Pi의 안내 페이지: <https://www.raspberrypi.com/documentation/computers/raspberry-pi.html>
>   ("Cooling"과 "Frequency management and thermal control" 부분).
> - **메모리 카드는 닳아요.** 정원은 매일 데이터베이스와 기록을 써요. SD 카드는 그런
>   용도로 만들어지지 않았어요. **SSD**(Raspberry Pi의 M.2 HAT+로, 또는 USB 3 포트에)가
>   더 빠르고 훨씬 오래가요. 어느 쪽이든 자동 백업을 두 번째 장소로 보내 두세요(안내서 13).

왜 좋은 생각일까요: Windows 7은 2020년에 보안 업데이트가 끝났어요. 그래서 인터넷에
올리기에 안전하지 않아요. Ubuntu LTS는 여러 해 동안 무료 보안 업데이트를 받아요.

## 시간과 비용
2~3시간. 무료예요. 16 GB 이상의 USB 메모리가 필요해요(그 안의 내용은 지워져요. Ubuntu
공식 안내서는 최소 12 GB를 요구하고, 16 GB가 흔한 크기예요). 값싼 SSD(약 $25~$40)를 달면 예전 노트북이 훨씬 빨라져요.
이것은 선택이에요.

## 먼저 필요한 것
- 예전 노트북과 충전기(처음부터 끝까지 계속 꽂아 두세요).
- 설치 USB를 만들 **인터넷이 되는 두 번째 컴퓨터**. Windows 7 노트북밖에 없다면,
  B 부분, 방법 2를 보세요.
- 16 GB 이상의 USB 메모리.
- 노트북에 네트워크 포트가 있다면 공유기(router)에 연결할 랜선(설치하는 동안
  Wi-Fi보다 더 안정적이에요).

> **정지 표지: 이 일은 Windows 7을 지워요.** 아래 단계는 노트북에 있는 모든 것을
> 지워요. 먼저 파일을 다른 곳에 복사하세요(A 부분, 3단계). 되돌릴 수 없어요.

## 단계

### A 부분. 노트북이 이 일을 할 수 있는지 확인하기
1. **프로세서가 64-bit인가요?** Ubuntu 24.04와 Docker는 64-bit에서만 작동해요.
   노트북의 Windows 7에서: **Start**(시작)를 누르고, **Computer**(컴퓨터)를 오른쪽 클릭하고,
   **Properties**(속성)를 고르세요. **System type**(시스템 종류)을 찾으세요.
   - `64-bit Operating System` : 좋아요, 계속하세요.
   - `32-bit Operating System` : 그래도 프로세서는 64-bit*일 수도* 있어요.
     **CPU-Z**(cpuid.com)를 내려받아서 **Instructions**(명령어)를 보세요: `EM64T`,
     `AMD64`, 또는 `x86-64`가 있으면 64-bit이고, 계속해도 돼요. 없으면 이
     노트북에서는 Docker를 돌릴 수 없어요. 다른 일(간단한 Linux 공부용 컴퓨터)에
     쓰고, 정원용으로는 다른 노트북을 찾으세요.
2. **메모리(RAM)와 디스크는 얼마나 되나요?** 같은 Properties(속성) 창에서
   **Installed memory (RAM)**(설치된 메모리)를 보세요. 그다음 **Computer**(컴퓨터)를 열고
   C: 드라이브의 크기를 보세요.

   | 메모리 | 할 일 |
   |---|---|
   | 8 GB 이상 | Ubuntu Desktop이면 괜찮아요 |
   | 4 GB | 작동해요. 하지만 대신 **Xubuntu 24.04**를 설치하세요(더 가벼운 화면이고, 나머지는 모두 같아요. xubuntu.org에서 받아요). 아래 단계는 똑같아요. |
   | 2 GB 이하 | 웹사이트를 만들기에는 너무 작아요. 먼저 메모리를 추가하고(노트북 메모리는 싸요), 그다음 계속하세요. |

   Ubuntu 노트북에 Docker, 정원, 백업을 위한 빈 디스크 공간이 약 60 GB
   필요해요. Windows를 지우면 디스크 전체가 비어요. 그러니 C: 드라이브의 전체 크기를
   보세요: Ubuntu 자체가 약 25 GB를 쓰니까, 디스크가 약 90 GB 이상이면 충분해요.
3. **간직하고 싶은 것을 저장하세요.** 사진, 문서, 브라우저 즐겨찾기를 다른 USB
   메모리나 다른 컴퓨터에 복사하세요. 지금 하세요. Wi-Fi 비밀번호도 적어 두세요.
4. **Windows 7만 해당:** Windows 제품 키 스티커를 간직하고 싶다면, 노트북 바닥에 있는
   스티커를 사진으로 찍어 두세요. Ubuntu에는 필요 없어요.

### B 부분. 설치 USB 만들기
**방법 1(추천): 더 새로운 Windows 컴퓨터에서, Rufus로.**
Rufus는 `.iso`를 USB 메모리에 넣어 주는 작고 무료이고 잘 알려진 프로그램이에요
(<https://rufus.ie>, 만든 사람은 Pete Batard). 설치할 것이 없어요: 그냥 실행돼요.
5. <https://releases.ubuntu.com/24.04/>를 열고 **Ubuntu 24.04 LTS**의
   "Desktop image"(데스크톱 이미지)를 내려받으세요(`.iso`로 끝나는 파일, 약 6 GB). 메모리 4 GB
   노트북이라면 대신 <https://xubuntu.org/download>에서 Xubuntu를 내려받으세요.
   *Ubuntu 메인 페이지는 이제 26.04를 보여 주는데, 왜 24.04일까요?* 정원은 24.04에서
   만들고 테스트했어요. 24.04는 2029년까지 무료 보안 업데이트를 받아요. 26.04도 작동할
   것으로 보지만(Docker가 지원해요), 테스트한 것은 24.04예요.
6. <https://rufus.ie>에서 **Rufus**를 내려받으세요(맨 위 링크, "Rufus 4.x").
   여세요. Windows가 "allow this app to make changes?"(이 앱이 변경하도록 허용할까요?)라고
   물어요: **Yes**(예)(USB 메모리에 쓰려면 이 허락이 필요해요).
7. USB 메모리를 꽂으세요. Rufus에서: **Device**(장치) = 내 USB 메모리(크기를 확인하세요.
   다른 드라이브를 고르지 않도록요). **Boot selection**(부팅 선택): **SELECT**(선택)를 누르고
   `.iso`를 고르세요. 나머지는 모두 Rufus가 고른 대로 두세요. **START**(시작). **ISO image mode**나
   **DD image mode**를 물으면 **ISO**를 고르세요. USB 메모리가 지워진다고 경고해요: **OK**(확인).
   **READY**(준비됨)가 나올 때까지 기다리세요(5~15분).

*대신 Mac이나 Linux 컴퓨터에서라면:* **balenaEtcher**(<https://etcher.balena.io>)가
같은 일을 해요: **Flash from file**(파일에서 굽기)을 누르고 `.iso`를 고르고, **Select target**(대상 선택)을
누르고 USB 메모리를 고르고, **Flash!**(굽기!)를 누르세요.

**방법 2: Windows 7 노트북밖에 없을 때.**
5. Etcher는 Windows 7에서 돌아가지 않아요. Windows 7을 지원하는 마지막 버전인
   **Rufus 3.22 버전**을 쓰세요. 공식 다운로드 페이지에서 받으세요
   (<https://rufus.ie>, "Other versions"(다른 버전)나 "Previous versions"(이전 버전) 아래를 보세요).
6. Ubuntu나 Xubuntu `.iso`를 내려받으세요(방법 1, 5단계를 보세요).
7. Rufus에서 내 USB 메모리를 고르고, **SELECT**(선택)를 누르고 `.iso`를 고르세요. 다른 설정은
   Rufus가 고른 대로 두고(예전 BIOS 노트북이면 MBR을 골라요), **START**(시작)를 누르세요.
   추가 파일(Syslinux)을 내려받을지 물으면 Yes(예)라고 하세요. **ISO image mode**와 **DD mode** 중
   무엇으로 할지 물으면 먼저 **ISO**를 고르세요. 나중에 노트북이 USB 메모리로 시작하지
   않으면, **DD**로 다시 만드세요.

### C 부분. USB 메모리로 노트북 시작하기
8. USB 메모리를 노트북에 꽂으세요. 노트북을 **끄세요**(다시 시작이 아니에요).
9. 노트북을 켜고 바로 **부팅 메뉴 키를 톡톡** 계속 누르세요.
   키는 브랜드마다 달라요. 화면에 1초쯤 보여 주는 경우가 많아요.

   | 브랜드 | 부팅 메뉴 키 |
   |---|---|
   | Dell | F12 |
   | HP | Esc, 그다음 F9 |
   | Lenovo | F12(또는 Novo 버튼) |
   | Acer | F12 |
   | ASUS | Esc 또는 F8 |
   | Toshiba | F12 |
   | Samsung | Esc 또는 F10 |

10. 메뉴에서 USB 메모리를 고르세요. 이름이 `USB HDD`, `USB-HDD`,
    `Removable Device`이거나 내 USB 메모리의 이름일 수 있어요. 항목이 두 개 있고
    하나는 앞에 `UEFI:`가 붙어 있다면, 어느 쪽이든 될 수 있어요. Windows 7 노트북에서는
    먼저 `UEFI`가 없는 쪽을 해 보세요("legacy"(레거시) BIOS 노트북일 가능성이 커요).
    *이렇게 보여야 해요:* **Try or Install Ubuntu**(Ubuntu 체험 또는 설치)라고 쓰인 화면.

노트북이 USB를 건너뛰고 Windows로 시작하면: 끄고 다시 해 보세요. 또는 BIOS 설정에
들어가서(보통 `F2`나 `Del`) **Boot**(부팅) 순서에서 **USB**를 맨 위로 옮기세요.

### D 부분. 설치하기
이것은 Ubuntu 24.04 설치 프로그램의 화면이에요. Ubuntu 공식 안내가 보여 주는 순서
그대로예요: <https://ubuntu.com/tutorials/install-ubuntu-desktop>.
각 화면에서 **Next**(다음) 버튼을 눌러 넘어가요.
11. 검은 시작 메뉴에서 **Try or Install Ubuntu**(Ubuntu 체험 또는 설치)를 고르고 Enter를 누르세요.
    예전 노트북에서는 5분까지 기다리세요.
12. 처음 화면들, 하나씩:
    - **Language**(언어): 내 언어를 고르고, **Next**.
    - **Accessibility**(접근성): 여기서는 할 것이 없어요, **Next**.
    - **Keyboard layout**(키보드 배열): 내 키보드에 맞는 것, **Next**.
    - **Internet**(인터넷)("Connect to your network", 네트워크에 연결하기): 랜선이 있으면 유선
      연결을 고르세요. 아니면 내 Wi-Fi를 고르고 비밀번호를 입력하세요. **Next**.
      (Ubuntu는 설치하는 동안 인터넷으로 업데이트와 드라이버를 받아요.)
    - **Try or install**(체험 또는 설치): **Install Ubuntu**(Ubuntu 설치)를 고르고, **Next**.
13. **Type of installation**(설치 종류): **Interactive installation**(대화형 설치)을 고르고, **Next**.
    그다음 **Applications**(응용 프로그램): **Default selection**(기본 선택)을 고르세요(더 가벼운
    쪽이에요. Extended selection(확장 선택)은 사무용 프로그램 등을 더 넣어요), **Next**.
14. **Third-party software**(타사 소프트웨어): 그래픽과 Wi-Fi 하드웨어용 타사 소프트웨어를
    설치하는 상자에 체크하세요. 예전 Wi-Fi 칩은 이것이 필요한 경우가 많아서 중요해요.
    추가 미디어 형식 상자는 마음대로 고르세요. **Next**.
15. **Disk setup**(디스크 설정): **Erase disk and install Ubuntu**(디스크를 지우고 Ubuntu 설치)를
    고르세요(정지 표지를 보세요). 암호화 선택은 꺼 두세요: 켜면 노트북이 시작할 때마다
    키를 물어서, 정전 뒤에 스스로 시작할 수 없어요. **Next**.
16. **Create your account**(계정 만들기). 여기에는 이름이 세 개 있어요(설명은
    [START-HERE/00-READ-ME-FIRST.md, "Three names for one computer"(컴퓨터 하나에 이름 세 개) 부분](../00-READ-ME-FIRST.md#three-names-for-one-computer-people-mix-these-up-all-the-time), 영어):
    - **Your name**(내 이름): 아무거나, 예: `Jun`. 로그인 화면에만 보여요.
    - **Computer name**(컴퓨터 이름, host name(호스트 이름)이라고도 해요): `garden-lab`. 집에 있는
      다른 컴퓨터들이 노트북을 이 이름(`garden-lab.local`)으로 봐요.
    - **Username**(사용자 이름): 소문자로, 띄어쓰기 없이, 입력하기 쉽게, 예: `jun`. 로그인할 때
      입력하고, SSH도 이것을 써요(안내서 20).
    - **Password**(비밀번호, 두 번 입력해요): 길고, 기억할 수 있는 것으로. 비밀번호 관리자에
      저장하세요. **Ubuntu에는 따로 "root password"(루트 비밀번호)가 없어요.** Ubuntu(또는 정원
      메뉴)가 무언가를 바꾸려고 비밀번호를 물으면, 바로 이 비밀번호예요: `sudo`는 "관리자로서
      이것을 해 줘"라는 뜻이고, 내 계정은 그렇게 할 수 있어요.
    - **Require my password to log in**(로그인할 때 비밀번호 요구)은 **켜 둔 채로** 두세요(자동으로
      로그인하기를 고르지 마세요). **Next**.
17. **Time zone**(시간대): 지도에서 내가 있는 곳을 누르고, **Next**. 그다음 **Review**(검토)
    페이지가 내가 고른 것을 보여 줘요: 디스크가 지워진다고 쓰여 있는지 확인하고,
    **Install**(설치)을 누르세요. 기다리세요. 예전 하드 디스크는 30~60분 걸릴 수 있어요.
    뚜껑을 닫지 말고, 충전기를 빼지 마세요.
18. **Restart Now**(지금 다시 시작)가 나오면 누르세요. USB 메모리를 빼라고 하면 빼고
    Enter를 누르세요.

*이렇게 보여야 해요:* 내 이름이 있는 로그인 화면. 로그인하세요.

### E 부분. 첫 업데이트(이것이 안전을 위한 부분이에요)
19. Windows 키(Super)를 누르고, `terminal`을 입력하고, Enter를 누르세요.
20. 모든 것을 업데이트하세요. 이것을 입력하고 Enter를 누른 다음, 비밀번호를 입력하세요
    (입력하는 동안 아무것도 보이지 않아요. 정상이에요):

```
sudo apt update && sudo apt upgrade -y
```

   *이렇게 보여야 해요:* 글자가 많이 나오고, 그다음 다시 `$` 줄. 처음에는 10~40분
   걸릴 수 있어요. 그다음 다시 시작하세요: `sudo reboot`.

### F 부분. 도구와 내 주소
(정원 설치, 메뉴 1이 SSH 서버, 방화벽, 그리고 G 부분의 스왑도 대신 해 줘요. 방화벽은
집 네트워크에서만 SSH를 허용하는 규칙과 함께 켜요. 그러니 여기서 방화벽을 손으로
설정하지 마세요.)
21. 다시 시작한 뒤, 터미널에서:

```
sudo apt install -y git curl unzip htop openssh-server
```

   *이렇게 보여야 해요:* 프로그램이 설치된다는 줄들, 그다음 다시 `$`
   줄.
22. 집 네트워크에서 노트북의 주소를 찾으세요:

```
hostname -I
```

   *이렇게 보여야 해요:* `192.168.1.50` 같은 것. 적어 두세요.

   **터미널 없이 하려면:** **Settings**(설정)를 열고, **Wi-Fi**(랜선이면 **Network**(네트워크))를
   누르고, 내 연결 옆의 **톱니바퀴**를 누르세요. **Details**(자세히) 페이지에 나오는
   **IPv4 Address**(IPv4 주소)가 바로 그거예요. 컴퓨터 이름은 **Settings, System, About**(설정,
   시스템, 정보)에 있어요("Device Name"(장치 이름)).

### G 부분. 예전 노트북을 서버로 잘 쓰기
23. **스왑 추가하기**(디스크 위의 추가 "메모리"). 정원 웹사이트를 처음 만들 때(빌드) 작은
    노트북에 있는 것보다 메모리가 더 필요해요. 스왑이 있으면 중간에 멈추지 않아요.
    이것은 4 GB를 추가해요. **정원 설치(메뉴 1)를 쓸 거라면 이 단계는 건너뛰세요:**
    설치가 컴퓨터에 필요할 때 스왑을 스스로 추가해요.

```
sudo fallocate -l 4G /swapfile2
sudo chmod 600 /swapfile2
sudo mkswap /swapfile2
sudo swapon /swapfile2
echo '/swapfile2 none swap sw 0 0' | sudo tee -a /etc/fstab
free -h
```

   *이렇게 보여야 해요:* 4 GB 이상인 `Swap:` 줄. 웹사이트의 첫 빌드는 15~45분
   걸릴 거예요(아주 오래된 노트북이나 Pi에서는 1시간까지). 정상이에요.
24. **뚜껑을 닫아도 깨어 있게 하기:**

```
sudo nano /etc/systemd/logind.conf
```

   `#HandleLidSwitch=suspend`를 찾으세요. `#`를 지우고
   `HandleLidSwitch=ignore`로 바꾸세요. `#HandleLidSwitchExternalPower`도 똑같이 하세요.
   `Ctrl+O`, `Enter`로 저장하고, `Ctrl+X`로 나오세요. 그다음 노트북을 다시 시작하세요
   (`sudo reboot`, 또는 전원 메뉴의 Restart(다시 시작)). 그래야 바뀐 것이 적용돼요.
   (노트북 전체를 다시 시작하는 것이 로그인 서비스만 다시 시작하는 것보다 안전해요.
   로그인 서비스만 다시 시작하면 작업하는 도중에 로그아웃될 수 있어요.)
25. **Settings, Power**(설정, 전원): **Automatic Suspend**(자동 절전)를 **Off**(끔)로 하세요.
26. **열:** 뚜껑을 닫은 예전 노트북은 아주 뜨거워져요. 뚜껑을 조금 열어 두거나 뒤쪽을
    받쳐서 공기가 통하게 하세요. 팬이 하루 종일 세게 도는지 지켜보세요.
27. **배터리:** 배터리가 오래되어 부풀었다면(노트북이 기울어져 있거나, 터치패드가
    위로 밀려 올라와 있으면), 배터리를 빼고 충전기로만 쓰세요. 부푼 배터리는 불이
    날 위험이 있어요.

## 잘 됐는지 아는 법
- 로그인하고 터미널을 열 수 있어요.
- `sudo apt update`가 "All packages are up to date"(모든 패키지가 최신이에요)로 끝나요(또는 업데이트 목록을 보여 줘요).
- `free -h`가 스왑을 보여 줘요.
- 노트북의 주소를 적어 뒀어요.

## 다르게 되면
| 보이는 것 | 할 일 |
|---|---|
| 노트북이 USB가 아니라 Windows로 시작해요 | 부팅 키를 다시 눌러 보거나, BIOS에서 USB를 맨 앞으로 하세요. |
| 부팅 메뉴에 USB가 없어요 | Rufus **DD mode**로 USB 메모리를 다시 만들거나, 다른 USB 포트를 써 보세요(USB 2.0 포트가 있으면 그것을 쓰세요). |
| `This kernel requires an x86-64 CPU` | 프로세서가 32-bit예요. A 부분, 1단계를 보세요. |
| "Install"(설치) 뒤에 화면이 까매요 | 부팅 화면에서 `e`를 누르고, `linux`로 시작하는 줄 끝에 `nomodeset`을 추가하고, `F10`을 누르세요. 설치가 끝난 뒤 Gemini에 "how to install the right graphics driver on Ubuntu for my laptop's graphics chip"(내 노트북 그래픽 칩에 맞는 그래픽 드라이버를 Ubuntu에 설치하는 방법)이라고 물어보고, `lspci`의 "VGA" 줄을 붙여 넣으세요. |
| 설치 뒤에 Wi-Fi가 없어요 | 랜선을 꽂고, `sudo apt update`를 실행한 다음, 메뉴에서 **Additional Drivers**(추가 드라이버)를 찾아서 보여 주는 Broadcom이나 다른 드라이버를 설치하세요. |
| 아주 느려요 | Xubuntu를 쓰고, 쓰지 않을 때는 브라우저를 닫고, SSD를 생각해 보세요. Gemini에 물어보세요: "How do I make Ubuntu faster on an old laptop with 4 GB RAM?"(RAM 4 GB인 예전 노트북에서 Ubuntu를 더 빠르게 하려면 어떻게 하나요?) |
| 설치 프로그램이 끝에서 멈춰요 | 10분 기다리세요(예전 디스크는 느려요). 그다음에만 전원 버튼을 길게 누르세요. 그 뒤에 시작되지 않으면, 설치를 다시 하세요. |

## 나만의 것으로 만들기
- 내 정원 사진으로 바탕화면을 고르세요.
- 비밀번호 관리자를 설치하세요(Bitwarden이나 Proton Pass에 Linux 앱이 있어요).
- 오늘 명령 세 개를 배워 보세요: `ls`(파일 목록 보기), `cd`(폴더로 가기),
  `pwd`(나는 어디에 있나요?).

**다음:** Google Drive 폴더에서 `junsgarden-for-jun.zip`을 받아 압축을 풀고, 그 안의
`00-WELCOME.html`을 연 다음, `Start-Here-Ubuntu.sh`를 오른쪽 클릭하고 **Run as a Program**을 고르세요
(첫 시작이에요. `00-READ-ME-FIRST-KO.html`은 그때 프로그램과 함께 와요).
(설치, 메뉴 1이 Docker를 대신 설치해 줘요. [02 Docker](02-docker.md)는 설명만 해요.)
