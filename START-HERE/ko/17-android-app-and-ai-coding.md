# 17. 내 Android 앱, 그리고 AI로 코딩하기

> 영어 원문(전체 안내서): [../17-android-app-and-ai-coding.md](../17-android-app-and-ai-coding.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래 링크(영어)에 있어요.*

- Windows 데스크톱의 Android Studio로 Jun's Garden Android 앱을 만들고, 휴대폰에서 실행하고, AI의 도움으로 코드를 안전하게 바꿔요.
- 약 3시간, 몇 번에 나눠서 해요. 무료예요.
- A 부분: Android Studio를 설치하고, `android-app` 폴더를 열고, 휴대폰에서 USB debugging(USB 디버깅)을 켜고, 초록색 **Run** 삼각형을 누르세요.
- B 부분: Android Studio에서 Gemini를 켜세요(무료는 "Gemini for individuals").
- C 부분(선택): Ollama로 로컬 AI. `qwen2.5-coder:7b`부터 시작하세요.
- D 부분, 바꿀 때마다: 먼저 저장(git), 작은 변경 하나만 부탁하기, `git diff`로 읽기, 빌드, 테스트, 그다음 그대로 두기(commit) 또는 되돌리기(`git restore .`).
- 휴대폰에 웹사이트에서 받은 앱이 이미 있으면, Android가 내가 만든 앱의 설치를 거부할 수 있어요(서명 키 때문). 이것을 "고치려고" 가족 휴대폰에서 앱을 지우지 **마세요**.
- 비밀번호, 키 파일, `.env`, `local.properties`를 어떤 AI에도 절대 붙여 넣지 마세요. 무료 Gemini는 입력한 내용을 학습에 쓸 수 있어요.

---

전체 내용은 영어 안내서에 있어요: [../17-android-app-and-ai-coding.md](../17-android-app-and-ai-coding.md)
