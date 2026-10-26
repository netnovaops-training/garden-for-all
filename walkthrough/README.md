# The garden manual / 정원 사용 설명서

How to use the whole garden, with a picture for every step (computer and phone):

- [JunsGarden-guide-English.pdf](JunsGarden-guide-English.pdf)
- [JunsGarden-guide-Korean.pdf](JunsGarden-guide-Korean.pdf) (한국어)

A table of contents at the start (the PDF's bookmarks have it too), then: signing in, getting
around, the home page and posts, the Full Catalog in depth (search, A–Z, filters, beds, card themes,
the plant card and window, the main photo and its focus point, Bulk Move, the Source link, what each
fact means), search on every page, adding, editing and removing a plant, photos, every tab of the
admin page, beds and labels (QR codes), comments and the guestbook, the Gallery, the Data page,
Help, the phone app, changing the web address, and the garden computer's menu (backups, settings).
The last chapter holds two marked places for parts still being built. The pictures come from a test
copy with made-up sample plants.

정원 전체의 사용법을 단계마다 사진과 함께 보여 줘요(컴퓨터와 휴대폰). 맨 앞에 차례가 있고(PDF의 책갈피에도
있어요), 로그인, 둘러보기, 첫 페이지와 글, 전체 목록 자세히(찾기, A–Z, 거르기, 화단, 카드 색깔, 식물 카드와 창,
대표 사진과 초점, 한꺼번에 옮기기, Source 링크, 정보의 뜻), 모든 페이지의 찾기, 식물 더하기·고치기·빼기, 사진,
관리 페이지의 모든 탭, 화단과 이름표(QR 코드), 댓글과 방명록, 사진첩, 자료 페이지, 도움말, 휴대폰 앱, 웹 주소
바꾸기, 정원 컴퓨터의 메뉴(백업, 설정)가 있어요. 마지막 장에는 아직 만드는 두 부분의 자리가 표시되어 있어요.
사진은 지어낸 예시 식물이 든 시험용 정원에서 찍었어요.

**Making them again** (each release): `source/` has the text (`guide-en.md`, `guide-ko.md`) and the
scripts. `GARDEN_LAB=user@host bash walkthrough/source/run.sh <main sha>` builds a fresh copy of
main with the repo's sample data on a lab machine, takes the pictures (`shots.py`; `SHOTS_ONLY="20 3"`
takes only some again) and makes the PDFs (`build.py`, headless Chromium, with the contents, page
numbers and bookmarks). The phone pictures come from the app test
(`docs/rust-pages/app-matrix/matrix.sh`); the menu pictures (`@menu`) from `START-HERE/images/`.
