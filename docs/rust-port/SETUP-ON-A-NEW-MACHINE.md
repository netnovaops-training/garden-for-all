# Running Jun's Garden on your own machine

For Jun. This puts the whole site on one Linux box (a Raspberry Pi 4/5 or any
laptop running Linux). Three containers: the database, the site, and a web
server in front. Plain words, no secrets in this file.

## 1. What you need

- A Linux machine with Docker installed (`curl -fsSL https://get.docker.com | sh`).
- The site's files: the `www` folder (pages, style, scripts, PDFs) and the
  `images` folder (781 MB of photos). The author has both; copy them to
  `/srv/garden/www` and `/srv/garden/images`.
- A database backup (`database-all.sql.gz`, the author has it) **or** start empty
  with `db/schema.sql` from this repository.
- This repository (branch `rust-faithful`).

## 2. The database

```
docker run -d --name garden-db --restart unless-stopped \
  -e MARIADB_ROOT_PASSWORD=choose-a-long-password \
  -e MARIADB_DATABASE=juns_garden_db \
  -e MARIADB_USER=dbadmin -e MARIADB_PASSWORD=choose-another-password \
  -v /srv/garden/db:/var/lib/mysql mariadb:11
```

Load the backup (or the empty structure):

```
zcat database-all.sql.gz | docker exec -i garden-db sh -c 'mariadb -uroot -p"$MARIADB_ROOT_PASSWORD"'
# or, for an empty start:
docker exec -i garden-db sh -c 'mariadb -uroot -p"$MARIADB_ROOT_PASSWORD"' < db/schema.sql
```

## 3. The site

Build it once (takes a few minutes the first time):

```
docker build --build-arg APP=junsgarden -t junsgarden/rs rust/
```

First make the settings file for the email (section 4 says what goes in it):
`/srv/garden/garden.env`. It holds a password, so make it readable only by
you (`chmod 600 /srv/garden/garden.env`) and never put it in the repository.

Run it. Every setting is an environment variable:

```
docker run -d --name garden-site --restart unless-stopped \
  --link garden-db \
  --env-file /srv/garden/garden.env \
  -e JUNSGARDEN_DATABASE_URL='mysql://dbadmin:choose-another-password@garden-db:3306/juns_garden_db' \
  -e APP_PORT=8030 -e TZ=America/New_York \
  -e JUNSGARDEN_USERS='Jun=your.email@example.com' \
  -e JUNSGARDEN_PUBLIC_URL='https://garden.example.com' \
  -e JUNSGARDEN_UPLOADERS='Jun' \
  -v /srv/garden/www:/www \
  -v /srv/garden/images:/www/images \
  -p 127.0.0.1:8030:8030 junsgarden/rs
```

| Setting | What it is |
|---|---|
| `JUNSGARDEN_DATABASE_URL` | how to reach the database |
| `JUNSGARDEN_WWW_DIR` | the site folder inside the container (default `/www`) |
| `JUNSGARDEN_DATA_DIR` | where the guestbook and comments are written (default `/www/data`) |
| `JUNSGARDEN_USERS` | `Name=email,Name=email`: the only accounts that can log in and edit, and the address each one's password links go to. Default: Jun's and the author's. Nobody can make an account from the website |
| `JUNSGARDEN_PUBLIC_URL` | the site's own address, put into the password links (default `https://junsgarden.zapto.org`, so change it on your machine) |
| `JUNSGARDEN_SMTP_HOST`, `JUNSGARDEN_SMTP_PORT`, `JUNSGARDEN_SMTP_USER`, `JUNSGARDEN_SMTP_PASSWORD`, `JUNSGARDEN_MAIL_FROM` | how the site sends the password emails (section 4); put them in `garden.env`, not on the command line |
| `JUNSGARDEN_UPLOADERS` | names (from the list above) who may upload plant photos (default `the author,Joshua,Jun`) |
| `JUNSGARDEN_AI_CACHE_DIR` | where the AI readings are cached (default `/www/cashe/AI`) |
| `OLLAMA_URL`, `OLLAMA_MODEL` | the local AI model, if you run one; leave unset otherwise and the AI boxes simply say they are offline |
| `JUNSGARDEN_FORECAST_FILE`, `JUNSGARDEN_WEATHER_LATEST` | the forecast and latest-reading files the Data page reads, if you have them |

