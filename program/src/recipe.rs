//! One TrackMania map into a ReSkate map and its Studio compile: primase's TrackMania reader (the map placed from the
//! install's block models), the lowering to core meanings, the preparation for skate., primase's map writer, then
//! ReSkate Studio. Every choice is a typed value of [`crate::choices`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use primase_core::convert::{Result, convert};
use primase_core::pass::{PassManager, PassRegistry, Pipeline};
use primase_core::{Component, Entity, LodChoice, Marker, Name, SelectLod, World};
use primase_foundations::compress::oodle::{Oodle, Version};
use primase_gltf::prepare::FillMissingTexcoords;
use primase_image::{BakeNormalScale, CapTextures, NormalizeNormals, PackOrm};
use primase_meshops::{FirstTexcoordSet, OmitMateriallessPrimitives, SpecializeMaterialVariants};
use primase_reskate::MapSink;
use primase_reskate::compile::{Compiled, Studio, compile_map};
use primase_reskate::passes::{
    GroundReskateMarkers, PrepareReskateBlendedSurfaces, PrepareReskateCollision, PrepareReskateLaidLayers,
    PrepareReskateOverlays, PrepareReskatePauseMap, PrepareReskateTextures, PrepareReskateUvRepeats,
    PrepareReskateUvTransforms, PrepareReskateWrap, RaiseOptions, RaiseSkate, SnapReskateMarkers,
    SummarizeReskateMap,
};
use primase_trackmania::components::TrackmaniaSpawn;
use primase_trackmania::passes::{
    LowerTrackmaniaLights, LowerTrackmaniaMaterials, LowerTrackmaniaSpawns, LowerTrackmaniaSurfaces,
};
use primase_trackmania::read::GameFiles;
use primase_trackmania::{FRAME, TrackmaniaReader};

use trackmania_export::maps::{Map, map_name};

use crate::choices;

/// `map` (its `.Challenge.Gbx` bytes) read with the install's `files`, converted into `<out>/map/<map name>` and, with
/// `studio`, compiled into `<out>/stage`.
pub fn map(
    files: &GameFiles,
    bytes: &[u8],
    map: &Map,
    out: &Path,
    studio: Option<&Studio>,
) -> Result<(PathBuf, Option<Compiled>)> {
    let name = map_name(map);
    let e = choices::encoding(out.join("map"), &name);
    let mut reader = TrackmaniaReader::new(files, bytes, map.id, FRAME);

    // Run once on the map as read: TrackMania's own data lowered to core meanings.
    let select = Pipeline {
        passes: vec![
            Box::new(LowerTrackmaniaMaterials),
            Box::new(LowerTrackmaniaLights {
                lux_per_unit: choices::LIGHT_LUX_PER_UNIT,
            }),
            Box::new(LowerTrackmaniaSpawns {
                others: choices::OTHER_SPAWNS,
            }),
            Box::new(LowerTrackmaniaSurfaces),
            Box::new(NameWaypoints),
            Box::new(TravelWithinStudio),
        ],
    };
    select.check()?;

    // Run on the map before the writer: the core world prepared as the ReSkate writer encodes it.
    let passes = Pipeline {
        passes: vec![
            // each block model's finest level of detail (the game's nearest)
            Box::new(SelectLod {
                choice: LodChoice::Finest,
            }),
            Box::new(NormalizeNormals),
            // the layers laid on a surface drawn as decals, judged before blended surfaces become cutouts
            Box::new(PrepareReskateLaidLayers::new(
                choices::LAID_LAYER_GAP,
                choices::LAID_LAYER_SHARE,
            )?),
            Box::new(PrepareReskateBlendedSurfaces::new(
                choices::BLEND_CUTOFF,
                choices::BLENDED,
                choices::UNIFORM_ALPHA,
            )?),
            Box::new(FillMissingTexcoords),
            Box::new(SpecializeMaterialVariants::new(choices::VARIANTS)),
            // skate samples the first set only: the grass's and the dirt's coordinates, which the lowering projects as a
            // set of their own, become their primitives' first
            Box::new(FirstTexcoordSet),
            Box::new(PrepareReskateUvTransforms::new(
                choices::BASE_SLOT,
                choices::SAMPLING,
            )),
            Box::new(OmitMateriallessPrimitives),
            Box::new(PrepareReskateWrap::new(choices::BASE_SLOT, choices::WRAP_AXIS)?),
            Box::new(PackOrm::new(choices::ORM_SIZE, choices::ORM_MIPS)),
            Box::new(BakeNormalScale),
            Box::new(RaiseSkate::new(RaiseOptions::new(
                &name,
                choices::LIGHTS,
                choices::unclassified(),
            )?)),
            Box::new(
                GroundReskateMarkers::new(choices::grounded(), choices::GROUND_ABOVE)?
                    .with_from(choices::GROUND_FROM),
            ),
            Box::new(SnapReskateMarkers::for_encoding(&e)?),
            Box::new(PrepareReskatePauseMap::new(choices::pause_map())?),
            Box::new(CapTextures::new(choices::TEXTURE_CAP, choices::cap_mode())?),
            Box::new(PrepareReskateTextures),
            Box::new(PrepareReskateUvRepeats::new(
                choices::REPEAT_DECALS,
                choices::REPEAT_BLENDED,
            )),
            Box::new(PrepareReskateOverlays::new(choices::OVERLAY_STEP)?.for_encoding(&e)?),
            Box::new(PrepareReskateCollision::new(
                choices::COLLISION_SURFACES,
                choices::COLLISION_WELD,
                &e,
            )?),
            Box::new(SummarizeReskateMap::for_encoding(&e)?),
        ],
    };
    passes.check()?;

    // Read, lower, prepare, write `<out>/map/<name>`; what was left out or approximated comes back as notices.
    let mut registry = PassRegistry::new();
    primase_trackmania::register(&mut registry);
    let manager = PassManager::new(registry).with_pass_checks(true);
    let report = convert(
        &mut reader,
        Box::new(MapSink::new(Path::new(""), &e)),
        &manager,
        &select,
        &passes,
    )?;
    for notice in report.notices() {
        println!("{notice}");
    }
    for (count, n) in &report.counts {
        println!("{count}={n}");
    }
    let written = out.join("map").join(&name);

    // ReSkate Studio compiles the map folder into a mod, against an empty stage folder (it refuses one in use).
    let Some(studio) = studio else {
        return Ok((written, None));
    };
    let stage = out.join("stage");
    if stage.exists() {
        std::fs::remove_dir_all(&stage)?;
    }
    // SAFETY: the installed game's own Oodle library, loaded to read back what Studio wrote for it.
    let oodle = unsafe { Oodle::load(studio.game.join("oo2core_9_win64.dll"), Version::V9) }?;
    let compiled = compile_map(
        studio,
        &written,
        &stage,
        &name,
        &choices::studio_compile(),
        &oodle,
    )?;
    Ok((written, Some(compiled)))
}

