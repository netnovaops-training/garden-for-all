# 02. Install Docker

> **You do not need to do this by hand: the install (menu 1) installs Docker for
> you. This guide only explains it.**

## Quick version
*No time to read it all? This is the short version. The full guide is below.*

- This guide installs Docker on your Ubuntu laptop. Docker runs programs in sealed boxes called containers.
- It takes 30 minutes and is free.
- Steps 1 to 3 follow Docker's own page: remove old pieces, add Docker's software source, then install Docker and Compose. If Docker changes their steps, trust their page.
- Step 4: `sudo usermod -aG docker $USER`, then **log out and log back in**. This step is easy to forget.
- Test it: `docker run --rm hello-world`. You should see "Hello from Docker!".
- `permission denied ... docker.sock` means you did not log out and back in after step 4.

---

## What you will build
Docker on your Ubuntu laptop. Docker runs programs in sealed boxes called
**containers**. Every project after this one is "download a container, start it".
It keeps your laptop tidy, and you can delete a project without leaving a mess.

## Time and cost
30 minutes. Free.

## What you need first
- [01 Ubuntu](01-ubuntu-laptop.md) done, and internet working.

## Steps
This follows Docker's own instructions for Ubuntu
(<https://docs.docker.com/engine/install/ubuntu/>). If Docker changes their
steps, their page is the one to trust.

1. Open a terminal. Remove any old Docker pieces (it is fine if it says none
   are installed):

```
sudo apt remove -y docker.io docker-compose docker-compose-v2 docker-doc podman-docker containerd runc
```

2. Add Docker's official software source:

```
sudo apt update
sudo apt install -y ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt update
```

3. Install Docker and Compose:

```
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

4. Let your user run Docker without typing `sudo` every time:

```
sudo usermod -aG docker $USER
```

   Then **log out and log back in** (or restart). This step is easy to forget.

5. Test it:

```
docker run --rm hello-world
```

   *You should see:* "Hello from Docker!" and some text.

6. Test Compose:

```
docker compose version
```

   *You should see:* a version number.

## Six commands you will use all the time
| Command | What it does |
|---|---|
| `docker ps` | Lists containers that are running now |
| `docker ps -a` | Lists all containers, including stopped |
| `docker logs -f NAME` | Shows what a container is saying. `Ctrl+C` to stop watching |
| `docker compose up -d` | Starts everything in the compose file in the current folder |
| `docker compose down` | Stops and removes those containers (your data folders stay) |
| `docker compose pull` | Downloads newer versions |

## How you know it worked
`docker run --rm hello-world` printed the hello message, with no `sudo`.

## If something goes differently
| What you see | What to do |
|---|---|
| `permission denied ... docker.sock` | You did not log out and back in after step 4. Do that. |
| `Unable to locate package docker-ce` | Step 2 did not finish. Run its commands again, one at a time. |
| `E: Could not get lock` | Another update is running. Wait 2 minutes and try again. |
| The `echo "deb ..."` line fails | Ubuntu is very new and Docker may not have it yet. Ask Gemini: "docker.com has no repo for my Ubuntu version, what do I do?" and paste the error. |

## Make it yours
- Run `docker run --rm -it ubuntu bash` to step inside a throwaway Ubuntu. Type `exit` to leave. It is gone forever when you leave.
- Run `docker system df` to see how much space Docker uses. `docker system prune` cleans up unused stuff (it asks first).

**Next:** [03 Run your garden website](03-run-your-garden.md)
