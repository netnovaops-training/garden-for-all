<#
===============================================================================
 Garden-Setup.ps1: the SETUP WIZARD for Windows (windows and buttons, mouse or finger).
 Start it by double-clicking  Start-Here-Windows.cmd  (one folder up).

 A step-by-step wizard: Back and Next, a list of the steps on the left and a
 progress bar on top, a Help button on every page that opens the guide at that
 exact question (English or Korean), and a window you can make bigger.

 THE PAGES:
   Welcome (and the language of the guides) > Level: EASY or CUSTOM >
   Where the garden runs > About you (and your web name) >
   CUSTOM only: Optional software (tick what you want) > one page for each part
   you ticked > Passwords >
   Summary (every answer, and the ones answered for you) > Install > Done.

 EASY keeps today's automatic install: the black window asks for the passwords.
 CUSTOM lets you type the passwords and keys here instead. They go to the
 install in one private file (only your Windows user can read it), which is
 handed to the garden computer through a program's input (never on a command
 line, never in a log) and deleted here; garden.sh wipes its copy when the
 install ends. They are never written to garden-setup.json.

 Your other answers (no passwords) are kept in garden-setup.json in the garden
 folder: close the wizard half way, open it again, and it carries on at the
 same page. It is also the start of the garden's setup record.

 This file does not install anything by itself. Everything happens in the black
 window, through Garden-Menu.ps1 and garden.sh, which you can read.
===============================================================================
#>
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
# Sharp text on big screens: tell Windows this window scales itself.
try {
    Add-Type -Namespace GardenUi -Name Dpi -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();' -ErrorAction Stop
    [void][GardenUi.Dpi]::SetProcessDPIAware()
} catch { }
[System.Windows.Forms.Application]::EnableVisualStyles()

$Root = Split-Path -Parent $PSScriptRoot          # the garden folder
$Menu = Join-Path $PSScriptRoot "Garden-Menu.ps1"  # the program that does the work
. (Join-Path $PSScriptRoot "Garden-Network.ps1")  # this PC's address; find computers at home
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
        "The garden's backup copies (with all your photos) soon grow past that.`n`n" +
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


# ---------- sizes that follow the screen ------------------------------------------
# 1.0 on a normal screen, 1.5 at 150% scaling: every size below is multiplied by it.
$g0 = [Drawing.Graphics]::FromHwnd([IntPtr]::Zero); $UiScale = [Math]::Max(1.0, $g0.DpiX / 96.0); $g0.Dispose()
function Px([double]$n) { return [int][Math]::Round($n * $UiScale) }
function Pad($l, $t, $r, $b) { return New-Object Windows.Forms.Padding((Px $l), (Px $t), (Px $r), (Px $b)) }

$fontBody = New-Object Drawing.Font("Segoe UI", 10.5)
$fontSmall = New-Object Drawing.Font("Segoe UI", 9.5)
$fontBold = New-Object Drawing.Font("Segoe UI", 10.5, [Drawing.FontStyle]::Bold)
$fontHead = New-Object Drawing.Font("Segoe UI", 16, [Drawing.FontStyle]::Bold)
$fontField = New-Object Drawing.Font("Segoe UI", 11)
$fontStep = New-Object Drawing.Font("Segoe UI", 10)
$fontStepNow = New-Object Drawing.Font("Segoe UI", 10, [Drawing.FontStyle]::Bold)
$green = [Drawing.Color]::FromArgb(47, 125, 50)
$leaf = [Drawing.Color]::FromArgb(236, 245, 236)
$grey = [Drawing.Color]::FromArgb(105, 105, 105)
$red = [Drawing.Color]::DarkRed
# Words in Korean, written as character codes so this file reads the same on every
# Windows language setting.
function Ko([int[]]$codes) { return -join ([char[]]$codes) }
$KoWord = Ko @(0xD55C, 0xAD6D, 0xC5B4)                    # "Korean", in Korean
$Tick = [string][char]0x2713; $Now = [string][char]0x25B6  # a tick, and an arrow for "you are here"

# A message box, and a yes / no question (tests replace these two).
function Show-Problem($m) { [void][Windows.Forms.MessageBox]::Show($m, "Garden setup wizard") }
function Ask-YesNo($m) { return ([Windows.Forms.MessageBox]::Show($m, "Garden setup wizard", "YesNo") -eq "Yes") }

# ---------- the window: steps on the left, the page, Help / Back / Next below -------
$form = New-Object Windows.Forms.Form
$form.Text = "Garden setup wizard"
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "Sizable"; $form.MaximizeBox = $true
$wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
$form.Size = New-Object Drawing.Size([Math]::Min((Px 1040), $wa.Width - 20), [Math]::Min((Px 740), $wa.Height - 20))
$form.MinimumSize = New-Object Drawing.Size([Math]::Min((Px 800), $wa.Width - 20), [Math]::Min((Px 560), $wa.Height - 20))
$form.Font = $fontBody

$content = New-Object Windows.Forms.Panel; $content.Dock = "Fill"
$header = New-Object Windows.Forms.Panel; $header.Dock = "Top"; $header.Height = Px 104; $header.Padding = Pad 26 12 26 8
$bottom = New-Object Windows.Forms.Panel; $bottom.Dock = "Bottom"; $bottom.Height = Px 74; $bottom.Padding = Pad 18 12 18 12
$side = New-Object Windows.Forms.Panel; $side.Dock = "Left"; $side.Width = Px 250; $side.BackColor = $leaf; $side.Padding = Pad 16 20 8 8
# Docked from the last one added: the step list takes the whole left side.
$form.Controls.Add($content); $form.Controls.Add($header); $form.Controls.Add($bottom); $form.Controls.Add($side)

$lbTitle = New-Object Windows.Forms.Label; $lbTitle.Dock = "Top"; $lbTitle.Height = Px 42; $lbTitle.Font = $fontHead; $lbTitle.ForeColor = $green
$lbTitle.AutoEllipsis = $true   # a long title ends in "..." instead of being cut in half
$lbStep = New-Object Windows.Forms.Label; $lbStep.Dock = "Top"; $lbStep.Height = Px 24; $lbStep.ForeColor = $grey
$bar = New-Object Windows.Forms.ProgressBar; $bar.Dock = "Bottom"; $bar.Height = Px 12; $bar.Style = "Continuous"
$header.Controls.Add($bar); $header.Controls.Add($lbStep); $header.Controls.Add($lbTitle)
# The page's hot words, top right: bold #tags that open the guide, and a one-line overview.
$hot = New-Object Windows.Forms.FlowLayoutPanel; $hot.Dock = "Right"; $hot.Width = Px 360
$hot.FlowDirection = "TopDown"; $hot.WrapContents = $false; $hot.Padding = Pad 8 0 0 0
$hotTags = New-Object Windows.Forms.FlowLayoutPanel; $hotTags.FlowDirection = "LeftToRight"; $hotTags.WrapContents = $true
$hotTags.AutoSize = $true; $hotTags.AutoSizeMode = "GrowAndShrink"; $hotTags.MaximumSize = New-Object Drawing.Size((Px 350), 0); $hotTags.Margin = Pad 0 2 0 2
$hotBrief = New-Object Windows.Forms.Label; $hotBrief.AutoSize = $true; $hotBrief.Font = $fontSmall; $hotBrief.ForeColor = $grey
$hotBrief.MaximumSize = New-Object Drawing.Size((Px 350), 0); $hotBrief.Margin = Pad 0 0 0 0
$hot.Controls.Add($hotTags); $hot.Controls.Add($hotBrief)
$header.Controls.Add($hot)

$lbSide = New-Object Windows.Forms.Label; $lbSide.Text = "Your steps"; $lbSide.Font = $fontBold; $lbSide.ForeColor = $green; $lbSide.Dock = "Top"; $lbSide.Height = Px 30
$steps = New-Object Windows.Forms.FlowLayoutPanel; $steps.Dock = "Fill"; $steps.FlowDirection = "TopDown"; $steps.WrapContents = $false; $steps.AutoScroll = $true
$side.Controls.Add($steps); $side.Controls.Add($lbSide)

# Big buttons, easy to hit with a mouse or a finger.
function New-BigButton($text) {
    $b = New-Object Windows.Forms.Button; $b.Text = $text; $b.Font = $fontBold
    $b.Size = New-Object Drawing.Size((Px 150), (Px 48)); $b.Margin = Pad 8 0 0 0
    return $b
}
$btnHelp = New-BigButton "?  Help"; $btnHelp.Dock = "Left"
$navRow = New-Object Windows.Forms.FlowLayoutPanel; $navRow.Dock = "Right"; $navRow.Width = Px 330
$navRow.FlowDirection = "RightToLeft"; $navRow.WrapContents = $false
$btnNext = New-BigButton "Next  >"; $btnBack = New-BigButton "<  Back"
$navRow.Controls.Add($btnNext); $navRow.Controls.Add($btnBack)
$bottom.Controls.Add($navRow); $bottom.Controls.Add($btnHelp)

