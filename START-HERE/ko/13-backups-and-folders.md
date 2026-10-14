# 13. 백업, 복원, 폴더 이름

> 영어 원문: [../13-backups-and-folders.md](../13-backups-and-folders.md)

## 짧은 버전
*다 읽을 시간이 없나요? 짧은 버전이에요. 전체 안내서는 아래에 있어요.*

- 백업은 하나하나가 `backups/` 폴더의 `.tar` 파일 하나예요. 이름이 날짜, 시간, 라벨을 알려 줘요.
- 매일 밤 오전 2시 30분에 백업이 저절로 만들어지고, 7일 동안 보관돼요. 내가 직접 만든 백업은 자동으로 지워지지 않아요.
- 백업마다 사진이 전부 들어 있어요(약 800 MB). 그래서 매일 밤 백업 7개에 디스크 공간이 약 6 GB 필요해요.
- 만들기: 메뉴 `6`, 그다음 `1`. 확인하기: `bash garden.sh verify`. 복원하기: 메뉴 `7`(먼저 `before-restore` 백업을 저장해요).
- 사본을 3개 두세요: 정원 컴퓨터에, NAS에(메뉴 `6`, 그다음 `4`), Google Drive에(메뉴 `6`, 그다음 `5`). 두 번째 NAS나 PC의 공유 폴더도 모든 백업을 받을 수 있어요(메뉴 `6`, 그다음 `13`. 네트워크 장소는 3곳까지).
- Windows PC가 매주 사본을 가져와서 내가 고른 폴더에 둘 수 있어요: Windows 메뉴, `M`, 그다음 `13`.
- 같은 노트북에 있는 백업은 노트북이 고장 나면 소용없어요. 가끔 USB 스틱에 하나 복사하세요.
- 파일이나 폴더(사진, 페이지, 백업 등)의 이름을 바꾸거나 옮기려면 메뉴 `M`, 그다음 `18`(또는 메뉴 `8`, 그다음 `8`)을 쓰세요. 그것을 쓰는 모든 곳이 함께 바뀌어요: 안내서 26.
- 사진 폴더 안의 폴더나 사진 파일의 이름은 손으로 바꾸지 마세요: 그 메뉴를 쓰면 웹사이트와 데이터베이스가 따라와요.

---

## 백업 파일: 이름이 모든 것을 알려 줘요

모든 백업은 `backups/` 폴더 안의 파일 '하나'예요:

```
junsgarden-backup_2026-10-13_0230_nightly.tar
                  |          |    |
                  날짜       시간 라벨(있을 수도 없을 수도)
```

| 라벨 | 만든 것 |
|---|---|
| `nightly` | 새벽 2시 30분의 자동 백업. 7일 보관하고, 그다음 자동으로 지워져요. |
| (없음) | 본인이 메뉴 `6`, 그다음 `1`로 만든 것. 자동으로 지워지지 않아요. |
| `before-restore` | 복원이 예전 백업을 되돌리기 바로 전에 만든 것. 안전망이에요. |
| `from-the-author` | zip에 들어 있던 것: 예전 모습 그대로의 정원 전체 |

각 백업 안에 있는 것:

| 조각 | 들어 있는 것 |
|---|---|
| `database.sql.gz` | 정원 데이터베이스: 식물, 댓글, 로그인 |
| `wiki.sql.gz` | 위키 페이지(위키가 설치되어 있을 때) |
| `images.tar` | 모든 사진, 각자의 폴더 안에(`Database/`, `Gallery/`, `post/`, `blog/`, `plan/`, `HeroBanner/`) |
| `data.tar` | 댓글, 방명록, 홈 페이지의 정보 |
| `cashe.tar` | AI가 저장해 둔 답(철자 "cashe"는 일부러예요: 웹사이트가 이 이름을 기대해요) |
| `n8n.tar`, `auth.tar` | n8n 작업 흐름, 그리고 로그인 페이지의 설정과 계정(설치되어 있을 때) |
| `MANIFEST.txt` | 안에 무엇이 있는지, 파일 개수와 함께 |
| `SHA256SUMS` | 모든 조각의 지문(fingerprint). 복원은 무엇이든 바꾸기 전에 이것을 확인해요. |

