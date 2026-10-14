<#
===============================================================================
 Garden-Setup.ps1: the GUIDED installer for Windows (windows and buttons).
 Start it by double-clicking  Start-Here-Windows.cmd  (one folder up).

 WHAT IT DOES, page by page:
   1. Welcome: what this is, with links to read first.
   2. Where: which computer will run the garden (another computer by its IP
      address, or this PC's built-in Ubuntu), with a "Test" button.
   3. About you: your name and email for the garden's login.
   4. What gets installed: every program with a link to where it comes from.
      You tick the optional parts you want.
   5. Install: opens a black window that runs the real install (the same
      garden.sh as on Ubuntu). It explains each step there, and asks for
      passwords there (hidden). This page shows the steps as they finish.
   6. Done: open your garden, or open the menu for everyday jobs.

 This file does not install anything by itself. Everything happens in the
 black window, through Garden-Menu.ps1 and garden.sh, which you can read.
 It never asks for, saves or shows a password.
===============================================================================
#>
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[System.Windows.Forms.Application]::EnableVisualStyles()

$Root = Split-Path -Parent $PSScriptRoot          # the garden folder
$Menu = Join-Path $PSScriptRoot "Garden-Menu.ps1"  # the program that does the work
$Progress = $null                                   # where the install writes its steps

# ---------- first: not inside OneDrive -------------------------------------------
# OneDrive uploads everything in its folders to the internet, and its free plan is
# 5 GB. The garden's downloads and backup copies grow past that. If the garden
# folder is inside OneDrive, offer to move it first. Nothing moves without a click,
# and the old folder is never deleted (you delete it yourself afterwards).
function OneDrive-Roots { @($env:OneDrive, $env:OneDriveCommercial, $env:OneDriveConsumer) | Where-Object { $_ } | Select-Object -Unique }
function In-OneDrive($path) {
    foreach ($od in (OneDrive-Roots)) { if ($path.StartsWith($od.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) { return $true } }
    return $false
}

# The little window with three buttons. Returns the form; its Tag holds the choice.
function New-OneDrive-Choice($from, $suggest) {
    $f = New-Object Windows.Forms.Form
    $f.Text = "Garden Bridge: a better place for the garden folder"
    $f.Size = New-Object Drawing.Size(640, 330); $f.StartPosition = "CenterScreen"
    $f.FormBorderStyle = "FixedDialog"; $f.MaximizeBox = $false; $f.MinimizeBox = $false
    $f.Font = New-Object Drawing.Font("Segoe UI", 10.5)
    $l = New-Object Windows.Forms.Label
    $l.Location = New-Object Drawing.Point(20, 16); $l.Size = New-Object Drawing.Size(590, 150)
    $l.Text = "Your garden folder is inside OneDrive:`n$from`n`n" +
        "OneDrive uploads everything in it to the internet, and its free plan is 5 GB. " +
        "The garden's downloads (about 900 MB) and its backup copies grow past that.`n`n" +
        "Move it to a folder OneDrive does not upload? The installer then opens again from there."
    $f.Controls.Add($l)
    $y = 172
    foreach ($b in @(@("move", "Move it to $suggest  (recommended)"), @("pick", "Choose another place..."), @("keep", "Keep it in OneDrive"))) {
        $btn = New-Object Windows.Forms.Button
        $btn.Text = $b[1]; $btn.Tag = $b[0]
        $btn.Location = New-Object Drawing.Point(20, $y); $btn.Size = New-Object Drawing.Size(590, 32)
        $btn.Add_Click({ $this.FindForm().Tag = $this.Tag; $this.FindForm().Close() })
        $f.Controls.Add($btn); $y += 38
    }
    $f.Tag = "keep"
    return $f
}

# Copy the garden folder to $to and check every file arrived. Also moves the photos
# and wiki zips out of a OneDrive Desktop into it. Returns $true when it is all there.
function Move-Garden-Folder($from, $to) {
    if ((Test-Path $to) -and (Get-ChildItem $to -Force -ErrorAction SilentlyContinue | Select-Object -First 1)) {
        [Windows.Forms.MessageBox]::Show("There is already something in`n$to`n`nNothing was moved. Choose another place, or empty that folder first.", "Garden Bridge") | Out-Null
        return $false
    }
    & robocopy.exe $from $to /E /R:1 /W:1 /NFL /NDL /NJH /NJS /NP | Out-Null
    if ($LASTEXITCODE -ge 8) { [Windows.Forms.MessageBox]::Show("The copy did not finish (robocopy code $LASTEXITCODE). Nothing was changed in the old folder.", "Garden Bridge") | Out-Null; return $false }
    $n1 = @(Get-ChildItem $from -Recurse -File -Force).Count; $n2 = @(Get-ChildItem $to -Recurse -File -Force).Count
    if ($n2 -lt $n1) { [Windows.Forms.MessageBox]::Show("Only $n2 of $n1 files arrived in $to. The old folder is unchanged; please try again.", "Garden Bridge") | Out-Null; return $false }
    $desk = [Environment]::GetFolderPath("Desktop")
    foreach ($z in "junsgarden-photos-addon.zip", "junsgarden-wiki-addon.zip") {
        $p = Join-Path $desk $z
        if ((Test-Path $p) -and (In-OneDrive $p) -and -not (Test-Path (Join-Path $to $z))) { Move-Item $p $to }
    }
    # The garden's scheduled tasks (More 13's weekly copy, "keep running") name the
    # folder they run from: point them at the new place, or they stop working once
    # the old folder is deleted.
    Get-ScheduledTask -TaskName "Jun's Garden*" -ErrorAction SilentlyContinue | ForEach-Object {
        $acts = @($_.Actions | ForEach-Object {
            if ($_.Arguments) { $_.Arguments = $_.Arguments.Replace($from, $to) }
            if ($_.WorkingDirectory) { $_.WorkingDirectory = $_.WorkingDirectory.Replace($from, $to) }
            $_ })
        try { Set-ScheduledTask -TaskName $_.TaskName -TaskPath $_.TaskPath -Action $acts -ErrorAction Stop | Out-Null } catch { }
    }
    try {   # a shortcut on the Desktop, so the folder is easy to find
        $lnk = (New-Object -ComObject WScript.Shell).CreateShortcut((Join-Path $desk "Garden folder.lnk"))
        $lnk.TargetPath = $to; $lnk.Save()
    } catch { }
    return $true
}

if (In-OneDrive $Root) {
    $suggest = Join-Path $env:USERPROFILE (Split-Path -Leaf $Root)
    $choice = New-OneDrive-Choice $Root $suggest
    [void]$choice.ShowDialog()
    $to = $null
    if ($choice.Tag -eq "move") { $to = $suggest }
    elseif ($choice.Tag -eq "pick") {
        $dlg = New-Object Windows.Forms.FolderBrowserDialog
        $dlg.Description = "Pick where the garden folder goes (a folder named $(Split-Path -Leaf $Root) is made inside it)"
        $dlg.ShowNewFolderButton = $true
        if ($dlg.ShowDialog() -eq [Windows.Forms.DialogResult]::OK) {
            $to = Join-Path $dlg.SelectedPath (Split-Path -Leaf $Root)
            if (In-OneDrive $to) { [Windows.Forms.MessageBox]::Show("That place is inside OneDrive too. The folder stays where it is.", "Garden Bridge") | Out-Null; $to = $null }
        }
    }
    if ($to -and (Move-Garden-Folder $Root $to)) {
        [Windows.Forms.MessageBox]::Show("Done. The garden folder is now in:`n$to`n`n" +
            "A 'Garden folder' shortcut is on your Desktop. The installer opens again from the new place.`n`n" +
            "The old folder is still in OneDrive. After the install, delete it yourself (right-click, Delete) to free the space.",
            "Garden Bridge") | Out-Null
        Start-Process powershell.exe -ArgumentList "-NoProfile -STA -ExecutionPolicy Bypass -File `"$(Join-Path $to 'windows\Garden-Setup.ps1')`""
        exit 0
    }
}

# ---------- the window and its fonts ---------------------------------------------
$form = New-Object Windows.Forms.Form
$form.Text = "Garden Bridge: from Windows to Linux"
$form.Size = New-Object Drawing.Size(760, 600)
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedDialog"
$form.MaximizeBox = $false
$fontBody = New-Object Drawing.Font("Segoe UI", 10.5)
$fontHead = New-Object Drawing.Font("Segoe UI", 16, [Drawing.FontStyle]::Bold)
$green = [Drawing.Color]::FromArgb(47, 125, 50)
$form.Font = $fontBody

# One panel per page; only one is visible at a time.
$pages = @()
# Make one page of the guided installer (only one page shows at a time).
function New-Page($title) {
    $p = New-Object Windows.Forms.Panel
    $p.Location = New-Object Drawing.Point(0, 0)
    $p.Size = New-Object Drawing.Size(744, 500)
    $p.Visible = $false
    $h = New-Object Windows.Forms.Label
    $h.Text = $title; $h.Font = $fontHead; $h.ForeColor = $green
    $h.Location = New-Object Drawing.Point(24, 16); $h.AutoSize = $true
    $p.Controls.Add($h)
    $form.Controls.Add($p)
    $script:pages += $p
    return $p
}
# Put a block of text on a page.
function Add-Text($page, $text, $y, $h = 60) {
    $l = New-Object Windows.Forms.Label
    $l.Text = $text; $l.Location = New-Object Drawing.Point(26, $y)
    $l.Size = New-Object Drawing.Size(690, $h)
    $page.Controls.Add($l); return $l
}
# A clickable link that opens a web page or a file in this folder.
function Add-Link($page, $text, $target, $x, $y) {
    $l = New-Object Windows.Forms.LinkLabel
    $l.Text = $text; $l.Location = New-Object Drawing.Point($x, $y); $l.AutoSize = $true
    $l.Tag = $target
    $l.add_LinkClicked({ param($s, $e)
        $t = $s.Tag
        if ($t -notmatch '^https?://') { $t = Join-Path $Root $t }
        Start-Process $t })
    $page.Controls.Add($l); return $l
}

# ---------- page 1: welcome ---------------------------------------------------------------
$p1 = New-Page "Welcome to your garden"
Add-Text $p1 ("This installs Jun's Garden: the whole website with all your plants, photos, blog, wiki and garden " +
    "tools, on a computer you choose. From then on it is yours: your logins, your passwords, your computer.") 64 60 | Out-Null
Add-Text $p1 ("Nothing is hidden. Every program comes from its official home (links on page 4). The install shows " +
    "each step before it runs it, with the exact commands, and saves everything it does in a 'logs' folder. " +
    "Nothing gives anybody else access to your computer.") 130 80 | Out-Null
Add-Text $p1 "Read these first if you like (they open in your browser or editor):" 220 24 | Out-Null
Add-Link $p1 "How it all works: a map of the install and every feature (start here if you are unsure)" "00-HOW-IT-WORKS.html" 40 250 | Out-Null
Add-Link $p1 "Read me first (step by step)" "00-READ-ME-FIRST.html" 40 278 | Out-Null
Add-Link $p1 "What gets installed, and where each program comes from" "WHAT-GETS-INSTALLED.md" 40 306 | Out-Null
Add-Link $p1 "The files that come from Jason (with fingerprints)" "FILES-FROM-JASON.txt" 40 334 | Out-Null
Add-Text $p1 "Time: about 1 to 2 hours, mostly waiting while the website is built." 380 24 | Out-Null
Add-Text $p1 ("In the black window, choose 1 EASY when it asks (recommended). Advanced parts (going online, " +
    "email, NAS and Google Drive backups, subdomains) come later from the menu.") 410 44 | Out-Null
# The credit line, small and quiet at the bottom of the first page.
$credit = Add-Text $p1 "Garden Bridge: from Windows to Linux. Imagined by Jason, prototyped by Claude." 462 22
$credit.ForeColor = [Drawing.Color]::FromArgb(122, 99, 181); $credit.Font = New-Object Drawing.Font("Segoe UI", 9)

# ---------- page 2: where ---------------------------------------------------------------------
$p2 = New-Page "Where should your garden run?"
$rbRemote = New-Object Windows.Forms.RadioButton
$rbRemote.Text = "Another computer at home: an Ubuntu laptop or a Raspberry Pi (recommended)"
$rbRemote.Location = New-Object Drawing.Point(26, 64); $rbRemote.AutoSize = $true; $rbRemote.Checked = $true
$p2.Controls.Add($rbRemote)
Add-Text $p2 ("It must be switched on, with Ubuntu (or Raspberry Pi OS) and the SSH server (on it, once: " +
    "sudo apt install -y openssh-server). Its LOCAL IP address: on it, type  hostname -I  (it looks like 192.168.1.50). " +
    "Later the router sends ports 80 and 443 to THAT address only.") 92 66 | Out-Null
$lbIp = Add-Text $p2 "Its IP address:" 166 22; $lbIp.Size = New-Object Drawing.Size(140, 22)
$tbIp = New-Object Windows.Forms.TextBox; $tbIp.Location = New-Object Drawing.Point(170, 163); $tbIp.Width = 180; $p2.Controls.Add($tbIp)
$lbUser = Add-Text $p2 "Your user name on it:" 198 22; $lbUser.Size = New-Object Drawing.Size(150, 22)
Add-Text $p2 "(the name you log in with there; on it type  whoami )" 224 22 | Out-Null
$tbUser = New-Object Windows.Forms.TextBox; $tbUser.Location = New-Object Drawing.Point(170, 195); $tbUser.Width = 180; $p2.Controls.Add($tbUser)
$btnTest = New-Object Windows.Forms.Button; $btnTest.Text = "Test the connection"; $btnTest.Location = New-Object Drawing.Point(370, 161); $btnTest.AutoSize = $true
$p2.Controls.Add($btnTest)
$lbTest = Add-Text $p2 "" 248 22
$btnTest.add_Click({
    # Only checks that the SSH door (port 22) answers. Nothing is sent.
    $c = New-Object Net.Sockets.TcpClient
    try {
        $r = $c.BeginConnect($tbIp.Text.Trim(), 22, $null, $null)
        if ($r.AsyncWaitHandle.WaitOne(5000) -and $c.Connected) { $lbTest.Text = "It answers on the SSH port. Good."; $lbTest.ForeColor = $green }
        else { $lbTest.Text = "No answer. Check the address, and that the SSH server is installed on it."; $lbTest.ForeColor = [Drawing.Color]::DarkRed }
    } catch { $lbTest.Text = "That address does not look right."; $lbTest.ForeColor = [Drawing.Color]::DarkRed }
    finally { $c.Close() }
})
$rbWsl = New-Object Windows.Forms.RadioButton
$rbWsl.Text = "This Windows PC, inside Windows' own Ubuntu (WSL)"
$rbWsl.Location = New-Object Drawing.Point(26, 282); $rbWsl.AutoSize = $true
$p2.Controls.Add($rbWsl)
Add-Text $p2 ("Windows installs Ubuntu as an app (by Microsoft and Canonical) and the garden runs in it exactly like " +
    "on a real Ubuntu computer. The PC must stay on and signed in for the website to be up. " +
    "Switching WSL on the first time needs your permission and one restart.") 308 70 | Out-Null
Add-Link $p2 "What is WSL? (Microsoft)" "https://learn.microsoft.com/windows/wsl/about" 40 382 | Out-Null

# ---------- page 3: about you -------------------------------------------------------------
$p3 = New-Page "About you"
Add-Text $p3 "Your first name is your login name on the garden. Letters only." 64 24 | Out-Null
$tbName = New-Object Windows.Forms.TextBox; $tbName.Location = New-Object Drawing.Point(28, 92); $tbName.Width = 240; $tbName.Text = "Jun"; $p3.Controls.Add($tbName)
Add-Text $p3 "Your email address. 'Forgot password' links go here. Nobody else gets it." 134 24 | Out-Null
$tbEmail = New-Object Windows.Forms.TextBox; $tbEmail.Location = New-Object Drawing.Point(28, 162); $tbEmail.Width = 320; $p3.Controls.Add($tbEmail)
Add-Text $p3 ("Passwords are NOT typed here. The install asks for them in its black window, where nothing shows " +
    "while you type. They are stored only as scrambled hashes on the garden computer.") 196 44 | Out-Null
# Your web name: keep the one the files use now, or a new one (the Website name swap).
$curName = "junsgarden.zapto.org"
try { if ((Get-Content (Join-Path $Root "site\apps\version.json") -Raw) -match 'https://([^/"]+)') { $curName = $Matches[1] } } catch { }
Add-Text $p3 "Your web name (your garden's address on the internet later):" 250 22 | Out-Null
$rbKeepName = New-Object Windows.Forms.RadioButton
$rbKeepName.Text = "Keep $curName (recommended)"
$rbKeepName.Location = New-Object Drawing.Point(28, 274); $rbKeepName.AutoSize = $true; $rbKeepName.Checked = $true
$p3.Controls.Add($rbKeepName)
$rbNewName = New-Object Windows.Forms.RadioButton
$rbNewName.Text = "A new name:"
$rbNewName.Location = New-Object Drawing.Point(28, 300); $rbNewName.AutoSize = $true
$p3.Controls.Add($rbNewName)
$tbWebName = New-Object Windows.Forms.TextBox; $tbWebName.Location = New-Object Drawing.Point(150, 298); $tbWebName.Width = 260
$tbWebName.add_TextChanged({ if ($tbWebName.Text) { $rbNewName.Checked = $true } })
$p3.Controls.Add($tbWebName)
Add-Text $p3 ("A web name is free from No-IP: one free name per account, confirmed every 30 days by an email " +
    "link. You can choose it now and add it at No-IP before going online. Change it later: menu, More, 17.") 332 44 | Out-Null
Add-Link $p3 "How to get a web name (guide 04, Part A)" "START-HERE/04-go-public-no-ip-caddy.md" 40 380 | Out-Null
Add-Link $p3 "No-IP's free plan" "https://www.noip.com/free" 380 380 | Out-Null
Add-Text $p3 ("Later, if you like: more free names from DuckDNS for other things (Home Assistant panels, " +
    "automations), each behind your sign-in page. Guide 04, 'Advanced'.") 408 44 | Out-Null

# ---------- page 4: what gets installed -------------------------------------------------------
$p4 = New-Page "What gets installed"
Add-Text $p4 "NECESSARY (always): each link opens the program's official page." 58 22 | Out-Null
# The list on a soft baby-blue panel, a little larger; each link in its maker's
# own colour, softened (not their logos, just a hint of who made it).
$sw = New-Object Windows.Forms.Panel
$sw.Location = New-Object Drawing.Point(26, 82); $sw.Size = New-Object Drawing.Size(690, 168)
$sw.BackColor = [Drawing.Color]::FromArgb(234, 244, 252)
$p4.Controls.Add($sw)
$y = 8
foreach ($row in @(
    @("Ubuntu updates, firewall (ufw), SSH server", "https://ubuntu.com/server/docs", 201, 71, 27),
    @("Docker Engine: runs each program in its own box", "https://docs.docker.com/engine/install/ubuntu/", 29, 99, 237),
    @("MariaDB database (Docker Official Image)", "https://hub.docker.com/_/mariadb", 46, 94, 110),
    @("Your website, built from its code in rust/ (Rust, Debian)", "https://hub.docker.com/_/rust", 183, 65, 14),
    @("Toolbox for backups (Alpine Linux, Docker Official Image)", "https://hub.docker.com/_/alpine", 13, 89, 127),
    @("Your plants and photos, from the backup in this folder", "START-HERE/13-backups-and-folders.md", 47, 107, 58))) {
    $l = Add-Link $sw $row[0] $row[1] 14 $y
    $l.Font = New-Object Drawing.Font("Segoe UI", 11.5)
    $l.LinkColor = [Drawing.Color]::FromArgb($row[2], $row[3], $row[4])
    $l.ActiveLinkColor = $l.LinkColor; $l.VisitedLinkColor = $l.LinkColor
    $y += 26
}
$y = 82 + 168
Add-Text $p4 "OPTIONAL: tick what you want (you can change it later in the menu, 'Parts')." ($y + 6) 22 | Out-Null
$y += 32
$cbBeds = New-Object Windows.Forms.CheckBox; $cbBeds.Text = "Garden tools (/beds)"; $cbBeds.Checked = $true; $cbBeds.Location = New-Object Drawing.Point(40, $y); $cbBeds.AutoSize = $true; $p4.Controls.Add($cbBeds)
$l = Add-Link $p4 "Python base image" "https://hub.docker.com/_/python" 330 ($y + 3); $l.LinkColor = [Drawing.Color]::FromArgb(48, 105, 152); $y += 28
$cbWiki = New-Object Windows.Forms.CheckBox; $cbWiki.Text = "Wiki (pages: a separate download)"; $cbWiki.Checked = $false; $cbWiki.Location = New-Object Drawing.Point(40, $y); $cbWiki.AutoSize = $true; $p4.Controls.Add($cbWiki)
$l = Add-Link $p4 "Wiki.js on GitHub" "https://github.com/requarks/wiki" 330 ($y + 3); $l.LinkColor = [Drawing.Color]::FromArgb(25, 118, 210); $y += 28
$cbN8n = New-Object Windows.Forms.CheckBox; $cbN8n.Text = "n8n automations (advanced)"; $cbN8n.Checked = $false; $cbN8n.Location = New-Object Drawing.Point(40, $y); $cbN8n.AutoSize = $true; $p4.Controls.Add($cbN8n)
$l = Add-Link $p4 "n8n documentation" "https://docs.n8n.io" 330 ($y + 3); $l.LinkColor = [Drawing.Color]::FromArgb(201, 65, 95); $y += 34
Add-Text $p4 ("LATER, from the menu: the front door (Caddy, your sign-in page Authelia, No-IP, router ports 80 and 443), " +
    "email, local AI. Full list: WHAT-GETS-INSTALLED.md") $y 50 | Out-Null

# ---------- page 5: install -------------------------------------------------------------------
$p5 = New-Page "Install"
$lbInst = Add-Text $p5 ("Click 'Start the install'. A black window opens and does the real work. It explains every step, " +
    "shows each command, and asks for passwords there (hidden while you type). Keep it open until it says it is finished and that you can close it. " +
    "This page ticks off the steps as they finish.") 60 66
$btnGo = New-Object Windows.Forms.Button; $btnGo.Text = "Start the install"; $btnGo.Location = New-Object Drawing.Point(28, 132); $btnGo.AutoSize = $true
$p5.Controls.Add($btnGo)
$list = New-Object Windows.Forms.ListBox; $list.Location = New-Object Drawing.Point(28, 172); $list.Size = New-Object Drawing.Size(690, 260); $p5.Controls.Add($list)
$lbState = Add-Text $p5 "" 440 40

# ---------- page 6: done --------------------------------------------------------------------------
$p6 = New-Page "Your garden is ready"
Add-Text $p6 ("Open your garden below. For everyday jobs (start, stop, back up, settings, users) use the menu: " +
    "double-click Garden-Menu-Windows.cmd. To put the garden on the internet later, the menu's 'More', then 1.") 64 70 | Out-Null
$btnOpen = New-Object Windows.Forms.Button; $btnOpen.Text = "Open my garden"; $btnOpen.Location = New-Object Drawing.Point(28, 150); $btnOpen.AutoSize = $true; $p6.Controls.Add($btnOpen)
$btnMenu = New-Object Windows.Forms.Button; $btnMenu.Text = "Open the menu"; $btnMenu.Location = New-Object Drawing.Point(200, 150); $btnMenu.AutoSize = $true; $p6.Controls.Add($btnMenu)
$btnOpen.add_Click({ Start-Process powershell.exe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$Menu`" -Action open-website" })
$btnMenu.add_Click({ Start-Process (Join-Path $Root "Garden-Menu-Windows.cmd") })

# ---------- Back / Next buttons and page switching -----------------------------------------------
$btnBack = New-Object Windows.Forms.Button; $btnBack.Text = "< Back"; $btnBack.Location = New-Object Drawing.Point(500, 515); $btnBack.Width = 100
$btnNext = New-Object Windows.Forms.Button; $btnNext.Text = "Next >"; $btnNext.Location = New-Object Drawing.Point(615, 515); $btnNext.Width = 100
$form.Controls.Add($btnBack); $form.Controls.Add($btnNext)
$script:page = 0
# Show page number n and set the Back/Next buttons for it.
function Show-Page($n) {
    for ($i = 0; $i -lt $pages.Count; $i++) { $pages[$i].Visible = ($i -eq $n) }
    $script:page = $n
    $btnBack.Enabled = ($n -gt 0 -and $n -lt 5)
    $btnNext.Visible = ($n -lt 4)
}
# Checks before leaving a page, so mistakes are caught early.
function Page-OK($n) {
    if ($n -eq 1 -and $rbRemote.Checked) {
        if ($tbIp.Text.Trim() -notmatch '^\d{1,3}(\.\d{1,3}){3}$') { [Windows.Forms.MessageBox]::Show("Type the IP address, for example 192.168.1.50.") | Out-Null; return $false }
        if ($tbUser.Text.Trim() -eq "") { [Windows.Forms.MessageBox]::Show("Type your user name on that computer.") | Out-Null; return $false }
    }
    if ($n -eq 2) {
        if ($tbName.Text.Trim() -notmatch '^[A-Za-z]+$') { [Windows.Forms.MessageBox]::Show("Your first name: letters only.") | Out-Null; return $false }
        if ($tbEmail.Text.Trim() -notmatch '^[^@\s]+@[^@\s]+\.[^@\s]+$') { [Windows.Forms.MessageBox]::Show("That email address does not look right.") | Out-Null; return $false }
        if ($rbNewName.Checked) {
            # Tidied like the menu does: no http(s)://, www., port or path.
            $n = (($tbWebName.Text -replace '\s', '').ToLower() -replace '^https?://', '' -replace '^www\.', '' -replace '[/:].*$', '' -replace '\.$', '')
            if ($n -notmatch '^([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$') { [Windows.Forms.MessageBox]::Show("The web name should look like  momsgarden.zapto.org") | Out-Null; return $false }
            $tbWebName.Text = $n
        }
    }
    return $true
}
$btnNext.add_Click({ if (Page-OK $script:page) { Show-Page ($script:page + 1) } })
$btnBack.add_Click({ Show-Page ($script:page - 1) })

# ---------- starting the install and watching its progress ------------------------------------------
$timer = New-Object Windows.Forms.Timer; $timer.Interval = 5000
$btnGo.add_Click({
    $parts = @(); if ($cbBeds.Checked) { $parts += "beds" }; if ($cbWiki.Checked) { $parts += "wiki" }; if ($cbN8n.Checked) { $parts += "n8n" }
    $a = "-NoProfile -ExecutionPolicy Bypass -File `"$Menu`" -Action install -Name $($tbName.Text.Trim()) -Email $($tbEmail.Text.Trim()) -Parts `"$($parts -join ',')`""
    if ($rbRemote.Checked) { $a += " -Mode remote -RemoteHost $($tbIp.Text.Trim()) -RemoteUser $($tbUser.Text.Trim())" } else { $a += " -Mode wsl" }
    # A new web name, or the one it came with (then the black window does not ask again).
    if ($rbNewName.Checked -and $tbWebName.Text) { $a += " -WebName $($tbWebName.Text)" } else { $a += " -WebName $curName" }
    # -NoExit keeps the black window open at the end, so you can read it.
    # A fresh start: the black window writes RUNNING, then RESTART, FAILED or DONE.
    Remove-Item (Join-Path $Root "logs\windows-install-status.txt") -ErrorAction SilentlyContinue
    Start-Process powershell.exe -ArgumentList ("-NoExit " + $a)
    $btnGo.Enabled = $false
    $lbState.Text = "The install is running in the black window. Steps appear here as they finish."
    $timer.Start()
})
# Every 5 seconds, read the install's progress file from the garden computer.
$timer.add_Tick({
    # First, what the black window says (on this PC).
    $status = (Get-Content (Join-Path $Root "logs\windows-install-status.txt") -ErrorAction SilentlyContinue | Select-Object -First 1)
    if ($status -eq "RESTART") { $timer.Stop(); $lbState.Text = "Windows needs one restart. After it, double-click Start-Here-Windows.cmd again and click through the same pages."; $btnGo.Enabled = $true; return }
    if ($status -eq "FAILED") { $lbState.Text = "Something stopped. Read the black window, fix what it says, and click Start again."; $btnGo.Enabled = $true }
    if ($status -eq "DONE") { $timer.Stop(); $lbState.Text = "All done."; Show-Page 5; return }
    $txt = $null
    try {
        if ($rbRemote.Checked) {
            $txt = & ssh.exe -o BatchMode=yes -o ConnectTimeout=4 "$($tbUser.Text.Trim())@$($tbIp.Text.Trim())" "cat ~/junsgarden-for-jun/logs/install-progress.txt" 2>$null
        } else {
            $txt = & wsl.exe -d JunsGarden -- bash -c "cat ~/junsgarden-for-jun/logs/install-progress.txt" 2>$null
        }
    } catch { }
    if (-not $txt) { return }
    $list.Items.Clear()
    foreach ($line in ($txt | Select-Object -Last 16)) { [void]$list.Items.Add($line) }
    $last = ($txt | Select-Object -Last 1)
    if ($last -match 'DONE\s+step\s+14/14') { $lbState.Text = "All 14 steps done. In the black window: set your password (and AI keys), until it says it is finished." }
    elseif ($last -match 'FAILED') { $lbState.Text = "A step stopped. Read the black window, fix what it says, and click Start again."; $btnGo.Enabled = $true }
})

Show-Page 0
[void]$form.ShowDialog()
