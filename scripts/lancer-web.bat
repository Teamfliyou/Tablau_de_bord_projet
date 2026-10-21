@echo off
rem Lance la version Web en local et l'ouvre dans le navigateur (http://localhost:8000).
rem Compile le frontend et le serveur si besoin. Prerequis : Rust (rustup) et Trunk.
setlocal
cd /d "%~dp0.."

where cargo >nul 2>&1 || (echo Rust est introuvable : installez-le depuis https://rustup.rs & pause & exit /b 1)
where trunk >nul 2>&1 || (echo Trunk est introuvable : lancez  cargo install trunk --version 0.22.0 --locked & pause & exit /b 1)

if not exist "crates\frontend\dist\index.html" (
  echo [1/2] Compilation du frontend...
  rustup target add wasm32-unknown-unknown
  pushd crates\frontend
  trunk build --release || (popd & echo Echec de la compilation du frontend. & pause & exit /b 1)
  popd
)

echo [2/2] Demarrage du serveur sur http://localhost:8000  (Ctrl+C pour arreter)
start "" /b cmd /c "timeout /t 4 /nobreak >nul & start http://localhost:8000"
cargo run -p tdb-server --release
pause