**크기는요?** 백업마다 사진이 전부 들어 있어요(약 800 MB). 매일 밤 백업 7개를 합치면
디스크 공간이 약 6 GB 필요해요. 직접 만든 백업은 그만큼 더 필요해요(더 이상 필요 없는
내 백업은 `backups/` 폴더에서 지우세요).

## 만들기, 확인하기, 복원하기

![백업 메뉴(메뉴 6)](../images/screen-13-backups.png)

- **만들기:** 메뉴 `6`, 그다음 `1`(Back up now, 지금 백업하기). 1~2분 걸려요.
- **확인하기:** `bash garden.sh verify`가 최신 백업의 지문을 확인하고 MANIFEST를 보여 줘요.
- **복원하기:** 메뉴 `7`. 목록에서 백업을 고르세요. 프로그램은 먼저 지금 있는 것의
  `before-restore` 백업을 만들고, 웹사이트를 멈추고, 모든 것을 되돌려 놓고,
  다시 시작해요.

## 매일 밤 자동 백업(Ubuntu, 설치가 설정해요)
설치가 내 **cron**(Ubuntu의 알람 시계)에 한 줄을 추가해요. 할 일은 없지만, 보고 바꾸는
방법은 이래요:

1. 보기: `crontab -l`. 이렇게 끝나는 줄이 하나 보여야 해요:
   `garden.sh backup nightly >> backups/backup.log 2>&1`.
   맨 앞의 `30 2 * * *`는 "매일 2시 30분에"라는 뜻이에요.
2. 무엇을 했는지 보기: `tail -20 backups/backup.log`.
3. 시간 바꾸기(예: 새벽 1시 15분): `crontab -e`, 물어보면 `nano`를 고르고,
   `30 2`를 `15 1`로 바꾸고, 저장하세요(`Ctrl+O`, `Enter`, `Ctrl+X`).
4. 없어졌나요? 메뉴 `1`(설치)이 다시 넣어 줘요. `bash lab/08-install-nightly-backup.sh`도 같아요.

cron의 공식 안내서: <https://manpages.ubuntu.com/manpages/noble/man5/crontab.5.html>

## Windows PC에 사본 두기(원하면 자동으로)
집에 Windows PC가 있다면, 그 PC가 매주 스스로 최신 백업의 사본을 가져와서, 내가 고른
폴더에 둘 수 있어요.

**쉬운 방법:** Windows 메뉴(`Garden-Menu-Windows.cmd`), `M`, 그다음 `13`.

![Windows 메뉴, More, 13: 매주 사본을 C:\Backups\weekly로 보내기](../images/windows-13-backup-copy.png)