# ---------- making pages ------------------------------------------------------------
# Each page is a column that scrolls when the window is small; long lines wrap
# to the window's width (Fit-Page), also after it is made bigger or smaller.
$Pages = @{}
function New-Page($id, $title, $step) {
    $p = New-Object Windows.Forms.FlowLayoutPanel
    $p.Dock = "Fill"; $p.FlowDirection = "TopDown"; $p.WrapContents = $false; $p.AutoScroll = $true
    $p.Padding = Pad 24 8 24 24; $p.Visible = $false
    $p.add_Resize({ Fit-Page $this })
    $content.Controls.Add($p)
    $Pages[$id] = @{ Panel = $p; Title = $title; Step = $step }
    return $p
}
function All-Controls($c) { foreach ($x in $c.Controls) { $x; All-Controls $x } }
function Fit-Page($p) {
    if (-not $p) { return }
    $w = [Math]::Max((Px 260), $p.ClientSize.Width - $p.Padding.Horizontal - (Px 24))
    foreach ($c in @(All-Controls $p)) {
        if ($c -is [Windows.Forms.Label] -and $c.Tag -is [int]) { $c.MaximumSize = New-Object Drawing.Size(($w - (Px $c.Tag)), 0) }
        elseif ($c.Tag -eq "wide") { $c.Width = $w - (Px 8) }
    }
}
# A paragraph. -Indent moves it right (under a choice it belongs to).
function P-Text($p, $text, $Indent = 0, [switch]$Bold, [switch]$Small) {
    $l = New-Object Windows.Forms.Label; $l.Text = $text; $l.AutoSize = $true
    $l.Margin = Pad (3 + $Indent) 4 3 6; $l.Tag = [int]$Indent
    if ($Bold) { $l.Font = $fontBold; $l.Margin = Pad (3 + $Indent) 14 3 4 }
    if ($Small) { $l.Font = $fontSmall; $l.ForeColor = $grey }
    $p.Controls.Add($l); return $l
}
# A link that opens a web page, or a file in the garden folder.
function P-Link($p, $text, $target, $Indent = 0) {
    $l = New-Object Windows.Forms.LinkLabel; $l.Text = $text; $l.AutoSize = $true
    $l.Margin = Pad (3 + $Indent) 2 3 4; $l.Tag = [int]$Indent; $l.AccessibleDescription = $target
    $l.add_LinkClicked({ param($s, $e)
        $t = $s.AccessibleDescription
        if ($t -notmatch '^https?://') { $t = Join-Path $Root $t }
        Start-Process $t })
    $p.Controls.Add($l); return $l
}
# A row of things side by side (a box and its buttons).
function P-Row($p, $Indent = 0) {
    $r = New-Object Windows.Forms.FlowLayoutPanel; $r.FlowDirection = "LeftToRight"; $r.WrapContents = $true
    $r.AutoSize = $true; $r.AutoSizeMode = "GrowAndShrink"; $r.Margin = Pad (3 + $Indent) 2 3 6
    $p.Controls.Add($r); return $r
}
function Row-Button($r, $text) {
    $b = New-Object Windows.Forms.Button; $b.Text = $text; $b.AutoSize = $true
    $b.MinimumSize = New-Object Drawing.Size(0, (Px 36)); $b.Margin = Pad 10 0 0 0
    $r.Controls.Add($b); return $b
}
# A question with a box to type in. -Secret: dots instead of letters, and "Show".
function P-Field($p, $label, $Width = 320, [switch]$Secret, $Indent = 0) {
    [void](P-Text $p $label $Indent)
    $r = P-Row $p $Indent
    $tb = New-Object Windows.Forms.TextBox; $tb.Width = Px $Width; $tb.Font = $fontField
    $r.Controls.Add($tb)
    if ($Secret) {
        $tb.UseSystemPasswordChar = $true
        $cb = New-Object Windows.Forms.CheckBox; $cb.Text = "Show"; $cb.AutoSize = $true; $cb.Margin = Pad 12 6 0 0
        $cb.Tag = $tb
        $cb.add_CheckedChanged({ $this.Tag.UseSystemPasswordChar = -not $this.Checked })
        $r.Controls.Add($cb)
    }
    return $tb
}
# A choice (RadioButton or CheckBox) in bold, with a line saying what it means.
function P-Choice($p, $kind, $text, $desc, $Indent = 0) {
    $c = New-Object ("Windows.Forms." + $kind); $c.Text = $text; $c.AutoSize = $true; $c.Font = $fontBold
    $c.Margin = Pad (3 + $Indent) 10 3 0; $c.MinimumSize = New-Object Drawing.Size(0, (Px 30))
    $p.Controls.Add($c)
    if ($desc) { [void](P-Text $p $desc ($Indent + 26)) }
    return $c
}
# "More options": a button that opens and closes a part of the page.
function P-More($p, $text) {
    $b = New-Object Windows.Forms.Button; $b.Text = "+   $text"; $b.AutoSize = $true; $b.FlatStyle = "Flat"
    $b.MinimumSize = New-Object Drawing.Size(0, (Px 38)); $b.Margin = Pad 3 14 3 4
    $inner = New-Object Windows.Forms.FlowLayoutPanel; $inner.FlowDirection = "TopDown"; $inner.WrapContents = $false
    $inner.AutoSize = $true; $inner.AutoSizeMode = "GrowAndShrink"; $inner.Visible = $false; $inner.Margin = Pad 20 0 0 0
    $b.Tag = @{ Panel = $inner; Text = $text }
    $b.add_Click({ $i = $this.Tag.Panel; $i.Visible = -not $i.Visible
        $this.Text = $(if ($i.Visible) { "-   " } else { "+   " }) + $this.Tag.Text; Fit-Page $this.Parent })
    $p.Controls.Add($b); $p.Controls.Add($inner)
    return $inner
}

# ---------- page: welcome -----------------------------------------------------------
$pWelcome = New-Page "welcome" "Welcome to your garden" "Welcome"
[void](P-Text $pWelcome ("This installs Jun's Garden: the whole website with all your plants, photos, blog, wiki and garden " +
    "tools, on a computer you choose. From then on it is yours: your logins, your passwords, your computer."))
[void](P-Text $pWelcome ("Nothing is hidden. Every program comes from its official home. The install shows each step " +
    "before it runs it, with the exact commands, and saves everything it does in a 'logs' folder. " +
    "Nothing gives anybody else access to your computer."))
[void](P-Text $pWelcome "Language of the guides and of the install window" -Bold)
$cbLang = New-Object Windows.Forms.ComboBox; $cbLang.DropDownStyle = "DropDownList"; $cbLang.Width = Px 320; $cbLang.Font = $fontField
[void]$cbLang.Items.Add("English"); [void]$cbLang.Items.Add("$KoWord (Korean)"); [void]$cbLang.Items.Add("English + $KoWord (both)")
$cbLang.SelectedIndex = 0; $cbLang.Margin = Pad 3 2 3 6
$cbLang.add_SelectedIndexChanged({ if (-not $script:loading -and $script:page) { Update-HotWords $script:page } })
$pWelcome.Controls.Add($cbLang)
[void](P-Text $pWelcome "The Help button on every page opens the guide at that exact question, in this language." -Small)
[void](P-Text $pWelcome "Read these first if you like (they open in your browser or editor):" -Bold)
[void](P-Link $pWelcome "Overview: what the install does, in order, and where help is" "handbook/00-OVERVIEW.html" 16)
[void](P-Link $pWelcome "How it all works: a map of the install and every feature (start here if you are unsure)" "00-HOW-IT-WORKS.html" 16)
[void](P-Link $pWelcome "Read me first (step by step)" "00-READ-ME-FIRST.html" 16)
[void](P-Link $pWelcome "What gets installed, and where each program comes from" "WHAT-GETS-INSTALLED.md" 16)
[void](P-Link $pWelcome "The files that come from the author (with fingerprints)" "FILES-FROM-AUTHOR.txt" 16)
[void](P-Text $pWelcome "Time: about 1 to 2 hours, mostly waiting while the website is built.")
$lbResumed = P-Text $pWelcome ""; $lbResumed.ForeColor = $green
# The credit line, small and quiet at the bottom of the first page.
$credit = P-Text $pWelcome "Garden Bridge: from Windows to Linux. Imagined by Jason, prototyped by Claude." -Small
$credit.ForeColor = [Drawing.Color]::FromArgb(122, 99, 181)

