//! The TrackMania Nations Forever maps the program converts, where each is read from, and the names their maps, mods
//! and zips go by. The conversion and the tooling (`tools/trackmania-tools`) both name maps this way.

/// Where a map's file is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A campaign map in the install's GameData (its path there, as the install's file list spells it).
    Campaign(&'static str),
    /// A map from TrackMania Exchange (`https://tmnf.exchange/trackgbx/<id>`), read from `<exchange folder>/<id>.Gbx`.
    Exchange(u32),
}

/// A map the program converts: its id on the command line, its name as its author gave it (formatting codes taken
/// out), its author's login, and where it is read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Map {
    pub id: &'static str,
    pub title: &'static str,
    pub author: &'static str,
    pub source: Source,
}

/// The three maps: the Nations campaign's first race, and two of TrackMania Exchange's most awarded maps of all time
/// with other blocks than it (Turbulence: platforms, inflatables, dirt; Iron Maiden: water, loops, a dirt park).
pub const MAPS: [Map; 3] = [
    Map {
        id: "a01",
        title: "A01-Race",
        author: "Nadeo",
        source: Source::Campaign(r"tracks\campaigns\nations\white\a01-race.challenge.gbx"),
    },
    Map {
        id: "turbulence",
        title: "Turbulence",
        author: "matto",
        source: Source::Exchange(1354053),
    },
    Map {
        id: "iron-maiden",
        title: "Iron Maiden",
        author: "torress",
        source: Source::Exchange(136547),
    },
];

/// The maps of [`MAPS`] named in `wanted` (all when it is empty).
pub fn chosen(wanted: &[String]) -> Result<Vec<Map>, String> {
    if wanted.is_empty() {
        return Ok(MAPS.to_vec());
    }
    wanted
        .iter()
        .map(|id| {
            MAPS.iter().find(|m| m.id == id).copied().ok_or_else(|| {
                let ids: Vec<&str> = MAPS.iter().map(|m| m.id).collect();
                format!("no map {id} (the program converts {ids:?})")
            })
        })
        .collect()
}

/// The map's (and the mod folder's, and the zip's) name: `TrackMania_` and its title's letters and digits
/// (`TrackMania_A01Race`, `TrackMania_IronMaiden`).
pub fn map_name(map: &Map) -> String {
    let name: String = map.title.chars().filter(char::is_ascii_alphanumeric).collect();
    format!("TrackMania_{name}")
}

/// The map's name in skate.'s level list and on its page: `TrackMania | Iron Maiden`, as the other ports name theirs
/// (`SkateBird | Rooftop`).
pub fn display_name(map: &Map) -> String {
    format!("TrackMania | {}", map.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_program_converts_three_maps() {
        assert_eq!(chosen(&[]).unwrap().len(), 3);
        assert!(chosen(&["a02".into()]).is_err());
    }

    #[test]
    fn maps_are_named_by_their_titles() {
        assert_eq!(map_name(&MAPS[0]), "TrackMania_A01Race");
        assert_eq!(map_name(&MAPS[2]), "TrackMania_IronMaiden");
        assert_eq!(display_name(&MAPS[1]), "TrackMania | Turbulence");
    }
}
