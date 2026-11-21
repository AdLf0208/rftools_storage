#!/usr/bin/env bash
# In-game check for RFTools Storage slice 5: the **storage scanner**.
#
# Default run (offscreen renderer, creative launch):
#   1. `rftoolsstorage:storage_scanner` places, reads back with its `facing`;
#   2. a fresh scanner has **no power**: `/rftoolsstorage scan <pos>` answers
#      `no power (0/50000 FE)` and reads nothing;
#   3. charged to 5,000 FE (`/data merge block … {energy: 5000}`), the scanner at
#      its **default radius 1** finds nothing two blocks away — the 3 x 3 x 3 box;
#   4. `/rftoolsstorage radius <pos> 2` (the GUI's radius slider, which this slice
#      has no screen for) widens it: the scan answers **`minecraft:stone 64`** for
#      a chest two blocks away, and not the `minecraft:dirt 32` of a chest three
#      away (outside the box) — the counterfactual;
#   5. radius 3 brings the dirt too: the box is the scanner's, and the scan counts
#      each kind once across every inventory in it;
#   6. every scan costs `rfPerRequest` (100 FE): the reply prints the buffer after
#      it, and 5,000 → 4,900 → 4,800 is asserted at every step;
#   7. an emptied buffer (`energy: 0`) answers `no power (0/50000 FE)` and reads
#      nothing; charging it again makes the same scan answer again;
#   8. a scan of a block that is not a scanner answers `not a storage scanner`
#      (the modular storage and a chest alike);
#   9. the radius and the buffer survive a relaunch and the scan still answers;
#  10. the scanner's loot table drops its own item on a survival break;
#  11. the carried `storage_scanner` recipe is in the table: its pattern with a
#      stone block where `rftoolsbase:machine_frame` belongs crafts nothing, and
#      `/give rftoolsbase:machine_frame` is refused — the frame is RFToolsBase's
#      item and that mod is not in this build (the campaign's port of it pins
#      `0b14f8a`), so the recipe is **blocked**, recorded in `notes.md`;
#  12. the modular storage still answers its readout (`0/0 empty no module`).
#
# SELFTEST=1 keeps the scenario and asserts that the scan did NOT find the stone,
# so the run MUST fail with a `FAIL:` verdict (the assertion phase 4 relies on is
# proved able to fail). COUNTERFACTUAL=1 puts a vanilla barrel where the scanner
# would be and proves the same commands answer `not a storage scanner` on it.
# CONTROL_GAME runs the same rig on the Nightly without the package and fails
# there (`EXIT:1`): the block and its item do not exist.
#
# The SDK migration made the mod one flat `.hmod` the launcher compiles into the
# game, so GAME must name a game the launcher built with it:
#   ports/tools/launcher-game.sh build rftoolsstorage
#   eval "$(ports/tools/launcher-game.sh env rftoolsstorage)"
#   build-slot.sh run --job e2e-rftoolsstorage --port ports/rftoolsstorage \
#       --step e2e -- bash tools/e2e.sh
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsstorage
SHARED=$CLONE/ports/_shared/bin/horizon-1d63e0a5
# The shared `horizon-ctl` speaks the same control protocol at this pin; a
# launcher-built one beside GAME is what `launcher-game.sh env` exports.
CTL=${CTL:-$SHARED/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsstorage-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}