# ---------- page: the level -----------------------------------------------------------
$pLevel = New-Page "level" "How much do you want to choose?" "Level: Easy or Custom"
$rbEasy = P-Choice $pLevel RadioButton "EASY: automatic (recommended)" ("Everything the garden needs, plus the garden tools, chosen for you. " +
    "Next you say where it runs and who you are; then the install window asks for your password (hidden while you type) " +
    "and does the rest. Extra parts come later, from the menu, only if you want them.")
$rbCustom = P-Choice $pLevel RadioButton "CUSTOM: you choose, page by page" ("First a list of the optional software to tick, then one page " +
    "for each part you ticked, and your passwords and keys. You type everything here, " +
    "in this window; the install window only installs.")
$rbEasy.Checked = $true
[void](P-Text $pLevel "You can switch between them with Back at any time before the install starts." -Small)

# ---------- page: where -------------------------------------------------------------------
$pWhere = New-Page "where" "Where does your garden run?" "Where it runs"
$rbRemote = P-Choice $pWhere RadioButton "Another computer at home: an Ubuntu laptop or a Raspberry Pi (recommended)" ("It must be switched on, " +
    "with Ubuntu (or Raspberry Pi OS) and the SSH server (on it, once: sudo apt install -y openssh-server). Its LOCAL IP " +
    "address: on it, type  hostname -I  (it looks like 192.168.1.50). Later the router sends ports 80 and 443 to THAT address only.")
$rbRemote.Checked = $true
function Row-Label($r, $text) {
    $l = New-Object Windows.Forms.Label; $l.Text = $text; $l.AutoSize = $true; $l.Margin = Pad 0 8 8 0
    $r.Controls.Add($l); return $l
}
$rIp = P-Row $pWhere 26
[void](Row-Label $rIp "Its IP address:")
$tbIp = New-Object Windows.Forms.TextBox; $tbIp.Width = Px 180; $tbIp.Font = $fontField; $rIp.Controls.Add($tbIp)
$btnTest = Row-Button $rIp "Test the connection"
$btnFind = Row-Button $rIp "Find it for me"
$rUser = P-Row $pWhere 26
[void](Row-Label $rUser "Your user name on it:")
$tbUser = New-Object Windows.Forms.TextBox; $tbUser.Width = Px 180; $tbUser.Font = $fontField; $rUser.Controls.Add($tbUser)
[void](Row-Label $rUser "(on it, type  whoami )")
$lbTest = P-Text $pWhere "" 26
$btnTest.add_Click({
    # Only checks that the SSH door (port 22) answers. Nothing is sent.
    $c = New-Object Net.Sockets.TcpClient
    try {
        $r = $c.BeginConnect($tbIp.Text.Trim(), 22, $null, $null)
        if ($r.AsyncWaitHandle.WaitOne(5000) -and $c.Connected) { $lbTest.Text = "It answers on the SSH port. Good."; $lbTest.ForeColor = $green }
        else { $lbTest.Text = "No answer. Check the address, and that the SSH server is installed on it."; $lbTest.ForeColor = $red }
    } catch { $lbTest.Text = "That address does not look right."; $lbTest.ForeColor = $red }
    finally { $c.Close() }
})
# Find the garden computer for you: the computers at home with the SSH door open.
$mnFound = New-Object Windows.Forms.ContextMenuStrip
$btnFind.add_Click({
    $lbTest.Text = "Looking on your home network for computers with the SSH door (port 22)..."; $lbTest.ForeColor = [Drawing.Color]::Black
    $form.Cursor = [Windows.Forms.Cursors]::WaitCursor; [Windows.Forms.Application]::DoEvents()
    $found = @(Find-Home-Computers)
    $form.Cursor = [Windows.Forms.Cursors]::Default
    if ($found.Count -eq 0) {
        $lbTest.Text = "None found. Is it on, with the SSH server installed? Or type its address."; $lbTest.ForeColor = $red; return
    }
    $lbTest.Text = "Found $($found.Count). Pick the garden computer from the list."; $lbTest.ForeColor = $green
    $mnFound.Items.Clear()
    foreach ($f in $found) {
        $it = $mnFound.Items.Add(("{0}   {1}" -f $f.IP, $f.Name).Trim()); $it.Tag = $f.IP
        $it.add_Click({ param($s, $e) $tbIp.Text = $s.Tag; $lbTest.Text = "Chosen: $($s.Tag). Now your user name on it." })
    }
    $mnFound.Show($btnFind, (New-Object Drawing.Point(0, $btnFind.Height)))
})
$me = Get-This-PC
$wslText = "This Windows PC, inside Windows' own Ubuntu (WSL)"
if ($me) { $wslText = "This Windows PC ($($me.Name), $($me.IP)), inside Windows' own Ubuntu (WSL)" }
$rbWsl = P-Choice $pWhere RadioButton $wslText ("Windows installs Ubuntu as an app (by Microsoft and Canonical) and the garden runs " +
    "in it exactly like on a real Ubuntu computer. The PC must stay on and signed in for the website to be up. " +
    "Switching WSL on the first time needs your permission and one restart.")
[void](P-Link $pWhere "What is WSL? (Microsoft)" "https://learn.microsoft.com/windows/wsl/about" 26)
# On this PC only: which Docker runs the garden's programs (a group of its own,
# so these two choices do not switch off the two above).
$pnEngine = New-Object Windows.Forms.FlowLayoutPanel; $pnEngine.FlowDirection = "TopDown"; $pnEngine.WrapContents = $false
$pnEngine.AutoSize = $true; $pnEngine.AutoSizeMode = "GrowAndShrink"; $pnEngine.Margin = Pad 26 4 3 4
$pWhere.Controls.Add($pnEngine)
$rbDesktop = P-Choice $pnEngine RadioButton "Docker Desktop: the easy way to manage it, with buttons (recommended on this PC)" ""
$rbEngine = P-Choice $pnEngine RadioButton "Docker inside Ubuntu: the hands-on way, the same as on any Ubuntu computer" ""
$rbDesktop.Checked = $true
$lbEngine = P-Text $pnEngine ("Docker Desktop is a free Windows app (personal use) that shows the garden's programs with start, " +
    "stop and message buttons. The install gets it for you and shows the one switch to turn on.") 26
# Windows switches its Ubuntu off when nothing uses it: a small task keeps it on while you are signed in.
$cbKeep = P-Choice $pnEngine CheckBox "Keep the garden running while you are signed in to Windows (recommended)" ("A small task in " +
    "Task Scheduler, 'Jun's Garden keep running', keeps Windows' Ubuntu on, so the website stays up. Remove it there any time.")
$cbKeep.Checked = $true
$pnEngine.Enabled = $false

# ---------- page: about you ------------------------------------------------------------------
$pYou = New-Page "you" "About you" "About you"
$tbName = P-Field $pYou "Your first name: your login name on the garden (letters only)" 260
$tbName.Text = "Jun"
$tbEmail = P-Field $pYou "Your email address ('forgot password' links go here; nobody else gets it)" 360
# Your web name: keep the one the files use now, or a new one (the Website name swap).
$curName = "junsgarden.zapto.org"
try { if ((Get-Content (Join-Path $Root "site\apps\version.json") -Raw) -match 'https://([^/"]+)') { $curName = $Matches[1] } } catch { }
[void](P-Text $pYou "Your web name (your garden's address on the internet later)" -Bold)
$rbKeepName = P-Choice $pYou RadioButton "Keep $curName (recommended)" "Before going online, add it to your own free No-IP account (guide 04, Part A)."
$rbNewName = P-Choice $pYou RadioButton "A new name" ""
$rbKeepName.Checked = $true
$tbWebName = P-Field $pYou "The new web name, for example  momsgarden.zapto.org" 360 -Indent 26
$tbWebName.add_TextChanged({ if ($tbWebName.Text) { $rbNewName.Checked = $true } })
[void](P-Text $pYou ("A web name is free from No-IP: one free name per account, confirmed every 30 days by an email " +
    "link. Change it later: menu, More, 17. Later, if you like: more free names from DuckDNS for other things " +
    "(Home Assistant panels, automations), each behind your sign-in page. Guide 04, 'Advanced'.") -Small)
[void](P-Link $pYou "How to get a web name (guide 04, Part A)" "START-HERE/04-go-public-no-ip-caddy.md")
[void](P-Link $pYou "No-IP's free plan" "https://www.noip.com/free")


