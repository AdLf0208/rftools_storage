#!/usr/bin/env bash
# In-game check for RFTools Storage slice 1: the modular storage block.
#
# Default run (offscreen renderer, creative launch, then survival where a
# break must drop):
#   1. `rftoolsstorage:modular_storage` places and answers its readout
#      (`0/0 empty no module`);
#   2. CONTROL A — with no module the block refuses a stack: the readout stays
#      `0/0`, the stone stays in the hand, the action bar says so;
#   3. a `storage_module0` in hand installs, and the block reads `0/6400 empty
#      unlocked`;
#   4. CONTROL B — unlocked, the block still refuses a stack (upstream's
#      `if (!isLocked()) return stack`), and nothing is stored;
#   5. an empty-hand click seals it (the GUI padlock's stand-in) and a
#      right-click then stores the whole 64-stack;
#   6. `/rftoolsstorage lock` unlocks it again and re-locks it (the GUI
#      button's other half, which a module swap needs);
#   7. a second stack takes a second slot (128 stored: a barrel holds 64);
#   8. a third kind (dirt) takes its own slot (192 stored, 2 kinds);
#   9. an empty-hand right-click takes a stack of the first kind back out;
#  10. a second block filled to 6,400 refuses a stack — and takes it again
#      after one stack is withdrawn (the control inside the same block);
#  11. quitting and relaunching the same world keeps both blocks' contents;
#  12. CONTROL C (COUNTERFACTUAL=1 replaces the block with a vanilla barrel in
#      the insert phases): the same click does not store, and the readout says
#      `not a modular storage`.
#
# SELFTEST=1 keeps the scenario but asserts the barrel's 64-item capacity on
# the storage, so the run MUST fail with a `FAIL:` verdict: the capacity
# assertion is not vacuous.
#
# Since the SDK migration (2026-11-18) the mod is one flat `.hmod` the launcher
# compiles into the game, so GAME must name a game the launcher built with it:
#   horizon-launcher native-build ports/rftoolsstorage --instance <instance> --trust-native-code
# and then `GAME=<that horizon-client> tools/e2e.sh`. The retired `--dev-mod` and
# the embedded/control legs (one `.hnmod` writing its own content out) are gone
# with the format; the scenario, the counterfactual and the selftest remain.
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsstorage
SHARED=$CLONE/ports/_shared/bin/horizon-1d63e0a5
# A launcher-built `horizon-ctl` from the same build as GAME is the safe choice;
# the shared one speaks the same control protocol at this pin.
CTL=${CTL:-$SHARED/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsstorage-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}

STORAGE=rftoolsstorage:modular_storage
MODULE=rftoolsstorage:storage_module0
CAPACITY=6400

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME (build it with horizon-launcher native-build)" >&2; exit 1; }
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
# The readout answers in chat (native_feedback); the block id prefixes it, so a
# barrel in the counterfactual reads "minecraft:barrel: not a modular storage".
reply() { ctl chat "/rftoolsstorage $1 $2 $3 $4" >/dev/null; ctl wait 8 >/dev/null; ctl messages 8 | jq -r '.chat[]' | grep -m1 -E '^[a-z0-9_.-]+:[a-z0-9_./-]+:' || true; }
info() { reply info "$1" "$2" "$3"; }
info_amount() { info "$1" "$2" "$3" | sed -n 's/.*: \([0-9][0-9]*\)\/[0-9][0-9]*.*/\1/p'; }
info_state() { info "$1" "$2" "$3" | sed -n 's/.* \([a-z][a-z ]*\)$/\1/p'; }
lock() { reply lock "$1" "$2" "$3"; }
action_bar() { ctl messages 10 | jq -r '.actionbar // ""'; }
use() { ctl interact "$1" "$2" "$3" "$4" >/dev/null; ctl wait 6 >/dev/null; }

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
        --game "$GAME" --world RFToolsStorageE2E --seed 20261105 --creative --offscreen --wait \
        --game-dir "$SCRATCH/game" --resources "$RES" >/dev/null
    ctl wait-ready 180 >/dev/null
    if grep -q "content mods not loaded" "${SOCK}.log" 2>/dev/null; then
        grep "content mods not loaded" "${SOCK}.log" >&2
        fail "the content mod did not load"
    fi
}

