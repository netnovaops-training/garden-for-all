# 14. 정원 컴퓨터에 고정 주소를 주고, 포트는 두 개만 열기

> 영어 원문: [../14-reserve-your-ip.md](../14-reserve-your-ip.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 공유기에서 정원 컴퓨터에 고정 주소를 주세요("DHCP reservation", DHCP 예약). 그러면 다시 시작해도 주소가 그대로예요.
- 1단계: 메뉴 `M`, 그다음 `2`(Network)가 IP 주소와 MAC 주소를 보여 줘요.
- 2단계: 공유기 페이지에서(로그인 방법: "2단계 전에", 흔한 공유기 표와 함께) 그 두 가지로 예약을 추가하세요. 다시 시작하고, 메뉴 `M`, `2`를 다시 실행해서 IP가 같은지 확인하세요.
- 3단계는 사이트를 인터넷에 올릴 때만(메뉴 `M`, `1` 뒤에): TCP 포트 80과 443을 그 주소로 전달하세요.
- 그 밖에는 아무것도 안 돼요. 22(SSH)도, 8030도, 3306도 안 돼요.
- 4단계: Wi-Fi를 끈 휴대폰으로 `https://junsgarden.zapto.org`(또는 내가 고른 웹 이름)를 열어 테스트하세요.
- 공유기를 바꿨나요? 2단계와 3단계를 다시 하세요.
- 공유기가 예약을 못 하나요? 주소가 바뀌면 Windows 메뉴가 네트워크 카드로 정원 컴퓨터를 다시 찾아요(더 보기 `21`). 인터넷에 올리려면 컴퓨터 자체에 고정 주소를 주세요: 아래의 "공유기가 주소를 예약하지 못하면".
- Windows 안의 Ubuntu(WSL)에서 정원을 돌리나요? 이 안내서는 그 경우에 맞지 않아요: 아래의 "Windows PC"를 보세요.

---

## 왜
공유기(router)는 집 안의 기기마다 `192.168.1.50` 같은 주소를 줘요. 보통은 다시 시작한
뒤에 *다른* 주소를 줄 수도 있어요. 정원 컴퓨터가 같은 주소를 계속 써야 하는 이유가 두
가지 있어요:
- 다른 컴퓨터에서 하는 **SSH**(그 주소를 입력해요).
- **포트 포워딩**(port forwarding, 포트 전달): 공유기는 인터넷 방문자를 주소 '하나'로 보내요.
  정원 컴퓨터의 주소가 바뀌면 방문자는 아무 데도 못 가요.

**DHCP 예약**(DHCP reservation)은 공유기에게 "이 컴퓨터에는 항상 이 주소를 줘"라고 알려
줘요. 컴퓨터가 아니라 공유기에서 하는 일이라서, 컴퓨터에서 무엇을 해도 망가지지 않아요.

**Windows PC 안의 Ubuntu(WSL)에서 정원을 돌리고 있나요?** 이 안내서는 그 경우에 맞지 않아요.
Windows 안의 Ubuntu는 PC 안에 자기만의 내부 주소가 있어서, 공유기와 휴대폰이 거기에 닿을
수 없어요. 그래서 포트를 PC로 전달해도 정원까지 가지 않아요. 그 Windows PC에서 정원은
PC 자신에서 쓰는 용도예요. 정원을 인터넷에 올리거나, 집에 있는 다른 기기에 보여 주려면,
따로 있는 정원 컴퓨터(Ubuntu 노트북 또는 Raspberry Pi)를 쓰고, 그 컴퓨터로 이 안내서를
따라 하세요. WSL의 네트워크가 어떻게 작동하는지 더 알고 싶다면 Microsoft가 설명해요(읽기만
하세요. 정원은 이것을 설정하지 않아요): <https://learn.microsoft.com/windows/wsl/networking>

## 1단계. 컴퓨터의 주소와 MAC 찾기
정원 컴퓨터에서: 메뉴 `M`, 그다음 `2`(Network, 네트워크). 이런 것을 보여 줘요:
- **IP address**(IP 주소), 예: `192.168.1.50`
- **MAC address**(MAC 주소), 예: `a4:5e:60:12:34:56`: 네트워크 카드의 바뀌지 않는 ID
- **Router address**(공유기 주소), 예: `192.168.1.1`

## 2단계 전에: 공유기에 로그인하기(따라하기)
공유기는 인터넷 회사가 준(또는 내가 산) 상자예요. 설정 페이지는 집 안에만 있어요: 집
Wi-Fi(또는 케이블)에 연결된 기기만 열 수 있어요.

1. **집에서**, 집 Wi-Fi에 연결하세요.
2. **공유기 주소를 찾으세요.** 정원 컴퓨터에서: 메뉴 `M`, 그다음 `2`("Router address").
   Windows PC에서: PowerShell을 열고 `ipconfig`를 입력해서 "Default Gateway"(기본 게이트웨이)를
   보세요. 보통 `192.168.1.1`, `192.168.0.1`, `192.168.50.1`, `10.0.0.1`, `192.168.1.254` 중 하나예요.
3. **브라우저 주소창에 입력하세요**(검색창이 아니에요). 예: `http://192.168.1.1`, 그리고 Enter.
   이름으로 열리는 공유기도 많아요(아래 표). 집 공유기 페이지에서 "안전하지 않음" 경고는
   흔해요: **고급**, 그다음 **계속**. 휴대폰 앱만 있는 공유기도 있어요.
4. **로그인하세요.** 사용자 이름과 비밀번호는 보통 **공유기 라벨**(흔히 밑면)에 있거나, 공유기를
   설치할 때 정한 것이에요. 인터넷 회사의 공유기는 그 회사 앱에서 내 계정으로 다루는 경우가 많아요.
5. **아직 라벨의 비밀번호인가요? 바꾸세요.** 내 비밀번호로 바꾸고 비밀번호 관리자에 두세요.
   안 그러면 내 Wi-Fi에 있는 누구나 공유기를 바꿀 수 있어요.
6. **잊어버렸나요?** 공유기의 리셋 버튼(약 10초 누르기)은 라벨의 비밀번호로 되돌리지만, Wi-Fi
   이름과 비밀번호도 라벨의 것으로 돌아가요: 집의 모든 기기를 다시 연결해야 해요. 마지막 방법으로만 쓰세요.
7. 그다음 아래 2단계. **Remote management**(인터넷에서 공유기 관리)는 **꺼 두세요**.

흔한 집 공유기의 로그인 방법(메뉴는 바뀌어요. 믿을 것은 만든 회사의 페이지예요):

| 공유기 | 주소 또는 앱 | 사용자 이름과 비밀번호 | 만든 회사의 안내 |
|---|---|---|---|
| Asus | `www.asusrouter.com` 또는 `192.168.50.1` | 공유기 라벨에 있어요(`admin`을 쓰는 것도 있어요) | <https://www.asus.com/us/support/faq/1005263/> |
| AT&T | `192.168.1.254` | 게이트웨이 라벨의 Device Access Code | <https://www.att.com/support/article/u-verse-high-speed-internet/KM1206322/> |
| eero | eero 앱 | 내 eero 계정 | <https://eero.com/support/articles/how-do-i-set-up-port-forwarding> |
| Google Nest Wifi / Google Wifi | Google Home 앱 | 내 Google 계정 | <https://support.google.com/googlehome/> |
| Linksys | `myrouter.local` 또는 `192.168.1.1` | 공유기 비밀번호(바꾸기 전에는 `admin`) | <https://support.linksys.com/kb/article/378-en/> |
| Netgear | `routerlogin.net` 또는 `192.168.1.1`, 또는 Nighthawk 앱 | `admin`, 그리고 설치할 때 정한 비밀번호 | <https://kb.netgear.com/980/How-do-I-log-in-to-my-NETGEAR-router> |
| Spectrum | My Spectrum 앱 | 내 Spectrum 계정 | <https://www.spectrum.net/support/internet/managing-advanced-settings-my-spectrum-app> |
| TP-Link Deco | Deco 앱 | 내 TP-Link 계정 | <https://www.tp-link.com/support/> |
| TP-Link (routers) | `tplinkwifi.net` 또는 `192.168.0.1` | 처음에 직접 만들어요. 오래된 것은 `admin`과 `admin`을 쓰기도 해요 | <https://www.tp-link.com/us/support/faq/87/> |
| Verizon | `mynetworksettings.com` 또는 `192.168.1.1` | `admin`, 그리고 라벨의 비밀번호 | <https://www.verizon.com/support/knowledge-base-308501/> |
| Xfinity | Xfinity 앱, 또는 `10.0.0.1` | `admin`, 그리고 앱에서 Admin Tool을 허용할 때 만드는 비밀번호 | <https://www.xfinity.com/support/articles/admin-tool-access> |

목록에 없나요? 만든 회사의 웹사이트에서 라벨의 모델 이름과 함께 "log in to router"를 찾아보세요.
기본 비밀번호를 모아 둔 사이트가 아니라, 만든 회사의 페이지를 쓰세요.

## 2단계. 공유기에서 예약하기
1. 브라우저에서 공유기 주소로 공유기 페이지를 여세요(또는 공유기의 휴대폰 앱을 쓰세요).
   관리자 비밀번호는 공유기에 붙은 스티커에 적혀 있는 경우가 많아요.
2. 이런 이름 중 하나를 찾으세요: **DHCP reservation**, **Address reservation**,
   **Static IP**, **Fixed IP**, **IP & MAC binding**.
3. 컴퓨터를 추가하세요: 1단계의 MAC 주소와 IP 주소. 저장하세요.
4. 정원 컴퓨터를 다시 시작하세요. 메뉴 `M`, `2`를 다시 실행하세요: IP가 같아야 해요.
   예약했는지 물어보면 **y**라고 대답하세요. 이제 주소가 바뀌면 점검표가 알려 줘요.

흔한 공유기에서 찾는 곳(메뉴는 바뀌기 때문에, 공식 도움말 페이지를 믿으세요):

| 공유기 | 찾는 곳 | 공식 도움말 |
|---|---|---|
| Google Nest Wifi / Google Wifi | Google Home 앱: Wi-Fi, 그다음 network settings, "DHCP IP reservations"를 찾으세요 | <https://support.google.com/googlehome/> ("DHCP IP reservation"으로 검색) |
| Netgear | Advanced, Setup, LAN Setup, Address Reservation | <https://www.netgear.com/support/> ("address reservation"으로 검색) |
| Spectrum / Verizon / Xfinity 공유기 | 그 회사의 앱이나 공유기 페이지: "Reserved IP" 또는 "DHCP reservation" | 인터넷 회사의 도움말 사이트 |
| TP-Link Deco | Deco 앱: More, Advanced, Address Reservation | <https://www.tp-link.com/support/> ("Deco address reservation"으로 검색) |

## 3단계. 포트를 정확히 두 개만 열기(사이트를 인터넷에 올릴 때만)
정문 단계(메뉴 `M`, `1`)를 한 다음에 하세요. 그 전에는 하지 마세요.

공유기에서 **Port Forwarding**(가끔 "Virtual Servers" 또는 "NAT")을 찾으세요.
규칙 두 개를 추가하세요. 둘 다 정원 컴퓨터의 예약된 주소로 가요:

| Name(이름) | Protocol(프로토콜) | Outside port(바깥 포트) | Inside address(안쪽 주소) | Inside port(안쪽 포트) |
|---|---|---|---|---|
| garden-http | TCP | 80 | 예약한 IP | 80 |
| garden-https | TCP | 443 | 예약한 IP | 443 |

**다른 것은 하지 마세요.** 22(SSH), 8030, 3306은 안 돼요. 인터넷이 닿아도 되는 프로그램은
Caddy뿐이에요. Caddy가 각 방문이 무엇을 볼 수 있는지 정해요.

## 4단계. 밖에서 테스트하기
**Wi-Fi를 끈** 휴대폰(그래서 모바일 네트워크를 써요)으로
`https://junsgarden.zapto.org`(또는 내가 고른 웹 이름)를 여세요. 자물쇠와 함께 내 정원이 보여야 해요.
정원 컴퓨터에서는 점검표(메뉴 `5`)가 이름이 우리 집을 가리키는지 확인해요.

## 공유기를 바꾸면
새 공유기에서 2단계와 3단계를 다시 하세요. 정원 컴퓨터에서는 할 일이 없어요.

## 공유기가 주소를 예약하지 못하면
어떤 공유기는(인터넷 회사가 준 것에 많아요) 예약 페이지가 없거나 잠겨 있어요. 그러면
정원 컴퓨터의 주소가 다시 시작한 뒤에 바뀔 수 있어요.

**그래도 Windows PC는 계속 찾아요.** 설치 마법사의 "Find it for me"(찾아 줘요)
버튼이 SSH 문(포트 22)이 열린 집의 컴퓨터를 보여 줘서 고를 수 있어요. 그다음 Windows
메뉴가 정원 컴퓨터를 네트워크 카드의 고정 ID(MAC 주소)와 이름으로 기억해요. 저장한
주소에서 대답이 없으면, 모든 작업 전과 매주 백업 복사 전에 집에서 같은 네트워크 카드를
찾아 새 주소를 저장해요. 손으로 하려면: Windows 메뉴, `M`, 그다음 `21`(정원 컴퓨터 다시 찾기).

**사이트를 인터넷에 올리려면 그래도 고정 주소가 필요해요.** 공유기가 포트 80과 443을
한 주소로 보내기 때문이에요. 컴퓨터 자체에 주소를 주세요:
1. 공유기 페이지에서 공유기가 나눠 주는 주소 범위를 찾으세요(흔히 `192.168.1.100`에서
   `192.168.1.200`까지). 그 밖의 주소 하나를 고르세요. 예: 다른 기기가 쓰지 않는 `192.168.1.20`.
2. 정원 컴퓨터에서: Settings(설정), Network(네트워크, 또는 Wi-Fi), 연결 옆 톱니바퀴,
   IPv4, **Manual**(수동). 주소: 고른 주소. 넷마스크: `255.255.255.0`. 게이트웨이와 DNS:
   공유기 주소(메뉴 `M`, `2`가 보여 줘요). 적용한 다음 연결을 껐다 켜세요.
3. 메뉴 `M`, `2`를 다시 실행하세요: 새 IP가 보여요. 그 주소로 3단계를 하세요.

화면이 없는 Ubuntu에서는 netplan으로 똑같이 해요:
<https://ubuntu.com/server/docs/configuring-networks>

## 더 읽기
- 포트 포워딩이 무엇인지: <https://en.wikipedia.org/wiki/Port_forwarding>
- Caddy와 포트 80/443: <https://caddyserver.com/docs/automatic-https#activation>
