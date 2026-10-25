<#
 Test-only: drives the REAL setup wizard (windows\Garden-Setup.ps1)
 in a real window, from a throwaway copy, without installing anything:
   1. EASY: the short path, and the install command it builds.
   2. CUSTOM: every page (optional software, a page per part, passwords),
      the checks that stop Next, Back keeping the answers, a bigger and a smaller
      window, the summary, the private file of passwords and keys (its content,
      its permissions), garden-setup.json (no secret in it), and the command.
   3. Close and open again: it carries on at the same page with the answers.
   Every page's Help, in English and Korean, must point at a stable help anchor
   that exists in the built handbook (help-ids.tsv).
 Start-Process is replaced by a stand-in that only records the command; message
 boxes by stand-ins that record the message.
   powershell -ExecutionPolicy Bypass -File tools\release\tests\test-wizard-flow.ps1
 Needs Python with the "markdown" package (to build the handbook into the copy).
#>
$pkg = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $PSScriptRoot))
$tmp = Join-Path $env:TEMP ("wizard-flow-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force (Join-Path $tmp "site\apps") | Out-Null
Copy-Item -Recurse (Join-Path $pkg "windows") $tmp
Copy-Item -Recurse (Join-Path $pkg "START-HERE") $tmp
Copy-Item -Recurse (Join-Path $pkg "lang") $tmp   # the Korean lines, shared with garden.sh
Copy-Item (Join-Path $pkg "site\apps\version.json") (Join-Path $tmp "site\apps\")
# The handbook, built into the copy, so the Help anchors can be checked.
$env:PYTHONIOENCODING = "utf-8"
& python (Join-Path $pkg "tools\release\make_handbook.py") $tmp | Out-Null
$results = Join-Path $tmp "results.txt"
$PW = "Garden-test-" + [guid]::NewGuid().ToString("N").Substring(0, 10)   # made up here, never a real one
$KEYS = @{ wiki = "Wiki-test-pass-123"; gemini = "AIzaTESTKEY0123456789"; perenual = "sk-perenual-test-42"; smtp = "abcd efgh ijkl mnop" }

function Harness($name, $drive) {
    $src = Get-Content (Join-Path $tmp "windows\Garden-Setup.ps1") -Raw
    $stub = 'function Start-Process { $script:launched = ($args -join " ") }' + "`n"
    $src = $src.Replace('$Root = Split-Path -Parent $PSScriptRoot', $stub + '$Root = Split-Path -Parent $PSScriptRoot')
    $pre = @'
function Show-Problem($m) { $script:problems += $m }
function Ask-YesNo($m) { $script:asked += $m; return $script:answer }
$script:problems = @(); $script:asked = @(); $script:answer = $false
$form.add_Shown({
    $out = New-Object System.Collections.Generic.List[string]
    function Check-It($what, $ok) { $out.Add($(if ($ok) { "PASS  $what" } else { "FAIL  $what" })) }
    function Go-Next { $script:problems = @(); $btnNext.PerformClick(); [Windows.Forms.Application]::DoEvents() }
'@
    $post = @'
    [IO.File]::AppendAllLines('RESULTS', $out)
    $form.Close()
})
[void]$form.ShowDialog()
'@
    $src = $src.Replace('[void]$form.ShowDialog()', ($pre + "`n" + $drive + "`n" + $post).Replace('RESULTS', $results).Replace('THEPW', $PW).
        Replace('KWIKI', $KEYS.wiki).Replace('KGEMINI', $KEYS.gemini).Replace('KPERENUAL', $KEYS.perenual).Replace('KSMTP', $KEYS.smtp))
    $h = Join-Path $tmp "windows\_test-$name.ps1"
    Set-Content $h $src -Encoding UTF8
    $p = Start-Process powershell.exe -ArgumentList "-NoProfile -STA -ExecutionPolicy Bypass -File `"$h`"" -PassThru -WindowStyle Normal
    if (-not $p.WaitForExit(120000)) { $p.Kill(); Add-Content $results "FAIL  $name : the window did not finish" }
}

# ---- 1. EASY, and the help on every page ----------------------------------------------
Harness "easy" @'
    Check-It "it opens on the welcome page" ($script:page -eq "welcome")
    Check-It "EASY: 7 steps in the list (welcome, level, where, you, summary, install, done)" ((Get-Path).Count -eq 7 -and $steps.Controls.Count -eq 7)
    Check-It "the welcome page's hot words are shown, with an overview" ($hotTags.Controls.Count -ge 2 -and ($hotTags.Controls | ForEach-Object Text) -contains "#wizard" -and $hotBrief.Text.Length -gt 10)
    $missing = @($Pages.Keys | Where-Object { (Help-Rows $_).Count -eq 0 })
    Check-It "every page has a help id in help-ids.tsv ($($Pages.Count) pages)" ($missing.Count -eq 0)
    if ($missing.Count) { $out.Add("      no help id: " + ($missing -join ", ")) }
    $bad = @()
    foreach ($lang in 0, 1) {
        $cbLang.SelectedIndex = $lang
        foreach ($k in $Pages.Keys) {
            $t = Help-Target $k
            if ($t -notmatch '^file:.*/handbook/([^#]+)#(.+)$') { $bad += "$k ($lang): $t"; continue }
            $html = [IO.File]::ReadAllText((Join-Path $Root ("handbook\" + $Matches[1])), [Text.Encoding]::UTF8)
            if (-not $html.Contains('id="' + [Uri]::UnescapeDataString($Matches[2]) + '"')) { $bad += "$k ($lang): #$($Matches[2])" }
        }
    }
    Check-It "every page's Help opens an anchor that is in the handbook, English and Korean" ($bad.Count -eq 0)
    foreach ($b in $bad) { $out.Add("      " + $b) }
    $cbLang.SelectedIndex = 1; Update-HotWords "where"
    Check-It "Korean hot words when Korean is chosen" (($hotTags.Controls | ForEach-Object Text) -contains "#wsl" -and $hotBrief.Text -match '[가-힣]')
    Check-It "the Korean help goes to the -ko anchor" ((Help-Target "where") -like "*#help-where-ko")
    $cbLang.SelectedIndex = 0
    Go-Next; Check-It "Next: the level page" ($script:page -eq "level")
    Go-Next; Check-It "Next: where" ($script:page -eq "where")
    $rbWsl.Checked = $true; [Windows.Forms.Application]::DoEvents()
    Check-It "on this PC the router step cannot be ticked" (-not $PartBoxes["network"].Enabled)
    Go-Next; Check-It "Next: about you" ($script:page -eq "you")
    Check-It "Jun's name is filled in, and the web name is kept" ($tbName.Text -eq "Jun" -and $rbKeepName.Checked -and $rbKeepName.Text -like "*junsgarden.zapto.org*")
    $tbName.Text = ""; Go-Next; Check-It "an empty name stops Next, with a message" ($script:page -eq "you" -and $script:problems.Count -eq 1)
    $tbName.Text = "Jun"; $tbEmail.Text = "jun@example.com"
    Go-Next; Check-It "EASY goes straight to the summary" ($script:page -eq "summary")
    Check-It "the summary says EASY and that the password is asked in the install window" ($tbSummary.Text -match "EASY: automatic" -and $tbSummary.Text -match "asked in the install window")
    Go-Next; Check-It "Looks right: the install page" ($script:page -eq "install")
    $btnGo.PerformClick(); $timer.Stop(); [Windows.Forms.Application]::DoEvents()
    $c = $script:launched
    Check-It "EASY's command: -Wizard -Level easy, Jun, the kept web name, this PC, no parts and no private file" ($c -match '-Action install -Wizard -Level easy -Lang en' -and $c -match '-Name Jun -Email jun@example\.com -WebName junsgarden\.zapto\.org' -and $c -match '-Mode wsl -Engine desktop' -and $c -notmatch '-Parts' -and $c -notmatch '-SecretsFile')
    Check-It "EASY fetches the PDF guides in the language of the first page, without a page of its own (-Pdfs en)" ($c -match '-Pdfs en' -and -not ((Get-Path) -contains "pdfs"))
    Check-It "the summary of EASY says which PDF guides arrive" ($tbSummary.Text -match "PDF GUIDES     English")
    # The overview is the first thing shown when the install starts, and it is in the handbook.
    $menuSrc = [IO.File]::ReadAllText((Join-Path $Root "windows\Garden-Menu.ps1"))
    $iOv = $menuSrc.IndexOf("    Open-Overview   #"); $iPdf = $menuSrc.IndexOf("    Start-Pdfs      #"); $iPrep = $menuSrc.IndexOf("Prepare-Wsl)", $iOv)
    Check-It "Garden-Menu opens the overview first when the install starts (before the PDFs start and before anything is prepared)" ($iOv -gt 0 -and $iOv -lt $iPdf -and $iPdf -lt $iPrep)
    Check-It "the overview is a Markdown guide in the program and a page in the handbook (English and Korean)" ((Test-Path (Join-Path $Root "START-HERE\00-OVERVIEW.md")) -and (Test-Path (Join-Path $Root "START-HERE\ko\00-OVERVIEW.md")) -and (Test-Path (Join-Path $Root "handbook\00-OVERVIEW.html")))
    $out.Add("COMMAND-EASY  " + ($c -replace '-File "[^"]*"', '-File "...\Garden-Menu.ps1"'))
'@

# ---- 2. CUSTOM, every page --------------------------------------------------------------------
Remove-Item (Join-Path $tmp "garden-setup.json") -ErrorAction SilentlyContinue
Harness "custom" @'
    $w0 = $tbSummary.Width
    Show-Page "level"; $rbCustom.Checked = $true; [Windows.Forms.Application]::DoEvents()
    Check-It "CUSTOM adds the optional-software and passwords pages, no visitors page" ((Get-Path) -contains "parts" -and (Get-Path) -contains "passwords" -and -not ((Get-Path) -contains "community"))
    Go-Next; $rbRemote.Checked = $true; $tbIp.Text = "192.168.1.50"; $tbUser.Text = "jun"
    Go-Next; $tbName.Text = "Jun"; $tbEmail.Text = "jun@example.com"; $tbWebName.Text = "HTTPS://www.Moms.Example.org/x"; $tbFolder.Text = "/garden/"
    Go-Next; Check-It "the optional software page" ($script:page -eq "parts" -and $tbWebName.Text -eq "moms.example.org" -and $rbNewName.Checked)
    Check-It "the router step can be ticked for another computer" ($PartBoxes["network"].Enabled)
    foreach ($k in "wiki", "email", "keys", "plantdata", "network", "frontdoor", "tailscale") { $PartBoxes[$k].Checked = $true }
    [Windows.Forms.Application]::DoEvents()
    $want = @("welcome", "level", "where", "you", "parts", "part-beds", "part-wiki", "part-email", "part-keys", "part-plantdata", "part-network", "part-frontdoor", "part-tailscale", "passwords", "pdfs", "summary", "install", "done")
    Check-It "one page for each ticked part, then the PDF guides, in order (18 steps)" (((Get-Path) -join ",") -eq ($want -join ","))
    Check-It "the step list follows straight away" ($steps.Controls.Count -eq 18)
    Go-Next; Go-Next; Check-It "garden tools, then the wiki page" ($script:page -eq "part-wiki")
    $tbWikiPw.Text = "short"; Go-Next
    Check-It "a short wiki password stops Next" ($script:page -eq "part-wiki" -and $script:problems.Count -eq 1)
    $tbWikiPw.Text = "KWIKI"; $tbWikiPw2.Text = "KWIKI"; Go-Next
    Check-It "the email page, with your email filled in" ($script:page -eq "part-email" -and $tbSmtpUser.Text -eq "jun@example.com")
    $tbSmtpPw.Text = "not sixteen"; Go-Next
    Check-It "a wrong app password stops Next" ($script:page -eq "part-email")
    $tbSmtpPw.Text = "KSMTP"; Go-Next
    $tbGemini.Text = "KGEMINI"; Go-Next
    $tbPerenual.Text = "KPERENUAL"; Go-Next
    Check-It "the router page, then the later parts' pages" ($script:page -eq "part-network")
    Go-Next; Check-It "a later part's page says where it is in the menu" ($script:page -eq "part-frontdoor" -and (@(All-Controls $Pages["part-frontdoor"].Panel | Where-Object { $_.Text -like "*More, then 1*" }).Count -eq 1))
    Go-Next; Go-Next; Check-It "the passwords page" ($script:page -eq "passwords")
    $tbPw.Text = "THEPW"; $tbPw2.Text = "different!!"; Go-Next
    Check-It "two different passwords stop Next" ($script:page -eq "passwords")
    $tbPw2.Text = "THEPW"; $tbDbPw.Text = "bad pass"; Go-Next
    Check-It "a database password with spaces stops Next" ($script:page -eq "passwords")
    Show-Page "part-keys"
    Check-It "Back: the keys are still there" ($tbGemini.Text -eq "KGEMINI")
    Show-Page "passwords"
    Check-It "and the password too" ($tbPw.Text -eq "THEPW")
    $tbDbPw.Text = ""; Go-Next
    Check-It "the PDF guides page, starting on the language of the first page (English)" ($script:page -eq "pdfs" -and $rbPdfEn.Checked)
    $rbPdfBoth.Checked = $true; $script:pdfTouched = $true; Go-Next
    Check-It "the summary" ($script:page -eq "summary")
    $s = $tbSummary.Text
    Check-It "the summary lists the answers and what was answered for you" ($s -match "CUSTOM" -and $s -match "192\.168\.1\.50" -and $s -match "ANSWERED FOR YOU" -and $s -match "moms\.example\.org" -and $s -match "junsgarden-for-jun")
    Check-It "the summary names the PDF guides chosen (English and Korean)" ($s -match "PDF GUIDES     English and Korean")
    Check-It "the summary shows no password or key" (-not ($s.Contains("THEPW") -or $s.Contains("KGEMINI") -or $s.Contains("KWIKI") -or $s.Contains("abcd")))
    # A bigger and a smaller window: the page follows.
    $form.WindowState = "Normal"; $form.Size = New-Object Drawing.Size(1500, 950); [Windows.Forms.Application]::DoEvents(); $wBig = $tbSummary.Width
    $form.Size = $form.MinimumSize; [Windows.Forms.Application]::DoEvents(); $wSmall = $tbSummary.Width
    Check-It "the window can be made bigger and smaller, and the page follows ($wSmall to $wBig)" ($form.FormBorderStyle -eq "Sizable" -and $wBig -gt $wSmall + 300)
    Check-It "the buttons stay big enough for a finger" ($btnNext.Height -ge 44 -and $btnBack.Height -ge 44 -and $btnHelp.Height -ge 44)
    Go-Next
    $btnGo.PerformClick(); $timer.Stop(); [Windows.Forms.Application]::DoEvents()
    $c = $script:launched
    Check-It "the command: CUSTOM, the new web name, the parts, Gmail, the router, another computer, no visitors" ($c -match '-Level custom' -and $c -match '-Parts "beds,wiki"' -and $c -match '-WebName moms\.example\.org' -and $c -match '-SmtpUser jun@example\.com' -and $c -match '-ReserveIp' -and $c -notmatch '-Signups|-AgeCheck' -and $c -match '-Mode remote -RemoteHost 192\.168\.1\.50 -RemoteUser jun')
    Check-It "the command carries the PDF choice (-Pdfs both)" ($c -match '-Pdfs both')
    Check-It "the command carries the folder, without its last slash (-Folder /garden)" ($c -match '-Folder /garden(\s|$)')
    try { $lbl = Both "Folder (like /garden; empty = the top of the web name)" } catch { $lbl = "error: $_" }
    # Hangul syllables, by code (this file is read as ANSI by Windows PowerShell: keep it ASCII).
    $hangul = "[$([char]0xAC00)-$([char]0xD7A3)]"
    Check-It "the folder field is labelled in English and Korean ($($KoTsv.Count) Korean lines)" ($lbl -match " / .*$hangul")
    Check-It "no password or key on the command line" (-not ($c.Contains("THEPW") -or $c.Contains("KGEMINI") -or $c.Contains("KWIKI")))
    $sf = $(if ($c -match '-SecretsFile "([^"]+)"') { $Matches[1] } else { "" })
    Check-It "a private file is handed over" ($sf -and (Test-Path $sf))
    if ($sf -and (Test-Path $sf)) {
        $b = [IO.File]::ReadAllBytes($sf); $t = [Text.Encoding]::UTF8.GetString($b)
        Check-It "the private file: name=value lines, no BOM, no Windows line endings" (-not ($b[0] -eq 0xEF) -and -not ($b -contains 13))
        Check-It "the private file holds the password, the wiki password and the keys (app password without spaces)" ($t.Contains("GARDEN_PASSWORD=THEPW`n") -and $t.Contains("WIKI_ADMIN_PASSWORD=KWIKI`n") -and $t.Contains("GEMINI_API_KEY=KGEMINI`n") -and $t.Contains("PERENUAL_KEY=KPERENUAL`n") -and $t.Contains("SMTP_PASSWORD=abcdefghijklmnop`n") -and -not $t.Contains("DB_PASSWORD"))
        $acl = Get-Acl $sf; $me = [Security.Principal.WindowsIdentity]::GetCurrent().User
        $rules = @($acl.GetAccessRules($true, $true, [Security.Principal.SecurityIdentifier]))
        Check-It "only this Windows user can open it (no inherited permissions)" ($acl.AreAccessRulesProtected -and $rules.Count -eq 1 -and $rules[0].IdentityReference -eq $me)
        Remove-Item $sf -Force
    }
    Check-It "the typed passwords and keys are cleared from the window" (-not ($tbPw.Text -or $tbGemini.Text -or $tbWikiPw.Text -or $tbSmtpPw.Text))
    $rec = Join-Path $Root "garden-setup.json"; $j = [IO.File]::ReadAllText($rec)
    $r = $j | ConvertFrom-Json
    Check-It "garden-setup.json: the answers, status installing" ($r.level -eq "custom" -and $r.status -eq "installing" -and $r.owner.first_name -eq "Jun" -and $r.owner.web_name -eq "moms.example.org" -and (@($r.parts_now) -contains "wiki") -and (@($r.parts_later) -contains "tailscale") -and $r.pdf_guides -eq "both")
    Check-It "garden-setup.json holds no password or key" (-not ($j.Contains("THEPW") -or $j.Contains("KGEMINI") -or $j.Contains("KWIKI") -or $j.Contains("KPERENUAL") -or $j.Contains("abcd")))
    $leak = @(Get-ChildItem $Root -Recurse -File | Where-Object { $_.Length -lt 5MB -and $_.Name -notlike "_test-*" -and $_.Name -ne "results.txt" -and ([IO.File]::ReadAllText($_.FullName)).Contains("THEPW") })
    Check-It "no file in the garden folder holds the password" ($leak.Count -eq 0)
    $out.Add("COMMAND-CUSTOM  " + (($c -replace '-File "[^"]*"', '-File "...\Garden-Menu.ps1"') -replace '-SecretsFile "[^"]*"', '-SecretsFile "...\garden-wizard-XXXX.txt"'))
'@

# ---- 3. Close and open again ------------------------------------------------------------------
$rec = Join-Path $tmp "garden-setup.json"
$j = [IO.File]::ReadAllText($rec) -replace '"status":\s*"installing"', '"status": "wizard"' -replace '"page":\s*"[a-z-]+"', '"page": "part-keys"'
[IO.File]::WriteAllText($rec, $j)
Harness "resume" @'
    Check-It "it carries on at the page where it stopped" ($script:page -eq "part-keys")
    Check-It "with the answers back in place" ($rbCustom.Checked -and $rbRemote.Checked -and $tbIp.Text -eq "192.168.1.50" -and $tbName.Text -eq "Jun" -and $rbNewName.Checked -and $tbWebName.Text -eq "moms.example.org" -and $PartBoxes["wiki"].Checked -and $PartBoxes["tailscale"].Checked)
    Check-It "the PDF choice is back too" ($rbPdfBoth.Checked)
    Check-It "but no password or key (never saved)" (-not ($tbPw.Text -or $tbGemini.Text))
    Check-It "and it says so on the first page" ($lbResumed.Text -like "Carried on*")
    Check-It "the step list shows the earlier steps as done" (@($steps.Controls | Where-Object { $_.Text -like ([string][char]0x2713 + "*") }).Count -eq 8)
'@

Get-ChildItem $env:TEMP -Filter "garden-wizard-*.txt" -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-10) } | Remove-Item -Force
if (Test-Path $results) { Get-Content $results -Encoding UTF8 } else { "FAIL  no results were written" }
$lines = @(); if (Test-Path $results) { $lines = Get-Content $results }
$fails = @($lines | Where-Object { $_ -like "FAIL*" }).Count; $passes = @($lines | Where-Object { $_ -like "PASS*" }).Count
"RESULT: $passes passed, $fails failed   (copy: $tmp)"
