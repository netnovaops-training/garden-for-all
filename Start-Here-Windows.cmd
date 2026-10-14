@echo off
rem ===========================================================================
rem  Jun's Garden for Windows: the GUIDED INSTALL. Double-click this file.
rem
rem  It opens a window with Next / Back buttons (windows\Garden-Setup.ps1).
rem  It asks where the garden should run, shows what will be installed with
rem  links to where each program comes from, then starts the install.
rem
rem  For everyday jobs afterwards, double-click Garden-Menu-Windows.cmd.
rem
rem  This is a plain text file: right-click it, "Edit" (or open it in
rem  Notepad) to read it. "-ExecutionPolicy Bypass" lets Windows run the
rem  installer script that came in the zip. It applies to that one script
rem  only and changes no Windows settings. "-STA" is needed for windows and
rem  buttons in PowerShell.
rem ===========================================================================
title Garden Bridge: from Windows to Linux
cd /d "%~dp0"
powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File "%~dp0windows\Garden-Setup.ps1"
