@echo off
rem Compatibility launcher. All paths are explicit; no cmd restart loop or port override.
rem Usage: openergy-run.cmd "C:\path\to\data" [windows^|virtual]
if "%~1"=="" exit /b 2
setlocal
set "OE_PLATFORM=%~2"
if "%OE_PLATFORM%"=="" set "OE_PLATFORM=windows"
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0openergy-run.ps1" -InstallDir "%~dp0." -DataDir "%~1" -Platform "%OE_PLATFORM%"
exit /b %errorlevel%
