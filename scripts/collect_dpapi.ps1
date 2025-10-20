<#
.SYNOPSIS
    Collect deterministic DPAPI recovery fixtures for CI automation.

.DESCRIPTION
    Generates reproducible DPAPI recovery artifacts used by integration tests.
    The script writes fixture data under tests/fixtures/security/dpapi-recovery
    and exports the matching audit log into tests/golden/security/ so Linux
    jobs and documentation have a consistent source of truth.

.NOTES
    The implementation emits synthetic but realistic looking artifacts. It is
    intended for CI environments where direct access to Windows DPAPI recovery
    infrastructure is not available.
#>

[CmdletBinding()]
param(
    [Parameter()]
    [string]$OutputDir = "tests/fixtures/security/dpapi-recovery",

    [Parameter()]
    [string]$AuditLogPath = "tests/golden/security/dpapi-recovery-audit.jsonl",

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

function Write-JsonFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        $Object
    )

    $parent = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    $json = $Object | ConvertTo-Json -Depth 6
    Set-Content -LiteralPath $Path -Value $json -Encoding UTF8
}

function Write-Jsonl {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,

        [Parameter(Mandatory = $true)]
        [array]$Events
    )

    $parent = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    $lines = foreach ($entry in $Events) {
        $entry | ConvertTo-Json -Depth 6
    }

    Set-Content -LiteralPath $Path -Value $lines -Encoding UTF8
}

function Write-Checksum {
    param(
        [Parameter(Mandatory = $true)]
        [string]$InputPath,

        [Parameter(Mandatory = $true)]
        [string]$ChecksumPath
    )

    $parent = Split-Path -Parent $ChecksumPath
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }

    $hash = (Get-FileHash -LiteralPath $InputPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $fileName = Split-Path -Leaf $InputPath
    Set-Content -LiteralPath $ChecksumPath -Value "$hash  $fileName" -Encoding UTF8
}

$resolvedOutputPath = Resolve-RepoPath -Path $OutputDir
if (-not (Test-Path -LiteralPath $resolvedOutputPath)) {
    New-Item -ItemType Directory -Path $resolvedOutputPath -Force | Out-Null
}

$resolvedAuditPath = Resolve-RepoPath -Path $AuditLogPath
$auditParent = Split-Path -Parent $resolvedAuditPath
if (-not (Test-Path -LiteralPath $auditParent)) {
    New-Item -ItemType Directory -Path $auditParent -Force | Out-Null
}

$manifestPath = Join-Path $resolvedOutputPath "manifest.json"
$inventoryPath = Join-Path $resolvedOutputPath "master-key-inventory.json"
$certificatePath = Join-Path $resolvedOutputPath "certificate-inventory.json"
$eventsPath = Join-Path $resolvedOutputPath "recovery-events.jsonl"

$manifest = [ordered]@{
    capture_id        = "ZA-DPAPI-20240318-01"
    collector         = "scripts/collect_dpapi.ps1"
    collector_version = "1.0.0"
    host              = "WIN-CI"
    domain            = "ZA-LAB"
    recovery_agent    = [ordered]@{
        distinguished_name = "CN=Zaevrynth Recovery,OU=Key Recovery,DC=za,DC=lab"
        thumbprint         = "92B5C4F3C6E81BD2A2393E311410C67F6A589F79"
        certificate_serial = "6E4C97C42F8A9AB33BF2C1FEE9D7AD42"
    }
    certificate_expiration = "2025-06-01T00:00:00Z"
    policy_revision        = 42
    artifacts = @(
        [ordered]@{ path = "recovery-events.jsonl"; description = "Security log export for DPAPI recovery events" }
        [ordered]@{ path = "master-key-inventory.json"; description = "Recovered master key metadata" }
        [ordered]@{ path = "certificate-inventory.json"; description = "Active recovery certificate inventory" }
    )
}

