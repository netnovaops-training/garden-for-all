# 14. 정원 컴퓨터에 고정 주소를 주고, 포트는 두 개만 열기

> 영어 원문: [../14-reserve-your-ip.md](../14-reserve-your-ip.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 공유기에서 정원 컴퓨터에 고정 주소를 주세요("DHCP reservation", DHCP 예약). 그러면 다시 시작해도 주소가 그대로예요.
- 1단계: 메뉴 `M`, 그다음 `2`(Network)가 IP 주소와 MAC 주소를 보여 줘요.
- 2단계: 공유기 페이지에서 그 두 가지로 예약을 추가하세요. 다시 시작하고, 메뉴 `M`, `2`를 다시 실행해서 IP가 같은지 확인하세요.
- 3단계는 사이트를 인터넷에 올릴 때만(메뉴 `M`, `1` 뒤에): TCP 포트 80과 443을 그 주소로 전달하세요.
- 그 밖에는 아무것도 안 돼요. 22(SSH)도, 8030도, 3306도 안 돼요.
- 4단계: Wi-Fi를 끈 휴대폰으로 `https://junsgarden.zapto.org`(또는 내가 고른 웹 이름)를 열어 테스트하세요.
- 공유기를 바꿨나요? 2단계와 3단계를 다시 하세요.
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
| Google Nest Wifi / Google Wifi | Google Home 앱: Wi-Fi, 그다음 network settings, "DHCP IP reservations"를 찾으세요 | <https://support.google.com/googlenest> ("DHCP IP reservation"으로 검색) |
| TP-Link Deco | Deco 앱: More, Advanced, Address Reservation | <https://www.tp-link.com/support/> ("Deco address reservation"으로 검색) |
| Netgear | Advanced, Setup, LAN Setup, Address Reservation | <https://www.netgear.com/support/> ("address reservation"으로 검색) |
| Spectrum / Verizon / Xfinity 공유기 | 그 회사의 앱이나 공유기 페이지: "Reserved IP" 또는 "DHCP reservation" | 인터넷 회사의 도움말 사이트 |

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

## 더 읽기
- 포트 포워딩이 무엇인지: <https://en.wikipedia.org/wiki/Port_forwarding>
- Caddy와 포트 80/443: <https://caddyserver.com/docs/automatic-https#activation>
