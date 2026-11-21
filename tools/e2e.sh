#!/usr/bin/env bash
# In-game check for RFTools Storage slice 3: the storage's card slot.
#
# Default run (offscreen renderer, creative launch, then survival where a break
# must drop):
#   1. `rftoolsstorage:modular_storage` places and answers its readout
#      (`0/0 empty no module`);
#   2. the plain click opens the menu: 100 storage slots **and the card slot**
#      (101 container slots), the panel 366 x 210;
#   3. CONTROL A — the card slot refuses a stack that is not a storage module: the
#      stone stays on the cursor, the card slot stays empty, the readout stays
#      `0/0`;
#   4. a `storage_module0` put into the card slot installs it (`0/6400 empty
#      unlocked`, the item in slot 400, none left in the inventory);
#   5. it comes back out through the slot (`0/0 empty no module`, the item in the
#      inventory) — the gesture upstream's GUI has and this port did not;
#   6. the sneak click still installs it (`0/6400 empty unlocked`, slot 400 holds
#      it), and CONTROL B — unlocked, the grid still refuses the stone;
#   7. the padlock seals it and a locked storage stores a stack put into a grid
#      slot (`64/6400`); while locked, the sneak gesture refuses a module;
#   8. the module comes out of the card slot and the block keeps its contents:
#      `0/0 empty no module`, and the module back in the slot finds them
#      (`64/6400 minecraft:stone unlocked`) — the number this slice is about;
#   9. one kind a slot, a click on a filled slot, `/rftoolsstorage lock`, the fill
#      to 6,400 with the card slot untouched, and the withdrawal/re-insert;
#  10. quitting and relaunching keeps the contents, the lock **and the module in
#      the card slot**;
#  11. the carried `storage_module0` recipe crafts in a real crafting table; a
#      stick in the iron's cell and a stone block where
#      `rftoolsbase:machine_frame` belongs craft nothing;
#  12. CONTROL C (COUNTERFACTUAL=1 replaces the block with a vanilla barrel):
#      the same click opens a 27-slot chest menu and its slot takes the stack the
#      storage refuses while unlocked.
#
# SELFTEST=1 keeps the scenario and asserts the barrel's 27-slot menu on the
# storage, so the run MUST fail with a `FAIL:` verdict. NEGATIVE=1 stops after
# the card slot's refusal and asserts that a stone WAS taken, so that assertion
# is proved able to fail too.
#
# Since the SDK migration (2026-11-18) the mod is one flat `.hmod` the launcher
# compiles into the game, so GAME must name a game the launcher built with it:
#   ports/tools/launcher-game.sh build rftoolsstorage
#   eval "$(ports/tools/launcher-game.sh env rftoolsstorage)"
#   build-slot.sh run --job e2e-rftoolsstorage --port ports/rftoolsstorage --step e2e -- bash tools/e2e.sh
# The retired `--dev-mod` and the embedded legs (one `.hnmod` writing its own
# content out) are gone with the format; the scenario, the counterfactual and the
# selftest remain.
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsstorage
SHARED=$CLONE/ports/_shared/bin/horizon-1d63e0a5
# A launcher-built `horizon-ctl` from the same build as GAME is the safe choice;
# the shared one speaks the same control protocol at this pin.
CTL=${CTL:-$SHARED/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/rftoolsstorage-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}
NEGATIVE=${NEGATIVE:-0}

