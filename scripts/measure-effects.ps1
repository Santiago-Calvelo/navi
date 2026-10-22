param(
    [string]$Executable = "$PSScriptRoot\..\target\release\navi-app.exe",
    [string]$OutputDirectory = "$PSScriptRoot\..\artifacts\performance"
)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$OutputDirectory = (Resolve-Path -LiteralPath $OutputDirectory).Path
$gpuError = $null
try {
    # Warm the provider before starting the bounded nine-second workloads.
    $null = Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine
} catch { $gpuError = $_.Exception.Message }
$naviProcess = Start-Process -FilePath $Executable -ArgumentList '--benchmark', $OutputDirectory -WindowStyle Hidden -PassThru
$naviId = $naviProcess.Id
$dwmIds = @(Get-Process dwm -ErrorAction SilentlyContinue | Where-Object SessionId -eq $naviProcess.SessionId | Select-Object -ExpandProperty Id)
$counterIds = @($naviId) + $dwmIds
$counterFilter = ($counterIds | ForEach-Object { "Name LIKE 'pid_$($_)_%'" }) -join ' OR '
$samples = [System.Collections.Generic.List[object]]::new()
while (-not $naviProcess.HasExited) {
    $stageFile = Join-Path $OutputDirectory 'stage.json'
    if (-not (Test-Path -LiteralPath $stageFile)) { Start-Sleep -Milliseconds 100; continue }
    try { $before = (Get-Content -LiteralPath $stageFile -Raw | ConvertFrom-Json).stage } catch { continue }
    if ($before -eq 'done') { break }
    $naviProcess.Refresh()
    $cpuStart = $naviProcess.TotalProcessorTime.TotalMilliseconds
    $clockSample = [System.Diagnostics.Stopwatch]::StartNew()
    $engines = @()
    if (-not $gpuError) {
        try { $engines = @(Get-CimInstance -ClassName Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine -Filter $counterFilter) }
        catch { $gpuError = $_.Exception.Message }
    }
    Start-Sleep -Milliseconds 400
    $naviProcess.Refresh()
    if ($naviProcess.HasExited) { break }
    $clockSample.Stop()
    try { $after = (Get-Content -LiteralPath $stageFile -Raw | ConvertFrom-Json).stage } catch { continue }
    if ($before -ne $after) { continue }
    $naviEngines = @($engines | Where-Object Name -Like "pid_$($naviId)_*")
    $dwmEngines = @($engines | Where-Object { $engineName = $_.Name; @($dwmIds | Where-Object { $engineName -like "pid_$($_)_*" }).Count -gt 0 })
    $cpuMs = $naviProcess.TotalProcessorTime.TotalMilliseconds - $cpuStart
    $naviGpu = if ($naviEngines.Count) { [double](($naviEngines | Measure-Object UtilizationPercentage -Sum).Sum) } else { $null }
    $dwmGpu = if ($dwmEngines.Count) { [double](($dwmEngines | Measure-Object UtilizationPercentage -Sum).Sum) } else { $null }
    $samples.Add([pscustomobject]@{
        Stage = $before
        ElapsedMs = $clockSample.Elapsed.TotalMilliseconds
        CpuMs = $cpuMs
        WorkingSetMiB = $naviProcess.WorkingSet64 / 1MB
        PrivateMiB = $naviProcess.PrivateMemorySize64 / 1MB
        NaviGpuEngineSumPercent = $naviGpu
        NaviGpuInstances = $naviEngines.Count
        DwmGpuEngineSumPercent = $dwmGpu
    })
}
$summary = foreach ($group in ($samples | Group-Object Stage)) {
    $wallMs = ($group.Group | Measure-Object ElapsedMs -Sum).Sum
    $cpuMs = ($group.Group | Measure-Object CpuMs -Sum).Sum
    $naviGpuSamples = @($group.Group | Where-Object { $null -ne $_.NaviGpuEngineSumPercent })
    $dwmGpuSamples = @($group.Group | Where-Object { $null -ne $_.DwmGpuEngineSumPercent })
    [pscustomobject]@{
        Stage = $group.Name
        Samples = $group.Count
        CpuOneCorePercent = [math]::Round(100 * $cpuMs / $wallMs, 3)
        CpuMachinePercent = [math]::Round(100 * $cpuMs / $wallMs / [Environment]::ProcessorCount, 3)
        PeakWorkingSetMiB = [math]::Round(($group.Group | Measure-Object WorkingSetMiB -Maximum).Maximum, 2)
        PeakPrivateMiB = [math]::Round(($group.Group | Measure-Object PrivateMiB -Maximum).Maximum, 2)
        NaviGpuEngineMeanPercent = if ($naviGpuSamples.Count) { [math]::Round(($naviGpuSamples | Measure-Object NaviGpuEngineSumPercent -Average).Average, 3) } else { $null }
        DwmGpuEngineMeanPercent = if ($dwmGpuSamples.Count) { [math]::Round(($dwmGpuSamples | Measure-Object DwmGpuEngineSumPercent -Average).Average, 3) } else { $null }
    }
}
[pscustomobject]@{
    LogicalProcessors = [Environment]::ProcessorCount
    GpuCounterError = $gpuError
    Note = 'Nine-second diagnostic workloads repeat bounded bursts; DWM counters include the whole desktop. Null GPU means no matching instance or unavailable counter, not measured zero.'
    Summary = @($summary)
    Samples = @($samples.ToArray())
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'metrics.json')
$summary | Format-Table -AutoSize
