#!/usr/bin/env bash
# The maps' collision under the skater's physics, in one skate. launch, through the shared runner (reskate-tools ingame.sh
# in copy mode, as ingame_levels.sh): each map is loaded and the skater read and shot at the spawn, then dropped from
# DROP m (default 20) above each of its first DROPS (default 4) travel points (`trainer tp`, the points `views
# skate-commands` puts on the road) and read and shot after 15 s. The GTA engine's drop test (reskate-tools
# REPORT_ENGINE.md: a 27 m drop ends in a bail at rest on the surface): a reading within a metre or so of the point's
# height is the skater stopped by the map's own collision; falling through reads far below it.
#   tools/ingame_drop.sh TAG [MAP ...]     (default: every map)
# RESKATE_TOOLS: a reskate-tools checkout (default $MODDING_ROOT/reskate/reskate-tools). BUILD: the program's --out
# folder (default $MODDING_CACHE/trackmania/build). RUNS: where runs go (default $MODDING_CACHE/trackmania/runs).
# Output: $RUNS/TAG, with
# drops.tsv (map, point, target height, read position).
set -u
TAG=$1; shift
W=$(cd "$(dirname "$0")/.." && pwd)
E=${RESKATE_TOOLS:-${MODDING_ROOT:?set RESKATE_TOOLS (or MODDING_ROOT)}/reskate/reskate-tools}/ports/gta/engine
B=${BUILD:-${MODDING_CACHE:?set BUILD (or MODDING_CACHE)}/trackmania/build}
R=${RUNS:-${MODDING_CACHE:?set RUNS (or MODDING_CACHE)}/trackmania/runs}/$TAG
X=${TRACKMANIA_TOOLS:-$W/target/release/trackmania-tools.exe}
DROP=${DROP:-20}; DROPS=${DROPS:-4}
MAPS=("$@"); [ ${#MAPS[@]} -gt 0 ] || MAPS=(a01 turbulence iron-maiden)
SRCS=; NAMES=; CMDS="wait 100"; SECS=100; SHOTS=0
mkdir -p "$R"; : > "$R/targets.tsv"
for L in "${MAPS[@]}"; do
  M=$("$X" views map-name "$L" | tr -d '\r') || exit 2
  S=$B/$L/mod/$M; [ -d "$S" ] || S=$B/$L/stage/Patch
  [ -d "$S" ] || { echo "$L: no mod or stage in $B"; exit 2; }
  SRCS+=${SRCS:+,}$S; NAMES+=${NAMES:+,}$M
  C=$("$X" views skate-commands "$L" --out "$B" | tr -d '\r') || exit 2
  # the load and the spawn's reading (up to its first `trainer where`)
  P=${C#wait 100;}; P=${P%%trainer where*}
  CMDS+=";${P}trainer where;wait 10"; SHOTS=$((SHOTS + 1)); SECS=$((SECS + 70))
  printf '%s\tspawn\t-\n' "$L" >> "$R/targets.tsv"
  n=0
  while read -r x y z; do
    [ "$n" -lt "$DROPS" ] || break
    n=$((n + 1))
    CMDS+=";trainer tp $x $(awk "BEGIN{printf \"%.3f\", $y + $DROP}") $z;wait 15;trainer where;wait 5"
    SHOTS=$((SHOTS + 1)); SECS=$((SECS + 25))
    printf '%s\t%s %s %s\t%s\n' "$L" "$x" "$y" "$z" "$y" >> "$R/targets.tsv"
  done < <(tr ';' '\n' <<< "$C" | sed -n 's/^trainer tp //p')
done
printf '%s\n' "${MAPS[@]}" > "$R/maps.txt"
STOP_AFTER_WHERE=$SHOTS NO_DLL=1 "$BASH" "$E/ingame.sh" "$R/run" copy "$SRCS" "$NAMES" "$CMDS" "$SECS" > "$R/run.txt" 2>&1; e=$?
echo "runner exit $e; shots $(ls "$R"/run/where_*.png 2>/dev/null | grep -vc _small) of $SHOTS; crash dumps: $(grep -c 'crash dump' "$R/run.txt")"
# each reading beside its target: map, point, target height, read (x, y, z), read height minus target
grep -a "\[Console\] \[INFO\] (" "$R/run/ReSkate.log" 2>/dev/null | sed 's/.*INFO\] (\([^)]*\)).*/\1/' | tr -d ' ' \
  | paste "$R/targets.tsv" - | awk -F'\t' 'BEGIN{OFS="\t"} {split($4, p, ","); d = ($3 == "-") ? "-" : sprintf("%+.2f", p[2] - $3); print $1, $2, $3, $4, d}' \
  | tee "$R/drops.tsv"
exit $e
