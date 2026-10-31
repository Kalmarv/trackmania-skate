//! The conversion's choices, with their reasons: what the recipe ([`crate::recipe`]) states. None of them is a reader
//! default; every map takes the same ones.
use std::collections::BTreeSet;
use std::path::PathBuf;

use primase_core::pbr::TextureSlot;
use primase_image::{Beyond, CapMode, MissingLevels, NormalDownscale, OrmSize};
use primase_meshops::UnusedVariants;
use primase_reskate::EncodingOptions;
use primase_reskate::compile::{CompileChoices, Streaming, TimeOfDay};
use primase_reskate::components::{CollisionSurfaces, CollisionWeld, MarkerKind};
use primase_reskate::passes::{
    BlendedSelection, GroundFrom, LightCalibration, PauseColour, PauseDrawing, PauseExtent, PauseLeaveOut,
    PauseSource, RepeatBlended, RepeatDecals, SamplingConflicts, Unclassified, UniformAlpha,
};
use primase_trackmania::passes::OtherSpawns;

/// The checkpoints' and the finish's spawns become travel points, so a player can go round the track's stages; the
/// start's is the map's spawn.
pub const OTHER_SPAWNS: OtherSpawns = OtherSpawns::Travel;

/// Studio takes at most 25 travel points a compile; the spawn is one.
pub const MAX_TRAVEL: usize = 24;

/// A car's spawn stands above the road (the car's height); the spawn and the travel points are dropped onto the
/// collision below, searching from 1 m above where they stand, in every collision: a TrackMania map's ground is its
/// placed blocks (the ground collision alone, the world's soups, is empty).
pub fn grounded() -> BTreeSet<MarkerKind> {
    BTreeSet::from([MarkerKind::Spawn, MarkerKind::Travel])
}
pub const GROUND_ABOVE: f64 = 1.0;
pub const GROUND_FROM: GroundFrom = GroundFrom::All;

/// The blocks' lights (the start, finish and checkpoint gantries' spots, the stadium floodlights): one unit of the
/// game's light intensity is the noon sun's 100,000 lux. The game lights a surface by its lights and its sun on one
/// scale (unit intensity at a light's base, the sun's colour times its angle), and its moods store no sun intensity
/// to scale by, so a unit is taken as the sun's: a spot of 0.6 is 7.6 million candela.
pub const LIGHT_LUX_PER_UNIT: f32 = 100_000.0;

/// raise-skate's light calibration: the lights are physical candela by [`LIGHT_LUX_PER_UNIT`], as GTA's are.
pub const LIGHTS: LightCalibration = LightCalibration::Standard;

/// A physics surface the reader's classes leave without one (`NotCollidable`: the tribunes, trees, banners and other
/// scenery the car passes through; the gameplay tags) is not collision in skate. either: RaiseSkate counts it out by
/// name.
pub fn unclassified() -> Unclassified {
    Unclassified::LeaveOut
}

/// Blended surfaces are cutouts at 0.5 (skate draws no blended world geometry): the grandstand crowds' and the trees'
/// alpha-tested sprites as they draw in the game, any uniform-alpha glass cut the same way.
pub const BLEND_CUTOFF: f32 = 0.5;
pub const BLENDED: BlendedSelection = BlendedSelection::All;
pub const UNIFORM_ALPHA: UniformAlpha = UniformAlpha::Cutout;

/// skate. samples one texture-coordinate set: every slot as the base colour (a slot that tiles otherwise adopts it,
/// reported), and the base colour's wrap on its S axis for the material.
pub const BASE_SLOT: TextureSlot = TextureSlot::BaseColor;
pub const SAMPLING: SamplingConflicts = SamplingConflicts::Adopt;
pub const WRAP_AXIS: usize = 0;
/// The turned copies of a material (the reader's variants for blocks placed with a direction) that no placement uses
/// are dropped.
pub const VARIANTS: UnusedVariants = UnusedVariants::Discard;
pub const ORM_SIZE: OrmSize = OrmSize::MetallicRoughness;
pub const ORM_MIPS: MissingLevels = MissingLevels::Generate;

/// ReSkate's placement heuristic for laid layers (road markings, decals laid on another surface): within 5 mm of a
/// surface, on at least half of the prototype's placements.
pub const LAID_LAYER_GAP: f64 = 0.005;
pub const LAID_LAYER_SHARE: f64 = 0.5;

/// The game's textures are 1,024 pixels at most on their larger side (Stadium's terrain and signs); the cap keeps every
/// one as it is.
pub const TEXTURE_CAP: u32 = 1024;
pub fn cap_mode() -> CapMode {
    CapMode {
        beyond: Beyond::Filter,
        normals: NormalDownscale::Box,
    }
}

/// Repeats of a decal are left out (the writer cannot tile one); repeats of a blended surface draw as cutouts.
pub const REPEAT_DECALS: RepeatDecals = RepeatDecals::Omit;
pub const REPEAT_BLENDED: RepeatBlended = RepeatBlended::Cutout;

/// Coplanar overlay offset (m): a laid layer is lifted this far off what it lies on.
pub const OVERLAY_STEP: f32 = 0.004;

/// Collision triangles of the same skate surface are coalesced, and welded at a millimetre: the blocks meet edge to
/// edge on the 32 m grid.
pub const COLLISION_SURFACES: CollisionSurfaces = CollisionSurfaces::Coalesce;
pub const COLLISION_WELD: CollisionWeld = CollisionWeld::Millimetre;

/// The pause map: a 2,160 px textured view from above the map's content, nothing left out (a Stadium map is open to
/// the sky).
pub fn pause_map() -> PauseSource {
    PauseSource::Geometry {
        height: 2160,
        extent: PauseExtent::Content,
        drawing: PauseDrawing {
            leave_out: PauseLeaveOut::Nothing,
            colour: PauseColour::Texture,
        },
    }
}

/// 200 m render tiles; a prototype placed at least 3 times with at least 150 triangles instanced (the blocks repeat:
/// a road straight is placed dozens of times); Studio's 4,780 m limit; the cull IDs of the level's share of ReSkate's
/// pool.
pub fn encoding(out_root: PathBuf, name: &str) -> EncodingOptions {
    EncodingOptions {
        out_root,
        name: Some(name.to_owned()),
        tile: 200.0,
        min_instances: 3,
        min_instance_tris: 150,
        max_render_edge: None,
        limit: 4780.0,
        offset: [0.0, 0.0],
        cull_ids: EncodingOptions::default().cull_ids,
        scene_warnings: None,
    }
}

/// The Studio compile: resident (`--no-stream`: a Stadium map is at most 1,024 m across, and Studio streams a map over
/// 600 m with its own vista proxies, which hung the GPU on large maps), the game's own geometry at every distance
/// (`--no-lods`: Studio's would be decimations), no baked GI, compiled at noon.
pub fn studio_compile() -> CompileChoices {
    CompileChoices {
        streaming: Some(Streaming::Resident),
        lods: false,
        gi: false,
        time_of_day: Some(TimeOfDay::Noon),
        pause_map: None,
    }
}