# ---------- page: optional software (CUSTOM) ------------------------------------------------
# Every optional part the install and the menu offer, from garden.sh's own lists
# (PARTS_LIST, KEYS_LIST, email, network; the menu's More and Backups jobs).
$PartsNow = @(
    @("beds", "Garden tools (/beds)", "A bed planner, plant check-up, diagnose, tag scan and spend list. They open after signing in."),
    @("wiki", "Wiki: your garden encyclopedia (Wiki.js)", "A website of articles you write. One more program to download; you become its admin."),
    @("n8n", "n8n automations (advanced)", "'When this happens, do that' (for example: the soil is dry, send me an email). Private to the garden computer."),
    @("email", "Email (Gmail) for 'forgot password' links", "Your garden sends password links through Gmail. Needs a Gmail app password (16 letters)."),
    @("keys", "AI keys (Google Gemini, OpenRouter)", "Your own free keys for the AI boxes: reading plant tags, identifying, diagnosing."),
    @("plantdata", "Plant data key (Perenual)", "Plant facts for the plant admin's lookup (free plan)."),
    @("network", "A reserved address in your router", "So the garden computer keeps the same address (needed later for going online, and for SSH).")
)
$PartsLater = @(
    @("frontdoor", "The front door: HTTPS, your sign-in page, your web name", "Puts the garden on the internet safely (Caddy, Authelia, No-IP, router ports 80 and 443).", "More, then 1", "a free web name (No-IP), your router's admin login, about an hour"),
    @("nas", "Backup copies on your NAS (network drive)", "A second copy of every backup on a network drive at home.", "6 (Backups), then 4", "the NAS's address, a shared folder and a user on the NAS"),
    @("drive", "Backup copies on Google Drive", "A copy of the newest backup in your own Google Drive, every week.", "6 (Backups), then 5", "your Google account (you sign in in your browser; the garden never sees your password)"),
    @("subdomains", "Subdomains with DuckDNS (wiki., n8n., Home Assistant)", "More free web names for other parts, each with its own padlock.", "More, then 14", "a free DuckDNS account and its token; the front door first"),
    @("tailscale", "Reach the garden from anywhere, privately (Tailscale)", "Your phone and laptops reach the garden without opening the router.", "More, then 19", "a free Tailscale account, and its app on your phone"),
    @("litellm", "AI cost control (LiteLLM gateway)", "One key for your own projects, with a monthly spending limit.", "More, then 16", "an AI key (Gemini or OpenRouter)"),
    @("localai", "A local AI computer", "Use an AI running on your own computer (Ollama) instead of a cloud key.", "More, then 9", "another computer running Ollama, and its address")
)
$PartOrder = @($PartsNow | ForEach-Object { $_[0] }) + @($PartsLater | ForEach-Object { $_[0] })
$PartBoxes = @{}
$pParts = New-Page "parts" "Optional software" "Optional software"
[void](P-Text $pParts ("Tick what you want. Each part you tick gets its own page next, with its details. " +
    "Everything here can be switched on or off later too (menu 9, Parts; or menu M)."))
[void](P-Text $pParts "Set up now, during the install" -Bold)
foreach ($row in $PartsNow) { $PartBoxes[$row[0]] = P-Choice $pParts CheckBox $row[1] $row[2] }
[void](P-Text $pParts "Later, from the menu (ticking them puts their how-to on the last page)" -Bold)
foreach ($row in $PartsLater) { $PartBoxes[$row[0]] = P-Choice $pParts CheckBox $row[1] $row[2] }
$PartBoxes["beds"].Checked = $true
$lbNetWsl = P-Text $pParts "" 26 -Small

# ---------- pages: one for each part ---------------------------------------------------------
$pBeds = New-Page "part-beds" "Garden tools (/beds)" "Garden tools"
[void](P-Text $pBeds ("A small Python program next to your website: a bed planner, a plant check-up, diagnose, tag scan, " +
    "a spend list and a wiki queue. It also reads tag photos for the phone app."))
[void](P-Text $pBeds "They open only after signing in on your sign-in page (the front door step). Nothing to type here.")
[void](P-Link $pBeds "What the garden tools do (beds/README.md)" "beds/README.md")

$pWiki = New-Page "part-wiki" "Wiki: your garden encyclopedia" "Wiki"
[void](P-Text $pWiki ("Wiki.js: a website of articles. Your wiki pages come from the wiki add-on zip (put it next to the " +
    "garden folder, on the Desktop or in Downloads). You become its admin, with your email address and a password of its " +
    "own (it can differ from your garden password)."))
$tbWikiPw = P-Field $pWiki "Your wiki admin password (10 or more characters; empty = type it in the install window)" 320 -Secret
$tbWikiPw2 = P-Field $pWiki "Type it again" 320 -Secret
[void](P-Link $pWiki "Wiki.js documentation" "https://docs.requarks.io")

$pN8n = New-Page "part-n8n" "n8n automations" "n8n"
[void](P-Text $pN8n ("A drag-and-drop tool for 'when this happens, do that'. Private to the garden computer. The first time " +
    "you open it, n8n asks you to make its own owner account there. Nothing to type here."))
[void](P-Link $pN8n "n8n documentation" "https://docs.n8n.io")

$pEmail = New-Page "part-email" "Email (Gmail)" "Email"
[void](P-Text $pEmail ("The website, your sign-in page and n8n send email through Gmail's own mail server. Gmail needs an " +
    "APP PASSWORD (16 letters), not your normal password. Get it first (5 minutes, in your browser):"))
[void](P-Text $pEmail "1. Turn on 2-Step Verification in your Google account." 16)
[void](P-Link $pEmail "myaccount.google.com/signinoptions/twosv" "https://myaccount.google.com/signinoptions/twosv" 32)
[void](P-Text $pEmail "2. Open the app passwords page, type the name  garden , click Create. Google shows 16 letters ONCE: copy them." 16)
[void](P-Link $pEmail "myaccount.google.com/apppasswords" "https://myaccount.google.com/apppasswords" 32)
[void](P-Text $pEmail "3. Paste them below. Tip: a Gmail just for your website keeps your own inbox apart." 16)
$tbSmtpUser = P-Field $pEmail "Gmail address" 360
$tbSmtpPw = P-Field $pEmail "Gmail app password (16 letters; spaces do not matter)" 320 -Secret

$pKeys = New-Page "part-keys" "Your AI keys" "AI keys"
[void](P-Text $pKeys ("This garden has no keys from anybody else: the AI boxes work once you add your own free key. " +
    "Each is optional; empty = add it later (menu K). Most sites show a key only ONCE: copy it before you leave their page."))
$tbGemini = P-Field $pKeys "Google Gemini key (AI Studio)" 420 -Secret
[void](P-Link $pKeys "Get a Gemini key: aistudio.google.com/apikey" "https://aistudio.google.com/apikey" 16)
$tbOpenRouter = P-Field $pKeys "OpenRouter key (a backup when Gemini is busy; free models only)" 420 -Secret
[void](P-Link $pKeys "Get an OpenRouter key: openrouter.ai/keys" "https://openrouter.ai/keys" 16)

$pPlant = New-Page "part-plantdata" "Plant data key (Perenual)" "Plant data key"
[void](P-Text $pPlant "Plant facts for the plant admin and the garden tools' lookup. Free plan. Empty = add it later (menu K).")
$tbPerenual = P-Field $pPlant "Perenual key" 420 -Secret
[void](P-Link $pPlant "Get a key: perenual.com/docs/api" "https://perenual.com/docs/api" 16)

$pNet = New-Page "part-network" "A reserved address in your router" "Reserved address"
[void](P-Text $pNet ("At the end of the install, the install window shows the garden computer's address and its MAC (the network " +
    "card's fixed ID), and how to reserve that address in your router, so it never changes. You then say whether it is done."))
[void](P-Text $pNet "You need your router's admin login (often on a sticker on the router). Nothing to type here.")
[void](P-Link $pNet "Reserve your IP (guide 14)" "START-HERE/14-reserve-your-ip.md")

foreach ($row in $PartsLater) {
    $short = $row[1].Split(":")[0].Split("(")[0].Trim()
    $pl = New-Page ("part-" + $row[0]) $short $short
    [void](P-Text $pl $row[1] -Bold)
    [void](P-Text $pl $row[2])
    [void](P-Text $pl ("You will need: " + $row[4] + "."))
    [void](P-Text $pl ("After the install, in the Windows menu: " + $row[3] + ". The last page of this wizard reminds you."))
    [void](P-Text $pl "The Help button opens the guide's steps for it." -Small)
}

# ---------- page: passwords (CUSTOM) ------------------------------------------------------
$pPw = New-Page "passwords" "Your password" "Passwords"
[void](P-Text $pPw ("One password for your garden's login and your sign-in page. At least 10 characters. It is stored " +
    "only as a scrambled hash on the garden computer; nobody can read it back, not even you."))
