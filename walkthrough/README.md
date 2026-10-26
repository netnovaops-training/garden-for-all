# Picture walkthroughs / 사진으로 보는 사용법

How to use the garden, one picture per step (computer and phone):

- [JunsGarden-guide-English.pdf](JunsGarden-guide-English.pdf)
- [JunsGarden-guide-Korean.pdf](JunsGarden-guide-Korean.pdf) (한국어)

Signing in and your password, adding, editing and removing a plant, photos, beds and bed labels
(QR codes), comments and the guestbook, search, the admin page, the phone app, and changing the
garden's web address. The pictures come from a test copy with made-up sample plants.

정원 사용법을 단계마다 사진 한 장으로 보여 줘요(컴퓨터와 휴대폰): 로그인과 비밀번호, 식물 더하기·고치기·빼기,
사진, 화단과 화단 이름표(QR 코드), 댓글과 방명록, 찾기, 관리 페이지, 휴대폰 앱, 정원의 웹 주소 바꾸기.
사진은 지어낸 예시 식물이 든 시험용 정원에서 찍었어요.

**Making them again** (each release): `source/` has the text (`guide-en.md`, `guide-ko.md`) and the
scripts. `GARDEN_LAB=user@host bash walkthrough/source/run.sh <main sha>` builds a fresh copy of
main with the repo's sample data on a lab machine, takes the pictures (`shots.py`) and makes the
PDFs (`build.py`, headless Chromium). The phone pictures come from the app test
(`docs/rust-pages/app-matrix/matrix.sh`).

## Adding a plant with the local plant AI / 로컬 식물 AI로 식물 추가하기

The plant admin's short path: the Add plant tab (six boxes), watching the AI fill it in, the
review (filled, needs review, still missing), Approve, the Receipt and Pictures tabs, "Export
what's still missing" with its prompt, the JSON import under Advanced, and the desktop helper:

- [streamlit/JunsGarden-Streamlit-guide-English.pdf](streamlit/JunsGarden-Streamlit-guide-English.pdf)
- [streamlit/JunsGarden-Streamlit-guide-Korean.pdf](streamlit/JunsGarden-Streamlit-guide-Korean.pdf) (한국어)

식물 관리 화면의 짧은 길: 식물 추가 탭(칸 여섯 개), AI가 채우는 것 지켜보기, 검토(채운 것, 검토 필요, 아직
비어 있음), 승인, 영수증과 사진 탭, 프롬프트가 담긴 '아직 비어 있는 것 내보내기', 고급 아래의 JSON 가져오기,
데스크톱 도우미.

**Making them again**: `sh walkthrough/streamlit/run.sh` on any computer with Docker: a throwaway
garden from this folder with the sample data (`lab.sh`, no published port), the pictures
(`shots.py`; the desktop helper is played by a recorded sample run, `recorded-run.json`, sent
through the helper's real conversation with the website, so no AI model runs), then the PDFs
(`build.py`). `sh walkthrough/streamlit/run.sh pdf` remakes only the PDFs after a text edit.