1. 무엇을 하는지 설명한 다음, **사본을 어디에 둘지** 물어봐요:
   - `1` **Pick a folder in a window**(창에서 폴더 고르기). PC가 볼 수 있는 폴더면 다 돼요:
     이 PC의 폴더, USB 드라이브, **Google Drive for desktop**(Drive를 드라이브 문자로,
     흔히 `G:`로 보여 주는 Google의 프로그램), 또는 NAS: 창 왼쪽의 **Network**(네트워크)를
     누르고 NAS를 여세요.
   - `2` **Type it**(직접 입력하기). `C:\Backups\weekly` 같은 폴더, 또는
     `\\192.168.1.20\backups` 같은 네트워크 경로(NAS, 또는 다른 PC가 공유하는 폴더).
     네트워크 경로는 역슬래시 두 개, 주소, 역슬래시 하나, 공유 폴더 이름이에요.
   - `3` **Keep it as it is**(그대로 두기). 처음에는 이 PC의 정원 폴더 안
     `backups\from-garden-computer\`예요.
2. 고른 곳 안에 `junsgarden-backups`라는 폴더를 만들고, 거기에 쓸 수 있는지 확인해요.
   그림에서는 사본이 `C:\Backups\weekly\junsgarden-backups`로 가요. 다음을 위해 그 곳을
   기억해요.
3. 그 폴더가 OneDrive 안에 있으면 경고해요: 사본 8개는 몇 GB나 돼서, OneDrive 무료 5 GB를
   넘어요.
4. **Add the weekly copy now?**(지금 매주 복사를 추가할까요?) `y`를 입력하세요. Windows
   작업 스케줄러(Task Scheduler)에 **Jun's Garden - copy backup to this PC**라는 작업을
   추가해요. 매주 일요일 11:00에 실행되고(일요일을 놓쳤으면 PC가 켜지자마자), 최신 백업을
   거기로 복사하고, 최신 8개를 보관해요. 결과는 매번 `logs\backup-copy.log`에 적어요.
5. **Make the first copy right now?**(지금 첫 사본을 만들까요?) `y`를 누르면 지금 하나를
   복사해서, 잘 되는지 볼 수 있어요.

**NAS는 먼저 한 가지가 필요해요.** 파일 탐색기에서 NAS를 한 번 여세요(주소 줄에 내 NAS
주소로 `\\192.168.1.20`처럼 입력). NAS 사용자 이름과 비밀번호로 로그인하고
**Remember my credentials**(내 자격 증명 기억)에 체크하세요. 그러면 Windows가 로그인을
기억해서, 아무도 보고 있지 않을 때도 매주 복사가 들어갈 수 있어요. 이것을 건너뛰면 메뉴가
거기에 "cannot write"(쓸 수 없어요)라고 하고, 같은 방법을 알려 줘요.

**나중에 바꾸려면:** `13`을 다시 고르세요. 이미 예약되어 있다고 하고
**Change where the copies go?**(사본을 둘 곳을 바꿀까요?)를 물어요. `y`는 위의 세 가지
선택을 다시 물어요. `n`이면 **Remove the weekly copy?**(매주 복사를 지울까요?)로 넘어가요:
`y`는 작업을 지우고(이미 만든 사본은 그대로 남아요), `n`은 모두 그대로 둬요.

- **손으로, 한 번만:** Windows 메뉴, `M`, `10`(백업을 만들고 PC의 정원 폴더 안 `backups`
  폴더로 복사해요).
- **대신 직접 예약하기**(클릭으로 해 보고 싶다면):
  1. Windows 키를 누르고 **Task Scheduler**를 입력해서 여세요.
  2. **Create Basic Task**. Name(이름): `Jun's Garden backup copy`. Trigger(트리거): **Weekly**(매주), 일요일, 11:00.
  3. Action(동작): **Start a program**. Program(프로그램): `powershell.exe`. Arguments(인수, 한 줄로.
     폴더는 정원 폴더가 있는 곳으로 바꾸세요):
     `-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File "C:\Users\YOU\Desktop\junsgarden-for-jun\windows\Garden-Menu.ps1" -Action copy-newest-quiet`
  4. Finish(마침). 작업을 오른쪽 클릭하고 **Run**(실행)을 누른 다음, `logs\backup-copy.log`를 확인하세요.

  Microsoft의 안내서: <https://learn.microsoft.com/windows/win32/taskschd/task-scheduler-start-page>

복사하려면 그 시간에 정원 컴퓨터가 켜져 있어야 하고, Windows 설치가 만든 SSH 키가
필요해요(비밀번호는 어디에도 저장하지 않아요).

## 3-2-1: NAS와 Google Drive
안전한 규칙은 **사본 3개, 저장 장치 2종류, 1개는 집 밖에**예요:

| 사본 | 어디에 | 얼마나 자주 | 보관 |
|---|---|---|---|
| 1 | 정원 컴퓨터의 `backups/` | 매일 밤, 새벽 2시 30분 | 매일 밤 백업 7일치(약 6 GB) |
| 2 | 내 **NAS**(집에 있는 네트워크 드라이브), 그리고 더하면 네트워크 장소 2곳 더 | 백업할 때마다 | 장소마다 최신 14개 |
| 3 | **Google Drive**(집 밖) | 일주일에 한 번 | 최신 4개(약 3.2 GB. 무료 Drive는 15 GB) |

메뉴 `6`(Backups, 백업)에서 2번과 3번을 설정해요. 그다음부터는 백업할 때마다 저절로 돼요.
`logs/offsite.log`에 모든 복사가 날짜와 함께 적혀요(첫 네트워크 장소는 `NAS`, 다른 곳은
`NAS2`와 `NAS3`, Google Drive는 `CLOUD`).

