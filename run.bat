@echo off
REM Simple batch wrapper for Windows that calls the PowerShell run script

echo Running RecogNotes...
powershell -ExecutionPolicy Bypass -File "%~dp0run.ps1"
