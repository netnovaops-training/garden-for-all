#!/usr/bin/env python3
"""
ha_to_garden.py: copy sensor readings from Home Assistant into your garden
website's database, so they show on your own Data page.

Run it by hand to test:      python3 docker/extras/ha_to_garden.py
Run it every 15 minutes:     see START-HERE/projects/P08-readings-on-your-website.md

What it does, in order:
  1. reads your Home Assistant "long-lived access token" from the file
     docker/.ha.env (that file is private and is NOT in the project)
  2. asks Home Assistant for the current value of each sensor in SENSORS below
  3. checks each value is really a number
  4. writes ONE row into the `weather` table of the garden database

Only the standard Python library is used, so nothing needs installing.
"""

import json
import os
import subprocess
import sys
import urllib.request
from datetime import datetime

# ----- Settings you may change ------------------------------------------------
HA_URL = "http://localhost:8123"

# Left: the column in the garden's `weather` table.
# Right: the Home Assistant sensor name (find yours in Settings, Entities).
SENSORS = {
    "soil_moisture_6in": "sensor.garden_bed_1_moisture_percent",
    # "temperature": "sensor.garden_bed_1_air_temperature",
    # "humidity": "sensor.garden_bed_1_air_humidity",
}

# These two labels say where the row came from. `source` 'live' is the same
# label the site's own collector uses, so the Data page includes your rows.
SOURCE = "live"
LOCATION = "my_garden"
# ------------------------------------------------------------------------------

PROJECT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def read_env_file(path):
    """Read KEY=value lines from a private settings file into a dictionary."""
    values = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                key, value = line.split("=", 1)
                values[key.strip()] = value.strip()
    return values


def ha_state(token, entity):
    """Ask Home Assistant for one sensor's value. Returns a float or None."""
    request = urllib.request.Request(
        f"{HA_URL}/api/states/{entity}",
        headers={"Authorization": f"Bearer {token}"},
    )
    try:
        with urllib.request.urlopen(request, timeout=10) as reply:
            state = json.load(reply)["state"]
        return float(state)  # "unavailable" or "unknown" raise ValueError
    except (ValueError, KeyError, OSError) as error:
        print(f"  could not read {entity}: {error}", file=sys.stderr)
        return None


def main():
    """Read the sensors and store one row. Exit code 1 if nothing was stored."""
    ha_env_path = os.path.join(PROJECT, "docker", ".ha.env")
    token = read_env_file(ha_env_path).get("HA_TOKEN", "")
    if not token:
        sys.exit("No HA_TOKEN in docker/.ha.env. See the Project 8 guide, Part A.")

    readings = {}
    for column, entity in SENSORS.items():
        value = ha_state(token, entity)
        if value is not None:
            readings[column] = value
    if not readings:
        sys.exit("No readings were available, so nothing was saved.")

    # Column names come from this file (not from outside), and values are
    # numbers already checked above, so building the SQL text like this is safe.
    columns = ", ".join(f"`{c}`" for c in readings)
    values = ", ".join(repr(v) for v in readings.values())
    now = datetime.now().strftime("%Y-%m-%d %H:%M:00")
    sql = (
        f"INSERT INTO juns_garden_db.weather (obs_time, location, source, {columns}) "
        f"VALUES ('{now}', '{LOCATION}', '{SOURCE}', {values}) "
        f"ON DUPLICATE KEY UPDATE "
        + ", ".join(f"`{c}`=VALUES(`{c}`)" for c in readings)
        + ";"
    )

    root_password = read_env_file(os.path.join(PROJECT, ".env"))["DB_ROOT_PASSWORD"]
    subprocess.run(
        ["docker", "exec", "-i", "-e", f"MYSQL_PWD={root_password}",
         "garden-db", "mariadb", "-uroot"],
        input=sql.encode(),
        check=True,
    )
    print(f"Saved {len(readings)} reading(s) at {now}: {readings}")


if __name__ == "__main__":
    main()
