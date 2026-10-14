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

## Read more (official)
- Ollama: <https://ollama.com>, documentation: <https://github.com/ollama/ollama/tree/main/docs>
- The model list: <https://ollama.com/library>
