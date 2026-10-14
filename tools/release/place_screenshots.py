"""Put the screenshots into the guides, right under the heading they show.

    python tools/release/place_screenshots.py

Each entry: guide file, the exact heading text (English, Korean), and the
pictures with a plain caption. A picture already in a guide is not added again,
so this can be run again after the pictures are retaken.
"""
import io, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2] / "START-HERE"

# (guide, English heading, Korean heading, [(picture, English caption, Korean caption), ...])
PLACES = [
    ("00b-before-you-start.md", "## Easy or Custom?", "## Easy(쉽게)와 Custom(사용자 지정) 중 무엇?",
     [("screen-03-easy-or-custom.png", "What you will see: the install asks EASY or CUSTOM, then shows its plan",
       "보게 될 화면: 설치가 EASY와 CUSTOM 중 무엇인지 묻고, 계획을 보여 줘요")]),
    ("00b-before-you-start.md", "### On a Windows PC (the garden inside Windows' own Ubuntu, WSL)", "### Windows PC에서(Windows 자체의 Ubuntu, WSL 안의 정원)",
     [("wizard-1-welcome.png", "The setup wizard: welcome, your steps on the left, Help, Back and Next below", "설치 마법사: 환영, 왼쪽에 내 단계, 아래에 Help, Back, Next"),
      ("wizard-1b-level.png", "The level: EASY (automatic) or CUSTOM (you choose, page by page)", "단계: EASY(자동) 또는 CUSTOM(한 쪽씩 내가 골라요)"),
      ("wizard-2-where.png", "Where the garden runs", "정원이 실행되는 곳"),
      ("wizard-3-about-you.png", "About you and your web name", "나와 내 웹 이름에 대해"),
      ("wizard-4-optional-software.png", "CUSTOM: the optional software, then one page for each part you tick", "CUSTOM: 선택 소프트웨어, 그다음 체크한 부분마다 쪽 하나"),
      ("wizard-5-install.png", "Install: the steps appear as they finish", "설치: 끝난 단계가 차례로 보여요")]),
    ("03-run-your-garden.md", "### Part A. Open your garden", "### A 부분. 정원 열기",
     [("website-home.png", "Your garden's home page", "내 정원의 첫 페이지")]),
    ("03-run-your-garden.md", "### Part E. Log in and change a plant", "### E 부분. 로그인해서 식물 하나 바꾸기",
     [("website-login.png", "The login page (Forgot password? / First time? is under the button)", "로그인 페이지(버튼 아래에 Forgot password? / First time?)"),
      ("website-catalog.png", "The full catalogue", "전체 목록")]),
    ("03-run-your-garden.md", "## Everyday jobs (the menu)", "## 평소 작업(메뉴)",
     [("screen-01-main-menu.png", "The menu (type a number or letter, then Enter)", "메뉴(숫자나 글자를 입력하고 Enter)")]),
    ("04-go-public-no-ip-caddy.md", "## The three choices in the front door step (menu `M`, then `1`)", "## 정문 단계의 세 가지 선택(메뉴 `M`, 그다음 `1`)",
     [("screen-14-front-door.png", "What you will see: the front door's three choices", "보게 될 화면: 정문의 세 가지 선택")]),
    ("04-go-public-no-ip-caddy.md", "## Advanced (optional): subdomains with DuckDNS", "## 고급(선택): DuckDNS로 하위 도메인 만들기",
     [("screen-15-subdomains.png", "The Subdomains menu (menu M, then 13)", "하위 도메인 메뉴(메뉴 M, 그다음 13)")]),
    ("04-go-public-no-ip-caddy.md", "## Website name swap: a different web name (optional)", "## 웹사이트 이름 바꾸기: 다른 웹 이름(선택)",
     [("screen-18-website-name.png", "The Website name swap explains itself first", "웹사이트 이름 바꾸기는 먼저 스스로 설명해요")]),
    ("13-backups-and-folders.md", "## Make one, check one, restore one", "## 만들기, 확인하기, 복원하기",
     [("screen-13-backups.png", "The Backups menu (menu 6)", "백업 메뉴(메뉴 6)")]),
    ("16-ai-studio.md", "## 4. A free API key for the garden's AI boxes", "## 4. 정원의 AI 상자를 위한 무료 API 키",
     [("screen-12-keys.png", "The keys menu (menu K): keys are never shown", "키 메뉴(메뉴 K): 키는 절대 화면에 보이지 않아요")]),
    ("19-every-menu-option.md", "## Main menu", "## 메인 메뉴",
     [("screen-01-main-menu.png", "The main menu", "메인 메뉴")]),
    ("19-every-menu-option.md", "### 1) INSTALL  (first time, or carry on, or repair)", None,
     [("screen-04-install-start.png", "The install: its 14 steps, each with how long it usually takes", "설치: 14단계, 단계마다 보통 걸리는 시간"),
      ("screen-05-install-firewall.png", "Step 5: the firewall keeps SSH open on port 22, from home only", "5단계: 방화벽이 SSH를 포트 22에서 집에서만 열어 둬요"),
      ("screen-06-install-plants-photos.png", "Step 12: your plants, then your photos from their download", "12단계: 내 식물, 그다음 사진 파일에서 사진"),
      ("screen-07-install-done.png", "The last step, then ALL DONE", "마지막 단계, 그다음 ALL DONE")]),
    ("19-every-menu-option.md", "### 5) Checklist: is everything OK?", None,
     [("screen-19-checklist.png", "The checklist: PASS, WARN or FAIL, with what to do", "점검표: PASS, WARN, FAIL과 할 일")]),
    ("19-every-menu-option.md", "### A) My addresses: every web address, and which need signing in", None,
     [("screen-20-addresses.png", "My addresses", "내 주소")]),
    ("19-every-menu-option.md", "## More (M)", "## More (M): 더 보기 메뉴",
     [("screen-02-more-menu.png", "The More menu", "More(더 보기) 메뉴")]),
    ("19-every-menu-option.md", "## Backups (6)", "## Backups (6): 백업 메뉴",
     [("screen-13-backups.png", "The Backups menu", "백업 메뉴")]),
    ("19-every-menu-option.md", "## Settings (8)", "## Settings (8): 설정 메뉴",
     [("screen-10-settings.png", "The Settings menu (passwords are never shown)", "설정 메뉴(비밀번호는 절대 보이지 않아요)")]),
    ("19-every-menu-option.md", "## Parts (9)", "## Parts (9): 구성 요소 화면",
     [("screen-09-parts.png", "The Parts screen: type a number to switch it, then A", "구성 요소 화면: 숫자로 켜고 끄고, 그다음 A")]),
    ("19-every-menu-option.md", "## Users and access (U)", "## Users and access (U): 사용자와 접근 권한",
     [("screen-11-users.png", "Users and access", "사용자와 접근 권한")]),
    ("19-every-menu-option.md", "## API keys (K)", "## API keys (K): API 키",
     [("screen-12-keys.png", "The keys menu", "키 메뉴")]),
    ("19-every-menu-option.md", "## The front door's three choices", "## 정문의 세 가지 선택",
     [("screen-14-front-door.png", "The front door's three choices", "정문의 세 가지 선택")]),
    ("19-every-menu-option.md", "## Subdomains (More 13; Windows: More 14)", "## Subdomains (More 13; Windows: More 14): 하위 도메인",
     [("screen-15-subdomains.png", "Subdomains", "하위 도메인")]),
    ("19-every-menu-option.md", "## AI cost control (LiteLLM) (More 15; Windows: More 16)", "## AI cost control (LiteLLM) (More 15; Windows: More 16): AI 비용 관리",
     [("screen-16-litellm.png", "LiteLLM, before it is switched on", "켜기 전의 LiteLLM")]),
    ("19-every-menu-option.md", "## Uninstall (More 12)", "## Uninstall (More 12): 제거하기",
     [("screen-17-uninstall.png", "Uninstall: two levels", "제거하기: 두 단계")]),
    ("19-every-menu-option.md", "### 16) Website name swap: use another web name than junsgarden.zapto.org", None,
     [("screen-18-website-name.png", "The Website name swap", "웹사이트 이름 바꾸기")]),
]