### NAS 연결하기(메뉴 `6`, 그다음 `4`)
먼저 NAS에서 필요한 것(Synology에서는: **Control Panel > Shared
Folder**와 **Control Panel > User**):
- 네트워크 안의 NAS 주소(공유기에서 이 주소도 예약하세요, 안내서 14: [14-reserve-your-ip.md](14-reserve-your-ip.md))
- 백업용 공유 폴더, 예: `backups`
- 그 폴더에 쓸 수 있는 NAS 사용자와 그 비밀번호

NAS 주소를 몰라도 돼요: 메뉴가 **집 네트워크에서 파일 공유 기기를 찾아볼지** 물어봐요
(포트 445를 두드려 보기만 하고, 어디에도 로그인하지 않아요). 찾은 기기를 이름과 함께
보여 줘요. NAS 사용자 이름과 비밀번호를 입력하면, NAS의 **공유 폴더 목록**을 보여 줘서
번호로 고를 수 있어요. SMB 3으로 작동하고, 요즘 NAS가 쓰는 최신 SMB 3.1.1까지 돼요.

그다음 메뉴가:
1. `cifs-utils`와 `smbclient`(Windows 방식 네트워크 폴더, SMB를 쓰는 Ubuntu 프로그램)를 설치하고,
2. NAS 로그인 정보를 `/etc/garden-nas.credentials`에 저장하고(관리자만 읽을 수 있어요),
3. `/etc/fstab`(Ubuntu가 시작할 때 연결하는 드라이브 목록)에 한 줄을 추가해요.
   `nofail`이 있어서 NAS가 꺼져 있어도 노트북은 시작돼요. 예전 파일은
   `/etc/fstab.bak-garden`으로 사본을 남겨요.
4. 공유 폴더를 `/mnt/garden-nas`에 연결하고, 거기에 쓸 수 있는지 확인하고,
5. 최신 백업을 복사해요.

이 NAS가 **네트워크 장소 1**이에요. 두 곳을 더할 수 있어요(아래 "네트워크 장소를 여러 곳에"를
보세요).

**손으로 하기**(메뉴가 무엇을 하는지 배우기 위해):
```
sudo apt install -y cifs-utils
sudo mkdir -p /mnt/garden-nas
sudo nano /etc/garden-nas.credentials        # two lines: username=...  password=...
sudo chmod 600 /etc/garden-nas.credentials
sudo nano /etc/fstab                          # add the line below, then save
//192.168.1.20/backups /mnt/garden-nas cifs credentials=/etc/garden-nas.credentials,uid=1000,gid=1000,iocharset=utf8,_netdev,nofail,x-systemd.automount 0 0
sudo systemctl daemon-reload && sudo mount /mnt/garden-nas
ls /mnt/garden-nas
```
(위 명령의 `#` 뒤 설명: credentials 파일에는 `username=...`과 `password=...` 두 줄을 쓰고,
fstab에는 아래 줄을 추가한 뒤 저장하세요.)
(내 NAS 주소와 폴더를 쓰세요. `uid=1000`은 내 사용자 번호예요: `id -u`가 보여 줘요.)
공식 참고 자료: <https://manpages.ubuntu.com/manpages/noble/man8/mount.cifs.8.html>
그리고 Ubuntu 안내서: <https://manpages.ubuntu.com/manpages/noble/man8/mount.cifs.8.html>

### 네트워크 장소를 여러 곳에(메뉴 `6`, 그다음 `13`)
NAS 하나도 좋아요. 두 곳이면 더 좋아요: 한 기기가 고장 나도 다른 곳에 정원이 남아
있어요. `13`, **Add another network place (a second NAS, or a PC's shared folder)**(네트워크
장소 하나 더하기: 두 번째 NAS, 또는 PC의 공유 폴더)는 모든 백업을 받을 장소를 하나 더 만들어요.
**네트워크 장소**는 집 네트워크에 있는 공유 폴더라면 무엇이든 돼요: 두 번째 NAS, 또는
Windows PC가 공유하는 폴더.

- 장소는 **모두 3곳까지** 둘 수 있어요. 장소 1은 `4`에서 연결한 NAS예요(바뀌지 않아요).
  `13`이 장소 2를, 그다음 장소 3을 더해요.