$tbPw = P-Field $pPw "Your password" 320 -Secret
$tbPw2 = P-Field $pPw "Type it again" 320 -Secret
$moreDb = P-More $pPw "More options: the database password"
[void](P-Text $moreDb ("The database needs a password the website uses to talk to it. Leave it empty and a strong one is " +
    "made for you (recommended: you never type it). Menu 8 can renew it later.") 20)
$tbDbPw = P-Field $moreDb "Database password (16 or more letters and numbers; empty = make one for me)" 360 -Secret -Indent 20

# ---------- page: PDF guides (CUSTOM) ---------------------------------------------------------------
# An optional download that arrives while the install runs (tools\get-pdfs.ps1), checked against
# PDF-GUIDES-SHA256.txt. EASY does it without a page, in the language chosen on the first page.
$pPdf = New-Page "pdfs" "PDF guides" "PDF guides"
[void](P-Text $pPdf ("Every guide as a PDF, to read or print. They download while the garden installs " +
    "(about 27 MB in English and 25 MB in Korean, under a minute on a normal connection) and never slow the install down. " +
    "If a download fails, nothing is written and the garden works anyway: the menu's P item tries again."))
$rbPdfEn = P-Choice $pPdf RadioButton "English" ""
$rbPdfKo = P-Choice $pPdf RadioButton "$KoWord (Korean)" ""
$rbPdfBoth = P-Choice $pPdf RadioButton "Both" ""
$rbPdfNone = P-Choice $pPdf RadioButton "Not now" "You can get them any time from the menu (P)."
$rbPdfEn.Checked = $true
$script:pdfTouched = $false
foreach ($c in @($rbPdfEn, $rbPdfKo, $rbPdfBoth, $rbPdfNone)) { $c.add_Click({ $script:pdfTouched = $true }) }
[void](P-Text $pPdf "The same guides are always on your own website at /help once it runs." -Small)
# The PDF set(s) to fetch: EASY follows the language chosen on the first page; CUSTOM has its own page.
function Pdf-Code {
    if (-not $rbCustom.Checked) { return (Lang-Code) }
    if ($rbPdfKo.Checked) { return "ko" }
    if ($rbPdfBoth.Checked) { return "both" }
    if ($rbPdfNone.Checked) { return "none" }
    return "en"
}
function Pdf-Words { switch (Pdf-Code) { "en" { return "English" } "ko" { return "Korean" } "both" { return "English and Korean" } default { return "not now (menu P gets them later)" } } }
# Until the person picks one, the page starts on the language chosen on the first page.
function Pdf-Default {
    if ($script:pdfTouched) { return }
    $script:loading = $true
    try { switch (Lang-Code) { "ko" { $rbPdfKo.Checked = $true } "both" { $rbPdfBoth.Checked = $true } default { $rbPdfEn.Checked = $true } } } finally { $script:loading = $false }
}

# ---------- page: summary -------------------------------------------------------------------------
$pSum = New-Page "summary" "Check your answers" "Summary"
[void](P-Text $pSum "Everything the install will use. Back changes any of it. Passwords and keys are never shown, only whether they are set.")
$tbSummary = New-Object Windows.Forms.TextBox; $tbSummary.Multiline = $true; $tbSummary.ReadOnly = $true; $tbSummary.ScrollBars = "Vertical"
$tbSummary.Height = Px 380; $tbSummary.Tag = "wide"; $tbSummary.Font = New-Object Drawing.Font("Consolas", 10); $tbSummary.Margin = Pad 3 4 3 4
$pSum.Controls.Add($tbSummary)

# ---------- page: install ---------------------------------------------------------------------------
$pInst = New-Page "install" "Install" "Install"
[void](P-Text $pInst ("Click 'Start the install'. A black window opens and does the real work. It explains every step and shows each " +
    "command. Keep it open until it says it is finished and that you can close it. This page ticks off the steps as they finish."))
$btnGo = New-Object Windows.Forms.Button; $btnGo.Text = "Start the install"; $btnGo.Font = $fontBold
$btnGo.Size = New-Object Drawing.Size((Px 230), (Px 52)); $btnGo.Margin = Pad 3 10 3 10; $pInst.Controls.Add($btnGo)
$list = New-Object Windows.Forms.ListBox; $list.Height = Px 260; $list.Tag = "wide"; $list.Margin = Pad 3 4 3 4; $pInst.Controls.Add($list)
$lbState = P-Text $pInst ""

# ---------- page: done ---------------------------------------------------------------------------------
$pDone = New-Page "done" "Your garden is ready" "Done"
[void](P-Text $pDone ("Open your garden below. For everyday jobs (start, stop, back up, settings, users) use the menu: " +
    "double-click Garden-Menu-Windows.cmd."))
