use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../../assets/navi.ico");
    println!("cargo:rerun-if-env-changed=RC");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let icon = fs::canonicalize("../../assets/navi.ico").expect("NAVI icon");
    let resource = output.join("navi.rc");
    let compiled = output.join("navi.res");
    fs::write(&resource, format!("1 ICON \"{}\"\n", icon.display().to_string().trim_start_matches("\\\\?\\").replace('\\', "/")))
        .expect("Write icon resource");
    let compiler = env::var_os("RC").map(PathBuf::from).or_else(|| {
        let sdk = PathBuf::from(env::var_os("ProgramFiles(x86)")?)
            .join("Windows Kits/10/bin");
        let mut candidates: Vec<_> = fs::read_dir(sdk).ok()?.filter_map(|entry| {
            let path = entry.ok()?.path().join("x64/rc.exe");
            path.is_file().then_some(path)
        }).collect();
        candidates.sort();
        candidates.pop()
    }).unwrap_or_else(|| "rc.exe".into());
    let status = Command::new(compiler)
        .arg("/nologo").arg("/fo").arg(&compiled).arg(&resource)
        .status().expect("Run Windows SDK resource compiler");
    assert!(status.success(), "Icon resource compilation failed");
    println!("cargo:rustc-link-arg-bin=navi-app={}", compiled.display());
}
