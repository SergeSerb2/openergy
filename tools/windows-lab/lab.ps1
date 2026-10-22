# Openergy Windows interop lab: runs a private copy of the installed Synergy core in an
# SSH session (session 0, no desktop), so it never sees or moves the real mouse and keyboard.
# Usage (from an SSH session, PowerShell 7):
#   lab.ps1 setup
#   lab.ps1 server -Cert <pem> [-Port 24810] [-Name winlab] [-Peer oe-client] [-CheckClients]
#   lab.ps1 client -Cert <pem> [-Port 24812] [-Name winlab-client]
#   lab.ps1 trust-server|trust-client -Line 'v2:sha256:<hex>'
#   lab.ps1 clip-get | clip-set -Text <text>      (only while 'client' runs)
#   lab.ps1 stop server|client
#   lab.ps1 cleanup
param(
  [Parameter(Position = 0, Mandatory)] [string]$Command,
  [Parameter(Position = 1)] [string]$Role,
  [string]$Cert, [int]$Port, [string]$Name, [string]$Peer = "oe-client",
  [switch]$CheckClients, [string]$Line, [string]$Text
)
$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$Lab = Join-Path $env:USERPROFILE "openergy-winlab"
$Core = Join-Path $Lab "core\synergy-core.exe"
$Installed = "C:\Program Files\Synergy"
# The core reads trust files here when it runs elevated (as an SSH admin session does).
$TrustDir = "C:\ProgramData\Synergy\tls"
$RoamingConf = Join-Path $env:APPDATA "synergy\Synergy.conf"

function Marker([string]$name) { Join-Path $Lab "$name.created" }

function Write-Settings([string]$mode, [string]$name, [int]$port, [string]$pem, [bool]$checkClients) {
  $pem = (Resolve-Path $pem).Path -replace '\\', '\\'
  $lines = @("[core]", "coreMode=$(if ($mode -eq 'server') { 2 } else { 1 })", "computerName=$name", "port=$port",
    "preventSleep=false")
  if ($mode -eq "server") { $lines += "interface=127.0.0.1" } else { $lines += "", "[client]", "remoteHost=127.0.0.1" }
  $lines += "", "[log]", "level=Debug", "toFile=true", "file=$($Lab -replace '\\', '\\')\\$mode.log",
    "", "[security]", "tlsEnabled=true", "checkPeerFingerprints=$($checkClients.ToString().ToLower())", "certificate=$pem"
  if ($mode -eq "server") {
    $lines += "", "[server]", "externalConfig=true", "externalConfigFile=$($Lab -replace '\\', '\\')\\server.conf"
    @("section: screens", "`t${name}:", "`t${Peer}:", "end", "", "section: links", "`t${name}:", "`t`tleft = $Peer",
      "`t${Peer}:", "`t`tright = $name", "end", "", "section: options", "`theartbeat = 5000", "`tclipboardSharing = true",
      "end") | Set-Content -Encoding ascii (Join-Path $Lab "server.conf")
  }
  $file = Join-Path $Lab "$mode-settings.conf"
  $lines | Set-Content -Encoding ascii $file
  $file
}

function Start-Core([string]$mode, [string]$settings) {
  foreach ($ext in "log", "out", "err") { Remove-Item (Join-Path $Lab "$mode.$ext") -ErrorAction SilentlyContinue }
  $p = Start-Process -FilePath $Core -ArgumentList $mode, "--new-instance", "--settings", $settings -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $Lab "$mode.out") -RedirectStandardError (Join-Path $Lab "$mode.err")
  $p.Id | Out-File -Encoding ascii (Join-Path $Lab "$mode.pid")
  Write-Host "lab $mode pid $($p.Id)"
  $p
}

