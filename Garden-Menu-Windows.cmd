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
rem ===========================================================================
title Jun's Garden
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0windows\Garden-Menu.ps1" %*
echo.
pause