SCANNER=rftoolsstorage:storage_scanner
STORAGE=rftoolsstorage:modular_storage
MAX_FE=50000
PRICE=100

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME (build it with ports/tools/launcher-game.sh)" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "${E2E_TMP:-/tmp/opencode}"/rftoolsstorage-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
say() { echo "== $* =="; }
count_item() { ctl inventory | jq -r --arg i "$1" '.counts[$i] // 0'; }
block_name() { ctl block "$1" "$2" "$3" | jq -r '.block // "unknown"'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // "unknown"'; }
block_facing() { block_state "$1" "$2" "$3" | sed -n 's/.*facing=\([a-z]*\).*/\1/p'; }
# A mod command's answer in chat: native commands are heard through the player's
# chat, so `ctl chat` (not `ctl command`, which runs the console path).
reply() { ctl chat "/rftoolsstorage $*" >/dev/null; ctl wait 8 >/dev/null; ctl messages 8 | jq -r '.chat[]' | grep -m1 -E '^[a-z0-9_.-]+:[a-z0-9_./-]+:' || true; }
# `/data get block X Y Z path`: the block entity's saved value, with no mod code in
# the path. A path reply prints `<x>, <y>, <z> has the following block data: <value>`.
data_block() { # x y z path
    local x=$1 y=$2 z=$3 path=$4 line value
    ctl command "data get block $x $y $z $path" >/dev/null
    ctl wait 8 >/dev/null
    line=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'has the following block data' || true)
    [ -n "$line" ] || fail "no data get reply for $path at $x,$y,$z: $(ctl messages 6 | jq -c '.chat')"
    value=${line##*has the following block data: }
    value=${value#\"}
    value=${value%\"}
    echo "$value"
}
# Sets the scanner's saved buffer: the `energy` int the game's own energy handler
# reads (`ModEnergyBuffer`), written with vanilla's own `/data merge block`.
set_energy() { # x y z amount
    ctl command "data merge block $1 $2 $3 {energy: $4}" >/dev/null
    ctl wait 6 >/dev/null
}
# A chest at (x, y, z) holding `count` of `item` in its first slot.
fill_chest() { # x y z item count
    ctl setblock "$1" "$2" "$3" minecraft:chest >/dev/null
    ctl wait 2 >/dev/null
    ctl command "item replace block $1 $2 $3 container.0 with $4 $5" >/dev/null
    ctl wait 2 >/dev/null
    local n id
    n=$(data_block "$1" "$2" "$3" 'Items[0].count')
    id=$(data_block "$1" "$2" "$3" 'Items[0].id')
    echo "  chest at $1,$2,$3: $n x $id" >&2
    [ "$n" = "$5" ] || fail "the chest at $1,$2,$3 holds '$n', expected $5"
    [ "$id" = "$4" ] || fail "the chest at $1,$2,$3 holds '$id', expected $4"
}
# The scanner's scan reply (a command a player sends: `ctl chat`).
scan() { # x y z
    local line
    line=$(reply scan "$1" "$2" "$3")
    [ -n "$line" ] || fail "the scan at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
radius_cmd() { # x y z n
    local line
    line=$(reply radius "$1" "$2" "$3" "$4")
    [ -n "$line" ] || fail "the radius command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
energy_of() { data_block "$1" "$2" "$3" energy; }

# Stand two blocks in front of (x, z) on the rig's floor: `ctl place` refuses a
# cell the player's own box overlaps. The console `/tp` is used and verified.
stand_at() { # x z [y]
    local want_y=${3:-101} i arrived
    for i in 1 2 3; do
        ctl command "execute in minecraft:overworld run tp @s $1.5 $want_y $2.5" >/dev/null
        ctl wait 8 >/dev/null
        arrived=$(ctl state | jq -r --arg x "$1.5" --arg z "$2.5" \
            '.player.position as $p
             | ((($p[0] - ($x|tonumber)) | if . < 0 then -. else . end) < 1)
               and ((($p[2] - ($z|tonumber)) | if . < 0 then -. else . end) < 1)')
        [ "$arrived" = true ] && return 0
    done
    echo "  stand_at $1,$2: the player is at $(ctl state | jq -c '.player.position') after three teleports" >&2
}

# Place the held/given item at a position and check the block is there; a refusal
# keeps the stack and is retried from the same spot.
place_held() { # x y z block
    local x=$1 y=$2 z=$3 block=$4 i reply name
    for i in 1 2 3; do
        stand_at "$x" "$((z + 2))"
        reply=$(ctl place "$x" "$Y" "$z" --item "$block" 2>&1) || true
        ctl wait 8 >/dev/null
        name=$(block_name "$x" "$Y" "$z")
        [ "$name" = "$block" ] && return 0
        echo "  place attempt $i at $x,$Y,$z from $(ctl state | jq -c '.player.position'): $reply (block: ${name:-unknown})" >&2
        [ "$(count_item "$block")" != 0 ] \
            || fail "$block left the inventory and is not at $x,$Y,$z"
        ctl wait 10 >/dev/null
    done
    fail "$block did not place at $x,$Y,$z"
}

# --- the crafting table (the recipe leg) -----------------------------------
# One ingredient into a crafting grid cell: pick the stack up, right-click the
# cell (which places one), put the rest back.
place_one() {
    local item=$1 cell=$2 src
    src=$(ctl screen | jq -r --arg i "$item" '[.slots[]? | select(.slot < 36 and .item == $i) | .slot] | first // ""')
    [ -n "$src" ] || fail "no $item in the inventory for the recipe"
    ctl click-slot "$src" >/dev/null
    ctl wait 1 >/dev/null
    ctl click-slot "$cell" right >/dev/null
    ctl wait 1 >/dev/null
    ctl click-slot "$src" >/dev/null
    ctl wait 1 >/dev/null
}
# A crafting table at the given spot with the grid open (the harness's recipe path).
open_table() { # x y z
    ctl close >/dev/null 2>&1 || true
    ctl wait 2 >/dev/null
    ctl setblock "$1" "$2" "$3" minecraft:air >/dev/null
    ctl setblock "$1" "$2" "$3" minecraft:crafting_table >/dev/null
    ctl wait 2 >/dev/null
    ctl command "execute in minecraft:overworld run tp @s $(( $1 - 1 )).5 $2 $(( $3 )).5" >/dev/null
    ctl wait 4 >/dev/null
    ctl interact "$1" "$2" "$3" >/dev/null || fail "the crafting table did not take the click"
    ctl wait 6 >/dev/null
    # The crafting table's screen reports `inventory` in this build; its grid is
    # slots 100..108 with the result at 200 (36 + 9 + 1).
    local grid
    grid=$(ctl screen | jq '[.slots[]? | select(.slot >= 100)] | length')
    [ "$grid" = 10 ] || fail "the crafting table did not open its grid ($grid grid slots)"
}
craft_result() { ctl screen | jq -r '[.slots[]? | select(.slot == 200 and ((.item // "") != "")) | "\(.item) x\(.count)"] | first // "empty"'; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" 2>/dev/null || true
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() {
    "$CTL" --socket "$SOCK" launch \
        --game "$1" --world RFToolsStorageE2E --seed 20261105 --creative --offscreen --wait \
        --game-dir "$SCRATCH/game" --resources "$RES" >/dev/null
    ctl wait-ready 180 >/dev/null
    if grep -q "content mods not loaded" "${SOCK}.log" 2>/dev/null; then
        grep "content mods not loaded" "${SOCK}.log" >&2
        fail "the content mod did not load"
    fi
}

if [ -n "$CONTROL_GAME" ]; then
    # The control is the same rig on the same Nightly with the package left out:
    # the scenario cannot even start, so the leg MUST fail (`EXIT:1` in the log).
    say "control: the same rig on the game without the package (must fail)"
    launch "$CONTROL_GAME"
    ctl command "fill -8 100 -8 8 100 8 minecraft:stone" >/dev/null
    ctl command "fill -8 101 -8 8 110 8 minecraft:air" >/dev/null
    ctl tp 0.5 101 3.5 >/dev/null
    ctl wait 20 >/dev/null
    got=$(ctl give "$SCANNER" 1 2>&1 || true)
    echo "control give: $got"
    echo "$got" | grep -q 'unknown item' || fail "the control game accepted $SCANNER: $got"
    fail "without the package there is no $SCANNER to place or scan with, so nothing of this scenario can run"
fi

say "launch (offscreen renderer, creative)"
launch "$GAME"

say "an Overworld platform at y=100, away from the terrain"
PX=$(ctl state | jq -r '.player.position[0]' | cut -d. -f1)
PZ=$(ctl state | jq -r '.player.position[2]' | cut -d. -f1)
ctl command "fill $((PX-9)) 100 $((PZ-9)) $((PX+9)) 100 $((PZ+9)) minecraft:stone" >/dev/null
ctl command "fill $((PX-9)) 101 $((PZ-9)) $((PX+9)) 115 $((PZ+9)) minecraft:air" >/dev/null
ctl tp "$PX.5" 101 "$((PZ+3)).5" >/dev/null
ctl wait 20
ctl command "gamerule spawn_mobs false" >/dev/null
ctl command "kill @e[type=!minecraft:player]" >/dev/null
ctl wait 20
X=$PX
Y=102
Z1=$((PZ + 1))
SX=$X
SY=$Y
SZ=$Z1
ctl close >/dev/null 2>&1 || true
ctl look 180 0
ctl wait 6

if [ "$COUNTERFACTUAL" = 1 ]; then
    PLACE=minecraft:barrel
else
    PLACE=$SCANNER
fi

say "place $PLACE at $SX $SY $SZ"
ctl setblock "$SX" "$((SY - 1))" "$SZ" minecraft:stone >/dev/null
ctl give "$PLACE" 1 >/dev/null || fail "cannot give $PLACE"
ctl wait 6
ctl select 1 >/dev/null || true
ctl wait 6
ctl place "$SX" "$SY" "$SZ" --item "$PLACE" >/dev/null || fail "cannot place $PLACE"
ctl wait 6
[ "$(block_name "$SX" "$SY" "$SZ")" = "$PLACE" ] || fail "the placed block is $(block_name "$SX" "$SY" "$SZ"), expected $PLACE"
facing=$(block_facing "$SX" "$SY" "$SZ")
echo "placed: $(block_state "$SX" "$SY" "$SZ")"
ctl look-at block "$SX" "$SY" "$SZ" "${facing:-north}" >/dev/null || true
ctl wait 6

if [ "$COUNTERFACTUAL" = 1 ]; then
    say "counterfactual: a vanilla barrel answers 'not a storage scanner' to the same commands"
    line=$(scan "$SX" "$SY" "$SZ")
    echo "scan: $line"
    echo "$line" | grep -q 'not a storage scanner' || fail "the barrel answered as a scanner: $line"
    line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
    echo "radius: $line"
    echo "$line" | grep -q 'not a storage scanner' || fail "the barrel took a radius: $line"
    echo "PASS: a barrel is not a storage scanner — /rftoolsstorage scan and radius refuse it — so the scan's answers come from the port's own block, not from any block with a block entity"
    exit 0
fi

[ -n "$facing" ] || fail "the scanner block has no facing property"

say "a fresh scanner has no power: the scan refuses and reads nothing"
line=$(scan "$SX" "$SY" "$SZ")
echo "scan: $line"
echo "$line" | grep -q "^$SCANNER: no power (0/$MAX_FE FE)$" || fail "a fresh scanner answered: $line"
case "$line" in
    *minecraft:stone*) fail "an unpowered scan read an inventory: $line" ;;
    *) ;;
esac

say "charged to 5,000 FE, the default radius 1 finds nothing two blocks away"
set_energy "$SX" "$SY" "$SZ" 5000
charged=$(energy_of "$SX" "$SY" "$SZ")
echo "saved buffer: $charged"
[ "$charged" = 5000 ] || fail "the buffer reads $charged, expected 5000"

say "a chest two blocks away holds 64 stone; one three away holds 32 dirt"
CHEST_X=$((SX + 2))
FAR_X=$((SX + 3))
fill_chest "$CHEST_X" "$SY" "$SZ" minecraft:stone 64
fill_chest "$FAR_X" "$SY" "$SZ" minecraft:dirt 32

line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan: $line; buffer now $spent"
case "$line" in
    *minecraft:stone*) fail "radius 1 read a chest two blocks away: $line" ;;
    *) ;;
esac
echo "$line" | grep -q "empty ($((5000 - PRICE))/$MAX_FE FE)" || fail "the radius-1 scan answered: $line"
[ "$spent" = "$((5000 - PRICE))" ] || fail "the scan left $spent FE, expected $((5000 - PRICE))"

say "/rftoolsstorage radius 2 widens the box (the GUI's radius slider)"
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
echo "$line" | grep -q "^$SCANNER: radius 2$" || fail "the radius command answered: $line"
saved_radius=$(data_block "$SX" "$SY" "$SZ" radius)
echo "saved radius: $saved_radius"
[ "$saved_radius" = 2 ] || fail "the saved radius is $saved_radius, expected 2"

say "THE SLICE'S ASSERTION: the scan answers the chest two blocks away (minecraft:stone 64)"
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:stone 64 (4800/$MAX_FE FE)$" || fail "the radius-2 scan answered: $line"
[ "$spent" = "$((5000 - 2 * PRICE))" ] || fail "two scans left $spent FE, expected $((5000 - 2 * PRICE))"
case "$line" in
    *minecraft:dirt*) fail "the scan read the chest three blocks away at radius 2: $line" ;;
    *) ;;
esac

if [ "$SELFTEST" = 1 ]; then
    say "SELFTEST: assert the scan did NOT find the stone (must fail)"
    echo "$line" | grep -q 'minecraft:stone 64' \
        || fail "the scan found no stone, so the assertion this run inverts is dead: $line"
    fail "the scan found minecraft:stone 64 in the chest two blocks away ($line), so this inverted assertion fails as it must"
fi

say "radius 3 brings the chest three away too: both kinds, one entry each"
line=$(radius_cmd "$SX" "$SY" "$SZ" 3)
echo "radius: $line"
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan: $line; buffer now $spent"
echo "$line" | grep -q '^'"$SCANNER"': minecraft:dirt 32, minecraft:stone 64 (' || fail "the radius-3 scan answered: $line"
[ "$spent" = "$((5000 - 3 * PRICE))" ] || fail "three scans left $spent FE, expected $((5000 - 3 * PRICE))"

say "an emptied buffer refuses the scan with no power, and reads nothing"
set_energy "$SX" "$SY" "$SZ" 0
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: no power (0/$MAX_FE FE)$" || fail "an empty buffer answered: $line"
case "$line" in
    *minecraft:stone*) fail "an unpowered scan read an inventory: $line" ;;
    *) ;;
