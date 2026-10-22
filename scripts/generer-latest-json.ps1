<#
  Génère puis valide le manifeste de mise à jour Tauri (latest.json) à partir des fichiers de release.
  Échoue explicitement si l'installateur ou sa signature sont absents ou incohérents.
  Usage : ./scripts/generer-latest-json.ps1 -Version 4.2.0 -Depot Teamfliyou/Tablau_de_bord_projet -Dossier release -Notes "..."
#>
param(
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][string]$Depot,
    [string]$Dossier = "release",
    [string]$Notes = ""
)
$ErrorActionPreference = "Stop"

if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version stable attendue (X.Y.Z), reçu : $Version" }
$setup = "TableauDeBordEtudiant-Setup-$Version.exe"
$cheminSetup = Join-Path $Dossier $setup
$cheminSig = "$cheminSetup.sig"
foreach ($f in @($cheminSetup, $cheminSig)) {
    if (-not (Test-Path $f) -or (Get-Item $f).Length -eq 0) { throw "Fichier indispensable absent ou vide : $f" }
}
$signature = (Get-Content $cheminSig -Raw).Trim()
if (-not $signature) { throw "Signature vide : $cheminSig" }

$url = "https://github.com/$Depot/releases/download/v$Version/$setup"
$manifeste = [ordered]@{
    version   = $Version
    notes     = $Notes
    pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = [ordered]@{
        "windows-x86_64" = [ordered]@{ signature = $signature; url = $url }
    }
}
$sortie = Join-Path $Dossier "latest.json"
$json = $manifeste | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText((Join-Path (Get-Location) $sortie), $json, (New-Object System.Text.UTF8Encoding($false)))

# Validation du fichier écrit.
$lu = Get-Content $sortie -Raw | ConvertFrom-Json
$plateforme = $lu.platforms.'windows-x86_64'
if ($lu.version -ne $Version) { throw "latest.json : version incorrecte" }
if (-not $plateforme.signature -or $plateforme.signature -ne $signature) { throw "latest.json : signature incorrecte" }
if ($plateforme.url -ne $url -or -not $plateforme.url.StartsWith("https://")) { throw "latest.json : URL incorrecte" }
if ([System.IO.Path]::GetFileName($plateforme.url) -ne $setup) { throw "latest.json : l'URL ne correspond pas au fichier publié" }
Write-Host "latest.json valide : version $Version -> $url"
