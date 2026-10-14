# 12. Your database (MariaDB, which speaks MySQL)

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- Your garden's database is called `juns_garden_db`. It runs in the container `garden-db` and is never reachable from the network.
- Your plants are in the table `data_source`.
- Look inside in your browser with Adminer: menu `M`, then `8`, then open <http://localhost:8082>. Stop it afterwards with `bash garden.sh dbweb-stop`.
- Or type SQL: `bash garden.sh db`.
- Back up first (menu `6`, then `1`) every time you change data by hand. Made a mistake? Restore that backup (menu `7`).
- Always use `WHERE` on `UPDATE` and `DELETE`. Without it, every row changes.
- Most changes are easier and safer in the plant admin (`/streamlit/`).

---

## What it is
A database is a set of tables, like spreadsheets that programs can search fast.
Your garden's database is called `juns_garden_db`. It runs in the container
`garden-db` and is never reachable from the network: only the garden's own
programs talk to it.

## The main tables

| Table | What it holds |
|---|---|
| `data_source` | Your plants: the catalogue (one row per plant) |
| `future_plants` | The wish list and plant ideas |
| `plant_barcodes` | Barcodes linked to plants (the app's barcode lookup) |
| `flower_bed_qr` | The flower beds and their QR codes |
| `garden_blog` | Blog posts |
| `weather`, `weather_forecast` | Weather readings and forecasts |
| `user_favorites`, `user_likes`, `user_done`, `user_notes`, `user_prefs` | Visitors' likes, favourites and notes |
| `plant_purchase`, `plant_quantity_log` | What was bought, and how many |
| `wiki_draft` | Drafts waiting in the wiki queue |
| `vocab_alias`, `vocab_proposal` | The word lists the tag reader uses |
| `garden_users`, `garden_sessions`, `login_attempts` | Logins. Passwords are stored only as Argon2 hashes. |

The wiki keeps its pages in a second database, `junsgarden_wiki`.

## Look inside: two ways

**The easy way, in your browser (Adminer):** menu `M`, then `8`. Open
<http://localhost:8082> and fill in: System **MySQL**, Server **db**, Username
**dbadmin**, Database **juns_garden_db**. For the password, type
`grep ^DB_PASSWORD= .env` in the garden folder. Stop it afterwards:
`bash garden.sh dbweb-stop`.

**The command line (SQL):** `bash garden.sh db`, then try:

```sql
SHOW TABLES;
SELECT COUNT(*) FROM data_source;
SELECT id, plant_name FROM data_source ORDER BY id DESC LIMIT 10;
exit
```

(If a column name is different, `DESCRIBE data_source;` lists the real ones.)

## Safe habits
1. **Back up first** (menu `6`, then `1`), every time you change data by hand.
2. **Look before you change:** run the `SELECT` with the same `WHERE` first, and
   check it finds only the rows you mean.
3. **Always use `WHERE`** on `UPDATE` and `DELETE`. Without it, every row changes.
4. Most changes are easier and safer through the website's plant admin
   (`/streamlit/`) than by hand.
5. Made a mistake? Restore the backup you just made (menu `7`).

## Changing the shape of a table (advanced)
Adding a column: back up, `ALTER TABLE ... ADD COLUMN ...`, add the same line to
`db/schema.sql` so new installs have it, then change the Rust code that reads it
(guide 11). One change at a time.

## Read more (official)
- MariaDB SQL basics: <https://mariadb.com/kb/en/basic-sql-statements/>
- SELECT: <https://mariadb.com/kb/en/select/>
- Adminer: <https://www.adminer.org>