- 매일 밤 백업이 끝나면 사본이 **장소마다** 가고, 장소마다 최신 14개를 보관해요.
- 질문은 첫 NAS와 같아요(네트워크에서 찾거나 주소 입력, 사용자 이름, 비밀번호, 공유 폴더
  고르기). 새 장소는 `/mnt/garden-nas-2`(또는 `-3`)에 나타나고, 로그인 파일도 따로
  `/etc/garden-nas-2.credentials`에 있어요.
- 3곳이 모두 쓰이고 있으면 `13`이 그렇다고 알려 줘요: 먼저 한 곳을 멈추세요(아래의 `8`).

이 예에서 장소 1은 `192.168.1.20`에 있는 NAS이고, 장소 2는 `192.168.1.30`에 있는 Windows
PC가 공유하는 `GardenCopies`라는 폴더예요:

![장소 2 더하기: 192.168.1.30에 있는 PC의 공유 폴더 GardenCopies](../images/screen-21-nas-add.png)

**Windows PC를 네트워크 장소로 쓰기.** 그 PC에서 폴더를 하나 만들고(예: `GardenCopies`),
오른쪽 클릭해서 **Give access to**(액세스 권한 부여, Windows 11에서는 **Show more
options**(추가 옵션 표시) 안에 있어요), 그다음 **Specific people**(특정 사용자)을 고르고,
사용자를 골라 **Read/Write**(읽기/쓰기) 권한을 주세요. 정원 메뉴에서는 그 PC의 주소와,
그 Windows 계정의 사용자 이름과 비밀번호를 입력하세요. 그 PC는 새벽 2시 30분에 켜져 있어야
해요. 꺼져 있으면 그날 밤 그곳으로 가는 사본은 빠져요(메뉴가 `logs/offsite.log`에
`NAS2 FAILED (not reachable)`라고 적고, 다른 장소들은 그대로 사본을 받아요).
Microsoft의 안내서:
<https://support.microsoft.com/windows/file-sharing-over-a-network-in-windows-b58704b2-f53a-4b82-7bc1-80f9994725bf>

### 한 곳으로 복사 멈추기(메뉴 `6`, 그다음 `8`)
`8`, **Stop copying to the NAS or Google Drive**(NAS나 Google Drive로 복사 멈추기)는
네트워크 장소를 번호와 주소와 함께 보여 주고, 어느 곳을 멈출지 물어봐요. 그 번호를
입력하거나, 아무것도 멈추지 않으려면 `0`을 입력하세요. 그다음 Google Drive가 연결되어
있으면, 그것도 멈출지 물어봐요.

![장소 2 멈추기: 장소 목록, 그다음 그 번호](../images/screen-22-nas-stop.png)

장소를 멈추면 `/etc/fstab`에서 그 줄을 빼고, 그 로그인 파일을 지워요. **이미 거기에 있는
파일은 남아요.** 나중에 다시 쓰려면 `13`으로(장소 1은 `4`로) 다시 더하세요.

제거하기(More, `12`)도 모든 네트워크 장소의 연결을 같은 방법으로 끊어요.

### NAS에 정원 폴더 '전체' 두기(메뉴 `6`, 그다음 `10`)
매일 밤 백업 말고도, 메뉴는 **정원 폴더 전체**(코드, 설정, 사진, 백업)의 사본을 NAS(네트워크 장소 1)의
`whole-garden-folder/`에 둘 수 있어요. 매주 만들고, 최신 2개를 보관해요. 데이터베이스의
작업 파일은 빼요(정원이 돌아가는 동안 계속 바뀌기 때문이에요). 대신 사본 안의 최신
백업에 데이터베이스가 안전하게 들어 있어요. 이 사본에는 설정 파일 `.env`(비밀번호 포함)도
들어 있으니, NAS를 비공개로 두세요. 이 사본으로 새 컴퓨터에 정원을 되살리려면: `.tar`
파일을 그 컴퓨터로 복사하고, 압축을 풀고(`tar -xf FILE`), 메뉴 `1`을 실행하세요.