STORAGE=rftoolsstorage:modular_storage
MODULE=rftoolsstorage:storage_module0
CAPACITY=6400
# The port's menu: 100 storage slots, then the card slot, then the player's 36.
# The ctl numbers a container slot `300 + index`, so the card slot (index 100) is
# 400. The panel is 366 x 210 (`src/support.rs`): the padlock sits at [32, 6] with
# a 46 x 16 rectangle, so its centre is (55, 14) from the panel's top left.
MENU_SLOTS=101
CARD=400
PANEL_W=366
PANEL_H=210
LOCK_X=55
LOCK_Y=14

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
# The readout answers in chat (native_feedback); the block id prefixes it, so a
# barrel in the counterfactual reads "minecraft:barrel: not a modular storage".
reply() { ctl chat "/rftoolsstorage $1 $2 $3 $4" >/dev/null; ctl wait 8 >/dev/null; ctl messages 8 | jq -r '.chat[]' | grep -m1 -E '^[a-z0-9_.-]+:[a-z0-9_./-]+:' || true; }
info() { reply info "$1" "$2" "$3"; }
info_amount() { info "$1" "$2" "$3" | sed -n 's/.*: \([0-9][0-9]*\)\/[0-9][0-9]*.*/\1/p'; }
info_state() { info "$1" "$2" "$3" | sed -n 's/.* \([a-z][a-z ]*\)$/\1/p'; }
lock() { reply lock "$1" "$2" "$3"; }
action_bar() { ctl messages 10 | jq -r '.actionbar // ""'; }
# The plain click: upstream's GUI. The sneak click: this port's quick gesture.
use() { ctl interact "$1" "$2" "$3" "$4" >/dev/null; ctl wait 6 >/dev/null; }
quick() {
    ctl key shift down >/dev/null
    ctl wait 6 >/dev/null
    ctl interact "$1" "$2" "$3" "$4" >/dev/null
    ctl key shift up >/dev/null
    ctl wait 6 >/dev/null
}
# --- the open menu ---------------------------------------------------------
menu_slots() { ctl screen | jq '[.slots[]? | select(.slot >= 300)] | length'; }
slot_item() { ctl screen | jq -r --arg s "$1" '[.slots[]? | select((.slot|tostring) == $s) | .item] | first // ""'; }
slot_count() { ctl screen | jq -r --arg s "$1" '[.slots[]? | select((.slot|tostring) == $s) | .count] | first // 0'; }
inv_slot() { ctl screen | jq -r --arg i "$1" '[.slots[]? | select(.slot < 300 and .item == $i) | .slot] | first // ""'; }
empty_inv_slot() { ctl screen | jq -r '[.slots[]? | select(.slot < 36 and ((.item // "") == "")) | .slot] | first // ""'; }
craft_slot() { ctl screen | jq -r --arg s "$1" '[.slots[]? | select((.slot|tostring) == $s) | .item] | first // ""'; }
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
    grid=$(ctl screen | jq '[.slots[]? | select(.slot >= 100)] | length')
    [ "$grid" = 10 ] || fail "the crafting table did not open its grid ($grid grid slots)"
}
panel_origin() {
    local ui w h
    ui=$(ctl ui)
    w=$(jq -r '.gui_size[0] | floor' <<<"$ui")
    h=$(jq -r '.gui_size[1] | floor' <<<"$ui")
    OX=$(( (w - PANEL_W) / 2 ))
    OY=$(( (h - PANEL_H) / 2 ))
}
open_menu() { # open_menu X Y Z FACE -> the storage's menu, panel origin in OX/OY
    ctl close >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    use "$1" "$2" "$3" "$4" || true
    ctl wait-for screen container --timeout 10 >/dev/null \
        || fail "the click did not open a menu (screen=$(ctl state | jq -r '.screen // ""'))"
    panel_origin
}
# The open menu re-synced from the server: a mod menu's client copy can keep a
# click it predicted that the server refused (or miss one it accepted) until the
# menu is opened again, so every slot read after a click goes through this.
sync_menu() {
    ctl close >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    open_menu "$X" "$Y" "$Z1" "$facing"
}
press_lock() { ctl click-at "$((OX + LOCK_X))" "$((OY + LOCK_Y))" >/dev/null || fail "the padlock press did not send"; ctl wait 6 >/dev/null; }
# Puts the stack the player holds in `ITEM` on the cursor: a left click on its
# inventory slot. Prints that slot, which is empty until the stack goes back.
carry() {
    local slot
    slot=$(inv_slot "$1")
    [ -n "$slot" ] || fail "no $1 in the inventory to carry"
    ctl click-slot "$slot" >/dev/null || fail "cannot pick $1 up"
    ctl wait 2 >/dev/null
    echo "$slot"
}
put_back() { # put_back SLOT: the cursor stack into inventory slot SLOT
    ctl click-slot "$1" >/dev/null
    ctl wait 2 >/dev/null
}

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
    say "control: the game without the package does not know the storage"
    launch "$CONTROL_GAME"
    ctl command "fill -8 100 -8 8 100 8 minecraft:stone" >/dev/null
    ctl command "fill -8 101 -8 8 110 8 minecraft:air" >/dev/null
    ctl tp 0.5 101 3.5 >/dev/null
    ctl wait 20 >/dev/null
    if ctl give "$STORAGE" 1 2>/dev/null | grep -q '"ok":true'; then
        fail "the package-less game gave $STORAGE (the control is not a control)"
    fi
    got=$(ctl give "$STORAGE" 1 2>&1 || true)
    echo "control give: $got"
    echo "PASS: without the package the block does not exist, so nothing of this scenario can run"
    exit 0