say "launch (offscreen renderer, creative)"
launch

say "an Overworld platform at y=100, away from the terrain"
PX=$(ctl state | jq -r '.player.position[0]' | cut -d. -f1)
PZ=$(ctl state | jq -r '.player.position[2]' | cut -d. -f1)
ctl command "fill $((PX-8)) 100 $((PZ-8)) $((PX+8)) 100 $((PZ+8)) minecraft:stone" >/dev/null
ctl command "fill $((PX-8)) 101 $((PZ-8)) $((PX+8)) 110 $((PZ+8)) minecraft:air" >/dev/null
ctl tp "$PX.5" 101 "$((PZ+3)).5" >/dev/null
ctl wait 20
ctl command "gamerule spawn_mobs false" >/dev/null
ctl command "kill @e[type=!minecraft:player]" >/dev/null
ctl wait 20
X=$PX
Y=102
Z1=$((PZ + 1))
ctl close >/dev/null 2>&1 || true
ctl look 180 0
ctl wait 6

if [ "$COUNTERFACTUAL" = 1 ]; then
    PLACE=minecraft:barrel
else
    PLACE=$STORAGE
fi

say "place $PLACE at $X $Y $Z1"
ctl setblock "$X" "$Y" $((Z1 - 1)) minecraft:stone >/dev/null
ctl give "$PLACE" 1 >/dev/null || fail "cannot give $PLACE"
ctl wait 6
ctl select 1 >/dev/null || true
ctl wait 6
ctl place "$X" "$Y" "$Z1" --item "$PLACE" >/dev/null || fail "cannot place $PLACE"
ctl wait 6
[ "$(block_name "$X" "$Y" "$Z1")" = "$PLACE" ] || fail "the placed block is $(block_name "$X" "$Y" "$Z1"), expected $PLACE"
facing=$(block_facing "$X" "$Y" "$Z1")
echo "placed: $(block_state "$X" "$Y" "$Z1")"
[ -n "$facing" ] || fail "the storage block has no facing property"
ctl look-at block "$X" "$Y" "$Z1" "$facing" >/dev/null || fail "cannot aim at the block"
ctl wait 6

if [ "$COUNTERFACTUAL" = 1 ]; then
    say "counterfactual: a vanilla barrel is not a modular storage"
    line=$(info "$X" "$Y" "$Z1")
    echo "readout: $line"
    echo "$line" | grep -q 'not a modular storage' || fail "the barrel answered as a storage: $line"
    ctl command "clear @s" >/dev/null
    ctl give minecraft:stone 64 >/dev/null
    ctl wait 4
    ctl select minecraft:stone >/dev/null
    ctl wait 4
    use "$X" "$Y" "$Z1" "$facing" || true
    ctl close >/dev/null 2>&1 || true
    left=$(count_item minecraft:stone)
    echo "stone left in the hand after the barrel click: $left"
    [ "$left" -eq 64 ] || fail "the barrel swallowed the stone (a chest menu is not a storage)"
    echo "PASS: the same click does not store in a barrel; the storage behaviour is the port's own code"
    exit 0
fi

say "a fresh block: no module, and the readout says so"
line=$(info "$X" "$Y" "$Z1")
echo "readout: $line"
echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "unexpected empty readout: $line"

say "CONTROL A: with no module the block refuses a stack"
ctl command "clear @s" >/dev/null
ctl give minecraft:stone 64 >/dev/null || fail "cannot give stone"
ctl wait 6
ctl select minecraft:stone >/dev/null || fail "cannot select stone"
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "action bar: '$bar'; stored: ${amount:-none}; stone in the hand: $left"
case "$bar" in
    *"no storage module"*) ;;
    *) fail "the refusal did not name the missing module: '$bar'" ;;
