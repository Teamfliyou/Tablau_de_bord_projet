<#
  Compile le serveur MCP (tdb-mcp) en release et le place là où Tauri l'attend pour l'intégrer
  à l'installateur (bundle.externalBin = "binaries/tdb-mcp") : src-tauri/binaries/tdb-mcp-<cible>.exe.
  L'installateur NSIS le dépose ensuite à côté de TableauDeBordEtudiant.exe sous le nom tdb-mcp.exe.
  Usage : ./scripts/preparer-serveur-mcp.ps1
#>
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$cible = ((rustc -vV) | Select-String '^host: (.+)$').Matches.Groups[1].Value
if (-not $cible) { throw "Cible Rust introuvable (rustc -vV)" }

cargo build -p tdb-mcp --release --locked
if ($LASTEXITCODE -ne 0) { throw "Échec de la compilation du serveur MCP" }

$source = "target/release/tdb-mcp.exe"
if (-not (Test-Path $source)) { throw "Serveur MCP compilé introuvable : $source" }
New-Item -ItemType Directory -Force src-tauri/binaries | Out-Null
$destination = "src-tauri/binaries/tdb-mcp-$cible.exe"
Copy-Item $source $destination -Force
Write-Host "Serveur MCP prêt pour l'installateur : $destination"
