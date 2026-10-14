# Pictures of each page of the real Windows guided installer (Garden-Setup.ps1).
# Runs the real script, but instead of waiting for clicks it draws each page
# to a PNG and closes. Nothing is installed.
$pkg = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)   # the package folder (two levels up)
$out = Join-Path $pkg "START-HERE\images"
New-Item -ItemType Directory -Force $out | Out-Null
$src = Get-Content (Join-Path $pkg "windows\Garden-Setup.ps1") -Raw
$shoot = @'
$names = @("wizard-1-welcome", "wizard-2-where", "wizard-3-about-you", "wizard-4-what-gets-installed", "wizard-5-install", "wizard-6-done")
$form.add_Shown({
    $tbEmail.Text = "you@gmail.com"
    $rbWsl.Text = "This Windows PC (MY-PC, 192.168.1.23), inside Windows' own Ubuntu (WSL)"   # a sample, not this PC
    for ($i = 0; $i -lt $pages.Count; $i++) {
        Show-Page $i
        [Windows.Forms.Application]::DoEvents()
        $bmp = New-Object Drawing.Bitmap($form.Width, $form.Height)
        $form.DrawToBitmap($bmp, (New-Object Drawing.Rectangle(0, 0, $form.Width, $form.Height)))
        $bmp.Save((Join-Path $OUTDIR ($names[$i] + ".png")), [Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
    }
    # Page 2 again with "This Windows PC" ticked: the choice of Docker shows.
    Show-Page 1; $rbWsl.Checked = $true
    [Windows.Forms.Application]::DoEvents()
    $bmp = New-Object Drawing.Bitmap($form.Width, $form.Height)
    $form.DrawToBitmap($bmp, (New-Object Drawing.Rectangle(0, 0, $form.Width, $form.Height)))
    $bmp.Save((Join-Path $OUTDIR "wizard-2b-this-pc.png"), [Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
    $form.Close()
})
[void]$form.ShowDialog()
'@
$src = $src.Replace('[void]$form.ShowDialog()', $shoot.Replace('$OUTDIR', "'$out'"))
# Run it as if it lived in windows\ (it finds its files from there).
$tmp = Join-Path $pkg "windows\_shoot-tmp.ps1"
Set-Content $tmp $src -Encoding UTF8
try { & powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File $tmp } finally { Remove-Item $tmp -Force }
Get-ChildItem $out -Filter "wizard-*.png" | Select-Object Name, Length
