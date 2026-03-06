@echo off
setlocal

set "PS_EXE=C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
set "SCRIPT_DIR=%~dp0"

"%PS_EXE%" -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%trigger-gh-nightly-regression.ps1" %*
set "RC=%ERRORLEVEL%"
exit /b %RC%