fi

say "launch (offscreen renderer, creative)"
launch "$GAME"

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
    say "counterfactual: a vanilla barrel opens a chest menu, not the storage's grid"
    line=$(info "$X" "$Y" "$Z1")
    echo "readout: $line"
    echo "$line" | grep -q 'not a modular storage' || fail "the barrel answered as a storage: $line"
    ctl command "clear @a" >/dev/null
    ctl give minecraft:stone 64 >/dev/null
    ctl wait 4
    open_menu "$X" "$Y" "$Z1" "$facing"
    barrel_slots=$(menu_slots)
    echo "the barrel's menu shows $barrel_slots container slots"
    [ "$barrel_slots" = 27 ] || fail "the barrel's menu shows $barrel_slots slots, expected 27 (our storage's $MENU_SLOTS would mean the grid is not ours)"
    # The barrel has no card slot: the same click on our slot 400 lands past the
    # barrel's 27 and takes nothing.
    [ "$CARD" -ge $((300 + barrel_slots)) ] || fail "the card slot is inside a barrel's menu, so the differential proves nothing"
    carry minecraft:stone >/dev/null
    ctl click-slot 300 >/dev/null
    ctl wait 2 >/dev/null
    barrel_stone=$(slot_count 300)
    echo "the barrel took $barrel_stone into its first slot (the storage refuses an unlocked stack)"
    [ "$barrel_stone" = 64 ] || fail "the barrel took $barrel_stone, expected 64"
    ctl close >/dev/null 2>&1 || true
    echo "PASS: the same click opens a 27-slot chest menu on a barrel, which has no card slot at 400, and its first slot takes the stack, so the storage's menu, its card slot and the locked-only rule are the port's own"
    exit 0
fi

say "a fresh block: no module, and the readout says so"
line=$(info "$X" "$Y" "$Z1")
echo "readout: $line"
echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "unexpected empty readout: $line"

say "the plain click opens the storage's menu: 100 storage slots and the card slot"
ctl command "clear @a" >/dev/null
ctl give minecraft:stone 64 >/dev/null || fail "cannot give stone"
ctl wait 6
open_menu "$X" "$Y" "$Z1" "$facing"
slots=$(menu_slots)
card=$(slot_item "$CARD")
echo "panel origin $OX,$OY; the menu shows $slots container slots; the card slot: '${card:-empty}'"
[ "$slots" = "$MENU_SLOTS" ] || fail "the menu shows $slots container slots, expected $MENU_SLOTS"
[ -z "$card" ] || fail "a fresh storage's card slot holds '$card'"

say "CONTROL A: the card slot refuses a stack that is not a storage module"
carry minecraft:stone >/dev/null
ctl click-slot "$CARD" >/dev/null
ctl wait 4 >/dev/null
predicted=$(slot_item "$CARD")
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "the click left the open menu showing '$predicted'; the server reads: $line; stone in the inventory: $left"
if [ "$NEGATIVE" = 1 ]; then
    say "NEGATIVE: the run must fail — it asserts the card slot took the stone"
    echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "the card slot took the stone: the port's rule is broken, not this assertion"
    fail "the card slot refused the stone (the server reads $line), so this inverted assertion fails as it must"
