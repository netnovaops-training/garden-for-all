# 16. Google AI Studio and Gemini: research, coding help, and knowing your computer

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Google AI Studio (<https://aistudio.google.com>) is Google's free web page for Gemini: web search with sources, coding help, and a free API key.
- To search the web, switch on **Grounding with Google Search** under Tools. Open the sources it lists: trust the source, not the summary.
- A free key for the AI boxes: **Get API key**, **Create API key**, then on the garden computer menu `8`, `4`, `1` (Gemini), `1` again (Paste a new key) and paste it. It is hidden and saved in `.env`.
- Free keys have daily limits. When they are reached, the AI boxes wait and try again later. That is normal.
- To learn about your computer: `bash lab/10-collect-for-gemini.sh` collects facts with passwords and keys removed.
- Paying is your choice and never needed. If you ever add billing, set a budget alert first.
- Never paste `.env`, a password, an API key or a backup into any AI chat. If a key leaks, make a new one and delete the old one.

---

## What it is
**Google AI Studio** (<https://aistudio.google.com>) is Google's free web page
for working with Gemini models. It is more powerful than the Gemini app: you
can turn on tools such as searching the internet, give it files, and make a
free **API key** that lets your garden's AI boxes use Gemini.

## 1. Pull fresh information from the internet ("grounding")
Normal AI answers from what it learned before a certain date. With **Grounding
with Google Search** switched on, Gemini searches the web first and shows the
pages it used, so you can check them.

1. Open <https://aistudio.google.com> and sign in with your Google account.
2. Start a new chat. On the right, under **Tools**, switch on **Grounding with
   Google Search**.
3. Ask, for example:
   - "What is the last frost date for my town this year? Show your sources."
   - "Which of these plants are toxic to cats: hostas, bee balm, marigolds? Cite sources."
   - "What does the newest Caddy release change? Link the release notes."
4. **Open the sources it lists.** Trust the source, not the summary.

Official guide: <https://ai.google.dev/gemini-api/docs/google-search>

## 2. Coding help (Rust, bash, Docker)
- Paste the **exact error lines** and say which file you changed (guide 11).
- Ask it to explain before it fixes: "Explain this error in simple words, then
  suggest the smallest change."
- Give it the rules: paste the contents of `GEMINI.md` from the garden folder at
  the start of the chat. They tell it how to help you safely.
- Check every suggestion with `git diff` before you keep it, and undo with
  `git checkout -- FILE` if it breaks something.

On the garden computer you can also use **Gemini CLI**, which reads
`GEMINI.md` by itself (guide 08).

## 3. Learning about your own computer
AI can't see your computer, so you show it the facts:
1. In the garden folder run `bash lab/10-collect-for-gemini.sh`. It collects
   facts (Ubuntu version, memory, disk, running containers, recent errors) with
   passwords and keys removed.
2. Read what it collected, then paste it into AI Studio and ask, for example:
   "Here are facts about my Ubuntu server. What should I improve first, and why?"
3. For a problem: menu `H` makes a help file (no passwords) you can paste the same way.

## 4. A free API key for the garden's AI boxes

![The keys menu (menu K): keys are never shown](images/screen-12-keys.png)

1. In AI Studio click **Get API key**, then **Create API key**
   (<https://aistudio.google.com/apikey>).
2. On the garden computer: menu `8` Settings, then `4` (API keys), then `1` (Gemini),
   then `1` again (**Paste a new key (or replace the old one)**).
   Paste it there (`Ctrl+Shift+V`). It is hidden and saved in `.env`.
3. Free keys have daily limits. When they are reached, the AI boxes wait and
   try again later. That is normal.

Limits: <https://ai.google.dev/gemini-api/docs/rate-limits>

**Where it goes and how to change it.** The key is saved on the garden computer
in the file `.env` in the garden folder, on the line `GEMINI_API_KEY=`. You
never open that file: menu `8`, `4` shows whether a key is set, the date it was
changed, the exact line, and which programs use it. From there you can paste a
new one, test it, or remove it. After a paste it restarts the website and the
garden tools and checks they have the new key.

**Rotating a key** (making a new one, for safety, or if one leaked): create a new
key in AI Studio, paste it with menu `8`, `4`, `1`, `1`, test it, then delete the old
key in AI Studio (<https://aistudio.google.com/apikey>).

## More than the free limits (your choice)
The garden works on the free tier. If you ever want more, it is your own
decision and your own account:
1. In AI Studio, open your key's project and choose **Set up billing**. Google
   walks you through it in Google Cloud.
2. **Set a budget and alert first**: Google Cloud, Billing, **Budgets & alerts**,
   for example $5 a month, with an email at 50% and 90%.
   <https://cloud.google.com/billing/docs/how-to/budgets>
3. The garden does not need any change: the same key just gets higher limits.

Prices: <https://ai.google.dev/gemini-api/docs/pricing>

The garden's other AI service, OpenRouter, is only ever asked for its free
models (names ending in `:free`).

## What OpenRouter is (and is not)
OpenRouter (<https://openrouter.ai>) is a service that sends an AI request to
one of many AI models (from Google, Meta, Mistral and others) with a single
key. The garden uses it only as a **backup** when Gemini's free tier is busy,
and only its **free** models.

- It is **your own account**, made with your own login. Nobody else, the author
  included, can see it, its keys or its history.
- It never sees your computer. It only receives what the garden sends it to
  read: a plant tag photo, a plant name, a question about a plant.
- Be aware: free models may let the model's maker keep the text and photos sent
  to them (to improve their models). So the garden never sends passwords,
  personal files or anything private. You can choose what is allowed in your
  OpenRouter **privacy settings**: <https://openrouter.ai/settings/privacy>
- You don't need it at all: without an OpenRouter key the garden just waits for
  Gemini. Menu `K` adds or removes the key any time.

Docs: <https://openrouter.ai/docs>

## One place for AI, with a spending limit: LiteLLM (optional)
If you build your own projects that use AI (n8n workflows, coding tools,
scripts), menu `M`, then `15` switches on **LiteLLM**: an open-source "AI
gateway" on the garden computer at `http://localhost:4000`.
- Your projects call LiteLLM instead of Google directly. LiteLLM uses the keys
  you saved with menu `K`, so your projects never hold them.
- You set a **monthly limit in dollars**. When it is reached, LiteLLM refuses
  more requests until the next 30 days start.
- Its settings file is `local/litellm/config.yaml` (no keys in it). Its own key
  for your projects is `LITELLM_MASTER_KEY` in `.env`; see it with
  `grep ^LITELLM_MASTER_KEY= .env` when a project asks for it.
- In n8n or any OpenAI-style tool: base URL `http://localhost:4000`, model
  `gemini-flash` (or `free-openrouter`), key = that master key.

The garden website and garden tools do not go through LiteLLM; they use their
free keys directly. Switch LiteLLM off again with the same menu item.
Official docs: <https://docs.litellm.ai/docs/simple_proxy> and budgets: <https://docs.litellm.ai/docs/proxy/users>

Note: the spending limit was set up following LiteLLM's documentation but has
not been tested with real paid requests. If you add billing, also keep the
Google budget alert above: two safety nets are better than one.

## Safety rules (always)
- **Never paste** `.env`, a password, an API key, or a backup into any AI chat.
- Billing is your choice, never needed for the garden. If you ever add it, set the budget alert first (above).
- AI can be confidently wrong. For anything about safety (firewall, passwords,
  ports), check the official documentation it links to.
- If you ever pasted a key by mistake, make a new one in AI Studio, delete the
  old one, and put the new one in with Settings.

## Read more (official)
- AI Studio quickstart: <https://ai.google.dev/gemini-api/docs/ai-studio-quickstart>
- Gemini API documentation: <https://ai.google.dev/gemini-api/docs>
- Gemini CLI: <https://github.com/google-gemini/gemini-cli>
