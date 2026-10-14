# 05. Gemini prompts you can copy

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Starter questions for Gemini (gemini.google.com or Gemini CLI). Copy one, change the CAPITAL words, and paste.
- Paste prompt 1, the context block, first in every new chat.
- Prompt 2 explains a command before you run it. Prompt 3 is for when you are stuck.
- Use prompt 6 ("Check my plan") before anything involving power or water.
- Never put passwords, app passwords, API keys or your `.env` file into any AI chat. Check a file for secrets before you paste it.
- Gemini CLI reads `GEMINI.md` in the project folder by itself.

---

You can use Gemini (gemini.google.com) or Gemini CLI. These are starters. Copy
one, change the CAPITAL words, and paste.

**Do not put passwords, app passwords, API keys, or the contents of your `.env`
file into any AI chat.** If a prompt asks you to paste a file, check the file
for secrets first.

---

## 1. The context block (paste this FIRST in every new chat)

```
I am Jun. I am learning to build home and garden projects. I explain things
best when they are in short numbered steps, with plain words and with exactly
what I should see after each step. Please do not use idioms or jokes. Tell me
what a command does before I run it.

My setup:
- An old laptop running Ubuntu 24.04 LTS, with Docker and Docker Compose.
- My garden website runs from a folder called junsgarden-for-jun (Rust website,
  MariaDB database, a few containers) at http://localhost:8030.
- Projects I use or plan to use: Home Assistant (Docker), Govee lights,
  Mosquitto (MQTT), ESPHome with an ESP32 board, Uptime Kuma, Caddy, No-IP.
- I am a beginner. I am careful. I do not run anything that I do not understand.

Rules for you:
1. If a command could delete data or open my network to the internet, say so
   in a line that starts with WARNING before you give the command.
2. Prefer the official documentation. Tell me which page you used.
3. If you are not sure, say you are not sure.
4. Never ask me for a password or key.

Say "ready" and wait for my question.
```

## 2. "Explain this before I run it"

```
Explain this command in plain words, part by part. Tell me what it changes on
my laptop and whether I can undo it:

PASTE THE COMMAND HERE
```

## 3. "I am stuck"

```
I am stuck. I was doing: NAME OF THE GUIDE AND STEP NUMBER.
I expected to see: WHAT THE GUIDE SAID I SHOULD SEE.
Instead I see: PASTE THE EXACT TEXT (hide any passwords).

Give me the three most likely causes, most likely first. For each, give one
command that checks it, and tell me what the answer means. Do not change
anything until I tell you the check result.
```

## 4. "Teach me what this code does"

```
Here is a file from my garden website, written in Rust. Explain it in plain
words, one function at a time. After each function, stop and ask if I want to
continue. I do not know Rust yet, so explain any Rust word the first time it
appears.

PASTE THE FILE OR A PART OF IT
```

## 5. "Make a small change"

```
In my Rust garden website I want to change THIS: DESCRIBE IT.
Show me the smallest change, which file it goes in, and how to rebuild with
the garden menu (M, then 7, or "bash garden.sh rebuild"). Explain what each part of the change does.
Tell me how I can undo it.
```

## 6. "Check my plan" (before anything involving power or water)

```
I plan to build: DESCRIBE THE PROJECT.
Parts I will use: LIST THEM WITH VOLTAGES.
Check my plan for safety problems with water and electricity. Tell me anything
I should not do. Tell me what could break if the power fails halfway or if the
internet goes down. I am only using low-voltage parts (5 V or 12 V) and I will
never touch wall-outlet wiring.
```

## 7. "Design me a Home Assistant automation"

```
I use Home Assistant in Docker. I want an automation that: DESCRIBE IT IN ONE
SENTENCE (for example: turn on my Govee lights at sunset).
Show me how to build it in the Home Assistant screen first (Settings,
Automations), step by step. Then show the same automation as YAML so I can
read how it is written.
```

## 8. "Review my work" (after you finish something)

```
I finished: NAME OF THE PROJECT.
Here is what I did: SHORT LIST.
What is one thing I did well? What is one thing I could make safer or tidier?
What is a good next small project that builds on this?
```

## 9. "Write the resume line"

```
Turn what I did into one honest sentence for a hobby-projects list, starting
with a verb. Do not exaggerate.
What I did: DESCRIBE IT.
```

---

## For Gemini CLI users
If you use Gemini CLI inside the project folder, it reads `GEMINI.md`
automatically. That file tells it how this project works, how to talk to you,
and what it must not do. You can add your own notes at the bottom of it.