fi
echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "the card slot took the stone: the server reads $line"
[ "$left" = 64 ] || fail "the refused card-slot click lost $((64 - left)) stone"
# The client's prediction of a refused click is corrected when the menu syncs again.
open_menu "$X" "$Y" "$Z1" "$facing"
reopened=$(slot_item "$CARD")
echo "reopened, the card slot is '${reopened:-empty}'"
[ -z "$reopened" ] || fail "the card slot shows '$reopened' after the menu reopened"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "a storage module goes into the card slot (upstream's SLOT_STORAGE_MODULE)"
ctl command "clear @a" >/dev/null
ctl give "$MODULE" 1 >/dev/null || fail "cannot give the module"
ctl wait 6
open_menu "$X" "$Y" "$Z1" "$facing"
carry "$MODULE" >/dev/null
ctl click-slot "$CARD" >/dev/null
ctl wait 4 >/dev/null
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
modules=$(count_item "$MODULE")
echo "readout: $line; modules in the inventory: $modules"
echo "$line" | grep -q "^$STORAGE: 0/$CAPACITY empty unlocked$" || fail "the card slot's module did not set the storage: $line"
[ "$modules" = 0 ] || fail "the module was not taken from the inventory"
open_menu "$X" "$Y" "$Z1" "$facing"
card=$(slot_item "$CARD")
card_count=$(slot_count "$CARD")
echo "the reopened menu's card slot: '$card_count $card'"
[ "$card" = "$MODULE" ] || fail "the card slot holds '$card', expected $MODULE"
[ "$card_count" = 1 ] || fail "the card slot holds $card_count, expected 1"

say "…and it comes back out through the card slot (nothing could take it out before)"
ctl click-slot "$CARD" >/dev/null
ctl wait 2 >/dev/null
dst=$(empty_inv_slot)
[ -n "$dst" ] || fail "no empty inventory slot for the withdrawn module"
ctl click-slot "$dst" >/dev/null || fail "cannot put the module into the inventory"
ctl wait 4 >/dev/null
card=$(slot_item "$CARD")
echo "the card slot after taking the module out: '${card:-empty}'"
[ -z "$card" ] || fail "the card slot still holds '$card'"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
modules=$(count_item "$MODULE")
echo "readout: $line; modules in the inventory: $modules"
echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "a storage with no module reads: $line"
[ "$modules" = 1 ] || fail "the card slot gave back $modules modules, expected 1"

say "a storage module in the hand still installs (the sneak gesture, now into the slot)"
ctl select "$MODULE" >/dev/null || fail "cannot select the module"
ctl wait 6
quick "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
line=$(info "$X" "$Y" "$Z1")
modules=$(count_item "$MODULE")
echo "action bar: '$bar'; readout: $line; modules in the hand: $modules"
echo "$line" | grep -q "^$STORAGE: 0/$CAPACITY empty unlocked$" || fail "unexpected readout after the module: $line"
[ "$modules" = 0 ] || fail "the module was not taken from the hand"
open_menu "$X" "$Y" "$Z1" "$facing"
card=$(slot_item "$CARD")
echo "the gesture left the module in the card slot: '$card'"
[ "$card" = "$MODULE" ] || fail "the gesture did not leave the module in the card slot ('$card')"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "CONTROL B: unlocked, the menu's grid still refuses a stack"
ctl command "clear @a" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
open_menu "$X" "$Y" "$Z1" "$facing"
carry minecraft:stone >/dev/null
ctl click-slot 300 >/dev/null
ctl wait 4 >/dev/null
predicted=$(slot_count 300)
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
echo "the click left the open menu showing '$predicted' in slot 300; the server reads: $line"
echo "$line" | grep -q "^$STORAGE: 0/$CAPACITY empty unlocked$" || fail "an unlocked storage took the stack: $line"
# The client's prediction of a refused click is corrected when the menu syncs again.
open_menu "$X" "$Y" "$Z1" "$facing"
refused=$(slot_count 300)
echo "reopened, slot 300 is '$refused'"
[ "${refused:-0}" = 0 ] || fail "the reopened grid shows $refused in slot 300"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
ctl command "clear @a" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
ctl select minecraft:stone >/dev/null
ctl wait 6
quick "$X" "$Y" "$Z1" "$facing"
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