$rDone = P-Row $pDone
$btnOpen = Row-Button $rDone "Open my garden"; $btnOpen.Margin = Pad 0 0 0 0; $btnOpen.MinimumSize = New-Object Drawing.Size((Px 180), (Px 48))
$btnMenu = Row-Button $rDone "Open the menu"; $btnMenu.MinimumSize = New-Object Drawing.Size((Px 180), (Px 48))
$btnOpen.add_Click({ Start-Process powershell.exe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$Menu`" -Action open-website" })
$btnMenu.add_Click({ Start-Process (Join-Path $Root "Garden-Menu-Windows.cmd") })
$lbLater = P-Text $pDone ""

# ---------- the help for each page: the guide, at that exact question ----------------------
# START-HERE/help-ids.tsv lists the stable help ids: each one's guide, its hot words
# (#tags) and a one-line overview, in English and Korean, and the wizard pages that
# show it (the first one listed for a page is that page's main help). Every id has
# <a id="help-ID"></a> above its section in both guides: the handbook keeps it as
# #help-ID (English) and #help-ID-ko (Korean). So a reworded heading never breaks
# the Help button. tools/release/check_help_ids.py checks the file and the guides.
$HelpIds = @()
$RegFile = Join-Path $Root "START-HERE\help-ids.tsv"
if (Test-Path $RegFile) {
    $cols = $null
    foreach ($l in (Get-Content $RegFile -Encoding UTF8)) {
        if (-not $l -or $l.StartsWith("#")) { continue }
        $f = $l.Split("`t")
        if (-not $cols) { $cols = $f; continue }
        $o = @{}; for ($i = 0; $i -lt $cols.Count -and $i -lt $f.Count; $i++) { $o[$cols[$i]] = $f[$i] }
        $HelpIds += , $o
    }
}
function Lang-Code { return @("en", "ko", "both")[[Math]::Max(0, $cbLang.SelectedIndex)] }
function Help-Rows($page) { return @($HelpIds | Where-Object { ($_.wizard -split ',') -contains $page }) }
function Help-Row($hid) { return ($HelpIds | Where-Object { $_.id -eq $hid } | Select-Object -First 1) }
# The address of one help id: the handbook if it is built, else the guide's file.
function Help-Url($hid) {
    $r = Help-Row $hid
    $guide = $(if ($r) { $r.guide } else { "23-full-setup" }); if (-not $r) { $hid = "wizard" }
    $ko = ((Lang-Code) -eq "ko")
    $page = Join-Path $Root ("handbook\" + $guide + ".html")
    if (Test-Path $page) { return ([Uri]$page).AbsoluteUri + "#help-" + $hid + $(if ($ko) { "-ko" } else { "" }) }
    return (Join-Path $Root ("START-HERE\" + $(if ($ko) { "ko\" } else { "" }) + $guide + ".md"))
}
# A page's main help id (the first in the registry that names the page).
function Page-HelpId($page) { $r = Help-Rows $page | Select-Object -First 1; if ($r) { return $r.id } else { return "wizard" } }
function Help-Target($page) { return Help-Url (Page-HelpId $page) }
function Open-HelpId($hid) {
    $t = Help-Url $hid
    if ($t -match '^file:') {
        # Windows drops the #part of a file link when it starts the browser, so a
        # tiny page in TEMP sends the browser to the right place in the guide.
        $hop = Join-Path $env:TEMP "garden-help.html"
        $html = "<!doctype html><meta charset=`"utf-8`"><title>Garden help</title><meta http-equiv=`"refresh`" content=`"0;url=$t`">" +
                "<script>location.replace(`"$t`")</script><a href=`"$t`">Open the guide</a>"
        [IO.File]::WriteAllText($hop, $html, (New-Object Text.UTF8Encoding($false)))
        Start-Process $hop
    } elseif (Test-Path $t) { Start-Process $t }
    else { Show-Problem "The guides are in the START-HERE folder (guide 23)." }
}
$btnHelp.add_Click({ Open-HelpId (Page-HelpId $script:page) })

# The hot words in the top-right corner: the page's #tags in bold (each opens its
# section of the guide) and the main one's one-line overview under them.
$tip = New-Object Windows.Forms.ToolTip
function Update-HotWords($page) {
    $hotTags.SuspendLayout(); $hotTags.Controls.Clear()
    $ko = ((Lang-Code) -eq "ko")
    $rows = @(Help-Rows $page)
    foreach ($r in $rows) {
        foreach ($t in (($(if ($ko) { $r.tags_ko } else { $r.tags_en })) -split ' ')) {
            if (-not $t) { continue }
            $l = New-Object Windows.Forms.LinkLabel; $l.Text = $t; $l.AutoSize = $true; $l.Font = $fontBold
            $l.LinkColor = $green; $l.ActiveLinkColor = $green; $l.VisitedLinkColor = $green; $l.Margin = Pad 0 0 10 2
            $l.Tag = $r.id
            $l.add_LinkClicked({ param($s, $e) Open-HelpId $s.Tag })
            $tip.SetToolTip($l, $(if ($ko) { $r.brief_ko } else { $r.brief_en }))
            $hotTags.Controls.Add($l)
        }
    }
    $hotTags.ResumeLayout()
    $main = $rows | Select-Object -First 1
    $hotBrief.Text = $(if ($main) { if ($ko) { $main.brief_ko } else { $main.brief_en } } else { "" })
}

# ---------- the steps, and moving between them --------------------------------------------
# The pages this person goes through: EASY skips the CUSTOM ones; a part page
# only when its part is ticked.
function Get-Path {
    $p = @("welcome", "level", "where", "you")
    if ($rbCustom.Checked) {
        $p += "parts"
        foreach ($k in $PartOrder) { if ($PartBoxes[$k].Checked -and $PartBoxes[$k].Enabled) { $p += "part-$k" } }
        $p += "passwords"
        $p += "pdfs"
    }
    $p += @("summary", "install", "done")
    return ,$p
}
$script:page = "welcome"; $script:path = @(); $script:status = "wizard"; $script:loading = $false
function Draw-Steps {
    $steps.SuspendLayout(); $steps.Controls.Clear()
    $i = [array]::IndexOf($script:path, $script:page)
    for ($n = 0; $n -lt $script:path.Count; $n++) {
        $id = $script:path[$n]
        $l = New-Object Windows.Forms.Label; $l.AutoSize = $true; $l.Margin = Pad 0 3 0 3
        $l.MaximumSize = New-Object Drawing.Size(($side.Width - (Px 30)), 0)
        if ($n -lt $i) {
            $l.Text = "$Tick  " + $Pages[$id].Step; $l.Font = $fontStep; $l.ForeColor = $green
            $l.Cursor = [Windows.Forms.Cursors]::Hand; $l.Tag = $id
            if (-not $script:installing) { $l.add_Click({ Show-Page $this.Tag }) }
        } elseif ($n -eq $i) {
            $l.Text = "$Now  " + $Pages[$id].Step; $l.Font = $fontStepNow; $l.ForeColor = [Drawing.Color]::Black
        } else {
            $l.Text = "     " + $Pages[$id].Step; $l.Font = $fontStep; $l.ForeColor = $grey
        }
        $steps.Controls.Add($l)
    }
    $steps.ResumeLayout()
}
function Show-Page($id) {
    $script:path = Get-Path
    if ($script:path -notcontains $id) { $id = "welcome" }
    foreach ($k in @($Pages.Keys)) { $Pages[$k].Panel.Visible = ($k -eq $id) }
    $script:page = $id
    $i = [array]::IndexOf($script:path, $id)
    $lbTitle.Text = $Pages[$id].Title
    $lbStep.Text = "Step $($i + 1) of $($script:path.Count):  $($Pages[$id].Step)"
    $bar.Maximum = $script:path.Count; $bar.Value = $i + 1
    $btnBack.Enabled = ($i -gt 0 -and $id -ne "done" -and -not $script:installing)
    $btnNext.Visible = ($id -ne "install" -and $id -ne "done")
    $btnNext.Text = $(if ($id -eq "summary") { "Looks right  >" } else { "Next  >" })
    if ($id -eq "pdfs") { Pdf-Default }
    if ($id -eq "summary") { Build-Summary }
    if ($id -eq "done") { Build-Done }
    Draw-Steps
    Update-HotWords $id
    Fit-Page $Pages[$id].Panel
    Save-Record
}
function Next-Page { $i = [array]::IndexOf($script:path, $script:page); if ($i -lt $script:path.Count - 1) { Show-Page $script:path[$i + 1] } }
function Prev-Page { $i = [array]::IndexOf($script:path, $script:page); if ($i -gt 0) { Show-Page $script:path[$i - 1] } }
$btnNext.add_Click({ if (Page-OK $script:page) { Next-Page } })
$btnBack.add_Click({ Prev-Page })
# Choices that add or remove pages redraw the step list straight away.
foreach ($c in @($rbEasy, $rbCustom) + @($PartBoxes.Values)) { $c.add_CheckedChanged({ if (-not $script:loading) { $script:path = Get-Path; Draw-Steps } }) }
# On this PC (WSL) the router cannot reach the garden: no address to reserve there.
function Where-Changed {
    $pnEngine.Enabled = $rbWsl.Checked
    $net = $PartBoxes["network"]
    if ($rbWsl.Checked) { $net.Checked = $false; $net.Enabled = $false; $lbNetWsl.Text = "(A reserved address is not needed when the garden runs on this PC.)" }
    else { $net.Enabled = $true; $lbNetWsl.Text = "" }
}
$rbWsl.add_CheckedChanged({ Where-Changed })

# Checks before leaving a page, so mistakes are caught early.
function Page-OK($id) {
    switch ($id) {
        "where" {
            if ($rbRemote.Checked) {
                if ($tbIp.Text.Trim() -notmatch '^\d{1,3}(\.\d{1,3}){3}$') { Show-Problem "Type the IP address, for example 192.168.1.50."; return $false }
                if ($tbUser.Text.Trim() -notmatch '^[a-z_][a-z0-9_.-]*$') { Show-Problem "Type your user name on that computer (small letters, as whoami shows it)."; return $false }
            }
        }
        "you" {
            if ($tbName.Text.Trim() -notmatch '^[A-Za-z]+$') { Show-Problem "Your first name: letters only."; return $false }
            if ($tbEmail.Text.Trim() -notmatch '^[^@\s"''`$;&|<>]+@[^@\s"''`$;&|<>]+\.[A-Za-z]{2,}$') { Show-Problem "That email address does not look right."; return $false }
            if ($rbNewName.Checked) {
                # Tidied like the menu does: no http(s)://, www., port or path.
                $n = (($tbWebName.Text -replace '\s', '').ToLower() -replace '^https?://', '' -replace '^www\.', '' -replace '[/:].*$', '' -replace '\.$', '')
                if ($n -notmatch '^([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$') { Show-Problem "The web name should look like  momsgarden.zapto.org  (or choose Keep)"; return $false }
                $tbWebName.Text = $n
            }
            if (-not $tbSmtpUser.Text) { $tbSmtpUser.Text = $tbEmail.Text.Trim() }
        }
        "part-wiki" {
            if ($tbWikiPw.Text -and $tbWikiPw.Text.Length -lt 10) { Show-Problem "The wiki admin password: 10 or more characters (or leave it empty)."; return $false }
            if ($tbWikiPw.Text -ne $tbWikiPw2.Text) { Show-Problem "The two wiki passwords are not the same."; return $false }
        }
        "part-email" {
            $tbSmtpUser.Text = $tbSmtpUser.Text.Trim()
            if ($tbSmtpUser.Text -notmatch '^[^@\s"''`$;&|<>]+@[^@\s"''`$;&|<>]+\.[A-Za-z]{2,}$') { Show-Problem "The Gmail address does not look right."; return $false }
            if (($tbSmtpPw.Text -replace '\s', '') -notmatch '^[A-Za-z]{16}$') { Show-Problem "The app password is 16 letters (Google shows it in four groups of four). Or go Back and untick Email."; return $false }
        }
        "passwords" {
            if ($tbPw.Text.Length -lt 10) { Show-Problem "Your password: at least 10 characters."; return $false }
            if ($tbPw.Text -ne $tbPw2.Text) { Show-Problem "The two passwords are not the same."; return $false }
            if ($tbDbPw.Text -and $tbDbPw.Text -notmatch '^[A-Za-z0-9]{16,}$') { Show-Problem "The database password: 16 or more letters and numbers only (or leave it empty)."; return $false }
        }
    }
    return $true
}

# ---------- what the person chose, in plain lines ---------------------------------------------
function Chosen($list) { return @($list | Where-Object { $PartBoxes[$_[0]].Checked -and $PartBoxes[$_[0]].Enabled }) }
function Set-Or-Not($tb) { if ($tb.Text) { return "set (typed here)" } else { return "not set" } }
function Build-Summary {
    $custom = $rbCustom.Checked
    $o = New-Object Collections.Generic.List[string]
    $o.Add("LEVEL          " + $(if ($custom) { "CUSTOM: chosen page by page" } else { "EASY: automatic" }))
    $o.Add("LANGUAGE       " + $cbLang.Text)
    if ($rbRemote.Checked) { $o.Add("WHERE          another computer: $($tbUser.Text.Trim())@$($tbIp.Text.Trim())") }
    else { $o.Add("WHERE          this PC, in WSL, with " + $(if ($rbDesktop.Checked) { "Docker Desktop" } else { "Docker inside Ubuntu" }))
           $o.Add("KEEP RUNNING   " + $(if ($cbKeep.Checked) { "yes, while you are signed in (a task in Task Scheduler)" } else { "no (Windows may switch its Ubuntu off when idle)" })) }
    $o.Add("YOU            $($tbName.Text.Trim())  <$($tbEmail.Text.Trim())>")
    $o.Add("WEB NAME       " + (Web-Name) + $(if ($rbKeepName.Checked) { " (kept)" } else { " (new)" }))
    $o.Add("YOUR GARDEN    your plants, photos and settings from the backup in this folder;")
    $o.Add("               the photos and wiki add-on zips when they are next to it")
    $o.Add("")
    if ($custom) {
        $o.Add("SET UP NOW     " + ((Chosen $PartsNow | ForEach-Object { $_[1] }) -join "; "))
        $later = @(Chosen $PartsLater | ForEach-Object { $_[1] })
        $o.Add("LATER          " + $(if ($later.Count) { $later -join "; " } else { "(nothing ticked)" }))
        if ($PartBoxes["wiki"].Checked) { $o.Add("WIKI ADMIN     password " + $(if ($tbWikiPw.Text) { "set (typed here)" } else { "asked in the install window" })) }
        if ($PartBoxes["email"].Checked) { $o.Add("EMAIL          $($tbSmtpUser.Text), app password " + (Set-Or-Not $tbSmtpPw)) }
        if ($PartBoxes["keys"].Checked) { $o.Add("AI KEYS        Gemini " + (Set-Or-Not $tbGemini) + "; OpenRouter " + (Set-Or-Not $tbOpenRouter)) }
        if ($PartBoxes["plantdata"].Checked) { $o.Add("PLANT DATA     Perenual " + (Set-Or-Not $tbPerenual)) }
        $o.Add("PASSWORD       " + (Set-Or-Not $tbPw) + "   (website + sign-in page)")
        $o.Add("DATABASE       " + $(if ($tbDbPw.Text) { "password set (typed here)" } else { "a strong password made for you" }))
    } else {
        $o.Add("PARTS          the necessary ones, plus the garden tools (EASY's choice)")
        $o.Add("PASSWORD       asked in the install window (hidden while you type)")
    }
    $o.Add("PDF GUIDES     " + (Pdf-Words) + $(if ((Pdf-Code) -ne "none") { " (download while the install runs; never slows it)" } else { "" }))
    $o.Add("")
    $o.Add("ANSWERED FOR YOU (questions that only matter once):")
    $o.Add("  - the garden's folder: $GardenDir (the install window does not ask again)")
    $o.Add("  - where to start: the next step not done yet (a second run carries on from there)")
    $o.Add("  - time zone: taken from the garden computer")
    $o.Add("  - the long passwords the programs use between themselves: made at random, you never type them")
    $o.Add("  - firewall: on, SSH only from your home network")
    $o.Add("  - swap (extra memory on disk): added only on a small computer")
    $o.Add("  - backups: automatic (more often when the garden is busy)")
    $o.Add("")
    $o.Add("Your answers (no passwords or keys) are kept in garden-setup.json in the garden folder.")
    $tbSummary.Text = ($o -join "`r`n")
}
function Build-Done {
    $later = @(Chosen $PartsLater)
    if ($later.Count -eq 0) { $lbLater.Text = "Want more later? The menu's More list has every extra part (guide 19)."; return }
    $t = "LATER, FROM THE MENU (you ticked these):`n"
    foreach ($row in $later) { $t += "  - $($row[1]): Windows menu $($row[3]). You need $($row[4]).`n" }
    $lbLater.Text = $t + "The Help button opens the guide for each."
}

# ---------- the setup record: garden-setup.json (no passwords, no keys) ------------------------
$RecordFile = Join-Path $Root "garden-setup.json"
function Save-Record {
    if ($script:loading) { return }
    $r = [ordered]@{
        about       = "The setup wizard's answers (no passwords or keys). Garden-Setup.ps1 writes it and carries on from it."
        version     = 1
        updated     = (Get-Date -Format "yyyy-MM-dd HH:mm")
        status      = $script:status
        page        = $script:page
        language    = Lang-Code
        level       = $(if ($rbCustom.Checked) { "custom" } else { "easy" })
        where       = [ordered]@{ mode = $(if ($rbRemote.Checked) { "remote" } else { "wsl" }); ip = $tbIp.Text.Trim(); user = $tbUser.Text.Trim()
                                  engine = $(if ($rbDesktop.Checked) { "desktop" } else { "engine" })
                                  keep_running = $(if ($cbKeep.Checked) { "on" } else { "off" }) }
        owner       = [ordered]@{ first_name = $tbName.Text.Trim(); email = $tbEmail.Text.Trim()
                                  web_name = (Web-Name); web_name_kept = [bool]$rbKeepName.Checked; folder = $GardenDir }
        pdf_guides  = (Pdf-Code)
        parts_now   = @(Chosen $PartsNow | ForEach-Object { $_[0] })
        parts_later = @(Chosen $PartsLater | ForEach-Object { $_[0] })
        email_user  = $tbSmtpUser.Text
    }
    $json = ($r | ConvertTo-Json -Depth 5) -replace "`r`n", "`n"
    try { [IO.File]::WriteAllText($RecordFile, $json + "`n", (New-Object Text.UTF8Encoding($false))) } catch { }
}
# Read the record back: every answer into its place. Returns the page to show.
function Load-Record {
    if (-not (Test-Path $RecordFile)) { return "welcome" }
    try { $r = Get-Content $RecordFile -Raw -Encoding UTF8 | ConvertFrom-Json } catch { return "welcome" }
    $script:loading = $true
    try {
        $cbLang.SelectedIndex = [Math]::Max(0, [array]::IndexOf(@("en", "ko", "both"), [string]$r.language))
        if ($r.level -eq "custom") { $rbCustom.Checked = $true } else { $rbEasy.Checked = $true }
        if ($r.where) {
            if ($r.where.mode -eq "wsl") { $rbWsl.Checked = $true } else { $rbRemote.Checked = $true }
            $tbIp.Text = [string]$r.where.ip; $tbUser.Text = [string]$r.where.user
            if ($r.where.engine -eq "engine") { $rbEngine.Checked = $true } else { $rbDesktop.Checked = $true }
            $cbKeep.Checked = ($r.where.keep_running -ne "off")
        }
        if ($r.owner) {
            $tbName.Text = [string]$r.owner.first_name; $tbEmail.Text = [string]$r.owner.email
            if ($r.owner.web_name_kept -eq $false) { $tbWebName.Text = [string]$r.owner.web_name; $rbNewName.Checked = $true } else { $rbKeepName.Checked = $true }
        }
        foreach ($k in $PartOrder) { $PartBoxes[$k].Checked = ((@($r.parts_now) + @($r.parts_later)) -contains $k) }
        $tbSmtpUser.Text = [string]$r.email_user
        if ($r.pdf_guides) {
            $script:pdfTouched = $true
            switch ([string]$r.pdf_guides) { "ko" { $rbPdfKo.Checked = $true } "both" { $rbPdfBoth.Checked = $true } "none" { $rbPdfNone.Checked = $true } default { $rbPdfEn.Checked = $true } }
        }
        $script:status = [string]$r.status
        Where-Changed
    } finally { $script:loading = $false }
    $lbResumed.Text = "Carried on from where you stopped ($($r.updated)). Passwords and keys are never saved: type them again."
    $go = [string]$r.page
    if ($script:status -eq "installing") { $go = "install" }
    return $go
}

# ---------- the wizard's private file: passwords and keys for the install -------------------------
# Only this Windows user may read it (no inherited permissions). Garden-Menu.ps1
# hands it to the garden computer through a program's input and deletes it.
function Write-Secrets {
    $pairs = [ordered]@{}
    if ($rbCustom.Checked) {
        if ($tbPw.Text) { $pairs["GARDEN_PASSWORD"] = $tbPw.Text }
        if ($tbDbPw.Text) { $pairs["DB_PASSWORD"] = $tbDbPw.Text }
        if ($PartBoxes["wiki"].Checked -and $tbWikiPw.Text) { $pairs["WIKI_ADMIN_PASSWORD"] = $tbWikiPw.Text }
        if ($PartBoxes["keys"].Checked) {
            if ($tbGemini.Text) { $pairs["GEMINI_API_KEY"] = ($tbGemini.Text -replace '\s', '') }
            if ($tbOpenRouter.Text) { $pairs["OPENROUTER_API_KEY"] = ($tbOpenRouter.Text -replace '\s', '') }
        }
        if ($PartBoxes["plantdata"].Checked -and $tbPerenual.Text) { $pairs["PERENUAL_KEY"] = ($tbPerenual.Text -replace '\s', '') }
        if ($PartBoxes["email"].Checked -and $tbSmtpPw.Text) { $pairs["SMTP_PASSWORD"] = ($tbSmtpPw.Text -replace '\s', '') }
    }
    if ($pairs.Count -eq 0) { return "" }
    $f = Join-Path $env:TEMP ("garden-wizard-" + [guid]::NewGuid().ToString("N") + ".txt")
    [IO.File]::WriteAllText($f, "")
    $acl = New-Object Security.AccessControl.FileSecurity
    $acl.SetAccessRuleProtection($true, $false)
    $me = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($me, "FullControl", "Allow")))
    Set-Acl -Path $f -AclObject $acl
    $text = (($pairs.Keys | ForEach-Object { $_ + "=" + ($pairs[$_] -replace "[`r`n]", "") }) -join "`n") + "`n"
    [IO.File]::WriteAllText($f, $text, (New-Object Text.UTF8Encoding($false)))
    return $f
}
# An old private file left by a wizard that never got to the install: removed.
Get-ChildItem $env:TEMP -Filter "garden-wizard-*.txt" -ErrorAction SilentlyContinue |
    Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-1) } | Remove-Item -Force -ErrorAction SilentlyContinue

