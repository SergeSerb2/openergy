@echo off
rem Runs the Openergy daemon for the logon task and restarts it if it crashes.
rem A clean stop (exit code 0: Ctrl+C, sign-out, "openergy ctl daemon.stop") ends the loop.
set OPENERGY_ALLOW_PORT_24800=1
:run
"%~dp0openergy.exe" daemon --platform windows
if %errorlevel%==0 exit /b 0
ping -n 3 127.0.0.1 >nul
goto run
