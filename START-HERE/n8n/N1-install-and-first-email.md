# n8n 1. Install n8n and send yourself an email

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- n8n (say "n-eight-n") in Docker, connected to your own Gmail, and a first workflow that emails you when you click a button.
- Easy, 45 minutes, free. Do guide 02 (Docker) first.
- Start it: `docker compose -f docker/extras/n8n.yml up -d`, then open <http://localhost:5678> and make an owner account with a **new** password.
- Make a Gmail app password (2-Step Verification must be on). Google shows it only once.
- In n8n, make an SMTP credential: host `smtp.gmail.com`, port `465`, SSL/TLS on.
- Make a workflow: **Trigger manually**, then **Send Email**. Click **Execute workflow** and check your inbox (and spam).
- Never paste the app password into a chat, a document or a screenshot. If it leaks, delete it in your Google account and make a new one.

---

**Level:** easy. **Time:** 45 minutes. **Cost:** free.

## What you will build
n8n (say "n-eight-n") in Docker, connected to your own Gmail address
(**you@gmail.com**), and a first workflow that sends you an email when you
click a button.

A **workflow** is a set of boxes ("nodes") joined by arrows. The first box says
*when* to start. The next boxes say *what to do*. Nothing needs code.

## What you need first
- Guide 02 (Docker) done.
- Access to your own Google account (your Gmail). Only you do this part.
  Nobody else needs your Google password, and nobody else should have it.

## Steps

### Part A. Start n8n
1. From the `junsgarden-for-jun` folder:

```
docker compose -f docker/extras/n8n.yml up -d
```

   (Or: `bash lab/07-start-extras.sh n8n`.)
   *You should see:* `Container n8n Started`. The first start downloads about
   a gigabyte, so wait a few minutes. n8n listens on this laptop only.
2. Open **http://localhost:5678**.
   *You should see:* a screen asking you to set up an owner account.
3. Make the account. Use your email and a **new** password. Save it in your
   password manager. (This account is only for n8n. It is not your Google
   password.)

### Part B. Make a Gmail "app password"
Gmail does not let other programs use your real password. Instead you make a
separate password that works only for this one purpose, and you can delete it
any time.
4. Open your Google account: <https://myaccount.google.com> and go to **Security**.
5. Turn on **2-Step Verification** if it is not on. App passwords exist only
   when it is on.
6. Open <https://myaccount.google.com/apppasswords> (or search "App passwords"
   inside Security). Name it `n8n` and click **Create**.
7. Google shows 16 letters **once**. Copy them now and paste them into your
   password manager with the note "n8n app password". You do not need the spaces.

> **Never paste the app password into a chat, a document or a screenshot.**
> It goes only into n8n's credential box in the next part. If it leaks, delete
> it in your Google account and make a new one.

### Part C. Tell n8n about your email
8. In n8n click **Credentials** (left side), **Create credential**, search
   **SMTP**, and choose it. (SMTP is the standard way to send email.)
9. Fill in:

   | Box | Value |
   |---|---|
   | User | `you@gmail.com` |
   | Password | the app password from step 7 |
   | Host | `smtp.gmail.com` |
   | Port | `465` |
   | SSL/TLS | **on** |

10. Name it `My Gmail (send)` and click **Save**. Click **Test** if shown.
    *You should see:* "Connection tested successfully".

### Part D. Your first workflow
11. Click **Create workflow**. Name it `First email`.
12. Click **Add first step**, choose **Trigger manually** (a button you click).
13. Click the **+** after it, search **Send Email**, and choose it. Set:
    - Credential: `My Gmail (send)`
    - From Email: `you@gmail.com`
    - To Email: `you@gmail.com`
    - Subject: `Hello from my garden lab`
    - Email Format: Text
    - Text: `This email was sent by my own n8n in Docker.`
14. Click **Execute workflow** (bottom).
    *You should see:* green ticks on both boxes.
15. Open your Gmail. *You should see:* the email. (Check spam if it is not in the inbox.)
16. Click **Save**, then flip the **Inactive/Active** switch at the top right to
    **Active** later when you have a schedule.

## How you know it worked
The email arrived in your inbox, sent by your own computer.

## If something goes differently
| What you see | What to do |
|---|---|
| `Invalid login` / `535` | The app password was typed wrong, or 2-Step Verification is off. Make a new app password. |
| `ETIMEDOUT` or `ECONNREFUSED` | Your network blocks port 465. Try port `587` with SSL/TLS **off** (n8n then upgrades the connection to secure by itself). |
| Cannot open `localhost:5678` | `docker ps` should list `n8n`. If not: run step 1 again. `docker logs n8n` shows why. |
| Email went to spam | Mark "Not spam". Sending to yourself from yourself is usually fine after that. |

## Make it yours
- Change the subject to include the date. Click the **Subject** box, choose
  **Expression**, and type `Garden check {{ $now.toFormat('yyyy-LL-dd') }}`.
  The double braces mean "work this out first".
- Add a second recipient.

**Next:** [n8n 2: A dry-soil email](N2-dry-soil-email.md)
