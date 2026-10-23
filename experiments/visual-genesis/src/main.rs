mod experience;
mod experiment;
mod experiments;
mod platform;
mod render;

use experiment::CATALOG;

#[derive(Debug)]
pub struct Options {
    pub selection: Vec<usize>,
    pub seconds: f32,
    pub reduced: bool,
    pub monitor: usize,
    pub experience: bool,
    pub tour: bool,
    pub smoke: bool,
}

fn parse(args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut args = args.peekable();
    let mut options = Options {
        selection: vec![0],
        seconds: 16.0,
        reduced: false,
        monitor: 0,
        experience: false,
        tour: false,
        smoke: false,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" | "--help" | "-h" => {
                println!("NAVI / VISUAL GENESIS\nNative passive studies. All events and data are simulated.\n");
                for entry in CATALOG {
                    println!(
                        "  {:12} {} / {}\n               {}",
                        entry.id, entry.kind, entry.name, entry.thesis
                    );
                }
                println!("\n--experience             Persistent interactive experience: Margen\n--tour                   Guided experience; click/type to take control\n--experiment ID | --all   Passive studies; --seconds 4..120\n--monitor N (0 = primary) --reduced-motion --list\nMargen: click the edge to open, Enter to continue, Escape to fold, × to exit.\nAll data and actions are simulated. No global shortcuts or desktop observers.");
                return Ok(None);
            }
            "--experiment" => {
                let id = args.next().ok_or("Missing experiment ID")?;
                options.selection = vec![CATALOG
                    .iter()
                    .position(|x| x.id == id)
                    .ok_or_else(|| format!("Unknown experiment: {id}. Use --list."))?];
            }
            "--all" => options.selection = (0..CATALOG.len()).collect(),
            "--seconds" => {
                options.seconds = args
                    .next()
                    .ok_or("Missing seconds")?
                    .parse()
                    .map_err(|_| "Invalid seconds")?;
                if !options.seconds.is_finite() || !(4.0..=120.0).contains(&options.seconds) {
                    return Err("Seconds must be between 4 and 120".into());
                }
            }
            "--monitor" => {
                options.monitor = args
                    .next()
                    .ok_or("Missing monitor index")?
                    .parse()
                    .map_err(|_| "Invalid monitor index")?
            }
            "--reduced-motion" => options.reduced = true,
            "--experience" => options.experience = true,
            "--tour" => {
                options.experience = true;
                options.tour = true;
            }
            "--smoke" => {
                options.experience = true;
                options.tour = true;
                options.smoke = true;
            }
            _ => return Err(format!("Unknown option: {arg}")),
        }
    }
    Ok(Some(options))
}

fn main() {
    let result = match parse(std::env::args().skip(1)) {
        Ok(Some(options)) => {
            println!("NAVI visual laboratory / simulated responses and events");
            if options.experience {
                experience::run(options).map_err(|e| e.to_string())
            } else {
                platform::run(options).map_err(|e| e.to_string())
            }
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    };
    if let Err(e) = result {
        eprintln!("Visual Genesis: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_invocations() {
        for args in [
            vec!["--seconds", "NaN"],
            vec!["--seconds", "0"],
            vec!["--experiment", "missing"],
            vec!["--monitor", "-1"],
        ] {
            assert!(parse(args.into_iter().map(String::from)).is_err());
        }
    }
    #[test]
    fn playlist_is_complete() {
        assert_eq!(
            parse(["--all".to_string()].into_iter())
                .unwrap()
                .unwrap()
                .selection,
            vec![0, 1, 2, 3, 4]
        );
    }
}
