# Project 8. Your sensor readings on your own website

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- A small Python script copies your soil sensor's number from Home Assistant into your website's database every 15 minutes, for your **Data** page.
- Hard, about 3 hours, free. Do Projects 3 and 6, and guide 03, first.
- Make a Home Assistant long-lived token and save it in the private file `docker/.ha.env` as `HA_TOKEN=...`.
- Put your sensor's name in `SENSORS` in `docker/extras/ha_to_garden.py`.
- Run it once: `python3 docker/extras/ha_to_garden.py`. You should see `Saved 1 reading(s) at ...`.
- Make it run every 15 minutes with one line in `crontab -e`.
- Finding your number on the Data page is exploring. If it does not show, `rust/junsgarden/src/pages/data.rs` explains which widgets read it.

---

**Level:** hard. **Time:** about 3 hours. **Cost:** free.

## What you will build
A small Python script that copies your soil sensor's number from Home Assistant
into your garden website's database every 15 minutes, so it can show on your
website's **Data** page. This ties your hardware projects to your website.

```
ESP32 sensor --> Home Assistant --script (every 15 min)--> garden database --> Data page
```

## What you need first
- Projects 3 and 6 done, and guide 03.
- Python 3 (Ubuntu already has it: check with `python3 --version`).

## Steps

### Part A. Let the script read Home Assistant
1. In Home Assistant click your **name** (bottom left), **Security** tab, scroll
   to **Long-lived access tokens**, **Create token**, name it `garden script`.
   Home Assistant shows it **once**. Copy it.
2. Save it in a private file (this does not print it). In the project folder:

```
umask 077
nano docker/.ha.env
```

   Put exactly this one line, with your token after the `=`:

```
HA_TOKEN=paste-the-token-here
```

   Save (`Ctrl+O`, `Enter`, `Ctrl+X`). The project's `.gitignore` already
   keeps `*.env` files out of git.

### Part B. Tell the script which sensor
3. Find your sensor's name: HA, **Settings, Devices & Services, Entities**,
   search `moisture`. Copy the name that starts with `sensor.`.
4. Open `docker/extras/ha_to_garden.py` in an editor. Find `SENSORS`. Put your
   sensor's name on the right side.
5. Read the script from the top. Every step has a comment. Ask Gemini
   (prompt 4) to explain any part.

### Part C. Run it once
6. Run:

```
python3 docker/extras/ha_to_garden.py
```

   *You should see:* `Saved 1 reading(s) at ...`.
7. Check the database has it:

```
ROOTPW=$(grep '^DB_ROOT_PASSWORD=' .env | cut -d= -f2-)
docker exec -e MYSQL_PWD="$ROOTPW" garden-db mariadb -uroot -e \
  "SELECT obs_time, location, source, soil_moisture_6in FROM juns_garden_db.weather ORDER BY id DESC LIMIT 3;"
```

   *You should see:* your row at the top.

### Part D. Make it run by itself
8. `crontab -e` and add (change `jun`):

```
*/15 * * * * python3 /home/jun/junsgarden-for-jun/docker/extras/ha_to_garden.py >> /home/jun/junsgarden-for-jun/backups/ha-to-garden.log 2>&1
```

### Part E. Find it on the website
9. Open `http://localhost:8030/data.php` (the Data page). Look for a chart that
   uses soil moisture. *This part is exploring.* The Data page was built for
   The author's weather data and different widgets read different columns.
10. If your numbers do not appear where you expect, that is a real
    programming puzzle. Work it out like this:
    1. Open `rust/junsgarden/src/pages/data.rs` and search for
       `soil_moisture_6in`. The comments explain which widgets read it.
    2. Ask Gemini (prompt 4) to explain how that widget picks its rows (the
       `WHERE` part of the SQL).
    3. If it filters by a `location` or `source` you did not use, change the
       two labels at the top of `ha_to_garden.py` to match.

## How you know it worked
- A new row appears in the database every 15 minutes.
- Your sensor's number shows on your Data page.

## If something goes differently
| What you see | What to do |
|---|---|
| `No HA_TOKEN` | Part A, step 2: the line must be `HA_TOKEN=...` with nothing before it. |
| `could not read ...: HTTP Error 401` | The token was copied wrong or deleted. Make a new one. |
| `HTTP Error 404` | The sensor name is wrong. Copy it from Entities. |
| `No such container: garden-db` | `docker compose up -d` first. |
| Cron seems to do nothing | Look at `backups/ha-to-garden.log`. |

## Make it yours
- Add the air temperature and humidity (uncomment the two example lines).
- Add a new widget to the Data page. Start by reading how an existing one is
  written in `data.rs`. Make the smallest possible copy and change one word.
- Write a README for your script (3 sentences) for your MY-WINS file.

**Next:** [Project 9: More ideas](P09-more-ideas.md)
