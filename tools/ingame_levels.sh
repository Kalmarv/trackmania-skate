#!/usr/bin/env bash
# Every map in one skate. launch, through the shared runner (reskate-tools ingame.sh in copy mode with several mods:
# each map's mod copied in, every other mod disabled, all restored after; crash reporting off). Each map is loaded in
# turn, shot at the spawn and at up to three of its travel points (from its map.json; every `trainer where` is a shot).
#   kit/run game --job -- "$BASH" tools/ingame_levels.sh TAG [MAP ...]     (default: every map)
# BUILD: the program's --out folder (default <modding>/cache/trackmania/build). Output: cache/trackmania/runs/TAG.
set -u
TAG=$1; shift
E=/c/Users/kalmarv/modding/reskate/reskate-tools/ports/gta/engine
B=${BUILD:-/c/Users/kalmarv/modding/cache/trackmania/build}
R=/c/Users/kalmarv/modding/cache/trackmania/runs/$TAG
# the tooling (cargo build --release -p trackmania-tools in the repo); TRACKMANIA_TOOLS to use another build
X=${TRACKMANIA_TOOLS:-/c/Users/kalmarv/modding/games/trackmania/target/release/trackmania-tools.exe}
MAPS=("$@"); [ ${#MAPS[@]} -gt 0 ] || MAPS=(a01 turbulence iron-maiden)
SRCS=; NAMES=; CMDS="wait 100"; SECS=100; SHOTS=0
for L in "${MAPS[@]}"; do
  # the packaged mod folder (what a player installs) when there is one, else Studio's Patch
  M=$("$X" views map-name "$L" | tr -d '\r') || exit 2
  S=$B/$L/mod/$M; [ -d "$S" ] || S=$B/$L/stage/Patch
  [ -d "$S" ] || { echo "$L: no mod or stage in $B"; exit 2; }
  SRCS+=${SRCS:+,}$S; NAMES+=${NAMES:+,}$M
  C=$("$X" views skate-commands "$L" --out "$B" | tr -d '\r') || exit 2
  CMDS+=";${C#wait 100;}"
  N=$(grep -o "trainer where" <<< "$C" | wc -l)
  SHOTS=$((SHOTS + N))
  # load + settle + the spawn view held 25 s, each travel view 12 + 25 s, with room for the load
  SECS=$((SECS + 120 + 37 * (N - 1)))
done
mkdir -p "$R"
printf '%s\n' "${MAPS[@]}" > "$R/maps.txt"
STOP_AFTER_WHERE=$SHOTS NO_DLL=1 "$BASH" "$E/ingame.sh" "$R/run" copy "$SRCS" "$NAMES" "$CMDS" "$SECS" > "$R/run.txt" 2>&1; e=$?
echo "runner exit $e; shots $(ls "$R"/run/where_*.png 2>/dev/null | grep -vc _small) of $SHOTS; crash dumps: $(grep -c 'crash dump' "$R/run.txt")"
# the console lines in order: per map, the load and its `trainer where` readings (one per shot)
grep -a "load levels/game\|\[Console\] \[INFO\] (" "$R/run/ReSkate.log" 2>/dev/null | cut -c1-200
exit $e
