# 08. Set up Gemini as your lab helper

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Three ways to set up Gemini as your helper: A, a saved "Gem" on the Gemini website (10 minutes); B, Gemini CLI in the project folder (30 minutes); C, a free key for the website's AI boxes (10 minutes).
- Start with A. Add B when you want Gemini to see your actual files.
- Gemini CLI needs Node.js 20 or newer. Install it with `npm install -g @google/gemini-cli`, then type `gemini` inside the project folder.
- Never give Gemini a secret: no passwords, keys or `.env` file. Never let Gemini CLI read `.env`.
- Gemini can be wrong. Check what it says against the guide, and run one command at a time.
- Stuck? `bash lab/10-collect-for-gemini.sh` writes a file of facts with passwords and keys removed. Read it, then paste it with the "I am stuck" prompt.

---

## What you will build
Three ways to use Gemini, set up properly, so it knows your lab and helps you
safely:

| Way | Best for | Effort |
|---|---|---|
| **A. A saved "Gem" in the Gemini website** | Quick questions, explaining things, planning | 10 minutes |
| **B. Gemini CLI inside the project folder** | Reading your files, explaining code, suggesting changes | 30 minutes |
| **C. A free Gemini key for your garden website** | The AI boxes on the website | 10 minutes |

Start with A. Add B when you want Gemini to see your actual files.

## The 5 rules (best practice, in plain words)
1. **Never give Gemini a secret.** No passwords, app passwords, API keys, DDNS
   keys, or the `.env` file. If you do by accident, change that password.
2. **Gemini can be wrong.** It sounds sure even when it is not. Check what it
   says against the guide or the official page before you run a command.
3. **Understand before you run.** Ask "what does this command change, and how
   do I undo it?" every time. Do not run something you cannot explain.
4. **One change at a time.** Run one command. Look at the result. Then the
   next. If two things change at once you cannot tell which one broke it.
5. **Keep a backup before big changes.** `sh docker/backup.sh` takes seconds.

## Way A. A saved Gem (Gemini website)
1. Open <https://gemini.google.com> signed in with your Google account.
2. On the left find **Gems** and click **New Gem** (or "Explore Gems", then
   "New Gem"). The names of the buttons can change; look for "Gems".
3. Name it `My Garden Lab Helper`.
4. In the instructions box paste the **context block** from
   [05 Gemini prompts](05-gemini-prompts.md) (prompt 1). Add these lines at the end:

```
Always answer in short numbered steps. After each step say what I should see.
If a command can delete data or open my network to the internet, start the
line with WARNING. If you are not sure, say so. Ask me before you assume my
Ubuntu version, my router, or my sensor model.
```

5. Save. Start a chat with the Gem whenever you work on the lab.

Tip: start each work session with one sentence: *"Today I am on guide 03, step
5, and I see this: ..."*

## Way B. Gemini CLI (it reads your project)
Gemini CLI runs in the terminal and can read the files in the folder you start
it in. It reads `GEMINI.md` in the project folder on its own, which already
contains the rules for this lab.

Needs: Node.js 20 or newer, and about 4 GB of memory. Check the official page
first, because install steps change: <https://github.com/google-gemini/gemini-cli>.

1. Check Node:

```
node --version
```

   If it says `v20` or higher, go to step 3. If it says "command not found" or a
   lower number (Ubuntu 24.04's own Node is older), go to step 2.
2. Install a current Node. The simplest safe way is **nvm**. Open
   <https://github.com/nvm-sh/nvm#installing-and-updating>, copy the install
   command from that page (it changes with each version), run it, close the
   terminal, open a new one, then:

```
nvm install --lts
node --version
```

3. Install Gemini CLI (this installs it for your user only, no `sudo`):

```
npm install -g @google/gemini-cli
```

4. Start it **inside the project folder**:

```
cd ~/junsgarden-for-jun
gemini
```

5. The first time, choose **Sign in with Google** and follow the browser steps.
   The free level has daily limits that Google may change.
6. Try these first questions:
   - `Read GEMINI.md and tell me the rules you will follow.`
   - `Explain rust/junsgarden/src/main.rs to me, one part at a time.`
   - `Run nothing. Tell me what lab/09-health-check.sh checks.`

**How to keep it safe:** when Gemini CLI asks "Allow this command to run?" read
the command first. Say **no** to anything you do not understand. Never allow it
to read `.env` or `docker/.ha.env` (tell it: "Never open files named .env").

## Way C. A free Gemini key for the website
1. Open <https://aistudio.google.com/apikey> and create a key.
2. In the garden menu choose `K` (keys), then `1` (Gemini), and paste the key
   (nothing shows while you paste). The menu checks the key with Google, saves it
   in your private settings file and restarts what uses it. No file editing needed.
3. That is all: the menu did the restart.
4. The AI boxes on the website now work, within the free daily limits. If they
   say "busy", wait; free keys are shared and limited.
Keep this key only in `.env`.

## The 4-step help loop
When something goes wrong:
1. **Run the facts script:** `bash lab/10-collect-for-gemini.sh`. It writes a
   file with your lab's state and removes passwords and keys.
2. **Read the file** it names. Delete any line you do not want to share.
3. **Paste it into Gemini** with the "I am stuck" prompt from
   [05](05-gemini-prompts.md). Say which guide and step you were on.
4. **Check the answer** against the guide, run **one** check command, and tell
   Gemini what you saw.

## Good habits that make Gemini much better
- Tell it your goal, not just the error: *"I want my site to open on my phone."*
- Paste the exact text, not a retyped version.
- Ask for the official page: *"Which page of the Docker docs says this?"*
- Ask it to teach: *"Explain it like I am new, then show me the command."*
- Ask for the undo: *"How do I put it back the way it was?"*
- When you finish something, ask it to write 3 sentences for your MY-WINS file.

## If something goes differently
| What you see | What to do |
|---|---|
| Gemini CLI says Node is too old | Step 2. Check `node --version` again in a new terminal. |
| `npm: command not found` | Node is not installed. Do step 2. |
| `EACCES` permission error on npm install | You used `sudo` or a system Node. Use nvm (step 2) so installs go in your own folder. |
| Gemini gives a command that does not work | Paste the error back. Say which Ubuntu version you have (`lsb_release -ds`). |
| Gemini asks for a password or key | Say no. Nothing in this lab needs you to give a secret to Gemini. |

**Next:** [09 Troubleshooting](09-troubleshooting.md)
