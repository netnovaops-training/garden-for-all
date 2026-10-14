# 22. Bash, 한 단계씩: 터미널 처음 따라 해 보기

> 영어 원문(전체 안내서): [../22-bash-walkthrough.md](../22-bash-walkthrough.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래 링크(영어)에 있어요.*

- **터미널**은 클릭 대신 명령을 입력하는 창이에요. **Bash**는 그 안에서 내가 입력한 것을 읽고 실행하는 프로그램이에요.
- Ubuntu에서는 **Ctrl+Alt+T**로 열어요. Windows에서는 **Garden terminal** 바로 가기를 쓰세요(Windows 메뉴, More, 18).
- 명령을 입력하고 **Enter**를 누른 다음, 나오는 글을 읽으세요. Enter를 누르기 전에는 아무 일도 일어나지 않아요.
- 명령 다섯 개면 많은 것을 할 수 있어요: `pwd`(나는 어디에 있나요?), `ls`(여기 무엇이 있나요?), `cd`(폴더로 가기), `less`(파일 읽기), `clear`(화면 지우기).
- 맨 처음에는 정원 폴더에서 `bash Start-Here-Ubuntu.sh`를 실행하세요. GitHub에서 프로그램(`garden.sh`와 나머지)을 받아 확인해요. 그다음부터는 `cd ~/junsgarden-for-jun`, 그리고 `bash garden.sh`로 정원 프로그램을 실행해요.
- 컴퓨터가 비밀번호를 물으면 **입력하는 동안 아무것도 보이지 않아요**. 정상이에요: 입력하고 Enter를 누르세요.
- **Ctrl+C**는 실행 중인 것을 멈춰요. 터미널에서 복사는 **Ctrl+Shift+C**, 붙여 넣기는 **Ctrl+Shift+V**예요.
- `rm`은 영원히 지워요(휴지통이 없어요). 모르는 사람이 준 명령은 절대 붙여 넣지 마세요.
- **지름길:** 위쪽 화살표는 마지막 명령을 다시 불러오고, Tab은 이름을 끝까지 채워 주고, Ctrl+R은 전에 입력한 것을 찾아요.
- **파일 만들고 고치기:** `touch notes.txt`로 만들고, `nano notes.txt`로 고쳐요(Ctrl+O, Enter로 저장, Ctrl+X로 나가기).
- **멈춘 것 같나요?** Ctrl+C, 그다음 `q`, 그다음 창을 닫고 새로 여세요. 정원 설치는 멈춘 곳부터 이어서 해요.
- **업데이트:** 일주일에 한 번쯤 `sudo apt update`, 그다음 `sudo apt upgrade`.

---

전체 내용은 영어 안내서에 있어요: [../22-bash-walkthrough.md](../22-bash-walkthrough.md)
