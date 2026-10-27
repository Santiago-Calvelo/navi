$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$executable = Join-Path $repo 'target\release\navi-app.exe'
$logs = Join-Path $repo 'artifacts\build-run'
$log = Join-Path $logs 'build.log'

try {
    $null = New-Item -ItemType Directory -Force -Path $logs
    if ($env:CARGO_TARGET_DIR) { throw 'CARGO_TARGET_DIR redirige el ejecutable oficial.' }
    $cargo = (Get-Command cargo.exe -ErrorAction Stop).Source
    if (-not ('NaviBuild.Windows' -as [type])) {
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
namespace NaviBuild {
    public static class Windows {
        private delegate bool WindowCallback(IntPtr window, IntPtr parameter);
        [DllImport("user32.dll")] private static extern bool EnumWindows(WindowCallback callback, IntPtr parameter);
        [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetClassName(IntPtr window, StringBuilder name, int capacity);
        [DllImport("user32.dll", SetLastError = true)] private static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
        public static bool Quit(uint process) {
            bool posted = false;
            EnumWindows((window, parameter) => {
                uint owner;
                GetWindowThreadProcessId(window, out owner);
                if (owner != process) return true;
                var name = new StringBuilder(64);
                GetClassName(window, name, name.Capacity);
                if (name.ToString() != "NaviSessionWindow") return true;
                // WM_QUIT ends GetMessage normally; NAVI then drops its resources.
                posted = PostMessage(window, 0x0012, IntPtr.Zero, IntPtr.Zero);
                return false;
            }, IntPtr.Zero);
            return posted;
        }
    }
}
'@
    }
    foreach ($instance in @(Get-Process -Name 'navi-app' -ErrorAction SilentlyContinue)) {
        if ($instance.Path -ne $executable) { continue }
        if (-not [NaviBuild.Windows]::Quit([uint32]$instance.Id)) {
            if ($instance.HasExited) { continue }
            throw "No se pudo solicitar el cierre de NAVI (PID $($instance.Id))."
        }
        if (-not $instance.WaitForExit(10000)) {
            throw "NAVI no cerro en 10 segundos (PID $($instance.Id)); no se forzo su salida."
        }
    }
    $build = Start-Process -FilePath $cargo -WorkingDirectory $repo `
        -ArgumentList 'build','--release','--offline','-p','navi-app' `
        -RedirectStandardOutput (Join-Path $logs 'stdout.log') -RedirectStandardError $log `
        -WindowStyle Hidden -PassThru
    $null = $build.Handle
    $build.WaitForExit()
    if ($build.ExitCode -ne 0) { throw "Compilacion fallida (codigo $($build.ExitCode))." }
    $navi = Start-Process -FilePath $executable -WorkingDirectory $repo -WindowStyle Hidden -PassThru
    if ($navi.WaitForExit(1000)) { throw "NAVI termino durante el inicio (codigo $($navi.ExitCode))." }
    Write-Output 'OK'
} catch {
    [Console]::Error.WriteLine("ERROR: $($_.Exception.Message) Log de compilacion: $log")
    exit 1
}
