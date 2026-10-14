# n8n 3. A weekly garden report by email

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Every Sunday at 8 am, n8n emails you a short report: how many plants your website has, and the current soil moisture.
- Medium, 1 hour, free. Do n8n 1 first, with the garden and Home Assistant running.
- Make a Home Assistant long-lived token. Store it in n8n as a **Header Auth** credential (`Bearer `, then the token). Treat it like a password.
- The workflow: **Schedule Trigger** (Weeks, Sunday, 8:00), two **HTTP Request** boxes (the garden, then Home Assistant), then **Send Email**.
- Click **Execute workflow** once to test, then **Save** and switch it to **Active**.
- No email on Sunday? The workflow must be **Active**. Look in **Executions** for errors.

---

**Level:** medium. **Time:** 1 hour. **Cost:** free.

## What you will build
Every Sunday at 8 am n8n asks your website how many plants it has, asks Home
Assistant for the current soil moisture, and emails you a short report at
**you@gmail.com**.

```
Schedule --> ask the garden --> ask Home Assistant --> build a message --> email
```

## What you need first
- [n8n 1](N1-install-and-first-email.md) done.
- The garden website running (guide 03) and Home Assistant running (Project 3).
- A Home Assistant **long-lived token**: HA, click your name (bottom left),
  **Security**, **Long-lived access tokens**, **Create token**. Copy it once.
  The token lets n8n read Home Assistant. Treat it like a password.

## Steps

### Part A. Give n8n the Home Assistant token safely
1. In n8n: **Credentials, Create credential**, search **Header Auth**.
2. Name: `Home Assistant token`. Header Name: `Authorization`. Value:
   `Bearer ` followed by your token (the word Bearer, a space, then the token).
3. Save. The token is now stored inside n8n, not in the workflow.

### Part B. The workflow
4. **Create workflow**, name it `Weekly garden report`.
5. **Add first step: Schedule Trigger.** Trigger interval: **Weeks**. Day: Sunday.
   Hour: 8. Minute: 0. (Uses the time zone from `docker/extras/n8n.yml`.)
6. Add **HTTP Request**. Name it `Ask the garden`. Set:
   - Method: GET
   - URL: `http://127.0.0.1:8030/healthz`
   This asks the website for its plant count. (n8n runs on the laptop's own
   network, so `127.0.0.1` means "this laptop" and reaches your garden.)
7. Add another **HTTP Request**. Name it `Ask Home Assistant`. Set:
   - Method: GET
   - URL: `http://127.0.0.1:8123/api/states/sensor.garden_bed_1_moisture_percent`
   - Authentication: **Generic Credential Type**, Generic Auth Type: **Header
     Auth**, credential: `Home Assistant token`
8. Add **Send Email**. Credential `My Gmail (send)`; From and To
   `you@gmail.com`; Subject: `Your weekly garden report`.
   Text (choose **Expression**):

```
Good morning!

Plants in the catalogue: {{ $('Ask the garden').item.json.plant_count }}
Bed 1 soil moisture: {{ $('Ask Home Assistant').item.json.state }} percent

Have a good week in the garden.
```

9. Click **Execute workflow** once to test. *You should see:* green ticks and an
   email.
10. **Save** and switch the workflow to **Active**.

## How you know it worked
The test email arrives. The next Sunday at 8 am another one arrives by itself.

## If something goes differently
| What you see | What to do |
|---|---|
| `ECONNREFUSED` for port 8030 | The garden is not running. `docker compose up -d` in the project folder, then `curl http://127.0.0.1:8030/healthz`. |
| `ECONNREFUSED` for port 8123 | Home Assistant is not running: `bash lab/07-start-extras.sh ha`. |
| n8n will not open at `localhost:5678` | `docker logs n8n` shows why. Check that no other program uses port 5678: `sudo ss -ltnp` (look for :5678). |
| 401 from Home Assistant | The token is wrong or has no `Bearer ` in front. Make a new token. |
| `state` is `unavailable` | The sensor is offline. Check Project 6. |
| No email on Sunday | The workflow must be **Active**. Check **Executions** (left menu) for errors. |

## Safety and best practice
- The token is in n8n's credentials, not in the workflow text. If you export or
  share a workflow, credentials are not included. Check anyway before sharing.
- Back up `n8n-data/` with the rest of your lab.

## Make it yours
- Add the weather forecast from Project 5 to the email.
- Add the last watering time and how many minutes the valve ran.
- Send the report to a second person too.
- Use an **Email Trigger (IMAP)** box (host `imap.gmail.com`, port 993, SSL on,
  same app password in a new IMAP credential) to react to emails you send
  yourself, for example a subject of `water bed 1`. Ask Gemini to plan it first,
  and add a safe limit so a stranger's email cannot start the water.

Back to [the plan](../10-the-plan.md).