esac
[ "${amount:-0}" = 0 ] || fail "a storage with no module stored $amount"
[ "$left" = 64 ] || fail "the block took $((64 - left)) stone without a module"

say "a storage module installs from the hand"
ctl command "clear @s" >/dev/null
ctl give "$MODULE" 1 >/dev/null || fail "cannot give the module"
ctl wait 6
ctl select "$MODULE" >/dev/null || fail "cannot select the module"
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
line=$(info "$X" "$Y" "$Z1")
modules=$(count_item "$MODULE")
echo "action bar: '$bar'; readout: $line; modules in the hand: $modules"
echo "$line" | grep -q "^$STORAGE: 0/$CAPACITY empty unlocked$" || fail "unexpected readout after the module: $line"
[ "$modules" = 0 ] || fail "the module was not taken from the hand"

say "CONTROL B: unlocked, the block still refuses a stack"
ctl command "clear @s" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
ctl select minecraft:stone >/dev/null
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "action bar: '$bar'; stored: ${amount:-none}; stone in the hand: $left"
case "$bar" in
    *unlocked*) ;;
    *) fail "the unlocked refusal did not say so: '$bar'" ;;
esac
[ "${amount:-0}" = 0 ] || fail "an unlocked storage stored $amount"
[ "$left" = 64 ] || fail "an unlocked storage took $((64 - left)) stone"

say "an empty-hand click seals it (the GUI padlock's stand-in)"
ctl command "clear @s" >/dev/null
ctl wait 4
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
line=$(info "$X" "$Y" "$Z1")
echo "action bar: '$bar'; readout: $line"
echo "$line" | grep -q "^$STORAGE: 0/$CAPACITY empty locked$" || fail "the empty-hand click did not lock: $line"

say "a locked storage stores a whole stack"
ctl command "clear @s" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
ctl select minecraft:stone >/dev/null
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "action bar: '$bar'; stored: $amount; stone in the hand: $left"
[ "$amount" = 64 ] || fail "the storage holds $amount, expected 64"
[ "$left" = 0 ] || fail "the storage left $left stone in the hand"

say "the lock command unlocks and re-locks it (the GUI button's other half)"
line=$(lock "$X" "$Y" "$Z1")
unlocked=$(info_state "$X" "$Y" "$Z1")
line2=$(lock "$X" "$Y" "$Z1")
relocked=$(info_state "$X" "$Y" "$Z1")
echo "lock command: '$line' then '$line2'"
[ "$unlocked" = "unlocked" ] || fail "the lock command did not unlock the block (state: $unlocked)"
[ "$relocked" = "locked" ] || fail "a second lock command did not lock it again (state: $relocked)"
amount=$(info_amount "$X" "$Y" "$Z1")
[ "$amount" = 64 ] || fail "the lock toggle changed the contents to $amount"

say "a second stack takes a second slot (a barrel holds 64)"
ctl command "clear @s" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
ctl select minecraft:stone >/dev/null
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
amount=$(info_amount "$X" "$Y" "$Z1")
echo "stored: $amount"
[ "$amount" = 128 ] || fail "the storage holds $amount, expected 128 (two slots)"

say "another kind takes its own slot"
ctl command "clear @s" >/dev/null
ctl give minecraft:dirt 64 >/dev/null
ctl wait 6
ctl select minecraft:dirt >/dev/null
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
line=$(info "$X" "$Y" "$Z1")
dirt=$(count_item minecraft:dirt)
echo "readout: $line; dirt in the hand: $dirt"
echo "$line" | grep -q "^$STORAGE: 192/$CAPACITY minecraft:stone locked$" || fail "unexpected readout after the dirt: $line"
[ "$dirt" = 0 ] || fail "the storage left $dirt dirt in the hand"