say "the panel's padlock seals it (upstream's lockButton)"
ctl command "clear @a" >/dev/null
ctl wait 4
open_menu "$X" "$Y" "$Z1" "$facing"
press_lock
bar=$(action_bar)
line=$(info "$X" "$Y" "$Z1")
echo "action bar: '$bar'; readout: $line"
echo "$line" | grep -q "^$STORAGE: 0/$CAPACITY empty locked$" || fail "the padlock did not lock: $line"

say "a locked storage stores the stack put into a grid slot"
ctl command "clear @a" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
open_menu "$X" "$Y" "$Z1" "$facing"
carry minecraft:stone >/dev/null
ctl click-slot 300 >/dev/null
ctl wait 4 >/dev/null
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "stored: $amount (the server); stone in the hand: $left"
[ "$amount" = 64 ] || fail "the storage holds $amount, expected 64"
[ "$left" = 0 ] || fail "the storage left $left stone in the hand"
sync_menu
stored=$(slot_count 300)
item=$(slot_item 300)
echo "the re-synced grid's slot 300: '$stored $item'"
[ "$stored" = 64 ] || fail "slot 300 holds $stored, expected 64"
[ "$item" = minecraft:stone ] || fail "slot 300 holds $item, expected minecraft:stone"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "while locked, the card slot takes no module (the sneak gesture says so)"
ctl command "clear @a" >/dev/null
ctl give "$MODULE" 1 >/dev/null || fail "cannot give a module"
ctl wait 6
ctl select "$MODULE" >/dev/null || fail "cannot select the module"
ctl wait 6
quick "$X" "$Y" "$Z1" "$facing"
bar=$(action_bar)
line=$(info "$X" "$Y" "$Z1")
modules=$(count_item "$MODULE")
echo "action bar: '$bar'; readout: $line; modules in the hand: $modules"
case "$bar" in
    *locked*) ;;
    *) fail "a locked storage did not refuse the module: '$bar'" ;;
esac
[ "$modules" = 1 ] || fail "a locked storage took the module from the hand"
open_menu "$X" "$Y" "$Z1" "$facing"
card=$(slot_item "$CARD")
echo "the card slot still holds: '$card'"
[ "$card" = "$MODULE" ] || fail "the card slot lost its module ('$card')"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "the module comes out of the card slot, and the block keeps its contents"
ctl command "clear @a" >/dev/null
ctl wait 4 >/dev/null
open_menu "$X" "$Y" "$Z1" "$facing"
ctl click-slot "$CARD" >/dev/null
ctl wait 2 >/dev/null
dst=$(empty_inv_slot)
[ -n "$dst" ] || fail "no empty inventory slot for the withdrawn module"
ctl click-slot "$dst" >/dev/null || fail "cannot put the module into the inventory"
ctl wait 4 >/dev/null
card=$(slot_item "$CARD")
echo "the card slot after taking the module out: '${card:-empty}'"
[ -z "$card" ] || fail "the card slot still holds '$card'"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
modules=$(count_item "$MODULE")
echo "readout with the module out: $line; modules in the inventory: $modules"
echo "$line" | grep -q "^$STORAGE: 0/0 empty no module$" || fail "a storage whose card slot is empty reads: $line"
[ "$modules" = 1 ] || fail "the card slot gave back $modules modules, expected 1"

