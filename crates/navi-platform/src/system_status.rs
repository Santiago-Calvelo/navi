//! On-demand application CPU sampling on the text worker; no window titles or polling.
use super::*;
use navi_agent::model::{ErrorCode, ToolError};
use std::collections::{BTreeMap, BTreeSet};
use windows::Win32::{
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};

struct Process {
    parent: u32,
    name: String,
    usage: Option<(u64, u64, Instant)>,
}

fn ticks(time: FILETIME) -> u64 {
    u64::from(time.dwHighDateTime) << 32 | u64::from(time.dwLowDateTime)
}

fn sample() -> std::result::Result<BTreeMap<u32, Process>, ToolError> {
    unsafe {
        let snapshot =
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|_| unavailable())?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut processes = BTreeMap::new();
        let mut next = Process32FirstW(snapshot, &mut entry);
        while next.is_ok() {
            let name = String::from_utf16_lossy(
                &entry.szExeFile[..entry
                    .szExeFile
                    .iter()
                    .position(|c| *c == 0)
                    .unwrap_or(entry.szExeFile.len())],
            );
            let usage = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                entry.th32ProcessID,
            )
            .ok()
            .and_then(|handle| {
                let mut created = FILETIME::default();
                let mut exited = FILETIME::default();
                let mut kernel = FILETIME::default();
                let mut user = FILETIME::default();
                let read =
                    GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user);
                let at = Instant::now();
                let _ = CloseHandle(handle);
                read.ok()
                    .map(|_| (ticks(created), ticks(kernel) + ticks(user), at))
            });
            processes.insert(
                entry.th32ProcessID,
                Process {
                    parent: entry.th32ParentProcessID,
                    name,
                    usage,
                },
            );
            next = Process32NextW(snapshot, &mut entry);
        }
        let _ = CloseHandle(snapshot);
        if processes.is_empty() {
            Err(unavailable())
        } else {
            Ok(processes)
        }
    }
}

unsafe extern "system" fn visible_app(hwnd: HWND, parameter: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool()
        || GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0
        || GetWindow(hwnd, GW_OWNER).0 != 0
        || hwnd == GetShellWindow()
    {
        return BOOL(1);
    }
    let mut cloaked = 0u32;
    if DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as _, 4).is_ok()
        && cloaked != 0
    {
        return BOOL(1);
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid != 0 && pid != std::process::id() {
        (*(parameter.0 as *mut BTreeSet<u32>)).insert(pid);
    }
    BOOL(1)
}

fn unavailable() -> ToolError {
    ToolError::new(
        ErrorCode::ToolFailed,
        "No se pudieron consultar las aplicaciones abiertas.",
    )
}

fn application_name(executable: &str) -> String {
    match executable.to_ascii_lowercase().as_str() {
        "taskmgr.exe" => "Administrador de tareas".into(),
        "explorer.exe" => "Explorador de Windows".into(),
        "powershell.exe" => "Windows PowerShell".into(),
        "pwsh.exe" => "PowerShell".into(),
        "code.exe" => "Visual Studio Code".into(),
        "firefox.exe" => "Firefox".into(),
        "msedge.exe" => "Microsoft Edge".into(),
        "chrome.exe" => "Google Chrome".into(),
        _ => executable.trim_end_matches(".exe").to_string(),
    }
}

pub(super) fn applications() -> std::result::Result<Vec<(String, Option<f64>)>, ToolError> {
    let before = sample()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    let after = sample()?;
    let mut visible = BTreeSet::new();
    unsafe {
        EnumWindows(Some(visible_app), LPARAM(&mut visible as *mut _ as isize))
            .map_err(|_| unavailable())?;
    }
    let roots = visible
        .iter()
        .filter_map(|pid| {
            after
                .get(pid)
                .map(|process| (*pid, process.name.to_ascii_lowercase()))
        })
        .collect::<BTreeMap<_, _>>();
    // Keep idle applications and use a friendly label; never read window content.
    let mut groups = BTreeMap::<String, (String, f64, bool)>::new();
    for pid in &visible {
        if let Some(process) = after.get(pid) {
            groups.entry(process.name.to_ascii_lowercase()).or_insert((
                application_name(&process.name),
                0.0,
                true,
            ));
        }
    }
    let logical = unsafe {
        windows::Win32::System::Threading::GetActiveProcessorCount(
            windows::Win32::System::Threading::ALL_PROCESSOR_GROUPS,
        )
    }
    .max(1) as f64;
    for (pid, process) in &after {
        if *pid == std::process::id() {
            continue;
        }
        let mut ancestor = *pid;
        let mut owner = None;
        for _ in 0..after.len() {
            if let Some(root) = roots.get(&ancestor) {
                owner = Some(root.clone());
                break;
            }
            let Some(parent) = after.get(&ancestor) else {
                break;
            };
            if parent.parent == 0 || parent.parent == ancestor {
                break;
            }
            ancestor = parent.parent;
        }
        let key = owner.unwrap_or_else(|| process.name.to_ascii_lowercase());
        let Some((_, cpu, available)) = groups.get_mut(&key) else {
            continue;
        };
        match (before.get(pid).and_then(|p| p.usage), process.usage) {
            (Some((created, ticks_before, at)), Some((created_after, ticks_after, end)))
                if created == created_after && end > at =>
            {
                *cpu += ticks_after.saturating_sub(ticks_before) as f64
                    / 10_000_000.0
                    / end.duration_since(at).as_secs_f64()
                    / logical
                    * 100.0;
            }
            // Processes born during this sample have no complete interval yet.
            (None, Some(_)) if !before.contains_key(pid) => {}
            _ => *available = false,
        }
    }
    let mut ranked = groups
        .into_values()
        .map(|(name, cpu, available)| (name, available.then_some(cpu)))
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| {
        b.1.unwrap_or(-1.0)
            .total_cmp(&a.1.unwrap_or(-1.0))
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked.truncate(6);
    Ok(ranked)
}
