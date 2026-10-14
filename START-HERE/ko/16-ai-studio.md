# 16. Google AI Studio와 Gemini: 조사, 코딩 도움, 내 컴퓨터 알기

> 영어 원문: [../16-ai-studio.md](../16-ai-studio.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- Google AI Studio(<https://aistudio.google.com>)는 Gemini를 쓰는 Google의 무료 웹 페이지예요: 출처가 있는 웹 검색, 코딩 도움, 무료 API 키.
- 웹을 검색하려면 Tools에서 **Grounding with Google Search**를 켜세요. 보여 주는 출처를 열어 보세요: 요약이 아니라 출처를 믿으세요.
- AI 상자용 무료 키: **Get API key**, **Create API key**, 그다음 정원 컴퓨터에서 메뉴 `8`, `4`, `1`(Gemini), 다시 `1`(Paste a new key, 새 키 붙여 넣기)에서 붙여 넣으세요. 화면에 보이지 않고 `.env`에 저장돼요.
- 무료 키에는 하루 한도가 있어요. 한도에 닿으면 AI 상자는 기다렸다가 나중에 다시 해요. 정상이에요.
- 내 컴퓨터에 대해 알아보려면: `bash lab/10-collect-for-gemini.sh`가 비밀번호와 키를 뺀 정보를 모아 줘요.
- 돈을 내는 것은 본인의 선택이고, 전혀 필요 없어요. 결제를 추가한다면 먼저 예산 알림(budget alert)을 정하세요.
- `.env`, 비밀번호, API 키, 백업을 AI 채팅에 절대 붙여 넣지 마세요. 키가 새어 나가면 새 키를 만들고 예전 키를 지우세요.

---

## 무엇인가요
**Google AI Studio**(<https://aistudio.google.com>)는 Gemini 모델로 작업하는 Google의
무료 웹 페이지예요. Gemini 앱보다 기능이 많아요: 인터넷 검색 같은 도구를 켤 수 있고,
파일을 줄 수 있고, 정원의 AI 상자가 Gemini를 쓸 수 있게 해 주는 무료 **API 키**를
만들 수 있어요.

## 1. 인터넷에서 새 정보 가져오기("grounding", 근거 찾기)
보통 AI는 어느 날짜 전까지 배운 것으로 대답해요. **Grounding with Google Search**를
켜면, Gemini가 먼저 웹을 검색하고 쓴 페이지를 보여 줘요. 그래서 직접 확인할 수 있어요.

1. <https://aistudio.google.com>을 열고 Google 계정으로 로그인하세요.
2. 새 채팅을 시작하세요. 오른쪽의 **Tools**(도구) 아래에서 **Grounding with
   Google Search**를 켜세요.
3. 이렇게 물어보세요(예):
   - "What is the last frost date for my town this year? Show your sources."
     (올해 우리 동네의 마지막 서리 날짜는 언제인가요? 출처를 보여 주세요.)
   - "Which of these plants are toxic to cats: hostas, bee balm, marigolds? Cite sources."
     (이 식물 중 고양이에게 독이 있는 것은? 호스타, 베르가못, 메리골드. 출처를 밝혀 주세요.)
   - "What does the newest Caddy release change? Link the release notes."
     (최신 Caddy 버전에서 무엇이 바뀌었나요? 릴리스 노트 링크를 주세요.)
   한국어로 물어봐도 돼요.
4. **AI가 보여 준 출처를 열어 보세요.** 요약이 아니라 출처를 믿으세요.

공식 안내서: <https://ai.google.dev/gemini-api/docs/google-search>

## 2. 코딩 도움(Rust, bash, Docker)
- **오류 줄을 그대로** 붙여 넣고, 어느 파일을 바꿨는지 말하세요(안내서 11).
- 고치기 전에 설명해 달라고 하세요: "Explain this error in simple words, then
  suggest the smallest change."(이 오류를 쉬운 말로 설명하고, 가장 작은 변경을 제안해 주세요.)
- 규칙을 알려 주세요: 채팅을 시작할 때 정원 폴더의 `GEMINI.md` 내용을 붙여 넣으세요.
  AI에게 나를 안전하게 돕는 방법을 알려 줘요.
- 제안을 그대로 두기 전에 `git diff`로 모두 확인하고, 무언가 고장 나면
  `git checkout -- FILE`로 되돌리세요.

정원 컴퓨터에서는 **Gemini CLI**도 쓸 수 있어요. Gemini CLI는 `GEMINI.md`를 스스로
읽어요(안내서 08).

## 3. 내 컴퓨터에 대해 배우기
AI는 내 컴퓨터를 볼 수 없어요. 그래서 내가 사실을 보여 줘야 해요:
1. 정원 폴더에서 `bash lab/10-collect-for-gemini.sh`를 실행하세요. 비밀번호와 키를 뺀
   사실(Ubuntu 버전, 메모리, 디스크, 실행 중인 컨테이너, 최근 오류)을 모아 줘요.
2. 모은 내용을 먼저 읽고, AI Studio에 붙여 넣은 다음 이렇게 물어보세요(예):
   "Here are facts about my Ubuntu server. What should I improve first, and why?"
   (내 Ubuntu 서버에 대한 사실이에요. 무엇을 먼저 개선해야 하고, 왜 그런가요?)
3. 문제가 있을 때: 메뉴 `H`가 도움 파일(비밀번호 없음)을 만들어요. 같은 방법으로 붙여 넣으세요.

## 4. 정원의 AI 상자를 위한 무료 API 키

![키 메뉴(메뉴 K): 키는 절대 화면에 보이지 않아요](../images/screen-12-keys.png)

1. AI Studio에서 **Get API key**, 그다음 **Create API key**를 누르세요
   (<https://aistudio.google.com/apikey>).
2. 정원 컴퓨터에서: 메뉴 `8` 설정, 그다음 `4`(API 키), 그다음 `1`(Gemini),
   그다음 다시 `1`(**Paste a new key (or replace the old one)**, 새 키 붙여 넣기(또는 예전 키 바꾸기)).
   거기에 붙여 넣으세요(`Ctrl+Shift+V`). 화면에 보이지 않고 `.env`에 저장돼요.
3. 무료 키에는 하루 한도가 있어요. 한도에 닿으면 AI 상자는 기다렸다가 나중에 다시
   시도해요. 정상이에요.

한도: <https://ai.google.dev/gemini-api/docs/rate-limits>

**키가 어디에 저장되고, 어떻게 바꾸나요.** 키는 정원 컴퓨터의 정원 폴더 안 `.env`
파일, `GEMINI_API_KEY=` 줄에 저장돼요. 그 파일은 직접 열지 않아요: 메뉴 `8`, `4`가 키가
있는지, 바꾼 날짜, 정확한 줄, 어떤 프로그램이 쓰는지 보여 줘요. 거기서 새 키를 붙여 넣거나,
테스트하거나, 지울 수 있어요. 붙여 넣은 뒤에는 웹사이트와 정원 도구를 다시 시작하고, 새 키를
받았는지 확인해요.

**키 교체하기**(안전을 위해, 또는 키가 새어 나갔을 때 새로 만들기): AI Studio에서 새 키를
만들고, 메뉴 `8`, `4`, `1`, `1`로 붙여 넣고, 테스트한 다음, AI Studio에서 예전 키를
지우세요(<https://aistudio.google.com/apikey>).

## 무료 한도보다 더 쓰기(본인의 선택)
정원은 무료 등급으로 작동해요. 더 쓰고 싶다면, 그것은 본인의 결정이고 본인의 계정이에요:
1. AI Studio에서 키의 프로젝트를 열고 **Set up billing**(결제 설정)을 고르세요. Google이
   Google Cloud에서 차근차근 안내해 줘요.
2. **먼저 예산과 알림을 정하세요**: Google Cloud, Billing, **Budgets & alerts**(예산 및 알림).
   예: 한 달에 5달러, 50%와 90%에서 이메일 알림.
   <https://cloud.google.com/billing/docs/how-to/budgets>
3. 정원은 바꿀 것이 없어요: 같은 키가 더 높은 한도를 받을 뿐이에요.

가격: <https://ai.google.dev/gemini-api/docs/pricing>

정원의 다른 AI 서비스인 OpenRouter에는 항상 무료 모델(이름이 `:free`로 끝나는 것)만
요청해요.

## OpenRouter는 무엇인가요(그리고 무엇이 아닌가요)
OpenRouter(<https://openrouter.ai>)는 키 하나로 여러 AI 모델(Google, Meta, Mistral 등에서
만든 것) 중 하나에게 AI 요청을 보내 주는 서비스예요. 정원은 Gemini 무료 등급이 바쁠 때
쓰는 **예비**로만, 그리고 **무료** 모델만 써요.

- **본인의 계정**이고, 본인의 로그인으로 만들어요. 제작자를 포함해 다른 누구도 그 계정,
  키, 기록을 볼 수 없어요.
- OpenRouter는 내 컴퓨터를 절대 보지 못해요. 정원이 읽으라고 보낸 것만 받아요:
  식물 이름표 사진, 식물 이름, 식물에 대한 질문.
- 알아 두세요: 무료 모델은 모델을 만든 회사가 받은 글과 사진을 보관할 수도 있어요(모델을
  개선하려고요). 그래서 정원은 비밀번호, 개인 파일, 비공개 내용을 절대 보내지 않아요. 무엇을
  허용할지는 OpenRouter의 **privacy settings**(개인정보 설정)에서 고를 수 있어요:
  <https://openrouter.ai/settings/privacy>
- 꼭 필요한 것은 아니에요: OpenRouter 키가 없으면 정원은 그냥 Gemini를 기다려요.
  메뉴 `K`에서 언제든지 키를 추가하거나 지울 수 있어요.

설명서: <https://openrouter.ai/docs>

## 지출 한도가 있는 AI 한곳: LiteLLM(선택)
AI를 쓰는 나만의 프로젝트(n8n 작업 흐름, 코딩 도구, 스크립트)를 만든다면, 메뉴 `M`,
그다음 `15`가 **LiteLLM**을 켜요: 정원 컴퓨터의 `http://localhost:4000`에 있는 오픈 소스
"AI gateway"(AI 관문)예요.
- 내 프로젝트는 Google을 직접 부르지 않고 LiteLLM을 불러요. LiteLLM은 메뉴 `K`로 저장한
  키를 써요. 그래서 내 프로젝트는 키를 가지고 있지 않아요.
- **월 한도를 달러로** 정해요. 한도에 닿으면, 다음 30일이 시작될 때까지 LiteLLM이 요청을
  거절해요.
- 설정 파일은 `local/litellm/config.yaml`이에요(키는 없어요). 내 프로젝트가 쓸 LiteLLM
  자체 키는 `.env`의 `LITELLM_MASTER_KEY`예요. 프로젝트가 키를 물어보면
  `grep ^LITELLM_MASTER_KEY= .env`로 보세요.
- n8n이나 OpenAI 방식 도구에서: base URL `http://localhost:4000`, model
  `gemini-flash`(또는 `free-openrouter`), key = 그 master key.

정원 웹사이트와 정원 도구는 LiteLLM을 거치지 않아요. 무료 키를 직접 써요. LiteLLM을
다시 끄려면 같은 메뉴 항목을 쓰세요.
공식 설명서: <https://docs.litellm.ai/docs/simple_proxy>, 예산: <https://docs.litellm.ai/docs/proxy/users>

참고: 지출 한도는 LiteLLM 설명서를 따라 설정했지만, 실제 유료 요청으로는 테스트하지
않았어요. 결제를 추가한다면 위의 Google 예산 알림도 꼭 유지하세요: 안전망은 하나보다
두 개가 나아요.

## 안전 규칙(항상)
- `.env`, 비밀번호, API 키, 백업은 어떤 AI 채팅에도 **절대 붙여 넣지 마세요**.
- 결제는 선택이에요. 정원에는 전혀 필요 없어요. 결제를 추가한다면 먼저 예산 알림부터 설정하세요(위).
- AI는 자신 있게 틀릴 수 있어요. 안전에 관한 것(방화벽, 비밀번호, 포트)은 AI가 링크한
  공식 설명서에서 확인하세요.
- 실수로 키를 붙여 넣었다면, AI Studio에서 새 키를 만들고, 예전 키를 지우고, 설정에서
  새 키를 넣으세요.

## 더 읽기(공식)
- AI Studio 빠른 시작: <https://ai.google.dev/gemini-api/docs/ai-studio-quickstart>
- Gemini API 설명서: <https://ai.google.dev/gemini-api/docs>
- Gemini CLI: <https://github.com/google-gemini/gemini-cli>
