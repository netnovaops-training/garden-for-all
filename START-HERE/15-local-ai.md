# 15. Local AI on a strong computer (optional, advanced)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Optional and advanced: run the AI for the garden's AI boxes on a strong computer at home, with Ollama. It is private and free once set up.
- Not on the garden laptop: it needs a strong graphics card (8 GB of graphics memory or more is comfortable) or an Apple Silicon Mac.
- Install Ollama from <https://ollama.com/download>, then `ollama pull phi4`.
- Let only the garden computer reach it (step 3).
- Connect the garden: menu `M`, then `9`, and type the AI computer's address. To turn it off, run it again and leave the address empty.
- Never forward port 11434 in the router. Local AI is for your home only.
- If the AI computer is off, the AI boxes stay quiet. The rest of the garden is not affected.

---

## What it is
The garden has a few AI boxes (plant health, "discover", the plant admin's AI
tabs). They can use free cloud AI (guide 03, part D), or a **local AI**: an AI
model running on a computer in your home, using **Ollama**. Local AI is
private (nothing leaves your home), free once set up, and works without
internet limits.

**Not on the garden laptop.** AI models need a strong graphics card (GPU) and
lots of memory. An old laptop would be far too slow. Use a desktop with a
recent NVIDIA or AMD graphics card (8 GB of graphics memory or more is
comfortable), or an Apple Silicon Mac.

The old garden server used local AI this way (a model called `phi4`). This
guide puts that option back, if you want it.

## Step 1. Install Ollama on the strong computer
Download it from the official site and install it like any program:
<https://ollama.com/download> (Windows, Mac and Linux).

## Step 2. Download a model
In a terminal (Windows: PowerShell) on that computer:

```
ollama pull phi4
ollama run phi4 "Name three companion plants for tomatoes."
```

It should answer. Smaller and faster: `ollama pull llama3.2`. The list of
models: <https://ollama.com/library>.

## Step 3. Let ONLY the garden computer reach it
By default Ollama only answers on its own computer. To let the garden computer
ask it:
- **Windows:** Settings, System, About, Advanced system settings, Environment
  Variables. Add a user variable `OLLAMA_HOST` = `0.0.0.0`, then quit Ollama
  from the tray and start it again.
  Then allow the garden computer only, in PowerShell **as administrator**
  (put your garden computer's reserved address instead of 192.168.1.50):

```
New-NetFirewallRule -DisplayName "Ollama from garden" -Direction Inbound -Protocol TCP -LocalPort 11434 -RemoteAddress 192.168.1.50 -Action Allow
```

- **Linux:** follow "How do I configure Ollama server?" in the official FAQ:
  <https://docs.ollama.com/faq>

Never forward port 11434 in the router: local AI is for your home only.

Give the AI computer a reserved address too (guide 14), so the garden can always find it.

## Step 4. Connect the garden
On the garden computer: menu `M`, then `9` (Connect a local AI computer). Type
the AI computer's address. It checks the AI computer answers, shows its
models, and asks which one to use. To turn local AI off again, run it again
and leave the address empty.

## Good to know
- If the AI computer is switched off, the AI boxes simply stay quiet. The rest
  of the garden is not affected.
- Cloud AI keys (guide 03 part D) and local AI can both be set. Only free keys,
  never a paid one.

<a id="help-plant-ai"></a>

## Local plant AI: the desktop helper that fills in new plants
A second, separate use of local AI, for the plant admin's **Add plant** tab: you type the
genus, species, name, bed, quantity and price, and a helper on a Windows PC fills in the
rest from the grower's page and trusted garden pages, checks every value, and finds up to
5 photos. You then see what was filled, what needs a look and what is still missing, and
press **Approve** (only the garden owner can).

- **Windows PC only, with an NVIDIA card with 12 GB of graphics memory or more** (like an
  RTX 4070 Ti). The Windows setup wizard offers it (CUSTOM, "Local plant AI") only on such
  a PC, and says why not on any other. Later: the Windows menu, More, 22.
- **The Linux installer (garden.sh) never installs AI models.** This is the one thing the
  Windows and Ubuntu installers do differently: a garden laptop has no graphics card for it.
- What is installed on the PC: Ollama (a background task that starts with Windows and
  answers on that PC only, 127.0.0.1), the models `phi4` (fills in facts) and `gemma4:12b`
  (checks them and the photos), one loaded at a time, Python, and the helper (`plant-ai/`).
- **No new open port anywhere.** The website keeps a queue. The helper asks the website
  for work every minute, through its usual address (outgoing, like a browser), with its
  own key, and only from your home network. The website can run on another computer (a
  laptop); plants added while the PC is off wait in the queue.
- The key: on the website, Add plant, Desktop helper, **Make a helper key** (shown once).
  Paste it into the wizard's Local plant AI page, or the Windows menu, More, 22.
- The AI only fills **empty** fields and never replaces a value; a copy of the plant is
  kept before every change. Pictures are one `.webp` each (500 to 2000 px), no thumbnails.
- What the AI could not find: **Export what's still missing** gives a small file with the
  empty fields, their allowed values and a ready prompt for your own Claude. Bring the
  answer back with Advanced, JSON Pipeline, Import (you see every value first).

## Read more (official)
- Ollama: <https://ollama.com>, documentation: <https://github.com/ollama/ollama/tree/main/docs>
- The model list: <https://ollama.com/library>
