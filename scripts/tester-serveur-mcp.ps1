<#
  Lance tdb-mcp.exe comme le ferait Hermes Agent (stdio, JSON-RPC ligne par ligne) et vérifie
  la poignée de main MCP, la liste des outils et l'accès à la base (désactivé par défaut).
  Usage : ./scripts/tester-serveur-mcp.ps1 -Executable <tdb-mcp.exe> -Base <tableau-de-bord.db> -Version 5.1.0
#>
param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$Base,
    [Parameter(Mandatory)][string]$Version
)
$ErrorActionPreference = "Stop"
if (-not (Test-Path $Executable)) { throw "Serveur MCP introuvable : $Executable" }

$psi = New-Object System.Diagnostics.ProcessStartInfo $Executable
$psi.Arguments = "--base `"$Base`""
$psi.UseShellExecute = $false
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.StandardOutputEncoding = New-Object System.Text.UTF8Encoding($false)
$psi.CreateNoWindow = $true
$p = [System.Diagnostics.Process]::Start($psi)
try {
    function Requete([string]$json) {
        $p.StandardInput.WriteLine($json)
        $p.StandardInput.Flush()
        $ligne = $p.StandardOutput.ReadLine()
        if (-not $ligne) { throw "Aucune réponse du serveur MCP à : $json" }
        return $ligne | ConvertFrom-Json
    }
    $init = Requete '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"ci-windows","version":"1"}}}'
    if ($init.result.serverInfo.version -ne $Version) { throw "Version du serveur MCP : $($init.result.serverInfo.version) (attendue : $Version)" }
    $p.StandardInput.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized"}')
    $liste = Requete '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
    $nombre = @($liste.result.tools).Count
    if ($nombre -ne 14) { throw "tools/list : $nombre outils (14 attendus)" }
    # Intégration désactivée par défaut : l'appel doit être refusé proprement (la base est bien lue).
    $appel = Requete '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_projects","arguments":{}}}'
    if ($appel.result.isError -ne $true -or $appel.result.content[0].text -notmatch "désactivée") {
        throw "Réponse inattendue de list_projects : $($appel | ConvertTo-Json -Depth 6 -Compress)"
    }
    Write-Host "Serveur MCP $Version : poignée de main, $nombre outils, lecture seule par défaut — OK ($Executable)"
} finally {
    if (-not $p.HasExited) { $p.Kill() }
}
