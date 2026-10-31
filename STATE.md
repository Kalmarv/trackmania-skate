# TrackMania: state (trackmania-map-reader, 2026-10-31; rewrite, never append)

## Goal
Caleb: "finish the gbx reader, and verify it in lab (through core primase) and make 3 different reskate levels from
trackmania maps following the general style of the other reskate maps like skatebird and los santos - using primase as
a library to read, pass, write".

## What works (verified)
- Reader, all on GitHub mains: gbx-formats 68dffb8 (reads Gbx maps and the NadeoPak packs; pins primase-foundations
  d64ea51 for the packs' Inflater) and primase-trackmania on primase main d827770db (landed through land.sh: fmt,
  clippy, 1502 unit tests, real-data primase-trackmania 8/8, primase-reskate 403/403, primase-meshops 165/165). It places every block from the Stadium pack and reads every part of every solid: visuals,
  collision and lights (CPlugTreeLight -> CPlugLight -> GxLight, kept as `trackmania.lights`), every material image
  (DDS, and TGA through primase-image's new `read_tga`; only the two DefaultCubeReflect cube maps stay unread, core has
  no cube texture), and the decoration's mood (`trackmania.mood`: sun latitude/rise/fall, remapped start day time,
  shadows, light map mood, the Sunrise/Sunset folders' Directional.SceneLight). Real-data test (GAME_DATA set): all 65
  campaign maps wholly placed, no part skipped, every mood read (7 Day, 16 Night, 22 Sunrise, 20 Sunset).
- `lower-trackmania-lights{lux-per-unit}`: each spot/ball a core.light per instance; a spot shines along its tree's +Z
  (StadiumWarpSpotsGlow quads face +Z 82/82); candela = intensity / a2 x lux-per-unit (TM falloff 1/(1 + a2 d^2)),
  range = ball radius; colour sRGB -> linear. Frustum/directional counted and left out (none in the three maps).
- Lab (primase-lab master 3104571, source kind `trackmania`, primase d827770db; landed: clippy, 875 unit tests):
  `check --views shaded,unlit,lights,notices,collision` on A01-Race, Turbulence (TMX 1354053), Iron Maiden (TMX
  136547): ok, 0 magenta/lime/cyan, nothing unsupported, lights 156 / 417 / 92. Output scratch/trackmania/lab/final3; final6 (the landed
  branch, no local patch) has the same 15 images byte for byte.
  The shaded view is dark by the lab's rule: the world states lights and no sun, so the lab drops its key light and
  exposes for the spots. That is faithful: the game places its sun at run time from the mood and how is not
  established (GBX.NET's day-time convention contradicts TMNF's values), so no core sun is made. The unlit view shows
  the textured geometry; final2 holds the pre-light shaded renders.
- Program (github.com/Kalmarv/trackmania-skate): `trackmania-export --jobs 3 --studio ...` (primase d827770db,
  foundations f9cf4ec, the revision primase shares) reads, lowers (materials, lights, spawns, surfaces), prepares and
  writes each map; Studio compiles each in ~20 s (cache/trackmania/build5). build5's map folders (223/292/302 files)
  are byte-identical to build4's, which the game runs used. Studio's compiled cas/toc differ, but Studio differs from
  itself on identical input (a01 compiled twice: build5, build5b). A fresh clone builds with `--locked`, no patch.
- Light calibration (choices.rs LIGHT_LUX_PER_UNIT = 7,700, one unit = skate.'s noon sun): skate.'s Noon exposes at
  EV 7.0-11.3, 3.7 stops under a physical noon. Run tm3 at 100,000 (physical sun) washed the road and skater white
  under the gantries (mean luma +30 to +80 over tm2); run tm4 at 7,700: +0 to +23, no clipping, pools under the
  gantries, the sun's shadows kept (cache/trackmania/runs/tm4/pairs.jpg: tm2 left, tm4 right). Both 12 of 12 shots,
  0 crash dumps, every travel point read where map.json puts it.
- Collision under the skater's physics (run drop1, tools/ingame_drop.sh, build4): dropped from 20 m above each map's 3
  travel points, the skater is at rest after 15 s on 9 of 9 drops, none through: 3 on the road (+0.59 to +0.61 m, the
  standing height the spawns read too), 6 on the roof of the checkpoint/finish arch above the point (+6.9 to +8.1 m;
  where_3, where_12 show the skater standing on its flags). 12 of 12 shots, 0 crash dumps.
- Skating on the maps (run roll1, scratch/trackmania/pad/run_roll.sh: a virtual Xbox 360 pad through ViGEmBus, plugged
  for the run): on all 3 maps the skater walks (left stick 4 s: 18 m each), gets on the board (trial 2: Y, then X taps) and rolls on the
  roads (A taps with the stick up: 89 / 30 / 86 m; readings at 2.5 to 10 km/h; where_3, where_14 show the skater on
  the board on the road), heights staying on the road. 15 of 15 shots, 0 crash dumps. Keyboard input does not move
  the skater (W through kit/gs-drive, run move3). Grinding is not tested (it needs a line onto a rail).
- Packages 0.2.1 (not published): cache/trackmania/releases/0.2.1/TrackMania_{A01Race,Turbulence,IronMaiden}-0.2.1.zip
  (35.6, 40.8, 48.0 MB), built from build6, laid out as SkateBIRD's (manifest, icon, README, CHANGELOG, trainer.json,
  reskate-levels.json beside Studio's Patch). 0.2.1 names the spawn and travel points Start, Checkpoint n, Finish
  (program pass trackmania-name-waypoints; the travel list's stop names too) and gives each page SkateBIRD's form: a
  shot in skate., the pause map's picture, the open-repo link (pictures in media/). Run media1 (scratch/trackmania/pad/
  run_media.sh) loaded all 3 from the packaged mod folders: spawns read where map.json puts them, 0 crash dumps; each
  zip's payload is byte-identical to the folders it loaded. Pad Back and Start open skate.'s menus, but the shot
  (ffmpeg gfxcapture) holds only the menu backdrop, no UI, so media/'s overhead pictures are ui/map_image.png, the
  pause map's own picture. 0.2.0 (cache/trackmania/releases/0.2.0) is superseded.

## Open
- The sun from the mood (direction from latitude, rise/fall and day time) needs the game's model: RE of TmForever.exe
  or in-game measurement. Not needed for skate (the program chooses skate's noon sun).

## Choices (program/src/choices.rs)
Noon, resident, no LODs, no GI; lights at 7,700 lux per unit (skate.'s sun), standard calibration; markers snapped to all
collision; start = spawn, checkpoints and finish = travel points (at most 24); textures capped at 1024; blended
surfaces as 0.5 cutouts; materialless primitives (grass fins, glows, fake shadows) left out.

## Remaining
- Thunderstore: the three 0.2.0 packages (cache/trackmania/releases/0.2.0) wait for Caleb (he said repo only).
- Grinding is by hand (no grind curves, as Los Santos: skate grinds the collision's own edges).
- Local patch configs are moved aside as .cargo/config.toml.local-off (here, the lab worktree, .worktrees/trackmania,
  crates/gbx-formats and its gbx-reader worktree); land.sh refuses a landing while one is in effect.