### 다른 NAS로 바꾸기(메뉴 `6`, 그다음 `9`)
NAS를 옮겼거나 새 NAS를 샀나요(어느 회사 제품이든)? `9`를 고르세요: 예전 NAS의 연결을
깔끔하게 끊고(그 안의 파일은 남아요), 위와 같은 방법으로 새 NAS를 연결하고, 예전 NAS가
아직 켜져 있으면 최신 정원 백업 3개를 새 NAS로 복사할지 물어봐요. 그다음부터는 모든
것이 새 NAS로 가요. (`9`는 네트워크 장소 1을 바꿔요. 장소 2나 3을 바꾸려면 `8`로 멈추고
`13`으로 새 곳을 더하세요.)

아주 오래된 NAS가 SMB 2나 1만 쓴다면 연결되지 않아요: SMB 1은 안전하지 않아서 Ubuntu에서
일부러 꺼 두었어요. 더 새 NAS를 쓰거나 NAS의 소프트웨어를 업데이트하세요.

### Google Drive 연결하기(메뉴 `6`, 그다음 `5`)
**먼저, 내 Google "client ID"(한 번만, 약 10분, 무료).** rclone의 공용 ID는
2026년 중에 없어져서 작동이 멈춘다고 rclone이 직접 알리고 있어요. 그러면 매주 하는
복사도 멈춰요. 메뉴가 단계를 보여 줘요. 단계는
<https://rclone.org/drive/#making-your-own-client-id>에서 왔어요. 짧게 말하면:
<https://console.developers.google.com/>에서 프로젝트를 만들고, **Google Drive
API**를 사용 설정하고, 동의 화면을 설정하고(앱 이름 `rclone`, 대상 **External**,
범위 `https://www.googleapis.com/auth/drive.file`, 테스트 사용자로 나 자신), 유형이
**Desktop app**인 **OAuth client**를 만든 다음, **Publish app**(앱 게시)을 누르세요
(게시하지 않으면 Google이 매주 다시 로그인하라고 해요). 메뉴가 물으면 client ID와
client secret을 붙여 넣으세요(secret은 입력하는 동안 보이지 않아요). 아무것도 넣지
않고 Enter를 누르면 지금은 rclone의 공용 ID를 써요.

메뉴가 Ubuntu의 앱 스토어에서 **rclone**을 설치한 다음, 브라우저에 Google 로그인
페이지를 열어요: 내 계정을 고르고 **Allow**(허용)를 누르세요. rclone은 **자기가 만든**
파일에 대한 권한만 요청해요(`drive.file`). 그래서 Drive의 나머지는 볼 수 없어요.
Drive에 `JunsGarden-backups`라는 폴더가 생겨요. rclone의 설정은
`~/.config/rclone/rclone.conf`에 있어요(본인만 읽을 수 있어요).

정원 컴퓨터에 브라우저가 없나요(SSH로 작업 중)? rclone이 브라우저가 있는 컴퓨터에서
`rclone authorize "drive"`를 실행하고, 나온 내용을 다시 붙여 넣으라고 알려 줘요.
공식 안내서: <https://rclone.org/drive/>와 <https://rclone.org/remote_setup/>

### 또는 NAS가 Google Drive로 보내게 하기
많은 NAS는 폴더를 직접 Google Drive로 복사할 수 있어요. Synology에서는 앱 이름이
**Cloud Sync**(Package Center)예요: Google Drive를 추가하고, NAS 폴더
`backups/junsgarden`을 고르고, 방향은 "Upload local changes only"(로컬 변경만 올리기)로 하세요.
그러면 정원 컴퓨터는 NAS 사본만 만들면 돼요. Synology 도움말: <https://kb.synology.com>
("Cloud Sync"로 검색).

### 한 달에 한 번 사본 확인하기
- `tail logs/offsite.log`가 최근 복사를 보여 줘요(OK 또는 FAILED).
- NAS 폴더마다, 그리고 Drive 폴더를 열고 이번 주 파일이 있는지 보세요.
- NAS 사본으로 복원을 연습해 보세요: 파일을 `backups/`로 복사하고 메뉴 `7`을 쓰세요.

## 다른 곳에 사본 두기
같은 노트북 안의 백업은 실수는 막아 주지만, 노트북이 고장 나는 것은 막지 못해요.
가끔 최신 백업을 USB 메모리에 복사하세요: Files(파일)를 열고 `backups` 폴더로 가서,
최신 `.tar` 파일을 USB 메모리로 끌어다 놓으세요. Windows에서는 Windows 메뉴 `M`(More, 더 보기),
그다음 `10`이 백업을 만들고 PC로 복사해 줘요.

