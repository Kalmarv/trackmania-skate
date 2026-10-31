# TrackMania: state (trackmania-map-reader, 2026-10-31; rewrite, never append)

## Goal
Caleb: "finish the gbx reader, and verify it in lab (through core primase) and make 3 different reskate levels from
trackmania maps following the general style of the other reskate maps like skatebird and los santos - using primase as
a library to read, pass, write".

## What works (verified)
- Reader: gbx-formats (crates/gbx-formats, branch gbx-reader worktree) reads Gbx maps and the NadeoPak packs;
  b1c154c adds collector descs and pack-file heads (every pack: 2360 heads, 313 descs, 312 bodies name the same ident).
  primase-trackmania (primase worktree .worktrees/trackmania, branch trackmania, local commits 06651deff..3e37e6c73 on
  main 1f0952904) places every block from the Stadium pack.
- Lab (primase-lab worktree .worktrees/lab-trackmania, source kind `trackmania`, local commit ad9c601, primase pinned at 3e37e6c73):
  `check --views shaded,notices,collision` on A01-Race, Turbulence (TMX 1354053) and Iron Maiden (TMX 136547): ok,
  0 magenta/lime/cyan pixels, every draw faithful (3227, 2779, 3556). Reports and PNGs in
  scratch/trackmania/lab/final/<map>. Unread: CPlugTreeLight visuals (92-258 per map), 4-6 non-DDS/cube images.
- Program: `trackmania-export --jobs 3 --studio ...` reads, lowers, prepares and writes each map in 3-4 s and Studio
  compiles it in ~8 s (build cache/trackmania/build1). Studio warnings are the usual (render objects merged, collision
  edges joined, BAM Lighting_Global, layout superbundle without install chunk; Iron Maiden also a light-probe overlap).
- In game (run tm2, job 34fe93c2, `kit/run game`, ingame.sh copy mode, the packaged mod folders, all three in one
  launch): 12 of 12 shots, 0 crash dumps, profile restored exactly. Each map: spawn plus three travel points (first,
  middle, last); every `trainer where` reads its point 0.6-0.65 m up, standing (A01 spawn (171.20, 90.95, 688.00) for
  map.json [171.2, 90.3, 688.0]). Contact sheet cache/trackmania/runs/tm2/sheet.jpg: start arches, checkpoints,
  finishes, ramps, textured and lit. `trainer tp`/`where` use map.json's frame unchanged (tm1 negated z and fell).
- Packages 0.1.0 (not published): cache/trackmania/releases/0.1.0/TrackMania_{A01Race,Turbulence,IronMaiden}-0.1.0.zip
  (35.6, 40.8, 47.9 MB), SkateBIRD's layout, icon.png from tm2 shots 4, 8, 11 (cache/trackmania/icons).

## Choices (program/src/choices.rs)
Noon, resident, no LODs, no GI; standard light calibration; markers snapped to all collision (TM blocks are instanced
prototypes, not static ground); start = spawn, checkpoints and finish = travel points (at most 24); textures capped at
1024; blended surfaces as 0.5 cutouts; materialless primitives (grass fins, glows, fake shadows) left out.

## primase additions (trackmania branch, local)
- meshops `first-texcoord-set`: TM's projected grass/soil sample set 3, skate samples set 0.
- reskate `ground-reskate-markers{from=all}`.
- reskate `prepare-reskate-uv-transforms` leaves materials nothing draws.
- worlds/trackmania: reader, lower-trackmania-{materials,spawns,surfaces}.

## Waiting on Caleb (nothing pushed)
- Push gbx-formats; primase-foundations zlib-stream d64ea51; primase-testdata eb199df (then move the pin); primase
  branch trackmania (then the program's Cargo.toml pins one public rev and .cargo/config.toml goes); the lab source.
- Create github.com/Kalmarv/trackmania-skate (manifest website_url) and push this repo.
- Publishing the three packages on Thunderstore.

## Next
- Caleb: approve the pushes above, then pin primase by its public rev and drop .cargo/config.toml; play the maps
  (rolling, grinding and fast travel are untested by hand).
