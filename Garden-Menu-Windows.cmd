@echo off
rem ===========================================================================
rem  Jun's Garden for Windows: the MENU. Double-click this file.
rem
rem  The same menu as on Ubuntu (install, start, stop, checklist, backups,
rem  settings, users, ...). It runs each job on the garden computer: another
rem  computer over SSH, or this PC's built-in Ubuntu (WSL).
rem  The menu itself is windows\Garden-Menu.ps1; open it in Notepad to read it.
rem
rem  "-ExecutionPolicy Bypass" lets Windows run that one script from the zip.
rem  It changes no Windows settings.
rem  The first time, it gets the program from GitHub (tools\get-code.ps1),
rem  like Start-Here-Windows.cmd.
rem ===========================================================================
title Jun's Garden
cd /d "%~dp0"
if not exist "%~dp0windows\Garden-Menu.ps1" (
    echo   The first time, this gets the program from GitHub and checks every file.
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\get-code.ps1"
    if errorlevel 1 ( echo. & pause & exit /b 1 )
)
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0windows\Garden-Menu.ps1" %*
echo.
pause
REM CI test line (added to prove the fingerprint bot; removed in the next commit)