say "…and the module back in the card slot finds the 64 stone it left"
open_menu "$X" "$Y" "$Z1" "$facing"
carry "$MODULE" >/dev/null
ctl click-slot "$CARD" >/dev/null
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
echo "readout after the module came back: $line"
# The block's own lock byte outlived the module (the storage is shown unlocked
# and takes a module while it is out), so the storage is locked again with it.
echo "$line" | grep -q "^$STORAGE: 64/$CAPACITY minecraft:stone locked$" || fail "the contents did not survive the module's removal: $line"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "one kind a slot: dirt is refused by the stone's slot"
ctl command "clear @a" >/dev/null
ctl give minecraft:dirt 64 >/dev/null
ctl wait 6
open_menu "$X" "$Y" "$Z1" "$facing"
carry minecraft:dirt >/dev/null
ctl click-slot 300 >/dev/null
ctl wait 4 >/dev/null
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
echo "after the dirt click on the stone's slot: $line"
echo "$line" | grep -q "^$STORAGE: 64/$CAPACITY minecraft:stone locked$" || fail "the stone's slot took the dirt: $line"
sync_menu
still=$(slot_item 300)
echo "the re-synced grid's slot 300: '$still' (the stone keeps its slot)"
[ "$still" = minecraft:stone ] || fail "slot 300 became '$still', expected the stone to stay"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "…and takes the grid's last one"
ctl command "clear @a" >/dev/null
ctl give minecraft:dirt 64 >/dev/null
ctl wait 6
open_menu "$X" "$Y" "$Z1" "$facing"
carry minecraft:dirt >/dev/null
ctl click-slot 399 >/dev/null
ctl wait 4 >/dev/null
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
line=$(info "$X" "$Y" "$Z1")
echo "readout: $line"
echo "$line" | grep -q "^$STORAGE: 128/$CAPACITY minecraft:stone locked$" || fail "unexpected readout after the dirt: $line"
sync_menu
dirt=$(slot_count 399)
dirt_item=$(slot_item 399)
echo "the re-synced grid's slot 399 (the last of the grid) holds: '$dirt $dirt_item'"
[ "$dirt" = 64 ] || fail "slot 399 holds '$dirt', expected 64 dirt"
[ "$dirt_item" = minecraft:dirt ] || fail "slot 399 holds '$dirt_item'"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "a click on a filled grid slot takes the stack back"
open_menu "$X" "$Y" "$Z1" "$facing"
ctl click-slot 300 >/dev/null
ctl wait 2 >/dev/null
dst=$(empty_inv_slot)
[ -n "$dst" ] || fail "no empty inventory slot for the withdrawn stone"
ctl click-slot "$dst" >/dev/null || fail "cannot put the withdrawn stone into the inventory"
ctl wait 4 >/dev/null
amount=$(info_amount "$X" "$Y" "$Z1")
stone=$(count_item minecraft:stone)
echo "stored: $amount; stone in the inventory: $stone"
[ "$stone" = 64 ] || fail "the grid gave $stone stone, expected 64"
[ "$amount" = 64 ] || fail "the storage holds $amount after the withdrawal, expected 64"
[ "$(count_item minecraft:dirt)" = 0 ] || fail "the withdrawal took the wrong kind"
ctl close >/dev/null 2>&1 || true

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

say "fill the storage to its tier-1 capacity (6,400) and watch the grid refuse more"
# The harness fills the block entity's own container slots (`ctl container`
# writes slot by slot, first empty first, and answers an error once they are all
# taken); the refusal below is the storage's own capacity, and the readout is
# the assertion. The cap is the storage's hundred slots: the card slot is not one
# of them, and `ctl container` does not know that (it writes the first empty slot
# of the container, card slot included).
writes=0
while [ "$writes" -lt 99 ]; do
    if ctl container "$X" "$Y" "$Z1" minecraft:stone 64 | jq -e 'has("error") | not' >/dev/null 2>&1; then
        writes=$((writes + 1))
    else
        break
    fi