esac
[ "$spent" = 0 ] || fail "a refused scan changed the buffer to $spent"

say "charged again, the same scan answers again"
set_energy "$SX" "$SY" "$SZ" 5000
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan: $line; buffer now $spent"
echo "$line" | grep -q 'minecraft:stone 64' || fail "the recharged scan answered: $line"
[ "$spent" = "$((5000 - PRICE))" ] || fail "a scan cost $((5000 - spent)) FE, expected $PRICE"

say "a scan of a block that is not a scanner is refused, and changes nothing"
line=$(scan "$CHEST_X" "$SY" "$SZ")
echo "scan of the chest: $line"
echo "$line" | grep -q '^minecraft:chest: not a storage scanner$' || fail "the chest answered: $line"

say "the modular storage still answers its own readout, and is not a scanner"
STORAGE_AT=$((SZ + 5))
ctl command "clear @a" >/dev/null
ctl setblock "$((X + 5))" "$((Y - 1))" "$STORAGE_AT" minecraft:stone >/dev/null
stand_at "$((X + 5))" "$STORAGE_AT"
ctl give "$STORAGE" 1 >/dev/null || fail "cannot give $STORAGE"
ctl wait 6
ctl select 1 >/dev/null || true
ctl wait 6
place_held "$((X + 5))" "$Y" "$STORAGE_AT" "$STORAGE"
line=$(reply info "$((X + 5))" "$Y" "$STORAGE_AT")
echo "info: $line"
echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "the modular storage's readout: $line"
line=$(scan "$((X + 5))" "$Y" "$STORAGE_AT")
echo "scan of the modular storage: $line"
echo "$line" | grep -q "^$STORAGE: not a storage scanner$" || fail "the modular storage answered: $line"