def place(path, heading, pics, prefix, ko):
    if not path.exists() or heading is None:
        return 0
    s = io.open(path, encoding="utf-8", newline="").read()
    nl = "\r\n" if "\r\n" in s else "\n"
    lines = s.split(nl)
    try:
        # Exact heading; or (Korean guide 19) the English heading followed by " — Korean".
        i = next(k for k, l in enumerate(lines) if l.rstrip() == heading or l.startswith(heading + " — "))
    except StopIteration:
        print("  heading not found:", path.name, heading)
        return 0
    add = []
    for pic, cap_en, cap_ko in pics:
        if f"images/{pic}" in s and heading.startswith("##") and f"images/{pic})" in nl.join(lines[i:i + 12]):
            continue
        add += [f"![{cap_ko if ko else cap_en}]({prefix}images/{pic})", ""]
    if not add:
        return 0
    if i + 1 < len(lines) and lines[i + 1].strip() == "":
        add = add[:-1]          # the guide already has an empty line here
    lines[i + 1:i + 1] = [""] + add
    io.open(path, "w", encoding="utf-8", newline="").write(nl.join(lines))
    return len(add) // 2


made = 0
for guide, h_en, h_ko, pics in PLACES:
    made += place(ROOT / guide, h_en, pics, "", False)
    made += place(ROOT / "ko" / guide, h_ko or h_en, pics, "../", True)
print("pictures placed:", made)
