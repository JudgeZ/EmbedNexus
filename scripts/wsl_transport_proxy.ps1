<#
.SYNOPSIS
    Generate WSL bridge metadata and transport traces for CI automation.

.DESCRIPTION
    Produces deterministic WSL transport artifacts that model the bridge
    behavior between Windows and a WSL distribution. The script writes both the
    transport negotiation trace and auxiliary metadata so the Linux workflows
    can validate DPAPI bridging expectations without manual intervention.
#>

[CmdletBinding()]
param(
    [Parameter()]
    [string]$OutputPath = "tests/golden/transport/wsl-handshake-negotiation.trace",

    [Parameter()]
    [string]$BridgeMetadataPath = "tests/golden/security/wsl-bridge.json",

    [Parameter()]
    [string]$ProxyHost = "127.0.0.1",

    [Parameter()]
    [int]$ProxyPort = 5173,

    [Parameter()]
    [switch]$EmitChecksums
)

$script:ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$script:RepoRoot = Split-Path -Parent $script:ScriptRoot

function Resolve-RepoPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }

    return (Join-Path $script:RepoRoot $Path)
}

function Ensure-ParentDirectory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $parent = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
}

function Write-Checksum {
    param(
        [Parameter(Mandatory = $true)]
        [string]$InputPath,

        [Parameter(Mandatory = $true)]
        [string]$ChecksumPath
    )

    Ensure-ParentDirectory -Path $ChecksumPath
    $hash = (Get-FileHash -LiteralPath $InputPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $fileName = Split-Path -Leaf $InputPath
    Set-Content -LiteralPath $ChecksumPath -Value "$hash  $fileName" -Encoding UTF8
}

function Write-Json {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        $Object
    )

    Ensure-ParentDirectory -Path $Path
    $json = $Object | ConvertTo-Json -Depth 6
    Set-Content -LiteralPath $Path -Value $json -Encoding UTF8
}

$resolvedTracePath = Resolve-RepoPath -Path $OutputPath
$resolvedMetadataPath = Resolve-RepoPath -Path $BridgeMetadataPath

Ensure-ParentDirectory -Path $resolvedTracePath
Ensure-ParentDirectory -Path $resolvedMetadataPath

$traceContent = @"
# WSL Transport Bridge Trace
proxy_endpoint: $ProxyHost:$ProxyPort
bridge:
  version: 1
  mode: passthrough
windows_client:
  computer: WIN-CI
  session: wsl-trace
wsl_peer:
  distro: Ubuntu-22.04
  kernel: 5.15.133.1-microsoft-standard-WSL2
handshake:
  tls_version: TLS1.3
  cipher: TLS_AES_128_GCM_SHA256
  negotiated_alpn: h2
  dpapi_session_key: 7c9e6679-7425-40de-944b-e07fc1f90ae7
"@

Set-Content -LiteralPath $resolvedTracePath -Value $traceContent -Encoding UTF8

$metadata = [ordered]@{
    capture_id   = "wsl-bridge-zaevrynth-ci"
    generated_at = "2024-03-18T20:20:12Z"
    proxy        = [ordered]@{
        host = $ProxyHost
        port = $ProxyPort
    }
    windows_client = [ordered]@{
        computer       = "WIN-CI"
        transport_user = "ZA-LAB\\dpapi.test"
    }
    wsl_peer = [ordered]@{
        distro = "Ubuntu-22.04"
        kernel = "5.15.133.1-microsoft-standard-WSL2"
    }
    tls = [ordered]@{
        negotiated_version = "TLS1.3"
        cipher             = "TLS_AES_128_GCM_SHA256"
        dpapi_session_key  = "7c9e6679-7425-40de-944b-e07fc1f90ae7"
    }
}

Write-Json -Path $resolvedMetadataPath -Object $metadata

if ($EmitChecksums.IsPresent) {
    $traceChecksumPath = Join-Path (Split-Path -Parent $resolvedTracePath) ((Split-Path -Leaf $resolvedTracePath) + '.sha256')
    Write-Checksum -InputPath $resolvedTracePath -ChecksumPath $traceChecksumPath

    $metadataChecksumPath = Join-Path (Split-Path -Parent $resolvedMetadataPath) ((Split-Path -Leaf $resolvedMetadataPath) + '.sha256')
    Write-Checksum -InputPath $resolvedMetadataPath -ChecksumPath $metadataChecksumPath
}

Write-Output "WSL bridge trace written to $resolvedTracePath"
Write-Output "Bridge metadata written to $resolvedMetadataPath"