done
echo "the harness filled $writes slots"
line=$(info "$X" "$Y" "$Z1")
echo "readout: $line"
echo "$line" | grep -q "^$STORAGE: $CAPACITY/$CAPACITY minecraft:stone locked$" || fail "the filled storage reads: $line"
open_menu "$X" "$Y" "$Z1" "$facing"
card=$(slot_item "$CARD")
echo "the card slot with the storage full: '$card'"
[ "$card" = "$MODULE" ] || fail "the fill wrote into the card slot ('$card')"
ctl command "clear @a" >/dev/null
ctl give minecraft:stone 64 >/dev/null
ctl wait 6
carry minecraft:stone >/dev/null
ctl click-slot 300 >/dev/null
ctl wait 4 >/dev/null
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "stored: $amount; stone in the hand: $left (nothing fits, the stack stays)"
[ "$amount" = "$CAPACITY" ] || fail "a full storage changed to $amount"
[ "$left" = 64 ] || fail "a full storage took $((64 - left)) stone"
sync_menu
full_item=$(slot_item 300)
full_count=$(slot_count 300)
echo "the re-synced grid's slot 300 with the storage full: '$full_count $full_item'"
[ "$full_item" = minecraft:stone ] || fail "slot 300 holds '$full_item'"
[ "$full_count" = 64 ] || fail "slot 300 holds $full_count, expected its own 64"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "…and takes it again once one stack is withdrawn through the grid"
inv_before=$(count_item minecraft:stone)
open_menu "$X" "$Y" "$Z1" "$facing"
ctl click-slot 300 >/dev/null
ctl wait 2 >/dev/null
dst=$(empty_inv_slot)
[ -n "$dst" ] || fail "no empty inventory slot for the withdrawn stack"
ctl click-slot "$dst" >/dev/null || fail "cannot put the withdrawn stack into the inventory"
ctl wait 6 >/dev/null
withdrawn=$(( $(count_item minecraft:stone) - inv_before ))
before=$(info_amount "$X" "$Y" "$Z1")
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
ctl select minecraft:stone >/dev/null
ctl wait 6 >/dev/null
quick "$X" "$Y" "$Z1" "$facing"
amount=$(info_amount "$X" "$Y" "$Z1")
left=$(count_item minecraft:stone)
echo "the grid gave $withdrawn ($CAPACITY -> $before); the gesture took it back: stored $amount, stone in the inventory $inv_before -> $left"
[ "$withdrawn" = 64 ] || fail "the withdrawal from the full storage gave $withdrawn"
[ "$before" = "$((CAPACITY - 64))" ] || fail "the withdrawal left $before stored, expected $((CAPACITY - 64))"
[ "$amount" = "$CAPACITY" ] || fail "the storage did not take the stack back: $amount"
[ "$left" = "$inv_before" ] || fail "the re-inserted stack left $left in the inventory, expected $inv_before"

say "the contents, the lock, the grid and the module's slot survive a relaunch"
before1=$(info_amount "$X" "$Y" "$Z1")
echo "quit with $before1 stored"
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch "$GAME"
ctl wait 20 >/dev/null
after1=$(info_amount "$X" "$Y" "$Z1")
state1=$(info_state "$X" "$Y" "$Z1")
echo "relaunch: $before1 -> $after1 (state $state1)"
[ "$(block_name "$X" "$Y" "$Z1")" = "$STORAGE" ] || fail "the storage is gone after the relaunch"
[ "$after1" = "$before1" ] || fail "the relaunch changed the storage $before1 -> $after1"
[ "$state1" = "locked" ] || fail "the lock did not survive the relaunch (state $state1)"
open_menu "$X" "$Y" "$Z1" "$facing"
reopened=$(slot_count 300)
kept_card=$(slot_item "$CARD")
unlocked_bar=$(action_bar)
echo "the reopened grid shows $reopened in slot 300, the card slot '$kept_card'; action bar: '$unlocked_bar'"
[ "$reopened" = 64 ] || fail "the reopened grid shows $reopened in slot 300, expected 64"
[ "$kept_card" = "$MODULE" ] || fail "the card slot lost its module across the relaunch ('$kept_card')"

say "the carried storage-module recipe crafts in a real crafting table"
TX=$((X + 4))
TY=$Y
TZ=$Z1
ctl command "clear @a" >/dev/null
for item in minecraft:chest minecraft:gold_nugget minecraft:iron_ingot minecraft:quartz minecraft:redstone; do
    ctl give "$item" 4 >/dev/null
done
ctl wait 8
open_table "$TX" "$TY" "$TZ"
# " C " / "gig" / "qrq": chest 101, gold nuggets 103 and 105, iron ingot 104,
# quartz 106 and 108, redstone 107.
place_one minecraft:chest 101
place_one minecraft:gold_nugget 103
place_one minecraft:iron_ingot 104
place_one minecraft:gold_nugget 105
place_one minecraft:quartz 106
place_one minecraft:redstone 107
place_one minecraft:quartz 108
made=$(ctl screen | jq -r '[.slots[]? | select(.slot == 200) | "\(.item) x\(.count)"] | first // "empty"')
echo "the grid's result: '$made'"
[ "$made" = "$MODULE x1" ] || fail "the module recipe's result is '$made', expected $MODULE x1"
ctl click-slot 200 >/dev/null || fail "cannot take the crafted module"
ctl wait 2 >/dev/null
dst=$(empty_inv_slot)
ctl click-slot "$dst" >/dev/null || fail "cannot put the crafted module into the inventory"
ctl wait 4 >/dev/null
crafted=$(count_item "$MODULE")
echo "crafted modules in the inventory: $crafted"
[ "$crafted" = 1 ] || fail "the crafted module reached the inventory as $crafted"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

