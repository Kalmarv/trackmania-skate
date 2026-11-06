//! The TrackMania tooling around the conversion (`program/`, the recipe): each built map packaged as a release (mod
//! folder, icon, zip), and the skate. console commands of a map's in-game check.
//!
//! ```text
//! trackmania-tools package --out <build folder> --dest <folder> --version <version> [--icons <folder of MAP.png>] [MAP ...]
//! trackmania-tools views skate-commands MAP --out <build folder>
//! trackmania-tools views map-name MAP
//! ```

mod package;
mod views;

use std::path::PathBuf;

use trackmania_export::maps::{self, map_name};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "usage: trackmania-tools package --out <build folder> --dest <folder> --version <version> [--icons <folder of MAP.png>] [MAP ...]
       trackmania-tools views skate-commands MAP --out <build folder>
       trackmania-tools views map-name MAP";

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("package") => package_command(args.collect()),
        Some("views") => views_command(args.collect()),
        _ => Err(USAGE.into()),
    }
}

/// Each built map as an installable mod folder (`<out>/<map>/mod/TrackMania_<Title>`) and its zip in `--dest`
/// ([`package`]).
fn package_command(args: Vec<String>) -> Result<()> {
    let (mut out, mut dest, mut version, mut icons, mut wanted) = (None, None, None, None, Vec::new());
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(USAGE);
        match arg.as_str() {
            "--out" => out = Some(PathBuf::from(value()?)),
            "--dest" => dest = Some(PathBuf::from(value()?)),
            "--version" => version = Some(value()?),
            "--icons" => icons = Some(PathBuf::from(value()?)),
            _ if arg.starts_with("--") => return Err(format!("unknown option {arg}; {USAGE}").into()),
            _ => wanted.push(arg),
        }
    }
    let (Some(out), Some(dest), Some(version)) = (out, dest, version) else {
        return Err(format!("package needs --out, --dest and --version; {USAGE}").into());
    };
    std::fs::create_dir_all(&dest)?;
    for m in maps::chosen(&wanted)? {
        let name = map_name(&m);
        // the mod's picture: `<icons>/<map>.png`, a screenshot of the map in skate
        let shot = icons.as_ref().map(|d| d.join(format!("{}.png", m.id)));
        let png = match &shot {
            Some(p) if p.is_file() => Some(std::fs::read(p)?),
            _ => None,
        };
        if png.is_none() {
            println!(
                "{}: no icon ({})",
                m.id,
                shot.map_or("no --icons".into(), |p| format!("{} missing", p.display()))
            );
        }
        let folder = package::mod_folder(&out.join(m.id), &m, &name, &version, png.as_deref())?;
        let zip = dest.join(format!("{name}-{version}.zip"));
        let bytes = package::zip(&folder, &zip)?;
        println!("{}: {} ({:.1} MB)", m.id, zip.display(), bytes as f64 / 1e6);
    }
    Ok(())
}

/// `views skate-commands MAP --out <build>`: one run's console commands for the map's views ([`views`]), from
/// `<build>/<map>/map/<name>/map.json`; `views map-name MAP`: the map's (and its mod folder's) name.
fn views_command(args: Vec<String>) -> Result<()> {
    let mut out = None;
    let mut words = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = Some(PathBuf::from(args.next().ok_or(USAGE)?)),
            _ => words.push(arg),
        }
    }
    match &words[..] {
        [w, id] if w == "map-name" => println!("{}", map_name(&maps::chosen(std::slice::from_ref(id))?[0])),
        [w, id] if w == "skate-commands" => {
            let m = maps::chosen(std::slice::from_ref(id))?[0];
            let name = map_name(&m);
            let out = out.ok_or(USAGE)?;
            let file = out.join(m.id).join("map").join(&name).join("map.json");
            let map: serde_json::Value = serde_json::from_slice(
                &std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?,
            )?;
            println!("{}", views::skate_commands(&map, &name)?);
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}
