#requires -Version 5.1
<#
.SYNOPSIS
  Collect safe, local Level 1A Windows acceptance diagnostics.
.DESCRIPTION
  Does not record screen, capture microphone, upload anything, change settings,
  or claim physical sound/original-Flash fidelity has passed. Always stores a
  manual checklist with NOT_TESTED statuses until a player supplies evidence.
.PARAMETER ExePath
  Previously installed native Rust executable, or a CI-staged copy.
.PARAMETER OutputDirectory
  Local report directory. Avoid supplying personal folders if sharing results.
.PARAMETER Launch
  Optionally launch the actual game after reporting; default is no launch.
#>
[CmdletBinding()]
param(
    [string]$ExePath = (Join-Path (Join-Path $env:LOCALAPPDATA 'BatmanCobbleBot') 'batman_cobblebot_native.exe'),
    [string]$OutputDirectory = '',
    [switch]$Launch
)

Set-StrictMode -Version 2.0
$ErrorActionPreference = 'Stop'

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'This acceptance collector only runs on Windows.'
}
if (-not (Test-Path -LiteralPath $ExePath -PathType Leaf)) {
    throw "Native game EXE not found. Install the current green GitHub Actions build first."
}

$ExePath = (Resolve-Path -LiteralPath $ExePath).ProviderPath
$exeFile = Get-Item -LiteralPath $ExePath
$exeDirectory = $exeFile.DirectoryName
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
    $stamp = (Get-Date).ToUniversalTime().ToString('yyyyMMdd-HHmmss')
    $OutputDirectory = Join-Path (Join-Path $env:LOCALAPPDATA 'BatmanCobbleBot\AcceptanceReports') ("level1a-$stamp")
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$OutputDirectory = (Resolve-Path -LiteralPath $OutputDirectory).ProviderPath
$actualSha = (Get-FileHash -LiteralPath $ExePath -Algorithm SHA256).Hash.ToLowerInvariant()

function Read-InstallMetadata([string]$file) {
    if (Test-Path -LiteralPath $file -PathType Leaf) {
        return (Get-Content -LiteralPath $file -Raw).Trim()
    }
    return $null
}

$reportedSha = Read-InstallMetadata (Join-Path $exeDirectory 'last-sha256.txt')
$reportedRun = Read-InstallMetadata (Join-Path $exeDirectory 'last-run.txt')
$reportedCommit = Read-InstallMetadata (Join-Path $exeDirectory 'last-commit.txt')
$hashStatus = 'METADATA_ABSENT'
if ($reportedSha) {
    if ($actualSha -eq $reportedSha.ToLowerInvariant()) {
        $hashStatus = 'MATCH'
    } else {
        $hashStatus = 'MISMATCH'
    }
}

$audioStatus = 'UNKNOWN'
$endpointCount = $null
$audioDetail = 'Audio endpoints may be unavailable, disabled, or not enumerable; detection is not proof of audible playback.'
try {
    $getPnp = Get-Command Get-PnpDevice -ErrorAction Stop
    $endpoints = @(Get-PnpDevice -Class 'AudioEndpoint' -ErrorAction Stop |
        Where-Object { $_.Status -eq 'OK' })
    $endpointCount = $endpoints.Count
    if ($endpointCount -gt 0) {
        $audioStatus = 'ENDPOINT_DETECTED_PLAYBACK_NOT_TESTED'
    } else {
        $audioStatus = 'NO_ENABLED_ENDPOINT_DETECTED'
    }
} catch {
    $audioStatus = 'ENUMERATION_UNAVAILABLE'
}
$displays = @()
$systemDpi = $null
try {
    Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
    foreach ($screen in [System.Windows.Forms.Screen]::AllScreens) {
        $displays += [ordered]@{
            width_px = [int]$screen.Bounds.Width
            height_px = [int]$screen.Bounds.Height
            primary = [bool]$screen.Primary
        }
    }
} catch {
    $displays = @()
}
try {
    if (-not ('BatmanSystemDpiProbe' -as [type])) {
        Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
public static class BatmanSystemDpiProbe {
    [DllImport("user32.dll")] public static extern uint GetDpiForSystem();
}
'@ -ErrorAction Stop
    }
    $systemDpi = [int][BatmanSystemDpiProbe]::GetDpiForSystem()
} catch {
    $systemDpi = $null
}