say "control: a stick in the iron's cell, and a block where the machine frame goes, craft nothing"
ctl command "clear @a" >/dev/null
for item in minecraft:chest minecraft:gold_nugget minecraft:stick minecraft:quartz minecraft:redstone; do
    ctl give "$item" 4 >/dev/null
done
ctl wait 8
open_table "$TX" "$TY" "$TZ"
place_one minecraft:chest 101
place_one minecraft:gold_nugget 103
place_one minecraft:stick 104
place_one minecraft:gold_nugget 105
place_one minecraft:quartz 106
place_one minecraft:redstone 107
place_one minecraft:quartz 108
empty_result=$(ctl screen | jq -r '[.slots[]? | select(.slot == 200) | .item] | first // "empty"')
echo "the module recipe with a stick instead of iron: '$empty_result'"
[ "$empty_result" = "empty" ] || fail "a stick crafted $empty_result in the iron's cell"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null
# The modular_storage recipe needs `rftoolsbase:machine_frame`, a mod that is
# not in this build: the same pattern with a stone block there crafts nothing.
ctl command "clear @a" >/dev/null
for item in minecraft:chest minecraft:stone minecraft:quartz minecraft:redstone; do
    ctl give "$item" 8 >/dev/null
done
ctl wait 8
open_table "$TX" "$TY" "$TZ"
# "rCr" / "qFq" / "rqr": redstone in 100/102/106/108, chest 101, quartz 103/105,
# stone where the machine frame belongs (104).
place_one minecraft:redstone 100
place_one minecraft:chest 101
place_one minecraft:redstone 102
place_one minecraft:quartz 103
place_one minecraft:stone 104
place_one minecraft:quartz 105
place_one minecraft:redstone 106
place_one minecraft:redstone 108
no_frame=$(ctl screen | jq -r '[.slots[]? | select(.slot == 200) | .item] | first // "empty"')
echo "the storage recipe with a stone block where rftoolsbase:machine_frame belongs: '$no_frame'"
[ "$no_frame" = "empty" ] || fail "a stone block crafted $no_frame in the machine frame's cell"
ctl close >/dev/null 2>&1 || true
ctl wait 4 >/dev/null

if [ "$SELFTEST" = 1 ]; then
    say "SELFTEST: assert a barrel's 27-slot menu on the storage (must fail)"
    # Back to the storage: the crafting phases left the player at the table.
    ctl tp "$X.5" "$((Y + 1))" "$((Z1 + 2)).5" >/dev/null || true
    ctl wait 8 >/dev/null
    open_menu "$X" "$Y" "$Z1" "$facing"
    slots=$(menu_slots)
    card=$(slot_item "$CARD")
    echo "the storage's menu shows $slots container slots, the card slot '$card'"
    [ "$slots" = 27 ] || fail "the storage's menu shows $slots slots, not 27: the card-slot assertion is alive"
fi

ctl close >/dev/null 2>&1 || true
echo "PASS: the plain click opens the storage's own menu — 100 storage slots and the card slot, whose module sets the capacity and which refuses anything else and any second module, and takes the module back out (the contents it left come back with it) — with the padlock sealing it, the grid refusing stacks with no module and while unlocked (one kind a slot), stores and gives back stacks by click, reads 6400/6400 when full and refuses more (taking it again after a withdrawal), keeps its contents, lock, grid and module across a relaunch, the carried storage-module recipe crafting in a table (and a stick/a block where its keys are crafting nothing), and a barrel's chest menu and unlocked slot do not behave this way"