$inventory = [ordered]@{
    machine_sid = "S-1-5-21-523489101-2145879035-1792157894"
    keys = @(
        [ordered]@{
            guid            = "7c9e6679-7425-40de-944b-e07fc1f90ae7"
            status          = "recovered"
            sha1            = "6b9a1bafcc2a0eb0b3f0ab8dbe3c7b9f1f9f2f34"
            last_rotation   = "2024-03-01T14:03:12Z"
            recovery_reason = "WSL transport handshake"
        }
        [ordered]@{
            guid            = "e02fd0e4-00fd-090A-ca30-0d00a0038ba0"
            status          = "staged"
            sha1            = "2c9dc8341f03ed2a928c0889680c91f89c1e9e5d"
            last_rotation   = "2023-12-15T09:41:27Z"
            recovery_reason = "scheduled rotation"
        }
    )
}

$certificates = [ordered]@{
    authority = "ZA-LAB Key Recovery"
    issued_at = "2023-05-12T08:00:00Z"
    certificates = @(
        [ordered]@{
            thumbprint = "92B5C4F3C6E81BD2A2393E311410C67F6A589F79"
            not_before = "2023-05-12T00:00:00Z"
            not_after  = "2025-06-01T00:00:00Z"
            subject    = "CN=Zaevrynth Recovery,OU=Key Recovery,DC=za,DC=lab"
        }
        [ordered]@{
            thumbprint = "7F0AE98E21C6E1387E8B0D719B6F8C47E2923A1B"
            not_before = "2022-03-10T00:00:00Z"
            not_after  = "2024-03-10T00:00:00Z"
            subject    = "CN=Zaevrynth Recovery Legacy,OU=Key Recovery,DC=za,DC=lab"
        }
    )
}

$events = @(
    [ordered]@{
        timestamp          = "2024-03-18T20:14:05Z"
        event_id           = 4692
        event_source       = "Microsoft-Windows-DataProtectionManagement"
        event_type         = "DPAPI_RECOVERY_SUCCESS"
        host               = "WIN-CI"
        target_user        = "ZA-LAB\\dpapi.test"
        master_key_guid    = "7c9e6679-7425-40de-944b-e07fc1f90ae7"
        agent_thumbprint   = "92B5C4F3C6E81BD2A2393E311410C67F6A589F79"
        recovery_certificate = "CN=Zaevrynth Recovery,OU=Key Recovery,DC=za,DC=lab"
    }
    [ordered]@{
        timestamp          = "2024-03-18T20:14:07Z"
        event_id           = 4693
        event_source       = "Microsoft-Windows-DataProtectionManagement"
        event_type         = "DPAPI_AUDIT_LOGGED"
        host               = "WIN-CI"
        target_user        = "ZA-LAB\\dpapi.test"
        master_key_guid    = "7c9e6679-7425-40de-944b-e07fc1f90ae7"
        details            = "Audit event recorded for WSL bridge session"
    }
    [ordered]@{
        timestamp          = "2024-03-18T20:15:11Z"
        event_id           = 4692
        event_source       = "Microsoft-Windows-DataProtectionManagement"
        event_type         = "DPAPI_RECOVERY_SUCCESS"
        host               = "WIN-CI"
        target_user        = "ZA-LAB\\dpapi.rotate"
        master_key_guid    = "e02fd0e4-00fd-090A-ca30-0d00a0038ba0"
        agent_thumbprint   = "92B5C4F3C6E81BD2A2393E311410C67F6A589F79"
        recovery_certificate = "CN=Zaevrynth Recovery,OU=Key Recovery,DC=za,DC=lab"
    }
)

Write-JsonFile -Path $manifestPath -Object $manifest
Write-JsonFile -Path $inventoryPath -Object $inventory
Write-JsonFile -Path $certificatePath -Object $certificates
Write-Jsonl -Path $eventsPath -Events $events

Copy-Item -LiteralPath $eventsPath -Destination $resolvedAuditPath -Force

if ($EmitChecksums.IsPresent) {
    $auditChecksumPath = Join-Path (Split-Path -Parent $resolvedAuditPath) ((Split-Path -Leaf $resolvedAuditPath) + '.sha256')
    Write-Checksum -InputPath $resolvedAuditPath -ChecksumPath $auditChecksumPath

    $fixtureChecksum = Join-Path $resolvedOutputPath "recovery-events.jsonl.sha256"
    Write-Checksum -InputPath $eventsPath -ChecksumPath $fixtureChecksum
}

Write-Output "DPAPI recovery artifacts written to $resolvedOutputPath"
Write-Output "Audit log exported to $resolvedAuditPath"
