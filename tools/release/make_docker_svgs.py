"""Two simple pictures for the 'What is Docker?' page, in English and Korean.
Picture 1: a VM = a whole second computer inside your computer (heavy).
Picture 2: Docker = small boxes, one program each, sharing one computer (light)."""
import os
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "START-HERE", "images")

T = {
 "en": dict(
   vm_title="A virtual machine (VM): a whole extra computer inside your computer",
   your_pc="Your computer", vm="Pretend computer", os="Its own Windows or Ubuntu",
   prog="One program", mem="usually needs 2 to 4 GB of memory", slow="starts in minutes",
   dk_title="Docker: small boxes, each with one program, sharing your computer",
   ubuntu="Ubuntu (one, shared by all the boxes)", engine="Docker",
   box1="Website", box2="Database", box3="Garden tools", box4="Sign-in page",
   small="each box: only what its program needs", fast="starts in seconds"),
 "ko": dict(
   vm_title="가상 머신(VM): 내 컴퓨터 안의 또 다른 컴퓨터 한 대 전체",
   your_pc="내 컴퓨터", vm="흉내 내는 컴퓨터", os="자기만의 Windows나 Ubuntu",
   prog="프로그램 하나", mem="보통 메모리 2~4 GB 필요", slow="켜지는 데 몇 분",
   dk_title="Docker: 프로그램 하나씩 든 작은 상자들이 내 컴퓨터를 함께 써요",
   ubuntu="Ubuntu(하나, 모든 상자가 함께 써요)", engine="Docker",
   box1="웹사이트", box2="데이터베이스", box3="정원 도구", box4="로그인 페이지",
   small="상자마다: 그 프로그램에 필요한 것만", fast="켜지는 데 몇 초"),
}
FONT = "font-family='Segoe UI, Malgun Gothic, Noto Sans KR, Arial, sans-serif'"


def vm_svg(t):
    blocks = ""
    for i in range(2):
        x = 40 + i * 340
        blocks += f"""
  <rect x='{x}' y='92' width='300' height='190' rx='14' fill='#fde8d7' stroke='#c0632b' stroke-width='3'/>
  <text x='{x + 150}' y='122' text-anchor='middle' font-size='19' font-weight='bold' fill='#8a3b12'>{t['vm']} {i + 1}</text>
  <rect x='{x + 20}' y='138' width='260' height='58' rx='8' fill='#f7c9a6'/>
  <text x='{x + 150}' y='173' text-anchor='middle' font-size='17' fill='#5a2508'>{t['os']}</text>
  <rect x='{x + 70}' y='210' width='160' height='52' rx='8' fill='#ffffff' stroke='#c0632b'/>
  <text x='{x + 150}' y='242' text-anchor='middle' font-size='17' fill='#5a2508'>{t['prog']}</text>"""
    return f"""<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 720 380' {FONT}>
  <rect width='720' height='380' fill='#ffffff'/>
  <text x='360' y='34' text-anchor='middle' font-size='20' font-weight='bold' fill='#1f2a1f'>{t['vm_title']}</text>
  <rect x='20' y='58' width='680' height='240' rx='18' fill='none' stroke='#555' stroke-width='3' stroke-dasharray='8 6'/>
  <text x='36' y='82' font-size='16' fill='#555'>{t['your_pc']}</text>{blocks}
  <text x='360' y='330' text-anchor='middle' font-size='18' fill='#8a3b12'>&#9888; {t['mem']}, {t['slow']}</text>
</svg>
"""


def docker_svg(t):
    boxes = ""
    for i, k in enumerate(("box1", "box2", "box3", "box4")):
        x = 40 + i * 162
        boxes += f"""
  <rect x='{x}' y='96' width='146' height='92' rx='12' fill='#e3f1dd' stroke='#2f7d32' stroke-width='3'/>
  <text x='{x + 73}' y='148' text-anchor='middle' font-size='17' font-weight='bold' fill='#1d4d1f'>{t[k]}</text>"""
    return f"""<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 720 380' {FONT}>
  <rect width='720' height='380' fill='#ffffff'/>
  <text x='360' y='34' text-anchor='middle' font-size='20' font-weight='bold' fill='#1f2a1f'>{t['dk_title']}</text>
  <rect x='20' y='58' width='680' height='240' rx='18' fill='none' stroke='#555' stroke-width='3' stroke-dasharray='8 6'/>
  <text x='36' y='82' font-size='16' fill='#555'>{t['your_pc']}</text>{boxes}
  <rect x='40' y='204' width='632' height='34' rx='8' fill='#cfe0f5' stroke='#2b5c99'/>
  <text x='356' y='227' text-anchor='middle' font-size='17' fill='#163a66'>{t['engine']}</text>
  <rect x='40' y='246' width='632' height='38' rx='8' fill='#dfe3e8' stroke='#555'/>
  <text x='356' y='271' text-anchor='middle' font-size='17' fill='#222'>{t['ubuntu']}</text>
  <text x='360' y='330' text-anchor='middle' font-size='18' fill='#1d4d1f'>&#10003; {t['small']}, {t['fast']}</text>
</svg>
"""


for lang, t in T.items():
    open(os.path.join(OUT, f"docker-1-vm-{lang}.svg"), "w", encoding="utf-8").write(vm_svg(t))
    open(os.path.join(OUT, f"docker-2-boxes-{lang}.svg"), "w", encoding="utf-8").write(docker_svg(t))
print("ok")