say "an empty-hand click takes a stack of the first kind back out"
ctl command "clear @s" >/dev/null
ctl wait 4
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
amount=$(info_amount "$X" "$Y" "$Z1")
stone=$(count_item minecraft:stone)
echo "action bar: '$bar'; stored: $amount; stone in the hand: $stone"
[ "$stone" = 64 ] || fail "the storage gave $stone stone, expected 64"
[ "$amount" = 128 ] || fail "the storage holds $amount after the withdrawal, expected 128"
[ "$(count_item minecraft:dirt)" = 0 ] || fail "the withdrawal took the wrong kind"

say "fill the storage to its tier-1 capacity (6,400) and watch it refuse more"
# The harness fills the block entity's own container slots (`ctl container`
# writes slot by slot, first empty first); the refusal below is the mod's own
# insert path, and the extra lines fail harmlessly once the slots are full.
python3 -c "
import sys
x, y, z = sys.argv[1:4]
print(chr(10).join(f'container {x} {y} {z} minecraft:stone 64' for _ in range(120)))
" "$X" "$Y" "$Z1" > "$SCRATCH/fill.txt"
# `script --keep-going` still reports the last failure, and the last lines of
# this file hit "container is full": the readout below is the assertion.
ctl script "$SCRATCH/fill.txt" --keep-going >/dev/null || true
line=$(info "$X" "$Y" "$Z1")
echo "readout: $line"
echo "$line" | grep -q "^$STORAGE: $CAPACITY/$CAPACITY minecraft:stone locked$" || fail "the filled storage reads: $line"
ctl command "clear @s" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
ctl select minecraft:stone >/dev/null
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "action bar: '$bar'; stored: $amount; stone in the hand: $left"
case "$bar" in
    *full*) ;;
    *) fail "a full storage did not say it is full: '$bar'" ;;
esac
[ "$amount" = "$CAPACITY" ] || fail "a full storage changed to $amount"
[ "$left" = 64 ] || fail "a full storage took $((64 - left)) stone"

say "…and takes a stack again once one is withdrawn (the control inside the block)"
ctl command "clear @s" >/dev/null
ctl wait 4
use "$X" "$Y" "$Z1" "$facing"
withdrawn=$(count_item minecraft:stone)
before=$(info_amount "$X" "$Y" "$Z1")
ctl select minecraft:stone >/dev/null
ctl wait 6
use "$X" "$Y" "$Z1" "$facing"
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "withdrew $withdrawn ($CAPACITY -> $before), re-inserted: stored $amount, stone in the hand: $left"
[ "$withdrawn" = 64 ] || fail "the withdrawal from the full storage gave $withdrawn"
[ "$before" = "$((CAPACITY - 64))" ] || fail "the withdrawal left $before stored, expected $((CAPACITY - 64))"
[ "$amount" = "$CAPACITY" ] || fail "the storage did not take the stack back: $amount"
[ "$left" = 0 ] || fail "the storage left $left stone in the hand"

say "the contents survive a relaunch"
before1=$(info_amount "$X" "$Y" "$Z1")
echo "quit with $before1 stored"
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch
ctl wait 20 >/dev/null
after1=$(info_amount "$X" "$Y" "$Z1")
state1=$(info_state "$X" "$Y" "$Z1")
echo "relaunch: $before1 -> $after1 (state $state1)"
[ "$(block_name "$X" "$Y" "$Z1")" = "$STORAGE" ] || fail "the storage is gone after the relaunch"
[ "$after1" = "$before1" ] || fail "the relaunch changed the storage $before1 -> $after1"
[ "$state1" = "locked" ] || fail "the lock did not survive the relaunch (state $state1)"

if [ "$SELFTEST" = 1 ]; then
    say "SELFTEST: assert the barrel's 64-item capacity on the storage (must fail)"
    [ "$after1" = 64 ] || fail "the storage holds $after1, not 64: the capacity assertion is alive"
fi

echo "PASS: the modular storage refuses items with no module and while unlocked, stores 64-stack slots (128 in two, 192 with a second kind), gives a stack back, reads 6400/6400 when full and refuses more (taking it again after a withdrawal), keeps its contents and lock across a relaunch, and a barrel does not behave this way"
