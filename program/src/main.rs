//! The TrackMania conversion: each map into a ReSkate map and a Studio-compiled mod ([`recipe`]).
//!
//! ```text
//! trackmania-export --out <folder> [--install <TrackMania Nations Forever>] [--exchange <folder>]
//!                   [--studio <reskate_cli.exe> [--game <skate. install>]] [--jobs N] [MAP ...]
//! ```
//!
//! Converts the maps named (every map of [`maps::MAPS`] when none is), each into `<out>/<map>/`:
//! `map/TrackMania_<Title>`, and with `--studio` its compile `stage/Patch` against `--game` (default `$RESKATE_GAME`).
//! A campaign map is read from the install's `GameData`, an exchange map from `<exchange>/<id>.Gbx` (default the
//! program's `maps` folder). Maps run `--jobs` at a time (3), each in its own process with its output in
//! `<out>/<map>/build.log`; a map that fails is reported and the others still run, and the exit code says whether all
//! did. Packaging and in-game checks are the tooling's (`tools/trackmania-tools`).

mod choices;
mod recipe;

use std::path::{Path, PathBuf};

use primase_core::convert::Result;
use primase_reskate::compile::Studio;
use primase_trackmania::read::GameFiles;
use trackmania_export::maps::{self, Map, Source};

const USAGE: &str = "usage: trackmania-export --out <folder> [--install <TrackMania Nations Forever>] [--exchange <folder>] [--studio <reskate_cli.exe> [--game <skate. install>]] [--jobs N] [MAP ...]";

/// Where Steam installs the game.
const STEAM_INSTALL: &str = r"C:\Program Files (x86)\Steam\steamapps\common\TrackMania Nations Forever";

fn main() -> Result<()> {
    let (mut out, mut cli, mut game) = (None, None, None);
    let mut install = PathBuf::from(STEAM_INSTALL);
    let mut exchange = Path::new(env!("CARGO_MANIFEST_DIR")).join("../maps");
    let (mut jobs, mut wanted) = (3usize, Vec::new());
    // the options as given, for the per-map children
    let mut pass_on = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(USAGE);
        match arg.as_str() {
            "--jobs" => {
                let n = value()?;
                jobs = n
                    .parse()
                    .ok()
                    .filter(|&n| n > 0)
                    .ok_or(format!("--jobs {n}: a count above 0"))?;
            }
            "--out" | "--install" | "--exchange" | "--studio" | "--game" => {
                let v = value()?;
                pass_on.extend([arg.clone(), v.clone()]);
                let v = PathBuf::from(v);
                match arg.as_str() {
                    "--out" => out = Some(v),
                    "--install" => install = v,
                    "--exchange" => exchange = v,
                    "--studio" => cli = Some(v),
                    _ => game = Some(v),
                }
            }
            _ if arg.starts_with("--") => return Err(format!("unknown option {arg}; {USAGE}").into()),
            _ => wanted.push(arg),
        }
    }
    let Some(out) = out else {
        return Err(USAGE.into());
    };
    let studio = match (
        cli,
        game.or_else(|| std::env::var_os("RESKATE_GAME").map(PathBuf::from)),
    ) {
        (Some(cli), Some(game)) => Some(Studio { cli, game }),
        (Some(_), None) => return Err("--studio needs --game (or RESKATE_GAME)".into()),
        (None, _) => None,
    };
    let chosen = maps::chosen(&wanted)?;
    if jobs > 1 && chosen.len() > 1 {
        return in_children(&chosen, &pass_on, jobs, &out);
    }
    let files = GameFiles::open(&install)?;
    let mut lines = Vec::new();
    for m in chosen {
        println!("== {} ({} by {})", m.id, m.title, m.author);
        let started = std::time::Instant::now();
        let secs = || started.elapsed().as_secs_f64();
        let converted = map_bytes(&files, &exchange, &m)
            .and_then(|bytes| recipe::map(&files, &bytes, &m, &out.join(m.id), studio.as_ref()));
        let line = match converted {
            Ok((map, None)) => format!("{}: ok in {:.0} s, map {}", m.id, secs(), map.display()),
            Ok((_, Some(c))) => format!(
                "{}: ok in {:.0} s, map and Studio compile ({:.0} s, {} warnings, log {}): {}",
                m.id,
                secs(),
                c.seconds,
                c.warnings,
                c.log.display(),
                c.patch.display()
            ),
            Err(e) => format!("{}: FAILED after {:.0} s: {e}", m.id, secs()),
        };
        println!("{line}");
        lines.push(line);
    }
    summary(lines)
}

/// The map's `.Challenge.Gbx` bytes: a campaign map's from the install, an exchange map's from `exchange`.
fn map_bytes(files: &GameFiles, exchange: &Path, map: &Map) -> Result<Vec<u8>> {
    match map.source {
        Source::Campaign(path) => Ok(files
            .read(path)?
            .ok_or_else(|| format!("the install has no {path}"))?),
        Source::Exchange(id) => {
            let path = exchange.join(format!("{id}.Gbx"));
            std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()).into())
        }
    }
}

/// Each map in its own process (the same options, that one map), `jobs` at a time, each one's output in
/// `<out>/<map>/build.log`; then each map's result line.
fn in_children(chosen: &[Map], options: &[String], jobs: usize, out: &Path) -> Result<()> {
    let exe = std::env::current_exe()?;
    let started = std::time::Instant::now();
    let mut queue: std::collections::VecDeque<&Map> = chosen.iter().collect();
    let mut running = Vec::new();
    let mut lines = Vec::new();
    while !queue.is_empty() || !running.is_empty() {
        while running.len() < jobs
            && let Some(m) = queue.pop_front()
        {
            let dir = out.join(m.id);
            std::fs::create_dir_all(&dir)?;
            let log = dir.join("build.log");
            let file = std::fs::File::create(&log)?;
            let child = std::process::Command::new(&exe)
                .args(options)
                .args(["--jobs", "1", m.id])
                .stdout(file.try_clone()?)
                .stderr(file)
                .spawn()?;
            println!("{}: started, log {}", m.id, log.display());
            running.push((m, log, child));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mut i = 0;
        while i < running.len() {
            let Some(status) = running[i].2.try_wait()? else {
                i += 1;
                continue;
            };
            let (m, log, _) = running.swap_remove(i);
            let prefix = format!("{}: ", m.id);
            let text = std::fs::read_to_string(&log).unwrap_or_default();
            let line = text.lines().rfind(|line| line.starts_with(&prefix)).map_or_else(
                || format!("{}: FAILED, exit {status}, see {}", m.id, log.display()),
                str::to_owned,
            );
            println!("{line}");
            lines.push(line);
        }
    }
    println!("== {:.0} s, {jobs} at a time", started.elapsed().as_secs_f64());
    lines.sort();
    summary(lines)
}

/// The result lines, and an error when a map failed.
fn summary(lines: Vec<String>) -> Result<()> {
    println!("== summary");
    for line in &lines {
        println!("{line}");
    }
    let failed = lines.iter().filter(|l| l.contains(": FAILED")).count();
    if failed > 0 {
        return Err(format!("{failed} of {} maps failed", lines.len()).into());
    }
    Ok(())
}
