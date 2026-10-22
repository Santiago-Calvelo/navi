#![windows_subsystem = "windows"]

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let snapshot = if args.get(1).is_some_and(|v| v == "--snapshot") {
        args.get(2).map(std::path::PathBuf::from)
    } else {
        None
    };
    let result = if args.get(1).is_some_and(|v| v == "--observer-probe") {
        navi_platform::observer_probe(
            args.get(2)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| "artifacts/observer.json".into()),
        )
    } else if args.get(1).is_some_and(|v| v == "--benchmark") {
        navi_platform::benchmark(
            args.get(2)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| "artifacts/benchmark".into()),
        )
    } else if args.get(1).is_some_and(|v| v == "--composition-probe") {
        navi_platform::composition_probe(
            &args
                .get(2)
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| "composition.bmp".into()),
        )
    } else {
        navi_platform::run(snapshot)
    };
    if let Err(error) = result {
        // A desktop executable has no attached console: retain the startup diagnostic.
        let path = std::env::temp_dir().join("navi-error.log");
        let _ = std::fs::write(path, error.to_string());
        std::process::exit(1);
    }
}