switch ($Command) {
  "setup" {
    New-Item -ItemType Directory -Force (Join-Path $Lab "core"), (Join-Path $Lab "queue") | Out-Null
    Copy-Item (Join-Path $Installed "synergy-core.exe") (Split-Path $Core) -Force
    Get-ChildItem $Installed -Filter *.dll | Where-Object { $_.Name -match '^(Qt6|libssl|libcrypto)' } |
      Copy-Item -Destination (Split-Path $Core) -Force
    Copy-Item (Join-Path $Installed "platforms") (Split-Path $Core) -Recurse -Force
    # The running Synergy core owns the global close event and denies other processes full
    # access, so a second core aborts at start. Rename the event in the private copy only.
    $bytes = [IO.File]::ReadAllBytes($Core)
    $old = [Text.Encoding]::Unicode.GetBytes("Global\SynergyClose")
    $new = [Text.Encoding]::Unicode.GetBytes("Global\OpenergyLab1")
    $hits = 0
    for ($i = 0; $i -le $bytes.Length - $old.Length; $i++) {
      $match = $true
      for ($j = 0; $j -lt $old.Length; $j++) { if ($bytes[$i + $j] -ne $old[$j]) { $match = $false; break } }
      if ($match) { [Array]::Copy($new, 0, $bytes, $i, $new.Length); $hits++ }
    }
    if ($hits -ne 1) { throw "expected one close-event name in synergy-core.exe, found $hits" }
    [IO.File]::WriteAllBytes($Core, $bytes)
    # The core creates this file on first start; remember to remove it if it is ours.
    if (-not (Test-Path $RoamingConf)) { "" | Out-File (Marker "roaming-conf") }
    "lab ready: $Lab (synergy-core $((Get-Item (Join-Path $Installed 'synergy-core.exe')).VersionInfo.ProductVersion))"
  }
  "server" {
    $settings = Write-Settings "server" ($Name ? $Name : "winlab") ($Port ? $Port : 24810) $Cert $CheckClients.IsPresent
    $p = Start-Core "server" $settings
    $p.WaitForExit(); "lab server exited $($p.ExitCode)"
  }
  "client" {
    $settings = Write-Settings "client" ($Name ? $Name : "winlab-client") ($Port ? $Port : 24812) $Cert $false
    $queue = Join-Path $Lab "queue"; Get-ChildItem $queue | Remove-Item -Force
    $p = Start-Core "client" $settings
    # Clipboards belong to a window station, and each SSH login gets its own. This loop runs
    # queued clipboard commands inside the client's station.
    while (-not $p.HasExited) {
      foreach ($f in Get-ChildItem $queue -Filter *.cmd) {
        $cmd = (Get-Content -Raw $f.FullName).Trim(); $out = $f.FullName -replace '\.cmd$', '.out'
        $result = if ($cmd -eq "get") { [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes((Get-Clipboard -Raw))) }
        elseif ($cmd -like "set *") {
          Set-Clipboard -Value ([Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($cmd.Substring(4)))); "ok"
        } else { "unknown" }
        Remove-Item $f.FullName; $result | Out-File -Encoding ascii "$out.tmp"; Move-Item "$out.tmp" $out
      }
      Start-Sleep -Milliseconds 100
    }
    "lab client exited $($p.ExitCode)"
  }
  { $_ -in "clip-get", "clip-set" } {
    $id = [guid]::NewGuid().ToString("n"); $queue = Join-Path $Lab "queue"
    $cmd = if ($Command -eq "clip-get") { "get" } else { "set " + [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($Text)) }
    $cmd | Out-File -Encoding ascii (Join-Path $queue "$id.cmd")
    $out = Join-Path $queue "$id.out"
    for ($i = 0; $i -lt 50 -and -not (Test-Path $out); $i++) { Start-Sleep -Milliseconds 100 }
    if (-not (Test-Path $out)) { throw "no reply: is 'lab.ps1 client' running?" }
    $reply = (Get-Content -Raw $out).Trim(); Remove-Item $out
    if ($Command -eq "clip-get") { [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($reply)) } else { $reply }
  }
  { $_ -in "trust-server", "trust-client" } {
    if ($Line -notmatch '^v2:sha256:[0-9a-f]{64}$') { throw "-Line must be v2:sha256:<64 lowercase hex>" }
    $file = Join-Path $TrustDir ($Command -eq "trust-server" ? "trusted-servers" : "trusted-clients")
    if (-not (Test-Path $TrustDir)) { New-Item -ItemType Directory $TrustDir | Out-Null; "" | Out-File (Marker "trust-dir") }
    if ((Test-Path $file) -and -not (Test-Path (Marker (Split-Path -Leaf $file)))) {
      throw "$file already exists and is not the lab's; refusing to change it"
    }
    "$Line" | Set-Content -Encoding ascii $file; "" | Out-File (Marker (Split-Path -Leaf $file))
    "wrote $file"
  }
  "stop" {
    $pidFile = Join-Path $Lab "$Role.pid"
    if (-not (Test-Path $pidFile)) { "no lab $Role"; break }
    $id = [int](Get-Content $pidFile)
    $p = Get-Process -Id $id -ErrorAction SilentlyContinue
    # Only the lab's private copy is ever stopped; the installed Synergy is never touched.
    if ($p -and $p.Path -eq $Core) { Stop-Process -Id $id -Force; "stopped lab $Role $id" } else { "no lab $Role running" }
  }
  "cleanup" {
    foreach ($role in "server", "client") { & $PSCommandPath stop $role }
    foreach ($leaf in "trusted-servers", "trusted-clients") {
      if (Test-Path (Marker $leaf)) { Remove-Item (Join-Path $TrustDir $leaf) -ErrorAction SilentlyContinue; "removed $leaf" }
    }
    if ((Test-Path (Marker "trust-dir")) -and -not (Get-ChildItem $TrustDir -ErrorAction SilentlyContinue)) {
      Remove-Item $TrustDir; "removed $TrustDir"
    }
    if ((Test-Path (Marker "roaming-conf")) -and (Test-Path $RoamingConf)) { Remove-Item $RoamingConf; "removed $RoamingConf" }
    Remove-Item $Lab -Recurse -Force; "removed $Lab"
  }
  default { throw "unknown command $Command" }
}
