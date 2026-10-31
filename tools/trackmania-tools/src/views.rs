//! The views an in-game check shoots of a built map: its spawn, and up to [`TRAVEL_VIEWS`] of its travel points spread
//! over their list (first, middle, last), from the map's own `map.json` as the program wrote it.
//!
//! The map's frame is +Y up and -Z forward (`map.json` says so), and skate.'s `trainer tp` takes the scene's (x, y,
//! -z): a position `[x, y, z]` is `trainer tp x y -z`. skate gives no console way to turn the skater: the spawn view is
//! shot where the level starts (facing the spawn's yaw), and `trainer tp` leaves the skater at heading 0.
use serde_json::Value;

/// Metres above a travel point a teleport lands (skate drops the skater onto the ground).
const LIFT: f64 = 0.3;
/// Seconds a view is held after its shot is asked for: under load the runner's capture starts 15-20 s late.
const HOLD: u32 = 25;
/// Travel points shot besides the spawn.
pub const TRAVEL_VIEWS: usize = 3;

/// The positions of up to [`TRAVEL_VIEWS`] travel points of `map` (a `map.json`): first, middle and last of the list.
pub fn travel_views(map: &Value) -> Result<Vec<[f64; 3]>, String> {
    let points = map["travel_points"]
        .as_array()
        .ok_or("map.json has no travel_points")?;
    let mut picked: Vec<usize> = match points.len() {
        0 => vec![],
        n => vec![0, n / 2, n - 1],
    };
    picked.dedup();
    picked.truncate(TRAVEL_VIEWS);
    picked
        .into_iter()
        .map(|i| {
            let p: Vec<f64> = points[i]["position"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_f64)
                .collect();
            <[f64; 3]>::try_from(p).map_err(|_| format!("travel point {i} has no position"))
        })
        .collect()
}

/// The asset path skate. loads a map named `name` by (`levels/game/dingolevel_reskate_<name>/...`, lowercase).
pub fn level_asset(name: &str) -> String {
    let l = format!("dingolevel_reskate_{}", name.to_lowercase());
    format!("levels/game/{l}/{l}")
}

/// RESKATE_STARTUP_COMMANDS for one map's run: load it, settle, a shot at the spawn, then a teleport and a shot per
/// travel view (each `trainer where` is a shot).
pub fn skate_commands(map: &Value, name: &str) -> Result<String, String> {
    let mut c = vec![
        "wait 100".to_owned(),
        format!("load {}", level_asset(name)),
        "wait 45".into(),
        "trainer option hud 0".into(),
        "hideui 1".into(),
        "trainer where".into(),
        format!("wait {HOLD}"),
    ];
    for [x, y, z] in travel_views(map)? {
        c.push(format!("trainer tp {x} {:.3} {}", y + LIFT, -z));
        c.extend(["wait 12".into(), "trainer where".into(), format!("wait {HOLD}")]);
    }
    Ok(c.join(";"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn views_spread_over_the_travel_points_in_skates_frame() {
        let map = serde_json::json!({ "travel_points": [
            { "position": [1.0, 2.0, 3.0] }, { "position": [4.0, 5.0, 6.0] }, { "position": [7.0, 8.0, 9.0] },
            { "position": [10.0, 11.0, 12.0] }, { "position": [13.0, 14.0, 15.0] },
        ]});
        assert_eq!(
            travel_views(&map).unwrap(),
            [[1.0, 2.0, 3.0], [7.0, 8.0, 9.0], [13.0, 14.0, 15.0]]
        );
        let c = skate_commands(&map, "TrackMania_A01Race").unwrap();
        assert!(c.contains(
            "load levels/game/dingolevel_reskate_trackmania_a01race/dingolevel_reskate_trackmania_a01race"
        ));
        assert!(c.contains("trainer tp 7 8.300 -9"), "{c}");
        assert_eq!(c.matches("trainer where").count(), 4);
        let one = serde_json::json!({ "travel_points": [{ "position": [1.0, 2.0, 3.0] }] });
        assert_eq!(travel_views(&one).unwrap().len(), 1);
    }
}
