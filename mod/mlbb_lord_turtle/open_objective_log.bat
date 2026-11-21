@echo off
set "LORD_PROBE_FILE=%APPDATA%\TeamSamoyed\TeamfightManager2\data\mlbb_lord_turtle_objective_probe.log"
if exist "%LORD_PROBE_FILE%" (
  explorer.exe /select,"%LORD_PROBE_FILE%"
) else (
  echo Objective log not found. Enable this mod and restart the game first.
  pause
)
