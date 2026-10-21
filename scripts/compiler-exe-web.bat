@echo off
rem Cree un dossier portable "release\TableauDeBord-Web" : double-cliquez sur Lancer.bat pour ouvrir l'application dans le navigateur.
rem Prerequis : Rust (rustup) et Trunk.
setlocal
cd /d "%~dp0.."

where cargo >nul 2>&1 || (echo Rust est introuvable : installez-le depuis https://rustup.rs & pause & exit /b 1)
where trunk >nul 2>&1 || (echo Trunk est introuvable : lancez  cargo install trunk --version 0.22.0 --locked & pause & exit /b 1)

echo [1/3] Compilation du frontend (Rust/WebAssembly)...
rustup target add wasm32-unknown-unknown
pushd crates\frontend
trunk build --release || (popd & echo Echec de la compilation du frontend. & pause & exit /b 1)
popd

echo [2/3] Compilation du serveur...
cargo build -p tdb-server --release || (echo Echec de la compilation du serveur. & pause & exit /b 1)

echo [3/3] Assemblage du dossier portable...
set "SORTIE=release\TableauDeBord-Web"
if not exist "%SORTIE%" mkdir "%SORTIE%"
copy /y "target\release\tdb-server.exe" "%SORTIE%\TableauDeBord.exe" >nul
xcopy /e /i /y /q "crates\frontend\dist" "%SORTIE%\frontend-dist" >nul
(
  echo @echo off
  echo cd /d "%%~dp0"
  echo set FRONTEND_DIR=%%~dp0frontend-dist
  echo set DATABASE_PATH=%%~dp0data\tableau-de-bord.db
  echo start "" /b cmd /c "timeout /t 3 /nobreak ^>nul ^& start http://localhost:8000"
  echo TableauDeBord.exe
  echo pause
) > "%SORTIE%\Lancer.bat"

echo.
echo Termine : %CD%\%SORTIE%
echo Double-cliquez sur Lancer.bat pour demarrer l'application dans le navigateur.
pause
