@echo off
rem Lance l'application et l'ouvre dans le navigateur. Fermez cette fenetre pour l'arreter.
cd /d "%~dp0"
set "FRONTEND_DIR=%~dp0frontend-dist"
set "DATABASE_PATH=%~dp0data\tableau-de-bord.db"
start "" cmd /c "timeout /t 3 /nobreak >nul & start http://localhost:8000"
TableauDeBord.exe
pause
