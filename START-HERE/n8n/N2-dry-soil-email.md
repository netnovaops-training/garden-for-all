# n8n 2. An email when the soil is dry

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- When Home Assistant sees dry soil, n8n emails you which bed is dry and how dry it is.
- Medium, 1 hour, free. Do n8n 1 and Project 3 first.
- In n8n: a **Webhook** (POST, path `dry-soil`), then **Send Email**.
- In Home Assistant: add a `rest_command` to `homeassistant-config/configuration.yaml`, then restart Home Assistant.
- Test with a fake value from **Developer Tools, Actions**. You do not need a dry plant.
- Flip the workflow to **Active**, so it uses the Production URL (without `-test`).
- Do not forward port 5678 on your router.

---

**Level:** medium. **Time:** 1 hour. **Cost:** free.

## What you will build
When Home Assistant sees dry soil, it tells n8n, and n8n sends an email to
**you@gmail.com** that says which bed is dry and how dry it is.

```
Home Assistant --"bed 1 is dry"--> n8n webhook --> Send Email --> your inbox
```

A **webhook** is a private web address that n8n listens on. When something
visits it, the workflow starts.

## What you need first
- [n8n 1](N1-install-and-first-email.md) done (n8n runs; the SMTP credential works).
- [Project 3](../projects/P03-home-assistant.md) done (Home Assistant runs).
- If you have no soil sensor yet, you can use any number that changes, or test with
  a fake value (see step 12).

## Steps

### Part A. The n8n side
1. In n8n click **Create workflow**. Name it `Dry soil email`.
2. **Add first step**, search **Webhook**, choose it. Set:
   - HTTP Method: `POST`
   - Path: `dry-soil`
3. Click **Test URL**. Copy the address it shows (it looks like
   `http://localhost:5678/webhook-test/dry-soil`). The test address works only
   while you have clicked **Listen for test event**.
4. Click the **+** after the Webhook, add **Send Email**. Set:
   - Credential: `My Gmail (send)`
   - From and To: `you@gmail.com`
   - Subject (choose **Expression**): `Dry soil: {{ $json.body.bed }}`
   - Text (choose **Expression**):
     `{{ $json.body.bed }} is at {{ $json.body.moisture }} percent.
     Time: {{ $now.toFormat('HH:mm') }}`
5. Click **Save**.

### Part B. The Home Assistant side
6. HA needs a "message sender". Open the HA configuration file. In your project
   folder:

```
nano homeassistant-config/configuration.yaml
```

   Add these lines at the end (keep the spaces exactly):

```yaml
rest_command:
  tell_n8n_dry_soil:
    url: "http://localhost:5678/webhook/dry-soil"
    method: post
    content_type: "application/json"
    payload: '{"bed": "{{ bed }}", "moisture": "{{ moisture }}"}'
```

   Save. Then **Settings, System, Restart Home Assistant** (or
   `docker restart homeassistant`).
   Home Assistant uses the laptop's own network, so `localhost:5678` reaches n8n.
7. In HA open the automation from [Project 6](../projects/P06-soil-sensor.md)
   (soil below 30 for 30 minutes). **Add action**, choose **Call a service**,
   pick `rest_command.tell_n8n_dry_soil`, and in **YAML mode** give:

```yaml
action: rest_command.tell_n8n_dry_soil
data:
  bed: "Bed 1"
  moisture: "{{ states('sensor.garden_bed_1_moisture_percent') }}"
```

### Part C. Test it
8. In n8n open `Dry soil email` and click **Listen for test event**.
9. In HA open **Developer Tools, Actions**, pick `rest_command.tell_n8n_dry_soil`,
   switch to **YAML mode**, paste:

```yaml
action: rest_command.tell_n8n_dry_soil
data:
  bed: "Test bed"
  moisture: "12"
```

   and click **Perform action**. This sends a fake value, so you do not need a
   dry plant to test.
   *You should see:* n8n shows the data, both boxes go green, and the email
   arrives.
10. Make it permanent: in n8n flip the workflow to **Active**. When it is active,
    the **Production URL** (`/webhook/dry-soil`, without `-test`) is used. That is
    the address in the HA settings above.

## How you know it worked
- A test email with "Test bed is at 12 percent".
- After you leave the real sensor in the air for 30 minutes, the real email
  arrives.

## If something goes differently
| What you see | What to do |
|---|---|
| HA says the service does not exist | Restart HA after editing `configuration.yaml`. Check there are no tab characters, only spaces. |
| n8n shows nothing | The workflow is not listening. For the test URL click **Listen for test event** first. For production the workflow must be **Active** and the address must not have `-test`. |
| 404 from the webhook | The path is wrong. It must be `dry-soil` in both places. |
| Email does not arrive | Run n8n 1 again; check the SMTP credential test; look in spam. |

## Safety
- The webhook address is open to anyone who can reach port 5678. n8n is only
  reachable from this laptop, so that is fine. Do not forward port 5678 on your
  router.

## Make it yours
- Add the time since the last watering.
- Add an **IF** box: only send one email per day for the same bed.
- Add a second **Send Email** box to send a copy to someone else.

**Next:** [n8n 3: A weekly garden report](N3-weekly-garden-report.md)