say "the radius and the buffer survive a relaunch, and the scan still answers"
before=$(energy_of "$SX" "$SY" "$SZ")
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch "$GAME"
ctl wait 20 >/dev/null
after=$(energy_of "$SX" "$SY" "$SZ")
saved_radius=$(data_block "$SX" "$SY" "$SZ" radius)
line=$(scan "$SX" "$SY" "$SZ")
echo "relaunch: buffer $before -> $after, radius $saved_radius; scan: $line"
[ "$after" = "$before" ] || fail "the relaunch changed the buffer $before -> $after"
[ "$saved_radius" = 3 ] || fail "the relaunch changed the radius to $saved_radius"
echo "$line" | grep -q 'minecraft:stone 64' || fail "the scan after the relaunch answered: $line"

say "the scanner's loot table drops its own item on a survival break"
BX=$((X + 2))
BZ=$((Z1 + 5))
ctl command "clear @a" >/dev/null
ctl setblock "$BX" "$((Y - 1))" "$BZ" minecraft:stone >/dev/null
ctl give "$SCANNER" 1 >/dev/null || fail "cannot give a scanner to break"
ctl wait 6
ctl select 1 >/dev/null || true
ctl wait 6
place_held "$BX" "$Y" "$BZ" "$SCANNER"
ctl command "gamemode survival" >/dev/null
ctl wait 20 >/dev/null
ctl command "clear @a" >/dev/null
ctl give minecraft:diamond_pickaxe 1 >/dev/null || fail "cannot give the pickaxe"
ctl wait 6
ctl select minecraft:diamond_pickaxe >/dev/null || fail "cannot select the pickaxe"
ctl wait 6
ctl command "execute in minecraft:overworld run tp @s $((BX + 3)).5 $Y $((BZ + 1)).5" >/dev/null
ctl wait 8 >/dev/null
BREAK=$(ctl break "$BX" "$Y" "$BZ") || fail "the scanner did not break"
ctl wait 10 >/dev/null
[ "$(block_name "$BX" "$Y" "$BZ")" = "minecraft:air" ] || fail "the scanner is still there after mining"
echo "break: $(jq -c '{drops, picked_up, items_left_on_ground}' <<<"$BREAK")"
[ "$(jq -r --arg i "$SCANNER" '.drops[$i] // 0' <<<"$BREAK")" = 1 ] \
    || fail "the break dropped no $SCANNER: $(jq -c '.drops' <<<"$BREAK")"