# ---------- starting the install and watching its progress ------------------------------------------
# The command for the black window: the menu's install, with every answer.
# The garden's folder and Ubuntu name on the garden computer (as the menu makes them).
$GardenDir = "junsgarden-for-jun"
# The web name the install uses: the one kept, or the new one typed.
function Web-Name { if ($rbNewName.Checked -and $tbWebName.Text) { return $tbWebName.Text } else { return $curName } }
function Install-Command($secrets) {
    $custom = $rbCustom.Checked
    $a = "-NoProfile -ExecutionPolicy Bypass -File `"$Menu`" -Action install -Wizard -Level $(if ($custom) { 'custom' } else { 'easy' }) -Lang $(Lang-Code)"
    $a += " -Name $($tbName.Text.Trim()) -Email $($tbEmail.Text.Trim()) -WebName $(Web-Name)"
    if ($custom) {
        $parts = @(Chosen $PartsNow | ForEach-Object { $_[0] } | Where-Object { $_ -in @("beds", "wiki", "n8n") })
        $a += " -Parts `"$($parts -join ',')`""
        if ($PartBoxes["email"].Checked) { $a += " -SmtpUser $($tbSmtpUser.Text)" }
        if ($PartBoxes["network"].Checked -and $PartBoxes["network"].Enabled) { $a += " -ReserveIp" }
    }
    if ($rbRemote.Checked) { $a += " -Mode remote -RemoteHost $($tbIp.Text.Trim()) -RemoteUser $($tbUser.Text.Trim())" }
    else { $a += " -Mode wsl -Engine $(if ($rbDesktop.Checked) { 'desktop' } else { 'engine' }) -KeepRunning $(if ($cbKeep.Checked) { 'on' } else { 'off' })" }
    $a += " -Pdfs $(Pdf-Code)"
    if ($secrets) { $a += " -SecretsFile `"$secrets`"" }
    return $a
}
# The Ubuntu inside Windows that runs the garden: the one the menu saved
# (.garden-target-windows.txt, "distro="), or the one the menu makes.
function Wsl-Distro {
    $f = Join-Path $Root ".garden-target-windows.txt"
    $d = (Get-Content $f -ErrorAction SilentlyContinue | Where-Object { $_ -match '^distro=(.+)$' } | ForEach-Object { $Matches[1].Trim() } | Select-Object -First 1)
    if ($d) { return $d } else { return "JunsGarden" }
}
# The install's progress file on the garden computer (one line per step started or done).
function Read-Progress {
    try {
        if ($rbRemote.Checked) {
            return (& ssh.exe -o BatchMode=yes -o ConnectTimeout=4 "$($tbUser.Text.Trim())@$($tbIp.Text.Trim())" "cat ~/$GardenDir/logs/install-progress.txt" 2>$null)
        }
        return (& wsl.exe -d (Wsl-Distro) -- bash -c "cat ~/$GardenDir/logs/install-progress.txt" 2>$null)
    } catch { return $null }
}
$script:progressSeen = 0
$timer = New-Object Windows.Forms.Timer; $timer.Interval = 5000
$script:installing = $false
$btnGo.add_Click({
    $secrets = Write-Secrets
    $a = Install-Command $secrets
    # Typed passwords and keys are not kept in this window any longer.
    foreach ($tb in @($tbPw, $tbPw2, $tbDbPw, $tbWikiPw, $tbWikiPw2, $tbGemini, $tbOpenRouter, $tbPerenual, $tbSmtpPw)) { $tb.Text = "" }
    $script:status = "installing"; Save-Record
    # A fresh start: the black window writes RUNNING, then RESTART, FAILED or DONE.
    Remove-Item (Join-Path $Root "logs\windows-install-status.txt") -ErrorAction SilentlyContinue
    $script:progressSeen = @(Read-Progress).Count     # what an earlier run wrote is not this run's news
    # -NoExit keeps the black window open at the end, so you can read it.
    Start-Process powershell.exe -ArgumentList ("-NoExit " + $a)
    $btnGo.Enabled = $false; $script:installing = $true; $btnBack.Enabled = $false; Draw-Steps
    $lbState.Text = "The install is running in the black window. Steps appear here as they finish."
    $timer.Start()
})
# Every 5 seconds, read the install's progress file from the garden computer.
$timer.add_Tick({
    # First, what the black window says (on this PC).
    $status = (Get-Content (Join-Path $Root "logs\windows-install-status.txt") -ErrorAction SilentlyContinue | Select-Object -First 1)
    if ($status -eq "RESTART") { $timer.Stop(); $script:installing = $false; $btnGo.Enabled = $true
        $lbState.Text = "Windows needs one restart. After it, double-click Start-Here-Windows.cmd again: the wizard carries on here."; return }
    if ($status -eq "FAILED") { $script:installing = $false; $btnGo.Enabled = $true; $script:status = "failed"; Save-Record
        $lbState.Text = "Something stopped. Read the black window, fix what it says, and click Start again (type any passwords again first: Back)." }
    if ($status -eq "DONE") { $timer.Stop(); $script:installing = $false; $script:status = "done"; $lbState.Text = "All done."; Show-Page "done"; return }
    $txt = @(Read-Progress)
    if (-not $txt) { return }
    $list.Items.Clear()
    foreach ($line in ($txt | Select-Object -Last 16)) { [void]$list.Items.Add($line) }
    # Only lines written since Start count: an earlier run's "FAILED" line is history.
    $new = @($txt | Select-Object -Skip $script:progressSeen)
    if (-not $new) { return }
    $last = ($new | Select-Object -Last 1)
    if ($last -match 'DONE\s+step\s+14/14') { $lbState.Text = "All 14 steps done. In the black window: the last questions, until it says it is finished." }
    elseif ($last -match 'FAILED') { $lbState.Text = "A step stopped. Read the black window, fix what it says, and click Start again."; $btnGo.Enabled = $true }
    elseif ($status -ne "FAILED") { $lbState.Text = "The install is running in the black window. Steps appear here as they finish." }
})
$form.add_FormClosing({ Save-Record })

# ---------- open: earlier answers back in place, then the page it stopped at ---------------------
$start = Load-Record
$form.add_Shown({ Show-Page $start; Fit-Page $Pages[$script:page].Panel })
[void]$form.ShowDialog()
