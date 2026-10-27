@echo off
rem Compile l'application Windows native (Tauri) : exe portable + installateur NSIS.
rem Prerequis : Rust (cible MSVC), "Build Tools for Visual Studio" (charge Developpement Desktop en C++), Trunk et la CLI Tauri.
setlocal
cd /d "%~dp0.."

where cargo >nul 2>&1 || (echo Rust est introuvable : installez-le depuis https://rustup.rs & pause & exit /b 1)
where link >nul 2>&1 || (echo L'editeur de liens MSVC est introuvable : installez "Build Tools for Visual Studio" avec la charge "Developpement Desktop en C++", puis lancez ce script depuis "x64 Native Tools Command Prompt". & pause & exit /b 1)
where trunk >nul 2>&1 || (echo Trunk est introuvable : lancez  cargo install trunk --version 0.22.0 --locked & pause & exit /b 1)
where cargo-tauri >nul 2>&1 || (echo La CLI Tauri est introuvable : lancez  cargo install tauri-cli --version "^2" --locked & pause & exit /b 1)

rustup target add wasm32-unknown-unknown
rustup default stable-x86_64-pc-windows-msvc
rem Serveur MCP pour Hermes Agent : compile puis copie dans src-tauri\binaries pour l'installateur.
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\preparer-serveur-mcp.ps1 || (echo Echec de la compilation du serveur MCP. & pause & exit /b 1)
rem Build local : pas de cle de signature des mises a jour (reservee a la CI), donc pas d'artefacts de mise a jour.
cargo tauri build --bundles nsis --config "{\"bundle\":{\"createUpdaterArtifacts\":false,\"externalBin\":[\"binaries/tdb-mcp\"]}}" || (echo Echec de la compilation. & pause & exit /b 1)

echo.
echo Termine :
echo   target\release\TableauDeBordEtudiant.exe
echo   target\release\tdb-mcp.exe  (serveur MCP, integre a l'installateur)
echo   target\release\bundle\nsis\
pause
