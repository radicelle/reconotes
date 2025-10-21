@echo off
REM Simple batch wrapper for Windows that calls the PowerShell script

echo Running build script...
powershell -ExecutionPolicy Bypass -File "%~dp0build.ps1"
