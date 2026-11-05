# TrackMania for skate.

A converter that turns TrackMania Nations Forever maps into custom maps for skate. The output is a ReSkate map, which ReSkate Studio compiles into a mod for the ReSkate mod loader.

![The three maps in skate.](media/levels.jpg)

## Features

- Converts three maps: A01-Race from the Nations campaign, and Turbulence and Iron Maiden from TrackMania Exchange
- Rebuilds each map block by block from the models, materials and collision in your own game install
- Uses primase and its TrackMania reader to read, convert and write the map
- Keeps the game's collision for every block and leaves out scenery that has none
- Turns the start into the spawn and each checkpoint and the finish into a fast-travel point
- Names those points Start, Checkpoint 1, 2, ... and Finish, and drops them onto the ground below
- Places the blocks' own lights, such as the gantry spots and the stadium floodlights
- Compiles each map with ReSkate Studio at noon, fully resident, without LODs or baked GI
- Renders the pause-menu map picture from the map's geometry
- Packages each map as a ReSkate mod zip with a manifest, icon, page, changelog and trainer spots
- Converts maps in parallel, one process per map, and reports a failed map without stopping the others
- Leaves out grass blade fins, additive glows and fake shadows, which have no material skate. can draw
- Prints the converter's notices and counts for anything left out or approximated

## Getting Started

### Requirements

- Windows, for the ReSkate Studio compile
- Rust 1.99.0 (pinned in `rust-toolchain.toml`)
- Your own install of TrackMania Nations Forever (free on Steam)
- skate. and ReSkate Studio (`reskate_cli.exe`)

Nothing from the game or from TrackMania Exchange is in this repository. Download the two exchange maps from
`https://tmnf.exchange/trackgbx/<id>` and save them as `maps/<id>.Gbx`:

| Map | Id | Author | Source |
| --- | --- | --- | --- |
| A01-Race | `a01` | Nadeo | The Nations campaign in the game install |
| Turbulence | `turbulence` | matto | [TrackMania Exchange 1354053](https://tmnf.exchange/trackshow/1354053) |
| Iron Maiden | `iron-maiden` | torress | [TrackMania Exchange 136547](https://tmnf.exchange/trackshow/136547) |

### Installation

The workspace has two binaries. Neither is published; run them from a clone with `cargo run`.

| Package | Description |
| --- | --- |
| `trackmania-export` (`program/`) | Converts the maps and compiles them with ReSkate Studio |
| `trackmania-tools` (`tools/trackmania-tools/`) | Packages compiled maps as mod zips and prints the console commands for in-game checks |

```sh
git clone https://github.com/Kalmarv/trackmania-skate.git
cd trackmania-skate
```

### Usage

Convert and compile every map into `build/`, then package them as version 0.2.1 into `releases/`:

```sh
cargo run --release -p trackmania-export -- --out build --studio <path to reskate_cli.exe> --game <skate. install>
cargo run --release -p trackmania-tools -- package --out build --dest releases --version 0.2.1
```

This writes `build/<map>/map/TrackMania_<Title>` (the ReSkate map), `build/<map>/stage/Patch` (the Studio compile),
`build/<map>/mod/TrackMania_<Title>` (the mod folder) and `releases/TrackMania_<Title>-0.2.1.zip`. Install a zip by
dragging it onto the ReSkate launcher and pick `TrackMania | <Title>` from the level list.

Other options and commands:

- Name map ids (`a01 turbulence iron-maiden`) after the options to convert only those maps.
- `--install <folder>` reads the game from a folder other than Steam's default, and `--exchange <folder>` reads the exchange maps from a folder other than `maps/`.
- `--game` can be replaced by the `RESKATE_GAME` environment variable. Without `--studio`, only the ReSkate map folders are written.
- `--jobs N` sets how many maps convert at once (default 3); each map's output goes to `build/<map>/build.log`.
- `package --icons <folder>` uses `<folder>/<map id>.png`, a screenshot, as each mod's 256 x 256 icon.
- `trackmania-tools views skate-commands <map> --out build` prints the skate. console commands that load the map and visit its spawn and travel points.

`tools/ingame_levels.sh` and `tools/ingame_drop.sh` run those views and a collision drop test in skate. They use
hard-coded paths and the shared runner from a separate reskate-tools checkout, so they only run on the maintainer's machine.

Run the unit tests with `cargo test`. The repository has no license file. TrackMania Nations Forever is by Nadeo and
Ubisoft, and each map belongs to its author.