ctl command "gamemode creative" >/dev/null
ctl wait 20 >/dev/null

say "the carried storage-scanner recipe is in the table; its frame is not in this build"
# The recipe is loaded and parsed by the game's own recipe manager: `recipe give`
# names it, and an unknown id is refused.
loaded=$(ctl command "recipe give @a rftoolsstorage:storage_scanner" 2>&1 || true)
echo "recipe give rftoolsstorage:storage_scanner: $loaded"
case "$loaded" in
    *'"error"'*) fail "the storage_scanner recipe is not loaded: $loaded" ;;
    *) ;;
esac
TX=$((X - 5))
TY=$Y
TZ=$((Z1 + 4))
ctl command "clear @a" >/dev/null
for item in minecraft:redstone_torch minecraft:gold_ingot minecraft:ender_pearl minecraft:stone; do
    ctl give "$item" 4 >/dev/null
done
ctl wait 8
open_table "$TX" "$TY" "$TZ"
# "ToT" / "gFg" / "ToT": torches 100/102/106/108, ender pearls 101/107, gold 103/105,
# and a stone block where `rftoolsbase:machine_frame` belongs (104).
place_one minecraft:redstone_torch 100
place_one minecraft:ender_pearl 101
place_one minecraft:redstone_torch 102
place_one minecraft:gold_ingot 103
place_one minecraft:stone 104
place_one minecraft:gold_ingot 105
place_one minecraft:redstone_torch 106
place_one minecraft:ender_pearl 107
place_one minecraft:redstone_torch 108
no_frame=$(craft_result)
echo "the scanner recipe with a stone block where rftoolsbase:machine_frame belongs: '$no_frame'"
[ "$no_frame" = "empty" ] || fail "a stone block crafted $no_frame in the machine frame's cell"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
frame=$(ctl give rftoolsbase:machine_frame 1 2>&1 || true)
echo "give rftoolsbase:machine_frame: $frame"
echo "$frame" | grep -q 'unknown item' || fail "this build has rftoolsbase:machine_frame, so the recipe is not blocked and must be crafted: $frame"

ctl close >/dev/null 2>&1 || true
echo "PASS: the storage scanner places and reads back with its facing; a fresh one has no power (no power (0/50000 FE)) and reads nothing; at its default radius 1 it finds nothing two blocks away; /rftoolsstorage radius 2 widens the box and the scan answers minecraft:stone 64 for a chest two blocks away while the chest three away (minecraft:dirt 32) is outside it; radius 3 lists both, one entry a kind; every scan costs rfPerRequest (5,000 -> 4,900 -> 4,800 -> 4,700, printed in the reply); an emptied buffer refuses with no power and reads nothing, and charging it again makes the scan answer again; scan and radius refuse a chest and the modular storage (not a storage scanner); the radius and the buffer survive a relaunch; the loot table drops the scanner on a survival break; the modular storage still answers 0/0 empty no module; and a barrel answers not a storage scanner to the same commands"