/// Names each spawn marker (the spawn and the travel points) by its block's way point ([`choices::waypoint_name`]):
/// the names the trainer and the travel list show.
#[derive(Debug)]
struct NameWaypoints;

impl primase_core::pass::Pass for NameWaypoints {
    fn name(&self) -> &'static str {
        "trackmania-name-waypoints"
    }
    fn requires(&self) -> &[&'static str] {
        &[Marker::NAME, TrackmaniaSpawn::NAME]
    }
    fn modifies(&self) -> &[&'static str] {
        &[Name::NAME]
    }
    fn run(&self, w: &mut World, log: &mut primase_core::pass::PassLog) -> std::result::Result<(), String> {
        let marked: Vec<(Entity, Option<i32>)> = w
            .iter::<TrackmaniaSpawn>()
            .filter(|(e, _)| w.get::<Marker>(*e).is_some())
            .map(|(e, s)| (e, s.waypoint.map(|t| if t == 4 { 0 } else { t })))
            .collect();
        let mut nth: BTreeMap<Option<i32>, usize> = BTreeMap::new();
        for (e, waypoint) in &marked {
            let of = marked.iter().filter(|(_, t)| t == waypoint).count();
            let n = nth.entry(*waypoint).or_default();
            *n += 1;
            if let Some(name) = choices::waypoint_name(*waypoint, *n, of) {
                w.insert(*e, Name(name));
                log.count("markers named by their way point", 1);
            }
        }
        Ok(())
    }
}

/// Refuses a map with more travel points than Studio compiles ([`choices::MAX_TRAVEL`]): a map with more checkpoints
/// needs its own choice of which to keep, not a silent cut.
#[derive(Debug)]
struct TravelWithinStudio;

impl primase_core::pass::Pass for TravelWithinStudio {
    fn name(&self) -> &'static str {
        "trackmania-travel-within-studio"
    }
    fn requires(&self) -> &[&'static str] {
        &[Marker::NAME]
    }
    fn run(&self, w: &mut World, log: &mut primase_core::pass::PassLog) -> std::result::Result<(), String> {
        let travel = w.iter::<Marker>().filter(|(_, m)| m.kind == "travel").count();
        log.count("travel markers", travel as u64);
        if travel > choices::MAX_TRAVEL {
            return Err(format!(
                "{travel} travel markers; Studio compiles at most {}",
                choices::MAX_TRAVEL
            ));
        }
        Ok(())
    }
}
