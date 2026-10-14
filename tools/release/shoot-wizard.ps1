# Pictures of the pages of the real Windows setup wizard (Garden-Setup.ps1).
# Runs the real script with sample answers (the sample answers: Jun, jun@example.com), but instead of
# waiting for clicks it draws each page to a PNG and closes. Nothing is installed,
# and nothing is saved (no garden-setup.json).
# It also writes clean-pictures\wizard-marks.json: where the buttons to click are
# on each picture, so mark_pictures.py rings the right places.
$pkg = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)   # the package folder (two levels up)
$out = Join-Path $pkg "START-HERE\images"
$marks = Join-Path $PSScriptRoot "clean-pictures\wizard-marks.json"
New-Item -ItemType Directory -Force $out | Out-Null
$src = Get-Content (Join-Path $pkg "windows\Garden-Setup.ps1") -Raw
$shoot = @'
function Save-Record { }                     # pictures only: nothing is saved
$script:marks = [ordered]@{}
# Where a control is on the picture (the whole window, title bar included).
function Box($c) {
    $r = $c.RectangleToScreen($c.ClientRectangle)
    return @(($r.Left - $form.Left), ($r.Top - $form.Top), ($r.Right - $form.Left), ($r.Bottom - $form.Top))
}
function Shoot($name, $rings) {
    [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 150; [Windows.Forms.Application]::DoEvents()
    $bmp = New-Object Drawing.Bitmap($form.Width, $form.Height)
    $form.DrawToBitmap($bmp, (New-Object Drawing.Rectangle(0, 0, $form.Width, $form.Height)))
    $bmp.Save((Join-Path $OUTDIR ($name + ".png")), [Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
    if ($rings[0] -is [Windows.Forms.Control]) { $rings = , $rings }   # one ring: PowerShell unpacked the list
    $list = @(); foreach ($g in $rings) { $list += , (@(Box $g[0]) + @($g[1], $g[2])) }
    $script:marks[$name + ".png"] = $list
}
$form.add_Shown({
    $form.Size = New-Object Drawing.Size((Px 1040), (Px 720)); $form.Location = New-Object Drawing.Point(20, 20)
    $rbWsl.Text = "This Windows PC (MY-PC, 192.168.1.23), inside Windows' own Ubuntu (WSL)"   # a sample, not this PC
    $tbIp.Text = "192.168.1.50"; $tbUser.Text = "jun"
    $tbName.Text = "Jun"; $tbEmail.Text = "jun@example.com"
    Show-Page "welcome"; Shoot "wizard-1-welcome" @(@($btnNext, "click Next", "above"), @($btnHelp, "the guide, at this page", "above"), @($cbLang, "the guides' language", "right"))
    Show-Page "level"; Shoot "wizard-1b-level" @(@($rbEasy, "automatic", "right"), @($rbCustom, "page by page", "right"), @($btnNext, "then Next", "above"))
    $rbRemote.Checked = $true; Show-Page "where"
    Shoot "wizard-2-where" @(@($tbIp.Parent, "its address, or Find it", "right"), @($tbUser.Parent, "your user name there", "right"), @($btnNext, "then Next", "above"))
    $rbWsl.Checked = $true; Show-Page "where"
    Shoot "wizard-2b-this-pc" @(@($rbDesktop, "the easy way", "above"), @($btnNext, "then Next", "above"))
    $rbRemote.Checked = $true
    Show-Page "you"; Shoot "wizard-3-about-you" @(@($btnNext, "check, then Next", "above"))
    $rbCustom.Checked = $true
    foreach ($k in "wiki", "email", "keys", "frontdoor", "nas") { $PartBoxes[$k].Checked = $true }
    Show-Page "parts"; Shoot "wizard-4-optional-software" @(@($PartBoxes["email"], "tick what you want", "right"), @($hotTags, "#words: the guide", "below"))
    $tbSmtpUser.Text = "jun@example.com"; $tbSmtpPw.Text = "abcdefghijklmnop"
    Show-Page "part-email"; Shoot "wizard-4b-a-part-page" @(@($tbSmtpPw.Parent, "paste the 16 letters", "right"), @($btnHelp, "stuck? Help", "above"))
    $tbPw.Text = "sample-password"; $tbPw2.Text = "sample-password"
    Show-Page "passwords"
    foreach ($b in @(All-Controls $pPw | Where-Object { $_ -is [Windows.Forms.Button] })) { $b.PerformClick() }   # open "More options"
    Shoot "wizard-4c-passwords" @(@($tbPw.Parent, "10 or more characters", "right"), @($tbDbPw.Parent, "empty = made for you", "right"))
    Show-Page "pdfs"; Shoot "wizard-4d-pdf-guides" @(@($rbPdfEn, "English, Korean or both", "right"), @($rbPdfNone, "or not now", "right"), @($btnNext, "then Next", "above"))
    Show-Page "summary"; Shoot "wizard-4e-summary" @(@($btnNext, "Looks right", "above"))
    Show-Page "install"; Shoot "wizard-5-install" @(@($btnGo, "click to start", "right"))
    Show-Page "done"; Shoot "wizard-6-done" @(@($btnOpen.Parent, "your garden, or the menu", "right"))
    [IO.File]::WriteAllText($MARKS, ($script:marks | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding($false)))
    $form.Close()
})
[void]$form.ShowDialog()
'@
$src = $src.Replace('[void]$form.ShowDialog()', $shoot.Replace('$OUTDIR', "'$out'").Replace('$MARKS', "'$marks'"))
# Run it as if it lived in windows\ (it finds its files from there).
$tmp = Join-Path $pkg "windows\_shoot-tmp.ps1"
Set-Content $tmp $src -Encoding UTF8
try { & powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File $tmp } finally { Remove-Item $tmp -Force }
Get-ChildItem $out -Filter "wizard-*.png" | Select-Object Name, Length