## 새 컴퓨터로 옮기기
1. 새 컴퓨터에 정원을 설치하세요(zip에 들어 있던 백업을 불러와요).
2. 최신 백업을 새 컴퓨터의 `backups/` 폴더로 복사하세요.
3. 메뉴 `7`, 그 백업을 고르세요.
4. 설정 파일(`.env`)은 일부러 백업에 들어가지 않아요: 키와 비밀번호가 들어 있으니까요.
   그래서 새 컴퓨터에서 키를 다시 붙여 넣고(메뉴 `K`), 이메일을 다시 설정하고(메뉴 `8`, 그다음 `3`),
   수정할 수 있는 사람을 다시 추가하세요(메뉴 `U`, 그다음 `2`). 로그인 페이지의 손님 계정은
   백업과 함께 옮겨져요.

백업은 각 조각을 폴더 이름이 아니라 하는 일(이미지, 데이터, ...)로 저장해요. 그래서
새 컴퓨터에서 폴더 이름이 달라도 잘 복원돼요.

## 폴더 이름 바꾸기 또는 옮기기
정원이 쓰는 모든 폴더는 '한곳'에서 정해지고, **설정**(Settings)이 대신 바꿔 줘요:
메뉴 `8`, 그다음 `8`(Rename or move a file or folder, 파일이나 폴더 이름 바꾸기 또는 옮기기),
그다음 `2`. 정원을 멈추고, 폴더를 옮기고(다른 디스크로도), 새 위치를 저장하고, 다시
시작해요. 그다음 점검표가 모든 폴더를 찾는지 확인해요. 안내서 26에 모든 단계가 있어요.

| 설정 | 기본값 | 들어 있는 것 |
|---|---|---|
| `SITE_DIR` | `./site` | 웹사이트의 스타일, 스크립트, 앱 다운로드 |
| `IMAGES_DIR` | `./local/images` | 내 사진 |
| `DATA_DIR` | `./local/data` | 댓글과 방명록 |
| `CACHE_DIR` | `./local/cashe` | AI 캐시 |
| `DB_DIR` | `./local/db` | 데이터베이스 자체 파일 |
| `N8N_DIR` | `./local/n8n` | n8n |
| `AUTH_DIR` | `./local/authelia` | 로그인 페이지 |
| `BACKUP_DIR` | `./backups` | 백업 |

**이것들은 손으로 이름을 바꾸지 마세요**(웹사이트가 잃어버려요):
- **사진 폴더 '안의' 폴더들**(`Database`, `Gallery`, `post`, `blog`,
  `plan`, `HeroBanner`). 웹사이트는 정확히 그 이름을 찾아요.
- **사진 파일 이름.** 데이터베이스는 각 사진을 이름으로 기억해요.

내 이름을 붙이려면 메뉴 `M`, 그다음 `18`, 그다음 `1`을 쓰세요: 폴더나 파일의 이름을
바꾸고, 그 이름을 쓰는 모든 곳(코드, 페이지, 사진 목록, 데이터베이스)도 바꿔요. 먼저
백업도 해요. 안내서에는 계속 처음 이름이 나와요. `logs/renames.tsv`에 지금 무엇이
무엇으로 불리는지 적혀 있어요. 안내서 26.

**정원 폴더 전체 옮기기**(예: 더 큰 디스크로): 메뉴 `M`, 그다음 `18`, 그다음 `3`.
정원을 멈추고, 옮기고, 파일 수를 세고, 폴더의 전체 경로를 기억하는 바탕화면 아이콘,
매일 밤 백업 줄, Docker의 프로그램을 고쳐 줘요. 안내서 26.

## 더 읽기
- tar(압축 파일 형식): <https://www.gnu.org/software/tar/manual/>
- SHA-256 지문: <https://www.gnu.org/software/coreutils/manual/html_node/sha2-utilities.html>
- 3-2-1 백업 규칙: <https://www.backblaze.com/blog/the-3-2-1-backup-strategy/>