$manualNames = @(
    'original_flash_side_by_side',
    'intro_story_timing_and_skip',
    'hud_score_lives_ammo_and_readability',
    'f_fullscreen_and_esc_restore',
    'background_focus_cutscene_audio_position',
    'sound_music_effects_and_volume_flyout',
    'windows_explorer_and_pinned_taskbar_icon',
    'different_scaling_and_multi_monitor',
    'level1a_gameplay_camera_collision_exit'
)
$manual = [ordered]@{}
foreach ($name in $manualNames) { $manual[$name] = 'NOT_TESTED_BY_SCRIPT' }

$report = [ordered]@{
    schema_version = 1
    batch = 'MANUAL-ACCEPTANCE-PREP-11'
    generated_at_utc = [DateTime]::UtcNow.ToString('o')
    local_only_no_automatic_upload = $true
    no_screen_or_audio_capture = $true
    exe = [ordered]@{
        file_name = $exeFile.Name
        size_bytes = [long]$exeFile.Length
        sha256 = $actualSha
        sha256_vs_install_metadata = $hashStatus
        last_successful_run_id_recorded_at_install = $reportedRun
        last_commit_recorded_at_install = $reportedCommit
    }
    local_environment = [ordered]@{
        windows_version = [Environment]::OSVersion.Version.ToString()
        enabled_audio_endpoint_count = $endpointCount
        audio_endpoint_detection = $audioStatus
        audio_endpoint_caveat = $audioDetail
        displays = @($displays)
        system_dpi_not_per_monitor = $systemDpi
    }
    manual_checks = $manual
    limitation = 'This collector cannot verify original Flash fidelity, sound output/MCI position, icon cache, gameplay feel, or physical multi-DPI behavior. Mark these only from actual human observations.'
}
$reportPath = Join-Path $OutputDirectory 'level1a-local-environment.json'
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $reportPath -Encoding UTF8

$checklist = @'
# Batman: The CobbleBot Caper — Level 1A physical Windows acceptance

This is a player checklist, NOT an automated PASS. Run the newest installed
game and compare it with an original Flash reference. Check the boxes only
after personally seeing/hearing the described behavior.

- [ ] Original Flash and Rust Level1A scene/animation/timing comparison is acceptable.
- [ ] Intro dialogue, panels, natural completion, mouse SKIP and fade feel right.
- [ ] HUD BATARANG/HEALTH/SCORE, yellow icons, X + lives digits and outlines look right at different values.
- [ ] F enters fullscreen, Esc restores, and Windows controls are usable.
- [ ] Audio is actually audible on speakers/headphones, and neither soundtrack nor intro sound drifts after Alt+Tab or Windows volume-flyout focus changes.
- [ ] Batman icon shows correctly in Explorer and the actual pinned taskbar shortcut; refresh an old pin/cache if necessary.
- [ ] Windows 100%/125%/150% scaling and multiple monitors (where available) do not crop or misalign the game.
- [ ] Walk, run, jump, glide, attacks, pickups, collision/camera, lives and Level1A exit behave as intended.

For discrepancies: capture only screenshots/video YOU choose to share.
Describe original expected behavior, native observed behavior, reproducible
actions, build SHA/run, and severity. Never upload private desktop contents
or audio automatically. Keep PR #18 draft and issue #17 open until approved.

The adjacent JSON file records technical context only. A detected audio
device is NOT a sound pass. This script itself does not test physical audio.
'@
$checklistPath = Join-Path $OutputDirectory 'level1a-player-checklist.md'
$checklist | Set-Content -LiteralPath $checklistPath -Encoding UTF8

Write-Host "Local diagnostic files written. No upload, screen recording, or audio capture occurred."
Write-Host "EXE SHA256: $actualSha"
Write-Host "Installation checksum comparison: $hashStatus"
Write-Host "Audio enumeration: $audioStatus (not playback proof)"
Write-Host "Checklist: $checklistPath"
Write-Host "Report: $reportPath"

if ($hashStatus -eq 'MISMATCH') {
    Write-Warning 'Installed EXE checksum differs from last-sha256.txt. Reinstall from the verified GitHub artifact before accepting this game build.'
}
if ($Launch) {
    Start-Process -FilePath $ExePath -WorkingDirectory $exeDirectory
    Write-Host 'Game launched. Manual checks remain NOT_TESTED until the player evaluates them.'
}