The container writes photos into `/www/images` and the guestbook into
`/www/data`, so those must be writable by the container's user (`chmod -R a+rwX`
on the two folders is the blunt way).

Check it: `curl http://127.0.0.1:8030/healthz` should say `{"status":"ok","plant_count":...}`.

## 4. The web server, the email and the login

Since 2026-10-06 the garden has its own login (username and password), so
you do **not** need a household sign-in, SSO or "forward auth" service any
more. Three steps.

### 4a. The web server

Put Caddy (or any reverse proxy) in front, with HTTPS. The site itself is
public; nothing special is needed for the login. The Caddy block is just:

```
garden.example.com {
    reverse_proxy 127.0.0.1:8030
}
```

### 4b. The email settings (for password links)

The site emails a link when someone uses "Forgot password? / First time?".
For that it needs an email account to send from. With Gmail:

1. Sign in to the Google account you want to send from, and open its
   **Security** settings.
2. Turn on **2-Step Verification** if it is not on yet. App passwords only
   exist when it is on.
3. In the same Security settings, find **App passwords** and make one (name
   it "garden"). Google shows it once. This is **not** your normal Gmail
   password; it is a separate password just for the site.
4. Put these lines in `/srv/garden/garden.env` (from section 3), with your
   own Gmail address and the app password from step 3 (typed without the
   spaces Google shows between the groups, and with no quotes):

```
JUNSGARDEN_SMTP_HOST=smtp.gmail.com
JUNSGARDEN_SMTP_PORT=587
JUNSGARDEN_SMTP_USER=your.address@gmail.com
JUNSGARDEN_SMTP_PASSWORD=paste-the-app-password-here
JUNSGARDEN_MAIL_FROM=your.address@gmail.com
```

Port 587 means the connection is upgraded to an encrypted one (STARTTLS)
before anything is sent. Keep this file out of the repository, and never
paste the app password anywhere else. If the mail settings are missing, the
site still runs; the password email simply is not sent, and the log says
"no outbox configured".

Also check that `JUNSGARDEN_PUBLIC_URL` is your site's own address, because
that is the address put into the emailed links.

### 4c. Start the site and set the passwords

1. Start (or restart) the site with the `docker run` line in section 3.
2. Each person opens `https://garden.example.com/login.php`, clicks
   **"Forgot password? / First time?"**, and types their name or email.
   There are no passwords until each person does this.
3. A one-time link arrives at the email address in `JUNSGARDEN_USERS`. It
   works for 30 minutes, and only once. Open it and choose a password of at
   least 10 characters.
4. From then on, log in with the login box on the home page or the
   catalogue page (Username, Password, Login).

The same "Forgot password?" link is how you reset a forgotten password
later. Setting a new password logs that person out everywhere.

## 5. Daily things

- **New photos in the gallery:** copy them into `images/Gallery/` and run
  `python3 images/Gallery/build_gallery_cache.py` so the gallery sees them.
- **Backups:** `docker exec garden-db sh -c 'mariadb-dump -uroot -p"$MARIADB_ROOT_PASSWORD" --all-databases' | gzip > garden-$(date +%F).sql.gz`
  plus a copy of the `images` folder. Keep them somewhere else.
- **Updating the site:** pull the repository, rebuild with the `docker build`
  line above, then `docker rm -f garden-site` and run it again.

## 6. What is not on your machine

- The weather collector (writes the `weather` table every 30 minutes) is a
  separate small Python container on the author's NUC (`iot/weather_collector.py`).
  Without it the Data page's weather widgets show the history they have and
  the Air & Temperature box shows its sample numbers.
- The Garden Orchestrator, wiki, bed planner and tag scanner are separate
  services, not part of this site.
