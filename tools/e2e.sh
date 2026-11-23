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
#  12. the modular storage still answers its readout (`0/0 empty no module`);
#  13. **the inject tick (slice 6)**: 64 stone in the scanner's in-slot is pushed
#      into the empty chest two blocks away on the first tick — the chest's
#      `Items[0]` reads `minecraft:stone` x64 and the buffer drops by exactly one
#      `rfPerInsert` (5,000 -> 4,980); with the buffer at 0 the next stack stays in
#      the slot and the chest keeps its 64, and recharging proves the stall was the
#      power (chest 128); the auto slot (2) routes too; `ctl container`'s items are
#      routed; a stack no inventory accepts is still charged one attempt (20 FE
#      pays one, 19 pays none) and stays in the slot; the walk passes a full barrel
#      for the chest behind it; the out slot is never written.
#  14. **the request path (slice 7)**: with a chest two blocks away holding 128
#      stone, radius 2 and the buffer at 5,000, `/rftoolsstorage request <pos>
#      minecraft:stone 32` pulls 32 stone into the player's inventory (the
#      reference's straight-to-inventory default), leaves the chest at 96 and the
#      buffer at 4,990 (an explicit count costs a tenth of `rfPerRequest`); a
#      request for `minecraft:dirt` (nothing in the box holds it) refuses with
#      `nothing to pull` and charges nothing; at buffer 9 the same request refuses
#      with the price line (`costs 10`); the same request through `ctl command`
#      also goes straight to the player (the reference's default), leaving the out
#      slot empty; and a counterfactual seeds the out slot with another kind, so
#      the same request is refused and the chest is untouched.
#  15. **the give path (slice 8)**: with the chest reseeded with 64 stone, the out
#      slot still holding the request counterfactual's dirt, radius 2 and the
#      buffer at 5,000, `/rftoolsstorage give <pos> minecraft:stone` hands the
#      **whole stack** straight to the player (64), empties the chest and charges
#      the **full** `rfPerRequest` (5,000 -> 4,900) without touching the out slot;
#      `give … minecraft:stone single` hands **one** item (chest 3 -> 2, buffer
#      4,900 -> 4,800); a give for a kind nothing holds refuses with `nothing to
#      give` and charges nothing; and at buffer 99 the same give refuses with the
#      price line (`costs 100`) and leaves the chest alone.
#  16. **the insert path (slice 9)**: with an empty chest two blocks away, radius
#      2, the buffer at 5,000 and 64 stone in the player's inventory,
#      `/rftoolsstorage insert <pos> minecraft:stone` pushes the stack into the
#      chest's `Items[0]` (id and count 64), empties the player's stack and charges
#      exactly one `rfPerInsert` (5,000 -> 4,980); the explicit count form inserts
#      10 of 64 (chest 74, player 54, one more `rfPerInsert`); a barrel with room
#      for only 32 takes 32 of the 64 and charges **nothing** (the remainder stays
#      with the player) — the reference charges only when the stack emptied; a full
#      barrel as the only inventory moves nothing, leaves the player's 64 stone and
#      the buffer at 5,000; a box with nothing routable answers `there are no
#      routable inventories` and charges nothing; and a chest addressed as a
#      scanner is refused.
#  17. **the scan's filter (slice 10)**: the GUI list's search box, as `scan`'s
#      optional term (`getMatcher`). At radius 3, with a chest two blocks away
#      holding 64 stone and one three away holding 32 dirt and the buffer at
#      5,000, `scan <pos> @minecraft` lists both kinds (4,900), `scan <pos> dirt`
#      only the dirt (4,800), `scan <pos> stone` only the stone (4,700) and
#      `scan <pos> @nomatch` answers `empty` with the price still paid (4,600);
#      the unfiltered `scan <pos>` lists both again (4,500) — the differential.
#  18. **the sort choice (slice 11)**: the GUI list's sort, as a saved mode. At
#      radius 3 with a chest two away holding 32 stone and one three away holding
#      64 dirt, `sort <pos> Amount-` + `scan` answers `minecraft:dirt 64,
#      minecraft:stone 32` and saves `AMOUNT_DESCENDING`; `Amount+` answers the
#      stone first; `Name` answers the dirt first (the differential); a third
#      chest's `rftoolsstorage:storage_module0` sorts after both `minecraft:`
#      kinds under `Mod`; and the saved `Amount-` order is read back after a
#      relaunch (the search phase's rig restored first).
#  19. **the routable star list (slice 12)**: the GUI's per-inventory star, as a
#      saved set. A fresh scanner's set is empty (`StorageScannerTileEntity`'s
#      `routable = new HashSet<>()`), so a tick with 64 stone in the in-slot and
#      two empty chests two blocks away (A at S+2 on x, B at S+2 on z) routes
#      nothing and still charges its 20 FE; starring B and un-starring A sends the
#      next stack to B while A stays empty; `scan` still lists A's 32 dirt and B's
#      64 stone (the star filters routing, not the list); re-starring A (and
#      un-starring B) makes A the recipient (A 96); and the saved `routable` list
#      keeps A across a relaunch, after which A takes another stack (160).
#  20. **the export choice (slice 13)**: the GUI's "Export to current container",
#      as a saved choice plus the list's selected inventory. At radius 2, with two
#      starred empty chests two blocks away (A at S+2 on x, B at S+2 on z, so B is
#      first in the x-major walk), `export <pos> current` + `select <pos> B` sends
#      the in-slot's 64 stone to **B only** (A 0, one rfPerInsert); `select <pos> A`
#      sends the next stack to A while B keeps its 64; the choice and the selection
#      are saved with the block (`exportc` 1b, `selected` = A) and survive a
#      relaunch; a **selected full barrel** refuses without falling through to the
#      walk (the in-slot keeps its 64, A and B unchanged); and `export <pos> first`
#      restores the starred walk, which routes the same stack to B (128).
#  21. **the row contents view (slice 14)**: the list's per-row contents
#      (`getInventoryForBlock`), as `/rftoolsstorage contents <pos> <target|all>`.
#      At radius 2, with A (S+2 on x, 64 stone) and B (S+2 on z, 128 stone) both
#      starred and the buffer at **0**, `contents <pos> B` answers
#      `minecraft:stone 128` and `contents <pos> A` answers `64` — a row listing is
#      not a request, so the buffer stays 0; un-starring B leaves the all row with
#      A's 64 while the single row B still lists its 128 (the star filters the
#      "All routable" row, not a row); with nothing starred the all row is `empty`;
#      a cell outside the box, the scanner's own cell and a cell with no inventory
#      are all refused with `is not an inventory in the scan box`, still free.
#
#  22. **the crafting grid (slice 15)**: the jar's ghost pattern (`CraftingGridInventory`:
#      nine memory cells and a derived output) and its four craft buttons
#      (`StorageCraftingTools.craftItems`), carried as `/rftoolsstorage grid` and
#      `/rftoolsstorage craft`. A fresh block's pattern is empty and a craft refuses;
#      a cobblestone cell matches no recipe (and takes nothing) while one plank is the
#      shapeless oak button and two planks in the stick shape are `minecraft:stick x4`
#      — the game's own table, not a hardcoded recipe — and the block's saved `grid`
#      key holds the cells; `craft 1` makes 4 sticks out of the storage's planks for
#      **one** set and takes exactly one plank a cell (8 -> 6), `craft 4` is one set
#      (4 -> 2) and `craft 8` two (2 -> 0, 16 sticks); an empty storage answers
#      `missing cell 1: 16 …, cell 4: 16 …` and takes nothing, and `craft stack` takes
#      16 planks a cell for 64 sticks; the **player's inventory is the first source**
#      (upstream's `InvWrapper` order): 2 planks of the player's go before the
#      storage's 8, which stays at 8; an **unlocked** storage hands nothing out
#      (`GlobalStorageItemWrapper.extractItem`), so the same craft answers `missing 1`
#      a cell and the storage keeps its planks, and a cell write is refused with
#      `lock the storage to change the pattern` (the jar's ghost cells); the pattern
#      is the block's saved `grid` key, so it survives a relaunch and crafts again;
#      `grid <pos> clear` empties it; a barrel answers `not a modular storage`.
#
# The newly-ported phase's inverted run is **CRAFT_SELFTEST=1**: it asserts the craft
# did NOT make the sticks (the run MUST end `FAIL:` + `EXIT:1`).
#
# SELFTEST=1 keeps the scenario and asserts that the tick did NOT push the stone
# into the chest, so the run MUST fail with a `FAIL:` verdict (the assertion the
# slice rests on is proved able to fail). REQUEST_SELFTEST=1 does the same for the
# request path (the chest must NOT have dropped to 96), GIVE_SELFTEST=1 for the
# give path (the give must NOT have handed the stone over), INSERT_SELFTEST=1
# for the insert path (the chest must NOT have received the 64 stone),
# SCAN_SELFTEST=1 for the scan filter (the `dirt` query must NOT list the stone)
# SORT_SELFTEST=1 for the sort (the `Amount-` reply must NOT start with stone),
# ROUTABLE_SELFTEST=1 for the star list (the un-starred chest must NOT have
# received the stack), EXPORT_SELFTEST=1 for the export choice (the selected
# chest must NOT have received the stack), and CONTENTS_SELFTEST=1 for the row
# view (B's row must NOT have listed 128 stone).
# COUNTERFACTUAL=1 puts a vanilla barrel where the scanner would be and proves the
# same commands answer `not a storage scanner` on it. CONTROL_GAME runs the same
# rig on the Nightly without the package and fails there (`EXIT:1`): the block and
# its item do not exist.
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
REQUEST_SELFTEST=${REQUEST_SELFTEST:-0}
GIVE_SELFTEST=${GIVE_SELFTEST:-0}
INSERT_SELFTEST=${INSERT_SELFTEST:-0}
SCAN_SELFTEST=${SCAN_SELFTEST:-0}
SORT_SELFTEST=${SORT_SELFTEST:-0}
ROUTABLE_SELFTEST=${ROUTABLE_SELFTEST:-0}
EXPORT_SELFTEST=${EXPORT_SELFTEST:-0}
CONTENTS_SELFTEST=${CONTENTS_SELFTEST:-0}
CRAFT_SELFTEST=${CRAFT_SELFTEST:-0}

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
# The scan with the GUI search box's filter (`getMatcher`): `scan <pos> <query>`.
scan_query() { # x y z query
    local line
    line=$(reply scan "$1" "$2" "$3" "$4")
    [ -n "$line" ] || fail "the scan at $1,$2,$3 for '$4' answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
radius_cmd() { # x y z n
    local line
    line=$(reply radius "$1" "$2" "$3" "$4")
    [ -n "$line" ] || fail "the radius command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
sort_cmd() { # x y z mode
    local line
    line=$(reply sort "$1" "$2" "$3" "$4")
    [ -n "$line" ] || fail "the sort command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
routable_cmd() { # x y z target-x target-y target-z on|off
    local line
    line=$(reply routable "$1" "$2" "$3" "$4" "$5" "$6" "$7")
    [ -n "$line" ] || fail "the routable command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
export_cmd() { # x y z current|first
    local line
    line=$(reply export "$1" "$2" "$3" "$4")
    [ -n "$line" ] || fail "the export command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
select_cmd() { # x y z target-x target-y target-z
    local line
    line=$(reply select "$1" "$2" "$3" "$4" "$5" "$6")
    [ -n "$line" ] || fail "the select command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
contents_cmd() { # x y z target-x target-y target-z
    local line
    line=$(reply contents "$1" "$2" "$3" "$4" "$5" "$6")
    [ -n "$line" ] || fail "the contents command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
contents_all_cmd() { # x y z
    local line
    line=$(reply contents "$1" "$2" "$3" all)
    [ -n "$line" ] || fail "the contents-all command at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
request_cmd() { # x y z item [count]
    local line
    if [ $# -ge 5 ]; then
        line=$(reply request "$1" "$2" "$3" "$4" "$5")
    else
        line=$(reply request "$1" "$2" "$3" "$4")
    fi
    [ -n "$line" ] || fail "the request at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
give_cmd() { # x y z item [single]
    local line
    if [ $# -ge 5 ]; then
        line=$(reply give "$1" "$2" "$3" "$4" "$5")
    else
        line=$(reply give "$1" "$2" "$3" "$4")
    fi
    [ -n "$line" ] || fail "the give at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
insert_cmd() { # x y z item [count]
    local line
    if [ $# -ge 5 ]; then
        line=$(reply insert "$1" "$2" "$3" "$4" "$5")
    else
        line=$(reply insert "$1" "$2" "$3" "$4")
    fi
    [ -n "$line" ] || fail "the insert at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
energy_of() { data_block "$1" "$2" "$3" energy; }
grid_cmd() { # x y z — the pattern readback
    local line
    line=$(reply grid "$1" "$2" "$3")
    [ -n "$line" ] || fail "the grid readback at $1,$2,$3 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
grid_set() { # x y z cell item
    local line
    line=$(reply grid "$1" "$2" "$3" "$4" "$5")
    [ -n "$line" ] || fail "grid $4 $5 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
grid_clear() { # x y z
    local line
    line=$(reply grid "$1" "$2" "$3" clear)
    [ -n "$line" ] || fail "grid clear answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
craft_cmd() { # x y z 1|4|8|stack
    local line
    line=$(reply craft "$1" "$2" "$3" "$4")
    [ -n "$line" ] || fail "craft $4 answered nothing: $(ctl messages 6 | jq -c '.chat')"
    echo "$line"
}
# A storage seeded for the crafting-grid legs: `count` oak planks in its first
# storage slot and a tier-1 module in the card slot (100), locked (so it hands
# items out, as upstream's wrapper does only while locked).
seed_crafting_storage() { # x y z planks [locked]
    local items
    items="{Slot: 0b, id: \"minecraft:oak_planks\", count: $4}"
    if [ "${5:-1}" = 1 ]; then
        items="$items, {Slot: 100b, id: \"rftoolsstorage:storage_module0\", count: 1}"
    fi
    ctl command "data merge block $1 $2 $3 {Items: [$items], locked: ${5:-1}b}" >/dev/null
    ctl wait 4 >/dev/null
}
# The planks in the crafting rig's storage slot 0. `Items` is a *list*, so
# `Items[0]` stops being slot 0 the moment a slot empties (the module's entry moves
# to the front): read the entry that names `Slot: 0b`.
planks_at() { # x y z
    local line n
    line=$(data_reply "$1" "$2" "$3" Items)
    n=$(printf '%s' "$line" | sed -n 's/.*{count: \([0-9]*\), Slot: 0b, id: "minecraft:oak_planks"}.*/\1/p')
    echo "${n:-0}"
}

# --- the inject tick (slice 6) --------------------------------------------
# A /data get whose path may not exist: `/data get block … Items[0].count` on an
# empty list answers `Found no elements matching Items`, not a value. The newest
# chat line is the reply (`ctl messages` lists it first).
data_reply() { # x y z path
    ctl command "data get block $1 $2 $3 $4" >/dev/null
    ctl wait 6 >/dev/null
    ctl messages 6 | jq -r '.chat[0] // "NONE"'
}
# The value of a /data get path, or the empty string when the path does not exist.
data_value() { # x y z path
    local line
    line=$(data_reply "$@")
    case "$line" in
        *"has the following block data: "*)
            line=${line##*has the following block data: }
            echo "${line//\"/}"
            ;;
        *) echo "" ;;
    esac
}
slot_count() { data_value "$1" "$2" "$3" "Items[$4].count"; }
slot_id() { data_value "$1" "$2" "$3" "Items[$4].id"; }
# The sum of a container's first thirteen slots (`Items[0..12].count`): enough for
# every stack this scenario routes, and 13 reads instead of 27.
inv_total() { # x y z
    local x=$1 y=$2 z=$3 sum=0 i v
    for i in $(seq 0 12); do
        v=$(slot_count "$x" "$y" "$z" "$i")
        [ -z "$v" ] && continue
        sum=$((sum + v))
    done
    echo "$sum"
}
# A full 27-slot container's item list body (one entry a slot, 64 each).
full_slots() { # item
    local i body=""
    for i in $(seq 0 26); do
        body="${body}${body:+, }{Slot: ${i}b, id: \"$1\", count: 64}"
    done
    echo "$body"
}
# A barrel with 26 slots full of `item` and the 27th holding `last` of it: room for
# exactly `64 - last` more, which makes a partial insert (and its free price).
nearly_full_slots() { # item last
    local i body=""
    for i in $(seq 0 25); do
        body="${body}${body:+, }{Slot: ${i}b, id: \"$1\", count: 64}"
    done
    body="${body}, {Slot: 26b, id: \"$1\", count: $2}"
    echo "$body"
}
# `/data merge block … {Items: […]}` replaces the container's item list: every slot
# not named is emptied, which is also how a phase starts from a clean slot.
seed_slots() { # x y z "<items list body>"
    ctl command "data merge block $1 $2 $3 {Items: [$4]}" >/dev/null
    ctl wait 2 >/dev/null
}

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

say "radius 3 brings the chest three away too: both kinds, one entry each"
line=$(radius_cmd "$SX" "$SY" "$SZ" 3)
echo "radius: $line"
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan: $line; buffer now $spent"
echo "$line" | grep -q '^'"$SCANNER"': minecraft:dirt 32, minecraft:stone 64 (' || fail "the radius-3 scan answered: $line"
[ "$spent" = "$((5000 - 3 * PRICE))" ] || fail "three scans left $spent FE, expected $((5000 - 3 * PRICE))"

say "THE SCAN FILTER: the GUI list's search box — every word is '@modid' (a namespace prefix) or a case-insensitive name contains"
# The box is the radius-3 one: a chest two blocks away with 64 stone, one three
# away with 32 dirt (the brief's rig). Each query is paid like the unfiltered scan
# (upstream's search narrows the list a request is made from, and this readout is
# that list — slice 5's deviation 1), so the buffer walks 5,000 -> 4,900 -> 4,800
# -> 4,700 -> 4,600 and the differential scan leaves it at 4,500.
set_energy "$SX" "$SY" "$SZ" 5000
line=$(scan_query "$SX" "$SY" "$SZ" "@minecraft")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan @minecraft: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 32, minecraft:stone 64 (4900/$MAX_FE FE)$" \
    || fail "the '@minecraft' scan answered: $line"
[ "$spent" = "$((5000 - PRICE))" ] || fail "the '@minecraft' scan left $spent FE, expected $((5000 - PRICE))"

line=$(scan_query "$SX" "$SY" "$SZ" dirt)
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan dirt: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 32 (4800/$MAX_FE FE)$" \
    || fail "the 'dirt' scan answered: $line"
[ "$spent" = "$((5000 - 2 * PRICE))" ] || fail "the 'dirt' scan left $spent FE, expected $((5000 - 2 * PRICE))"

if [ "$SCAN_SELFTEST" = 1 ]; then
    say "SCAN_SELFTEST: assert the 'dirt' query DID list the stone (must fail)"
    case "$line" in
        *minecraft:stone*)
            fail "the 'dirt' query listed the stone too ($line), so the filter is broken and this inverted assertion wrongly held" ;;
        *)
            fail "the 'dirt' query listed only minecraft:dirt 32 and not the stone 64 two blocks away ($line), so this inverted assertion fails as it must" ;;
    esac
fi

line=$(scan_query "$SX" "$SY" "$SZ" stone)
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan stone: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:stone 64 (4700/$MAX_FE FE)$" \
    || fail "the 'stone' scan answered: $line"
[ "$spent" = "$((5000 - 3 * PRICE))" ] || fail "the 'stone' scan left $spent FE, expected $((5000 - 3 * PRICE))"
case "$line" in
    *minecraft:dirt*) fail "the 'stone' query listed the dirt: $line" ;;
    *) ;;
esac

line=$(scan_query "$SX" "$SY" "$SZ" "@nomatch")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan @nomatch: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: empty (4600/$MAX_FE FE)$" \
    || fail "the '@nomatch' scan answered: $line"
[ "$spent" = "$((5000 - 4 * PRICE))" ] || fail "the '@nomatch' scan left $spent FE, expected $((5000 - 4 * PRICE)) (the price is paid even when nothing matches)"
case "$line" in
    *minecraft:*) fail "the '@nomatch' query listed an item: $line" ;;
    *) ;;
esac

say "the unfiltered scan still lists both — the differential for the filter"
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan (no query): $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 32, minecraft:stone 64 (4500/$MAX_FE FE)$" \
    || fail "the unfiltered scan after the queries answered: $line"
[ "$spent" = "$((5000 - 5 * PRICE))" ] || fail "the differential scan left $spent FE, expected $((5000 - 5 * PRICE))"

say "THE SORT CHOICE: the GUI list's sort — Amount+, Amount-, Mod, Name (Tag is blocked at this pin)"
# The brief's rig: a chest two away with 32 stone, one three away with 64 dirt,
# the buffer back at 5,000 and radius 3. The counts are the opposite of the
# search phase's, so Amount- (dirt first) and Name (dirt first) agree while
# Amount+ (stone first) differs — the differential that proves the order is the
# mode, not the rig. Every scan still costs one rfPerRequest.
fill_chest "$CHEST_X" "$SY" "$SZ" minecraft:stone 32
fill_chest "$FAR_X" "$SY" "$SZ" minecraft:dirt 64
set_energy "$SX" "$SY" "$SZ" 5000
line=$(sort_cmd "$SX" "$SY" "$SZ" 'Amount-')
echo "sort Amount-: $line"
echo "$line" | grep -q "^$SCANNER: sort Amount-$" || fail "the sort command answered: $line"
saved_sort=$(data_block "$SX" "$SY" "$SZ" sortMode)
echo "saved sortMode: $saved_sort"
[ "$saved_sort" = AMOUNT_DESCENDING ] || fail "the saved sortMode is '$saved_sort', expected AMOUNT_DESCENDING"
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan Amount-: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 64, minecraft:stone 32 (4900/$MAX_FE FE)$" \
    || fail "the Amount- scan answered: $line"
[ "$spent" = "$((5000 - PRICE))" ] || fail "the Amount- scan left $spent FE, expected $((5000 - PRICE))"

if [ "$SORT_SELFTEST" = 1 ]; then
    say "SORT_SELFTEST: assert the Amount- reply DID start with stone (must fail)"
    case "$line" in
        *"$SCANNER: minecraft:stone "*)
            fail "the Amount- scan started with stone ($line), so the order is broken and this inverted assertion wrongly held" ;;
        *)
            fail "the Amount- scan started with minecraft:dirt 64 and not the stone 32 ($line), so this inverted assertion fails as it must" ;;
    esac
fi

line=$(sort_cmd "$SX" "$SY" "$SZ" 'Amount+')
echo "sort Amount+: $line"
echo "$line" | grep -q "^$SCANNER: sort Amount+$" || fail "the sort command answered: $line"
line=$(scan "$SX" "$SY" "$SZ")
echo "scan Amount+: $line"
echo "$line" | grep -q "^$SCANNER: minecraft:stone 32, minecraft:dirt 64 (4800/$MAX_FE FE)$" \
    || fail "the Amount+ scan answered: $line"

line=$(sort_cmd "$SX" "$SY" "$SZ" Name)
echo "sort Name: $line"
echo "$line" | grep -q "^$SCANNER: sort Name$" || fail "the sort command answered: $line"
line=$(scan "$SX" "$SY" "$SZ")
echo "scan Name: $line"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 64, minecraft:stone 32 (4700/$MAX_FE FE)$" \
    || fail "the Name scan answered: $line"

say "the Mod order puts a rftoolsstorage stack after both minecraft kinds"
# A third chest (not adjacent to the row, so it cannot merge) holds the port's
# own module item: minecraft sorts before rftoolsstorage.
MODX=$((SX - 2))
MODZ=$SZ
fill_chest "$MODX" "$SY" "$MODZ" rftoolsstorage:storage_module0 1
line=$(sort_cmd "$SX" "$SY" "$SZ" Mod)
echo "sort Mod: $line"
echo "$line" | grep -q "^$SCANNER: sort Mod$" || fail "the sort command answered: $line"
line=$(scan "$SX" "$SY" "$SZ")
echo "scan Mod: $line"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 64, minecraft:stone 32, rftoolsstorage:storage_module0 1 (4600/$MAX_FE FE)$" \
    || fail "the Mod scan answered: $line"
ctl setblock "$MODX" "$SY" "$MODZ" minecraft:air >/dev/null
ctl wait 4 >/dev/null

# Restore the search phase's rig and the default order for the rest of the run.
fill_chest "$CHEST_X" "$SY" "$SZ" minecraft:stone 64
fill_chest "$FAR_X" "$SY" "$SZ" minecraft:dirt 32
line=$(sort_cmd "$SX" "$SY" "$SZ" Name)
echo "sort back to Name: $line"
line=$(scan "$SX" "$SY" "$SZ")
echo "scan after the sort phase: $line"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 32, minecraft:stone 64 (" \
    || fail "the restored NAME scan answered: $line"

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

say "the radius, the buffer and the saved sort choice survive a relaunch, and the scan still answers"
line=$(sort_cmd "$SX" "$SY" "$SZ" 'Amount-')
echo "sort before the relaunch: $line"
line=$(scan "$SX" "$SY" "$SZ")
echo "scan before the relaunch: $line"
echo "$line" | grep -q "^$SCANNER: minecraft:stone 64, minecraft:dirt 32 (" \
    || fail "the Amount- scan before the relaunch answered: $line"
before=$(energy_of "$SX" "$SY" "$SZ")
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch "$GAME"
ctl wait 20 >/dev/null
after=$(energy_of "$SX" "$SY" "$SZ")
saved_radius=$(data_block "$SX" "$SY" "$SZ" radius)
saved_sort=$(data_block "$SX" "$SY" "$SZ" sortMode)
line=$(scan "$SX" "$SY" "$SZ")
echo "relaunch: buffer $before -> $after, radius $saved_radius, sort $saved_sort; scan: $line"
[ "$after" = "$before" ] || fail "the relaunch changed the buffer $before -> $after"
[ "$saved_radius" = 3 ] || fail "the relaunch changed the radius to $saved_radius"
[ "$saved_sort" = AMOUNT_DESCENDING ] || fail "the relaunch lost the sort choice (saved '$saved_sort')"
echo "$line" | grep -q "^$SCANNER: minecraft:stone 64, minecraft:dirt 32 (" \
    || fail "the scan after the relaunch answered: $line"

say "THE ROUTABLE STAR LIST: a fresh scanner routes nothing, the star filters routing but not the list"
# Two chests two blocks away on different axes — A at S+2 on x, B at S+2 on z —
# so neither joins the other (two adjacent chests are one double chest). The walk
# is x, then y, then z, so B comes before A; a phase with only one of them starred
# cannot be explained by the order.
ROUTABLE_A=$((SX + 2))
ROUTABLE_B=$((SZ + 2))
line=$(sort_cmd "$SX" "$SY" "$SZ" Name)
echo "sort Name: $line"
ctl setblock "$FAR_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$ROUTABLE_A" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$ROUTABLE_A" "$SY" "$SZ" minecraft:chest >/dev/null
ctl setblock "$SX" "$SY" "$ROUTABLE_B" minecraft:air >/dev/null
ctl setblock "$SX" "$SY" "$ROUTABLE_B" minecraft:chest >/dev/null
ctl wait 4
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
seed_slots "$SX" "$SY" "$SZ" ''
saved=$(data_reply "$SX" "$SY" "$SZ" routable)
echo "saved routable (default): $saved"
case "$saved" in
    *"Found no elements"*) ;;
    *) fail "a fresh scanner has a routable list: $saved" ;;
esac
# 20 FE pays exactly one attempt (the tick charges even when nothing accepts the
# stack — slice 6), so the assertion does not depend on how many ticks the reads
# cost: the stack must still be in the slot and both chests empty.
ctl command "data merge block $SX $SY $SZ {energy: 20}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$ROUTABLE_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$ROUTABLE_B")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "no star: in-slot ${in_slot}, A ${a_total}, B ${b_total}, energy ${energy}"
[ "$in_slot" = 64 ] || fail "with no star the in-slot holds '$in_slot', expected its 64 stone"
[ "$a_total" = 0 ] || fail "with no star chest A took $a_total"
[ "$b_total" = 0 ] || fail "with no star chest B took $b_total"
[ "$energy" = 0 ] || fail "the unstarred attempt left $energy, expected 20 - 20 (one paid attempt)"

line=$(routable_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$ROUTABLE_B" on)
echo "star B: $line"
echo "$line" | grep -q "^$SCANNER: routable $SX $SY $ROUTABLE_B on$" || fail "the star command answered: $line"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$ROUTABLE_A" "$SY" "$SZ" on)
echo "star A: $line"
echo "$line" | grep -q "^$SCANNER: routable $ROUTABLE_A $SY $SZ on$" || fail "the star command answered: $line"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$ROUTABLE_A" "$SY" "$SZ" off)
echo "un-star A: $line"
echo "$line" | grep -q "^$SCANNER: routable $ROUTABLE_A $SY $SZ off$" || fail "the un-star command answered: $line"
saved=$(data_reply "$SX" "$SY" "$SZ" routable)
echo "saved routable (B only): $saved"
case "$saved" in
    *"[I; $SX, $SY, $ROUTABLE_B]"*) ;;
    *) fail "the saved routable list does not name B: $saved" ;;
esac
case "$saved" in
    *"[I; $ROUTABLE_A, $SY, $SZ]"*) fail "the un-starred A is still in the saved list: $saved" ;;
    *) ;;
esac

# The 64 stone the unstarred tick could not move is still in the in-slot: the
# star is what lets the very same stack go, for one rfPerInsert.
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$ROUTABLE_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$ROUTABLE_B")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "B starred, A unstarred: in-slot '${in_slot}', A ${a_total}, B ${b_total}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the starred walk left $in_slot stone in the in-slot"
[ "$a_total" = 0 ] || fail "the un-starred chest A took $a_total stone"
[ "$b_total" = 64 ] || fail "the starred chest B holds $b_total, expected 64"
[ "$energy" = "$((5000 - 20))" ] || fail "the starred walk left $energy, expected one rfPerInsert off 5000"

if [ "$ROUTABLE_SELFTEST" = 1 ]; then
    say "ROUTABLE_SELFTEST: assert the un-starred chest DID take the stack (must fail)"
    fail "the un-starred chest A stayed empty (${a_total}) and the starred B took the 64 stone (energy ${energy}), so the star filtered routing; this inverted assertion fails as it must"
fi

say "the scan still lists the un-starred inventory: the star filters routing, not the list"
fill_chest "$ROUTABLE_A" "$SY" "$SZ" minecraft:dirt 32
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
line=$(scan "$SX" "$SY" "$SZ")
spent=$(energy_of "$SX" "$SY" "$SZ")
echo "scan with A unstarred: $line; buffer now $spent"
echo "$line" | grep -q "^$SCANNER: minecraft:dirt 32, minecraft:stone 64 (4900/$MAX_FE FE)$" \
    || fail "the scan of the un-starred chest answered: $line"
[ "$spent" = "$((5000 - PRICE))" ] || fail "the scan left $spent FE, expected its $PRICE"

say "re-starring A makes it the walk's recipient again"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$ROUTABLE_A" "$SY" "$SZ" on)
echo "star A again: $line"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$ROUTABLE_B" off)
echo "un-star B: $line"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$ROUTABLE_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$ROUTABLE_B")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "A re-starred, B unstarred: in-slot '${in_slot}', A ${a_total}, B ${b_total}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the re-starred walk left $in_slot stone in the in-slot"
[ "$a_total" = "$((32 + 64))" ] || fail "the re-starred chest A holds $a_total, expected its 32 dirt and 64 stone"
[ "$b_total" = 64 ] || fail "the un-starred chest B changed to $b_total"

say "the saved star list survives a relaunch, and the starred inventory still receives"
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch "$GAME"
ctl wait 20 >/dev/null
saved=$(data_reply "$SX" "$SY" "$SZ" routable)
echo "saved routable after the relaunch: $saved"
case "$saved" in
    *"[I; $ROUTABLE_A, $SY, $SZ]"*) ;;
    *) fail "the relaunch lost the star on A: $saved" ;;
esac
case "$saved" in
    *"[I; $SX, $SY, $ROUTABLE_B]"*) fail "the relaunch kept B's un-star: $saved" ;;
    *) ;;
esac
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$ROUTABLE_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$ROUTABLE_B")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "after the relaunch: in-slot '${in_slot}', A ${a_total}, B ${b_total}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the relaunched walk left $in_slot stone in the in-slot"
[ "$a_total" = "$((32 + 128))" ] || fail "the relaunched chest A holds $a_total, expected 160"
[ "$b_total" = 64 ] || fail "the relaunched un-starred chest B changed to $b_total"
# B leaves the box for the rest of the run; A (the chest two away on x) stays
# starred, so the inject/request/give/insert phases route to it as before.
ctl setblock "$SX" "$SY" "$ROUTABLE_B" minecraft:air >/dev/null
ctl wait 4

say "THE INJECT TICK: 64 stone in the in-slot goes to the chest two blocks away, for one rfPerInsert"
# The chest must be empty (and alone: two adjacent chests are one double chest), so
# its Items[0] can only be the stack the tick pushed there. The star list filters
# every routing walk (slice 12), so the chest this phase routes into must be
# starred: A's position already is, and the call is the explicit assertion of it.
CHEST_X=$((SX + 2))
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:chest >/dev/null
ctl wait 4
line=$(routable_cmd "$SX" "$SY" "$SZ" "$CHEST_X" "$SY" "$SZ" on)
echo "star the inject chest: $line"
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl wait 4
echo "before the seed: in-slot '$(slot_count "$SX" "$SY" "$SZ" 0)' chest total $(inv_total "$CHEST_X" "$SY" "$SZ") energy $(energy_of "$SX" "$SY" "$SZ")"
case "$(data_reply "$SX" "$SY" "$SZ" 'Items[0].count')" in
    *"Found no elements"*) ;;
    *) fail "the scanner's in-slot is not empty before the seed" ;;
esac
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
echo "after the seed: in-slot $(slot_count "$SX" "$SY" "$SZ" 0) energy $(energy_of "$SX" "$SY" "$SZ")"
# The merge, the read above and the tick: ten ticks are plenty for one push.
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
chest_id=$(slot_id "$CHEST_X" "$SY" "$SZ" 0)
chest_count=$(slot_count "$CHEST_X" "$SY" "$SZ" 0)
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "after 10 ticks: in-slot '${in_slot}', chest Items[0] = ${chest_count} x ${chest_id}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the in-slot still holds $in_slot stone after the tick"
[ "$chest_id" = "minecraft:stone" ] || fail "the chest's Items[0] is '$chest_id', expected minecraft:stone"
[ "$chest_count" = 64 ] || fail "the chest's Items[0] holds $chest_count, expected 64"
[ "$energy" = "$((5000 - 20))" ] || fail "the buffer is $energy, expected one rfPerInsert off 5000 (4980)"

if [ "$SELFTEST" = 1 ]; then
    say "SELFTEST: assert the tick did NOT push the stone into the chest (must fail)"
    fail "the tick moved 64 stone from the in-slot into the chest two blocks away (energy ${energy}), so this inverted assertion fails as it must"
fi

say "counterfactual: buffer 0, another 64 stone in the in-slot -> the chest keeps 64 and the slot keeps the stone"
ctl command "data merge block $SX $SY $SZ {energy: 0}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
chest=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "buffer 0: in-slot ${in_slot}, chest total ${chest}, energy ${energy}"
[ "$in_slot" = 64 ] || fail "with no power the in-slot holds '$in_slot', expected its 64 stone"
[ "$chest" = 64 ] || fail "with no power the chest holds $chest, expected its 64"
[ "$energy" = 0 ] || fail "a refused tick changed the buffer to $energy"

say "recharged, the same stack goes: the stall was the power, not the walk"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
chest=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "recharged: in-slot '${in_slot}', chest total ${chest}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the recharged scanner still holds $in_slot stone"
[ "$chest" = 128 ] || fail "the chest holds $chest after the second stack, expected 128"
[ "$energy" = "$((5000 - 20))" ] || fail "the second stack cost more than one rfPerInsert ($energy)"

say "the automation input (slot 2) routes through the same tick"
seed_slots "$SX" "$SY" "$SZ" '{Slot: 2b, id: "minecraft:stone", count: 32}'
ctl wait 10
auto_slot=$(slot_count "$SX" "$SY" "$SZ" 2)
chest=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "auto slot: '${auto_slot}', chest total ${chest}, energy ${energy}"
[ "$auto_slot" = "" ] || fail "the auto slot still holds $auto_slot"
[ "$chest" = 160 ] || fail "the chest holds $chest after the auto slot, expected 160"
[ "$energy" = "$((5000 - 2 * 20))" ] || fail "two inserts left $energy, expected 5000 - 2 x 20"

say "items the game puts in the container are routed too (ctl container -> the tick)"
chest_before=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_before=$(energy_of "$SX" "$SY" "$SZ")
ctl container "$SX" "$SY" "$SZ" minecraft:stone 3 >/dev/null
ctl wait 10
chest=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "container path: chest ${chest_before} -> ${chest}, energy ${energy_before} -> ${energy}"
[ "$chest" = "$((chest_before + 3))" ] || fail "the container's 3 stone did not reach the chest (${chest_before} -> ${chest})"
[ "$energy" = "$((energy_before - 20))" ] || fail "the container's stack cost $((energy_before - energy)), expected one rfPerInsert"

say "a stack nothing accepts still costs its attempt: 20 FE pays one, 19 pays none"
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:barrel >/dev/null
ctl wait 4
seed_slots "$CHEST_X" "$SY" "$SZ" "$(full_slots minecraft:stone)"
ctl wait 4
echo "the full barrel holds $(slot_count "$CHEST_X" "$SY" "$SZ" 0) x 27 slots of stone"
ctl command "data merge block $SX $SY $SZ {energy: 20}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:dirt", count: 64}'
ctl wait 20
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "20 FE: in-slot ${in_slot} dirt, energy ${energy}"
[ "$in_slot" = 64 ] || fail "the refused dirt left the in-slot ($in_slot)"
[ "$energy" = 0 ] || fail "the refused attempt left $energy, expected 20 - 20"
ctl command "data merge block $SX $SY $SZ {energy: 19}" >/dev/null
ctl wait 20
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "19 FE: in-slot ${in_slot} dirt, energy ${energy}"
[ "$energy" = 19 ] || fail "19 FE paid for an attempt ($energy)"
[ "$in_slot" = 64 ] || fail "the refused dirt left the in-slot ($in_slot)"

say "the walk passes a refusing inventory for the one behind it (a full barrel at S+1, a chest at S+2)"
BARREL_X=$((SX + 1))
ctl setblock "$BARREL_X" "$SY" "$SZ" minecraft:barrel >/dev/null
ctl wait 4
line=$(routable_cmd "$SX" "$SY" "$SZ" "$BARREL_X" "$SY" "$SZ" on)
echo "star the refusing barrel: $line"
seed_slots "$BARREL_X" "$SY" "$SZ" "$(full_slots minecraft:stone)"
ctl wait 4
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:chest >/dev/null
ctl wait 4
# The dirt from the 19-FE phase is still in the slot and 19 cannot buy an attempt:
# seed the same stack (so the phase starts from one known stack), then pay.
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:dirt", count: 64}'
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
barrel_first=$(slot_count "$BARREL_X" "$SY" "$SZ" 0)
barrel_last=$(slot_count "$BARREL_X" "$SY" "$SZ" 26)
chest_dirt=$(slot_id "$CHEST_X" "$SY" "$SZ" 0)
chest_count=$(slot_count "$CHEST_X" "$SY" "$SZ" 0)
echo "walk: in-slot '${in_slot}', barrel ${barrel_first}..${barrel_last} stone, chest ${chest_count} x ${chest_dirt}"
[ "$in_slot" = "" ] || fail "the walk left $in_slot dirt in the in-slot"
[ "$barrel_first" = 64 ] && [ "$barrel_last" = 64 ] || fail "the full barrel changed ($barrel_first..$barrel_last)"
[ "$chest_dirt" = "minecraft:dirt" ] || fail "the chest behind the barrel holds '${chest_dirt}'"
[ "$chest_count" = 64 ] || fail "the chest behind the barrel holds $chest_count, expected 64"

say "the out slot (1) is declared and never written by the tick"
out_slot=$(data_reply "$SX" "$SY" "$SZ" 'Items[1].count')
echo "out slot reply: $out_slot"
case "$out_slot" in
    *"Found no elements"*) ;;
    *) fail "the tick wrote the out slot: $out_slot" ;;
esac
echo "scanner container: $(ctl container "$SX" "$SY" "$SZ" | jq -c .)"

say "THE REQUEST PATH: /rftoolsstorage request <pos> minecraft:stone 32 pulls from the chest"
# One chest two blocks away is the only inventory in radius 2: the full barrel at
# S+1 and the dirt chest at S+3 the inject phases left are removed. The chest is
# alone (two adjacent chests would merge into one double chest).
ctl setblock "$((SX + 1))" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$((SX + 3))" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:chest >/dev/null
ctl wait 4
seed_slots "$CHEST_X" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}, {Slot: 1b, id: "minecraft:stone", count: 64}'
# Clear the scanner's own slots (the out slot included) and charge the buffer.
seed_slots "$SX" "$SY" "$SZ" ''
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl command "clear @a" >/dev/null
ctl wait 6
stand_at "$SX" "$SZ"
ctl wait 6
chest_before=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_before=$(energy_of "$SX" "$SY" "$SZ")
echo "before the request: chest ${chest_before} stone, energy ${energy_before}, player stone $(count_item minecraft:stone)"
[ "$chest_before" = 128 ] || fail "the chest holds $chest_before stone before the request, expected 128"
[ "$energy_before" = 5000 ] || fail "the buffer is $energy_before before the request, expected 5000"

line=$(request_cmd "$SX" "$SY" "$SZ" minecraft:stone 32)
echo "request 32 stone: $line"
echo "$line" | grep -q "^$SCANNER: pulled 32 minecraft:stone (4990/$MAX_FE FE)$" || fail "the request answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the request: chest ${chest_after} stone, energy ${energy_after}, player stone ${held}"
[ "$chest_after" = 96 ] || fail "the chest holds $chest_after after pulling 32, expected 96"
[ "$energy_after" = "$((5000 - 10))" ] || fail "the request cost $((5000 - energy_after)) FE, expected 10 (a tenth of rfPerRequest)"
[ "$held" = 32 ] || fail "the player holds $held stone, expected the 32 pulled (requestStraightToInventory's default)"

if [ "$REQUEST_SELFTEST" = 1 ]; then
    say "REQUEST_SELFTEST: assert the request did NOT pull the stone (must fail)"
    fail "the request pulled 32 stone out of the chest (128 -> ${chest_after}) and charged 10 FE (5000 -> ${energy_after}), so this inverted assertion fails as it must"
fi

say "a request for a kind nothing holds refuses and charges nothing"
line=$(request_cmd "$SX" "$SY" "$SZ" minecraft:dirt 32)
echo "request 32 dirt: $line"
echo "$line" | grep -q "^$SCANNER: nothing to pull for minecraft:dirt (4990/$MAX_FE FE)$" || fail "the dirt request answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
echo "after the refused dirt request: chest ${chest_after} stone, energy ${energy_after}, player dirt $(count_item minecraft:dirt)"
[ "$chest_after" = 96 ] || fail "the refused dirt request changed the chest to $chest_after"
[ "$energy_after" = "$((5000 - 10))" ] || fail "the refused dirt request charged ($energy_after)"
[ "$(count_item minecraft:dirt)" = 0 ] || fail "the refused dirt request gave the player dirt"

say "with the buffer below the price the same request is refused with the price line"
ctl command "data merge block $SX $SY $SZ {energy: 9}" >/dev/null
ctl wait 6
line=$(request_cmd "$SX" "$SY" "$SZ" minecraft:stone 32)
echo "request 32 stone at 9 FE: $line"
echo "$line" | grep -q 'costs 10' || fail "the refusal does not print the price: $line"
echo "$line" | grep -q "9/$MAX_FE FE" || fail "the refusal does not print the buffer: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
echo "after the 9-FE refusal: chest ${chest_after} stone, energy ${energy_after}, player stone $(count_item minecraft:stone)"
[ "$chest_after" = 96 ] || fail "a request without the power changed the chest to $chest_after"
[ "$energy_after" = 9 ] || fail "a refused request changed the buffer to $energy_after"

say "the same request through ctl command goes straight to the player too (the reference's default)"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl wait 6
out_reply=$(ctl command "/rftoolsstorage request $SX $SY $SZ minecraft:stone 16" 2>&1 || true)
echo "command request 16 stone: $out_reply"
ctl wait 8
out_count=$(data_reply "$SX" "$SY" "$SZ" 'Items[1].count')
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
echo "after the command request: out slot reply '${out_count}', chest ${chest_after} stone, energy ${energy_after}, player stone $(count_item minecraft:stone)"
case "$out_count" in
    *"Found no elements"*) ;;
    *) fail "the out slot kept the delivered stack: $out_count" ;;
esac
[ "$chest_after" = 80 ] || fail "the chest holds $chest_after after the 16-stone pull, expected 80"
[ "$energy_after" = "$((5000 - 10))" ] || fail "the command request cost $((5000 - energy_after)) FE, expected 10"
[ "$(count_item minecraft:stone)" = 48 ] || fail "the player holds $(count_item minecraft:stone) stone, expected 32 + 16"

say "counterfactual: an out slot holding another kind refuses the request and leaves the chest alone"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 1b, id: "minecraft:dirt", count: 1}'
ctl wait 6
line=$(request_cmd "$SX" "$SY" "$SZ" minecraft:stone 32)
echo "request 32 stone with a dirt out slot: $line"
echo "$line" | grep -q 'out slot holds minecraft:dirt' || fail "the out-slot refusal answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
echo "after the out-slot counterfactual: chest ${chest_after} stone, energy ${energy_after}, player stone $(count_item minecraft:stone)"
[ "$chest_after" = 80 ] || fail "the out-slot counterfactual pulled from the chest ($chest_after)"
[ "$energy_after" = 5000 ] || fail "the out-slot counterfactual charged ($energy_after)"

say "THE GIVE PATH: /rftoolsstorage give <pos> minecraft:stone hands the stack straight to the player"
# The give path is the storage control screen module's click
# (`giveToPlayerFromScreen`): it has **no out slot at all**, so the dirt the request
# counterfactual left in slot 1 is not a guard here — it must still be there after
# the give. One chest two blocks away is the only inventory in radius 2, and the
# scanner's own in/auto slots stay empty (the tick runs during every `ctl wait`).
seed_slots "$CHEST_X" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl command "clear @a" >/dev/null
ctl wait 6
stand_at "$SX" "$SZ"
ctl wait 6
chest_before=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_before=$(energy_of "$SX" "$SY" "$SZ")
# The counterfactual seeded the out slot with `/data merge block … {Items: [{Slot:
# 1b, …}]}`, which writes **one list entry** carrying `Slot: 1b`: the list's first
# entry is the out slot (a merge replaces the whole list, so nothing else is in it).
out_before_slot=$(data_value "$SX" "$SY" "$SZ" 'Items[0].Slot')
out_before=$(slot_id "$SX" "$SY" "$SZ" 0)
echo "before the give: chest ${chest_before} stone, energy ${energy_before}, player stone $(count_item minecraft:stone), out slot '${out_before}' (Slot ${out_before_slot})"
[ "$chest_before" = 64 ] || fail "the chest holds $chest_before stone before the give, expected 64"
[ "$energy_before" = 5000 ] || fail "the buffer is $energy_before before the give, expected 5000"
[ "$out_before_slot" = "1b" ] || fail "the first Items entry carries Slot '$out_before_slot', expected the out slot's 1b"
[ "$out_before" = "minecraft:dirt" ] || fail "the out slot holds '$out_before', expected the request counterfactual's dirt"

line=$(give_cmd "$SX" "$SY" "$SZ" minecraft:stone)
echo "give stone: $line"
echo "$line" | grep -q "^$SCANNER: gave 64 minecraft:stone (4900/$MAX_FE FE)$" || fail "the give answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
out_after_slot=$(data_value "$SX" "$SY" "$SZ" 'Items[0].Slot')
out_after=$(slot_id "$SX" "$SY" "$SZ" 0)
echo "after the give: chest ${chest_after} stone, energy ${energy_after}, player stone ${held}, out slot '${out_after}' (Slot ${out_after_slot})"
[ "$chest_after" = 0 ] || fail "the chest holds $chest_after after giving its 64, expected 0"
[ "$energy_after" = "$((5000 - PRICE))" ] || fail "the give cost $((5000 - energy_after)) FE, expected $PRICE (the whole price, a stack or one item)"
[ "$held" = 64 ] || fail "the player holds $held stone, expected the 64 given"
[ "$out_after_slot" = "1b" ] && [ "$out_after" = "minecraft:dirt" ] \
    || fail "the give touched the out slot (now '${out_after}' at Slot '${out_after_slot}'): the give path has no out slot"

if [ "$GIVE_SELFTEST" = 1 ]; then
    say "GIVE_SELFTEST: assert the give did NOT hand the stone over (must fail)"
    fail "the give handed 64 stone to the player (chest 64 -> ${chest_after}, player holds ${held}) and charged $PRICE FE (5000 -> ${energy_after}), so this inverted assertion fails as it must"
fi

say "a give with single hands one item and drains one"
seed_slots "$CHEST_X" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 3}'
seed_slots "$SX" "$SY" "$SZ" ''
ctl wait 6
line=$(give_cmd "$SX" "$SY" "$SZ" minecraft:stone single)
echo "give single stone: $line"
echo "$line" | grep -q "^$SCANNER: gave 1 minecraft:stone (4800/$MAX_FE FE)$" || fail "the single give answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the single give: chest ${chest_after} stone, energy ${energy_after}, player stone ${held}"
[ "$chest_after" = 2 ] || fail "the chest holds $chest_after after a single give, expected 3 - 1"
[ "$energy_after" = "$((5000 - 2 * PRICE))" ] || fail "the single give cost more than the price ($energy_after)"
[ "$held" = 65 ] || fail "the player holds $held stone after a single give, expected 64 + 1"

say "a give for a kind nothing holds refuses and charges nothing"
line=$(give_cmd "$SX" "$SY" "$SZ" minecraft:dirt)
echo "give dirt: $line"
echo "$line" | grep -q "^$SCANNER: nothing to give for minecraft:dirt (4800/$MAX_FE FE)$" || fail "the dirt give answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
echo "after the refused dirt give: chest ${chest_after} stone, energy ${energy_after}, player dirt $(count_item minecraft:dirt)"
[ "$chest_after" = 2 ] || fail "the refused dirt give changed the chest to $chest_after"
[ "$energy_after" = "$((5000 - 2 * PRICE))" ] || fail "the refused dirt give charged ($energy_after)"
[ "$(count_item minecraft:dirt)" = 0 ] || fail "the refused dirt give handed the player dirt"

say "counterfactual: buffer 99 refuses the give with the price line and leaves the chest alone"
ctl command "data merge block $SX $SY $SZ {energy: 99}" >/dev/null
ctl wait 6
line=$(give_cmd "$SX" "$SY" "$SZ" minecraft:stone)
echo "give stone at 99 FE: $line"
echo "$line" | grep -q 'costs 100' || fail "the give refusal does not print the price: $line"
echo "$line" | grep -q "99/$MAX_FE FE" || fail "the give refusal does not print the buffer: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the 99-FE refusal: chest ${chest_after} stone, energy ${energy_after}, player stone ${held}"
[ "$chest_after" = 2 ] || fail "a give without the power changed the chest to $chest_after"
[ "$energy_after" = 99 ] || fail "a refused give changed the buffer to $energy_after"
[ "$held" = 65 ] || fail "a refused give changed the player's stone to $held"

say "a give on a block that is not a scanner is refused before anything is read"
line=$(give_cmd "$CHEST_X" "$SY" "$SZ" minecraft:stone)
echo "give on the chest: $line"
echo "$line" | grep -q '^minecraft:chest: not a storage scanner$' || fail "the chest took a give: $line"

say "THE INSERT PATH: /rftoolsstorage insert <pos> minecraft:stone pushes the player's stack out, for one rfPerInsert when it emptied"
# The screen's insert button (`injectStackFromScreen`): the player hands a stack
# over, the walk pushes it into the scanned inventories, and the remainder stays in
# the player's slot. The one inventory in radius 2 is the empty chest two blocks
# away (the barrel at S+1 and the dirt chest at S+3 were removed by the request
# phase), so its Items[0] can only be what the insert pushed there.
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:chest >/dev/null
ctl wait 4
seed_slots "$SX" "$SY" "$SZ" ''
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl command "clear @a" >/dev/null
ctl wait 6
stand_at "$SX" "$SZ"
ctl wait 6
ctl give minecraft:stone 64 >/dev/null || fail "cannot give the player 64 stone"
ctl wait 8
held=$(count_item minecraft:stone)
echo "before the insert: chest $(inv_total "$CHEST_X" "$SY" "$SZ") stone, energy $(energy_of "$SX" "$SY" "$SZ"), player stone ${held}"
[ "$held" = 64 ] || fail "the player holds $held stone before the insert, expected 64"

line=$(insert_cmd "$SX" "$SY" "$SZ" minecraft:stone)
echo "insert stone: $line"
echo "$line" | grep -q "^$SCANNER: inserted 64 minecraft:stone (4980/$MAX_FE FE)$" || fail "the insert answered: $line"
chest_id=$(slot_id "$CHEST_X" "$SY" "$SZ" 0)
chest_count=$(slot_count "$CHEST_X" "$SY" "$SZ" 0)
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the insert: chest Items[0] = ${chest_count} x ${chest_id}, energy ${energy_after}, player stone ${held}"
[ "$chest_id" = "minecraft:stone" ] || fail "the chest's Items[0] is '$chest_id', expected minecraft:stone"
[ "$chest_count" = 64 ] || fail "the chest's Items[0] holds $chest_count, expected 64"
[ "$energy_after" = "$((5000 - 20))" ] || fail "the insert cost $((5000 - energy_after)) FE, expected exactly one rfPerInsert (4980)"
[ "$held" = 0 ] || fail "the player holds $held stone after handing 64 over, expected 0"

if [ "$INSERT_SELFTEST" = 1 ]; then
    say "INSERT_SELFTEST: assert the insert did NOT move the stone into the chest (must fail)"
    fail "the insert moved 64 stone from the player into the chest (player 64 -> ${held}, chest 0 -> ${chest_count}) and charged one rfPerInsert (5000 -> ${energy_after}), so this inverted assertion fails as it must"
fi

say "an explicit count inserts that many: 10 of the player's 64, one rfPerInsert when they all went in"
ctl command "clear @a" >/dev/null
ctl wait 4
ctl give minecraft:stone 64 >/dev/null || fail "cannot give the player 64 stone"
ctl wait 8
line=$(insert_cmd "$SX" "$SY" "$SZ" minecraft:stone 10)
echo "insert 10 stone: $line"
echo "$line" | grep -q "^$SCANNER: inserted 10 minecraft:stone (4960/$MAX_FE FE)$" || fail "the count insert answered: $line"
chest_after=$(inv_total "$CHEST_X" "$SY" "$SZ")
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the 10-stone insert: chest ${chest_after} stone, energy ${energy_after}, player stone ${held}"
[ "$chest_after" = 74 ] || fail "the chest holds $chest_after after 64 + 10, expected 74"
[ "$energy_after" = "$((5000 - 2 * 20))" ] || fail "the count insert left $energy_after, expected 5000 - 2 x 20"
[ "$held" = 54 ] || fail "the player holds $held stone after inserting 10 of 64, expected 54"

say "counterfactual: a barrel with room for 32 takes 32 of the 64 and charges nothing (the stack did not empty)"
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:barrel >/dev/null
ctl wait 4
seed_slots "$CHEST_X" "$SY" "$SZ" "$(nearly_full_slots minecraft:stone 32)"
ctl wait 4
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl command "clear @a" >/dev/null
ctl wait 6
ctl give minecraft:stone 64 >/dev/null || fail "cannot give the player 64 stone"
ctl wait 8
line=$(insert_cmd "$SX" "$SY" "$SZ" minecraft:stone)
echo "insert stone into a barrel with room for 32: $line"
echo "$line" | grep -q "^$SCANNER: inserted 32 of 64 minecraft:stone; the rest stays with you, no charge (5000/$MAX_FE FE)$" || fail "the partial insert answered: $line"
barrel_last=$(slot_count "$CHEST_X" "$SY" "$SZ" 26)
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the partial insert: barrel slot 26 = ${barrel_last} stone, energy ${energy_after}, player stone ${held}"
[ "$barrel_last" = 64 ] || fail "the barrel's last slot holds $barrel_last, expected 32 + 32"
[ "$energy_after" = 5000 ] || fail "a partial insert charged ($energy_after): the reference charges only an emptied stack"
[ "$held" = 32 ] || fail "the player holds $held stone, expected the 32 that did not fit"

say "counterfactual: a full barrel as the only inventory moves nothing, charges nothing and leaves the player's 64 stone"
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:barrel >/dev/null
ctl wait 4
seed_slots "$CHEST_X" "$SY" "$SZ" "$(full_slots minecraft:stone)"
ctl wait 4
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl command "clear @a" >/dev/null
ctl wait 6
ctl give minecraft:stone 64 >/dev/null || fail "cannot give the player 64 stone"
ctl wait 8
line=$(insert_cmd "$SX" "$SY" "$SZ" minecraft:stone)
echo "insert stone into a full barrel: $line"
echo "$line" | grep -q "^$SCANNER: inserted 0 of 64 minecraft:stone; the rest stays with you, no charge (5000/$MAX_FE FE)$" || fail "the full-barrel insert answered: $line"
barrel_first=$(slot_count "$CHEST_X" "$SY" "$SZ" 0)
barrel_last=$(slot_count "$CHEST_X" "$SY" "$SZ" 26)
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the full-barrel insert: barrel ${barrel_first}..${barrel_last} stone, energy ${energy_after}, player stone ${held}"
[ "$barrel_first" = 64 ] && [ "$barrel_last" = 64 ] || fail "the full barrel changed ($barrel_first..$barrel_last)"
[ "$energy_after" = 5000 ] || fail "an insert that moved nothing charged ($energy_after)"
[ "$held" = 64 ] || fail "the player holds $held stone, expected the 64 the barrel refused"

say "a box with nothing routable refuses with 'there are no routable inventories' and charges nothing"
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl wait 4
line=$(insert_cmd "$SX" "$SY" "$SZ" minecraft:stone)
echo "insert stone into a scanner with no inventory in its box: $line"
echo "$line" | grep -q "^$SCANNER: there are no routable inventories (5000/$MAX_FE FE)$" || fail "the empty-box insert answered: $line"
energy_after=$(energy_of "$SX" "$SY" "$SZ")
held=$(count_item minecraft:stone)
echo "after the empty-box refusal: energy ${energy_after}, player stone ${held}"
[ "$energy_after" = 5000 ] || fail "the empty-box refusal charged ($energy_after)"
[ "$held" = 64 ] || fail "the empty-box refusal changed the player's stone to $held"

say "an insert on a block that is not a scanner is refused before anything is read"
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:chest >/dev/null
ctl wait 4
line=$(insert_cmd "$CHEST_X" "$SY" "$SZ" minecraft:stone)
echo "insert on the chest: $line"
echo "$line" | grep -q '^minecraft:chest: not a storage scanner$' || fail "the chest took an insert: $line"
ctl setblock "$CHEST_X" "$SY" "$SZ" minecraft:air >/dev/null
ctl wait 2

say "THE EXPORT CHOICE: 'Export to current container' sends the in-slot stack only to the selected inventory"
# The GUI's exportToStarred choice (`StorageScannerData.exportC`): with `current`
# the tick schedules its GUI-slot stack into the one inventory the list's selected
# row names (`getInventoryForBlock`'s lastSelectedInventory) and into nothing else —
# upstream returns from that attempt with the remainder, so a full selected
# container leaves the stack in the slot instead of falling through to the walk.
# The rig is the star phase's: scanner at S, radius 2, two empty chests two away on
# different axes (A at S+2 on x, B at S+2 on z; B comes first in the x-major walk).
EXPORT_A=$((SX + 2))
EXPORT_B=$((SZ + 2))
for at in "$((SX + 1)):$SY:$SZ" "$((SX + 3)):$SY:$SZ" "$EXPORT_A:$SY:$SZ" "$SX:$SY:$EXPORT_B"; do
    IFS=: read -r ex ey ez <<<"$at"
    ctl setblock "$ex" "$ey" "$ez" minecraft:air >/dev/null
done
ctl setblock "$EXPORT_A" "$SY" "$SZ" minecraft:chest >/dev/null
ctl setblock "$SX" "$SY" "$EXPORT_B" minecraft:chest >/dev/null
ctl wait 4
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
seed_slots "$SX" "$SY" "$SZ" ''
line=$(routable_cmd "$SX" "$SY" "$SZ" "$EXPORT_A" "$SY" "$SZ" on)
echo "star A: $line"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$EXPORT_B" on)
echo "star B: $line"
saved=$(data_reply "$SX" "$SY" "$SZ" exportc)
echo "saved exportc (default): $saved"
case "$saved" in
    *"Found no elements"*) ;;
    *) fail "a fresh scanner has an exportc flag: $saved" ;;
esac
line=$(export_cmd "$SX" "$SY" "$SZ" current)
echo "export current: $line"
echo "$line" | grep -q "^$SCANNER: export current$" || fail "the export command answered: $line"
saved=$(data_value "$SX" "$SY" "$SZ" exportc)
echo "saved exportc: $saved"
[ "$saved" = 1b ] || fail "the saved exportc is '$saved', expected 1b (exportC true)"
line=$(select_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$EXPORT_B")
echo "select B: $line"
echo "$line" | grep -q "^$SCANNER: select $SX $SY $EXPORT_B$" || fail "the select command answered: $line"
saved=$(data_reply "$SX" "$SY" "$SZ" selected)
echo "saved selected (B): $saved"
case "$saved" in
    *"[I; $SX, $SY, $EXPORT_B]"*) ;;
    *) fail "the saved selection does not name B: $saved" ;;
esac
# The 64 stone goes to B and nowhere else: A (starred too) stays empty.
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$EXPORT_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$EXPORT_B")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "export current, B selected: in-slot '${in_slot}', A ${a_total}, B ${b_total}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the current-container export left $in_slot stone in the in-slot"
[ "$a_total" = 0 ] || fail "the current-container export also put $a_total stone in the unselected A"
[ "$b_total" = 64 ] || fail "the selected chest B holds $b_total, expected 64"
[ "$energy" = "$((5000 - 20))" ] || fail "the current-container export left $energy, expected one rfPerInsert off 5000"

if [ "$EXPORT_SELFTEST" = 1 ]; then
    say "EXPORT_SELFTEST: assert the current-container export did NOT send the stone to the selected B (must fail)"
    fail "the current-container export sent the 64 stone to the selected B (${b_total}) and left A empty (${a_total}) for one rfPerInsert (energy ${energy}), so this inverted assertion fails as it must"
fi

say "the selection follows the list's row: selecting A sends the next stack to A, B keeps its 64"
line=$(select_cmd "$SX" "$SY" "$SZ" "$EXPORT_A" "$SY" "$SZ")
echo "select A: $line"
echo "$line" | grep -q "^$SCANNER: select $EXPORT_A $SY $SZ$" || fail "the select command answered: $line"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$EXPORT_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$EXPORT_B")
echo "export current, A selected: in-slot '${in_slot}', A ${a_total}, B ${b_total}"
[ "$in_slot" = "" ] || fail "the A-selected export left $in_slot stone in the in-slot"
[ "$a_total" = 64 ] || fail "the selected chest A holds $a_total, expected 64"
[ "$b_total" = 64 ] || fail "the unselected chest B changed to $b_total"

say "the export choice and the selection are saved with the block and survive a relaunch"
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch "$GAME"
ctl wait 20 >/dev/null
saved=$(data_value "$SX" "$SY" "$SZ" exportc)
echo "saved exportc after the relaunch: $saved"
[ "$saved" = 1b ] || fail "the relaunch lost the export choice (saved '$saved')"
saved=$(data_reply "$SX" "$SY" "$SZ" selected)
echo "saved selected after the relaunch: $saved"
case "$saved" in
    *"[I; $EXPORT_A, $SY, $SZ]"*) ;;
    *) fail "the relaunch lost the selection on A: $saved" ;;
esac

say "a selected full barrel refuses without falling through to the walk: the in-slot keeps its 64"
# The barrel is in the box and not starred; the walk would pass it for the starred
# A or B (both have room), so a stack that stays proves the selected attempt
# returned instead of walking on. The tick charges rfPerInsert every tick while a
# refused stack stays, so the assertion is on the stack, not on the buffer.
EXPORT_BARREL_Z=$((SZ + 1))
ctl setblock "$SX" "$SY" "$EXPORT_BARREL_Z" minecraft:air >/dev/null
ctl setblock "$SX" "$SY" "$EXPORT_BARREL_Z" minecraft:barrel >/dev/null
ctl wait 4
seed_slots "$SX" "$SY" "$EXPORT_BARREL_Z" "$(full_slots minecraft:stone)"
ctl wait 4
line=$(select_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$EXPORT_BARREL_Z")
echo "select the full barrel: $line"
echo "$line" | grep -q "^$SCANNER: select $SX $SY $EXPORT_BARREL_Z$" || fail "the select command answered: $line"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
seed_slots "$SX" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
barrel_last=$(slot_count "$SX" "$SY" "$EXPORT_BARREL_Z" 26)
a_total=$(inv_total "$EXPORT_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$EXPORT_B")
echo "selected full barrel: in-slot ${in_slot}, barrel slot 26 ${barrel_last}, A ${a_total}, B ${b_total}"
[ "$in_slot" = 64 ] || fail "a refusing selected barrel left the in-slot at '$in_slot', expected its 64 (no fall-through)"
[ "$barrel_last" = 64 ] || fail "the full barrel changed (slot 26 ${barrel_last})"
[ "$a_total" = 64 ] || fail "the selected full barrel fell through to A (${a_total})"
[ "$b_total" = 64 ] || fail "the selected full barrel fell through to B (${b_total})"

say "export first restores the starred walk, and the stack the barrel refused goes in"
line=$(export_cmd "$SX" "$SY" "$SZ" first)
echo "export first: $line"
echo "$line" | grep -q "^$SCANNER: export first$" || fail "the export command answered: $line"
saved=$(data_value "$SX" "$SY" "$SZ" exportc)
echo "saved exportc: $saved"
[ "$saved" = 0b ] || fail "the saved exportc is '$saved', expected 0b (the first routable container)"
ctl command "data merge block $SX $SY $SZ {energy: 5000}" >/dev/null
ctl wait 10
in_slot=$(slot_count "$SX" "$SY" "$SZ" 0)
a_total=$(inv_total "$EXPORT_A" "$SY" "$SZ")
b_total=$(inv_total "$SX" "$SY" "$EXPORT_B")
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "export first: in-slot '${in_slot}', A ${a_total}, B ${b_total}, energy ${energy}"
[ "$in_slot" = "" ] || fail "the walk left $in_slot stone in the in-slot"
[ "$a_total" = 64 ] || fail "the walk changed A to $a_total"
[ "$b_total" = 128 ] || fail "the walk's first routable chest B holds $b_total, expected 64 + 64"
# The buffer is not asserted here: the tick charges rfPerInsert every tick while a
# refused stack stays (the barrel phase), so the exact figure depends on how many
# ticks the chat round trip cost. The first export phase pinned the charge (4,980).
# The barrel and the two chests leave the rig for the loot leg.
ctl setblock "$SX" "$SY" "$EXPORT_BARREL_Z" minecraft:air >/dev/null
ctl setblock "$EXPORT_A" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$SX" "$SY" "$EXPORT_B" minecraft:air >/dev/null
ctl wait 4

say "THE ROW CONTENTS VIEW: a row lists its own inventory; 'all' lists only the starred rows"
# `StorageScannerTileEntity.getInventoryForBlock(cpos)`: with a cell it lists that
# inventory's contents (`addItemsFromInventory`, no star test), and with the list's
# first "All routable" row — upstream's `(-1, -1, -1)` sentinel — only the starred
# inventories (`routable.contains`), the `countItems(..., starred = true)` half of
# the star. A listing is not a request: `getInventoryForBlock` never calls
# `consumeEnergy`, so the phase runs at buffer 0 and leaves it there.
CONTENTS_A=$((SX + 2))
CONTENTS_B=$((SZ + 2))
for at in "$((SX + 1)):$SY:$SZ" "$((SX + 3)):$SY:$SZ" "$CONTENTS_A:$SY:$SZ" "$SX:$SY:$CONTENTS_B"; do
    IFS=: read -r cx cy cz <<<"$at"
    ctl setblock "$cx" "$cy" "$cz" minecraft:air >/dev/null
done
ctl setblock "$CONTENTS_A" "$SY" "$SZ" minecraft:chest >/dev/null
ctl setblock "$SX" "$SY" "$CONTENTS_B" minecraft:chest >/dev/null
ctl wait 4
seed_slots "$CONTENTS_A" "$SY" "$SZ" '{Slot: 0b, id: "minecraft:stone", count: 64}'
seed_slots "$SX" "$SY" "$CONTENTS_B" '{Slot: 0b, id: "minecraft:stone", count: 64}, {Slot: 1b, id: "minecraft:stone", count: 64}'
line=$(radius_cmd "$SX" "$SY" "$SZ" 2)
echo "radius: $line"
seed_slots "$SX" "$SY" "$SZ" ''
line=$(routable_cmd "$SX" "$SY" "$SZ" "$CONTENTS_A" "$SY" "$SZ" on)
echo "star A: $line"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$CONTENTS_B" on)
echo "star B: $line"
ctl command "data merge block $SX $SY $SZ {energy: 0}" >/dev/null
ctl wait 4
contents_line_b=$(contents_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$CONTENTS_B")
echo "contents B: $contents_line_b"
b_total=$(inv_total "$SX" "$SY" "$CONTENTS_B")
echo "  B holds $b_total stone"
[ "$contents_line_b" = "$SCANNER: contents $SX $SY $CONTENTS_B: minecraft:stone $b_total" ] \
    || fail "the row view of B answered '$contents_line_b', expected its $b_total stone"
[ "$b_total" = 128 ] || fail "B holds $b_total stone, expected 64 + 64"

if [ "$CONTENTS_SELFTEST" = 1 ]; then
    say "CONTENTS_SELFTEST: assert the row view of B DID NOT list its 128 stone (must fail)"
    fail "the row view of B answered '$contents_line_b' for the $b_total stone it holds, so this inverted assertion fails as it must"
fi

contents_line_a=$(contents_cmd "$SX" "$SY" "$SZ" "$CONTENTS_A" "$SY" "$SZ")
echo "contents A: $contents_line_a"
a_total=$(inv_total "$CONTENTS_A" "$SY" "$SZ")
[ "$contents_line_a" = "$SCANNER: contents $CONTENTS_A $SY $SZ: minecraft:stone $a_total" ] \
    || fail "the row view of A answered '$contents_line_a', expected its $a_total stone"
[ "$a_total" = 64 ] || fail "A holds $a_total stone, expected 64"
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "energy after two row listings: ${energy}/50000"
[ "$energy" = 0 ] || fail "a row listing was charged (energy $energy, expected 0 — a listing is not a request)"

say "the star filters the all row, not a single row: un-starring B leaves the all row with A's 64"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$CONTENTS_B" off)
echo "un-star B: $line"
all_line=$(contents_all_cmd "$SX" "$SY" "$SZ")
echo "contents all (B unstarred): $all_line"
[ "$all_line" = "$SCANNER: contents all: minecraft:stone 64" ] || fail "the all row answered: $all_line"
unstarred_row=$(contents_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$CONTENTS_B")
echo "contents B (unstarred): $unstarred_row"
[ "$unstarred_row" = "$SCANNER: contents $SX $SY $CONTENTS_B: minecraft:stone 128" ] \
    || fail "the un-starred row B answered '$unstarred_row': a row lists its own inventory whether or not it is starred"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$CONTENTS_A" "$SY" "$SZ" off)
echo "un-star A: $line"
all_line=$(contents_all_cmd "$SX" "$SY" "$SZ")
echo "contents all (nothing starred): $all_line"
[ "$all_line" = "$SCANNER: contents all: empty" ] || fail "the all row with nothing starred answered: $all_line"
line=$(routable_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$CONTENTS_B" on)
echo "star B again: $line"
all_line=$(contents_all_cmd "$SX" "$SY" "$SZ")
echo "contents all (B only): $all_line"
[ "$all_line" = "$SCANNER: contents all: minecraft:stone 128" ] \
    || fail "the all row with B starred answered: $all_line"

say "a cell the scan did not list is refused: outside the box, the scanner itself, an empty cell"
line=$(contents_cmd "$SX" "$SY" "$SZ" "$((SX + 3))" "$SY" "$SZ")
echo "contents at S+3 (outside radius 2): $line"
[ "$line" = "$SCANNER: $((SX + 3)) $SY $SZ is not an inventory in the scan box" ] \
    || fail "the outside cell answered: $line"
line=$(contents_cmd "$SX" "$SY" "$SZ" "$SX" "$SY" "$SZ")
echo "contents of the scanner itself: $line"
[ "$line" = "$SCANNER: $SX $SY $SZ is not an inventory in the scan box" ] \
    || fail "the scanner's own cell answered: $line"
line=$(contents_cmd "$SX" "$SY" "$SZ" "$((SX + 1))" "$SY" "$SZ")
echo "contents of an empty cell: $line"
[ "$line" = "$SCANNER: $((SX + 1)) $SY $SZ is not an inventory in the scan box" ] \
    || fail "the empty cell answered: $line"
energy=$(energy_of "$SX" "$SY" "$SZ")
echo "energy after three refusals: ${energy}/50000"
[ "$energy" = 0 ] || fail "a refused row listing was charged (energy $energy)"
# The rig leaves for the loot leg; the saved star (B) is inert once the scanner is
# broken, and the later legs reseed their own chests.
ctl setblock "$CONTENTS_A" "$SY" "$SZ" minecraft:air >/dev/null
ctl setblock "$SX" "$SY" "$CONTENTS_B" minecraft:air >/dev/null
ctl wait 4

say "THE CRAFTING GRID: the pattern's nine cells and the four craft buttons, out of the storage's own contents"
# Upstream's ghost panel: the nine cells are a recipe memory (CraftingGridInventory's
# slot limit is 0), and the four buttons (1/4/8/*) craft out of the player's
# inventory and then the (locked) storage. The pin has no ghost slot kind, so the
# memory is the block's saved `grid` key and the buttons are `/rftoolsstorage craft`.
CGX=$((X - 3))
CGZ=$((Z1 + 6))
ctl close >/dev/null 2>&1 || true
ctl command "clear @a" >/dev/null
ctl setblock "$CGX" "$((Y - 1))" "$CGZ" minecraft:stone >/dev/null
ctl give "$STORAGE" 1 >/dev/null || fail "cannot give a modular storage for the crafting rig"
ctl wait 6
ctl select 1 >/dev/null || true
ctl wait 6
place_held "$CGX" "$Y" "$CGZ" "$STORAGE"
seed_crafting_storage "$CGX" "$Y" "$CGZ" 8
line=$(reply info "$CGX" "$Y" "$CGZ")
echo "the crafting rig's storage: $line"
echo "$line" | grep -q "^$STORAGE: 8/6400 minecraft:oak_planks locked$" \
    || fail "the crafting rig's storage reads: $line"

line=$(grid_cmd "$CGX" "$Y" "$CGZ")
echo "pattern (fresh block): $line"
[ "$line" = "$STORAGE: pattern empty" ] || fail "a fresh pattern read: $line"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft with an empty pattern: $line"
[ "$line" = "$STORAGE: the pattern is empty" ] || fail "an empty pattern answered: $line"
[ "$(count_item minecraft:stick)" = 0 ] || fail "an empty pattern made sticks"

say "a cell of cobblestone is no recipe; one plank is the shapeless button — the game's own table matches"
line=$(grid_set "$CGX" "$Y" "$CGZ" 1 minecraft:cobblestone)
echo "grid 1 cobblestone: $line"
[ "$line" = "$STORAGE: pattern 1=minecraft:cobblestone -> no recipe" ] || fail "a cobblestone cell read: $line"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft with no recipe: $line"
[ "$line" = "$STORAGE: the pattern crafts nothing" ] || fail "a pattern no recipe matches crafted: $line"
[ "$(planks_at "$CGX" "$Y" "$CGZ")" = 8 ] || fail "a pattern no recipe matches took planks"
line=$(grid_set "$CGX" "$Y" "$CGZ" 1 minecraft:oak_planks)
echo "grid 1 oak_planks: $line"
[ "$line" = "$STORAGE: pattern 1=minecraft:oak_planks -> minecraft:oak_button x1" ] \
    || fail "one plank is the shapeless button recipe, and the readback answered: $line"

say "the stick recipe's shape (cells 1 and 4) makes 4 sticks; the pattern is the saved grid key"
line=$(grid_set "$CGX" "$Y" "$CGZ" 4 minecraft:oak_planks)
echo "grid 4 oak_planks: $line"
[ "$line" = "$STORAGE: pattern 1=minecraft:oak_planks, 4=minecraft:oak_planks -> minecraft:stick x4" ] \
    || fail "the stick pattern read: $line"
saved=$(data_block "$CGX" "$Y" "$CGZ" grid)
echo "saved grid key: $saved"
case "$saved" in
    *minecraft:oak_planks*) ;;
    *) fail "the pattern is not in the block's saved grid key: $saved" ;;
esac
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft 1: $line"
[ "$line" = "$STORAGE: crafted 4 minecraft:stick" ] || fail "the craft answered: $line"
sticks=$(count_item minecraft:stick)
planks=$(planks_at "$CGX" "$Y" "$CGZ")
echo "after craft 1: sticks $sticks, storage planks $planks"
[ "$sticks" = 4 ] || fail "the player holds $sticks sticks, expected 4"
[ "$planks" = 6 ] || fail "the storage holds $planks planks, expected 6 (two consumed)"

if [ "$CRAFT_SELFTEST" = 1 ]; then
    say "CRAFT_SELFTEST: assert the craft did NOT make the sticks (must fail)"
    fail "the craft made 4 minecraft:stick from the planks in the pattern's cells 1 and 4 (sticks 0 -> $sticks) and the storage fell to $planks planks, so this inverted assertion fails as it must"
fi

say "the batch buttons: 4 is one set, 8 is two, '*' is a whole stack of results"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 4)
echo "craft 4: $line"
[ "$line" = "$STORAGE: crafted 4 minecraft:stick" ] || fail "craft 4 answered: $line"
[ "$(planks_at "$CGX" "$Y" "$CGZ")" = 4 ] || fail "craft 4 did not take one plank per cell"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 8)
echo "craft 8: $line"
[ "$line" = "$STORAGE: crafted 8 minecraft:stick" ] || fail "craft 8 answered: $line"
[ "$(planks_at "$CGX" "$Y" "$CGZ")" = 0 ] || fail "craft 8 did not take two planks per cell"
[ "$(count_item minecraft:stick)" = 16 ] || fail "the player holds $(count_item minecraft:stick) sticks, expected 16"

say "an empty storage answers what is missing per cell, and takes nothing"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" stack)
echo "craft stack with nothing to pull: $line"
[ "$line" = "$STORAGE: missing cell 1: 16 minecraft:oak_planks, cell 4: 16 minecraft:oak_planks" ] \
    || fail "the missing report answered: $line"
[ "$(count_item minecraft:stick)" = 16 ] || fail "a refused batch still made sticks"
seed_crafting_storage "$CGX" "$Y" "$CGZ" 32
line=$(craft_cmd "$CGX" "$Y" "$CGZ" stack)
echo "craft stack from 32 planks: $line"
[ "$line" = "$STORAGE: crafted 64 minecraft:stick" ] || fail "craft stack answered: $line"
[ "$(planks_at "$CGX" "$Y" "$CGZ")" = 0 ] || fail "craft stack did not take 16 planks per cell (32)"
[ "$(count_item minecraft:stick)" = 80 ] || fail "the player holds $(count_item minecraft:stick) sticks, expected 80"

say "the player's inventory is the first source (upstream's InvWrapper order), the storage the second"
seed_crafting_storage "$CGX" "$Y" "$CGZ" 8
ctl give minecraft:oak_planks 2 >/dev/null || fail "cannot give the player planks"
ctl wait 8
held=$(count_item minecraft:oak_planks)
echo "the player holds $held planks, the storage 8"
[ "$held" = 2 ] || fail "the player holds $held planks, expected 2"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft 1: $line"
[ "$line" = "$STORAGE: crafted 4 minecraft:stick" ] || fail "the craft answered: $line"
held=$(count_item minecraft:oak_planks)
planks=$(planks_at "$CGX" "$Y" "$CGZ")
echo "after the craft: the player holds $held planks, the storage $planks"
[ "$held" = 0 ] || fail "the craft did not take the player's planks first ($held left)"
[ "$planks" = 8 ] || fail "the craft reached into the storage while the player had the planks ($planks)"

say "an unlocked storage hands nothing out: only the player's inventory is a source"
line=$(reply lock "$CGX" "$Y" "$CGZ")
echo "unlock: $line"
[ "$line" = "$STORAGE: unlocked" ] || fail "the unlock answered: $line"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft on an unlocked storage: $line"
[ "$line" = "$STORAGE: missing cell 1: 1 minecraft:oak_planks, cell 4: 1 minecraft:oak_planks" ] \
    || fail "an unlocked storage fed the craft: $line"
[ "$(planks_at "$CGX" "$Y" "$CGZ")" = 8 ] || fail "the unlocked storage lost planks"

say "the pattern's cells are the ghost ones: nothing is written while the storage is unlocked"
line=$(grid_set "$CGX" "$Y" "$CGZ" 2 minecraft:oak_planks)
echo "grid 2 while unlocked: $line"
[ "$line" = "$STORAGE: lock the storage to change the pattern" ] || fail "an unlocked pattern took a cell: $line"
line=$(reply lock "$CGX" "$Y" "$CGZ")
echo "lock again: $line"
[ "$line" = "$STORAGE: locked" ] || fail "the lock answered: $line"
line=$(grid_cmd "$CGX" "$Y" "$CGZ")
echo "pattern after the refused write: $line"
[ "$line" = "$STORAGE: pattern 1=minecraft:oak_planks, 4=minecraft:oak_planks -> minecraft:stick x4" ] \
    || fail "the refused write changed the pattern: $line"

say "the pattern is saved with the block: it survives a relaunch and still crafts"
ctl quit >/dev/null 2>&1 || true
sleep 5
rm -f "$SOCK"
launch "$GAME"
ctl wait 20 >/dev/null
line=$(grid_cmd "$CGX" "$Y" "$CGZ")
echo "pattern after the relaunch: $line"
[ "$line" = "$STORAGE: pattern 1=minecraft:oak_planks, 4=minecraft:oak_planks -> minecraft:stick x4" ] \
    || fail "the relaunch lost the pattern: $line"
seed_crafting_storage "$CGX" "$Y" "$CGZ" 2
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft after the relaunch: $line"
[ "$line" = "$STORAGE: crafted 4 minecraft:stick" ] || fail "the craft after the relaunch answered: $line"
[ "$(count_item minecraft:stick)" = 88 ] || fail "the player holds $(count_item minecraft:stick) sticks, expected 88"

say "the pattern's cells clear, and a cleared pattern crafts nothing"
line=$(grid_clear "$CGX" "$Y" "$CGZ")
echo "grid clear: $line"
[ "$line" = "$STORAGE: pattern empty" ] || fail "grid clear answered: $line"
line=$(craft_cmd "$CGX" "$Y" "$CGZ" 1)
echo "craft after the clear: $line"
[ "$line" = "$STORAGE: the pattern is empty" ] || fail "a cleared pattern crafted: $line"
barrel=$((CGZ + 2))
ctl setblock "$CGX" "$Y" "$barrel" minecraft:barrel >/dev/null
ctl wait 4
line=$(craft_cmd "$CGX" "$Y" "$barrel" 1)
echo "craft on a barrel (not a modular storage): $line"
[ "$line" = "minecraft:barrel: not a modular storage" ] || fail "a barrel answered: $line"

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
echo "PASS: the storage scanner places and reads back with its facing; a fresh one has no power (no power (0/50000 FE)) and reads nothing; at its default radius 1 it finds nothing two blocks away; /rftoolsstorage radius 2 widens the box and the scan answers minecraft:stone 64 for a chest two blocks away while the chest three away (minecraft:dirt 32) is outside it; radius 3 lists both, one entry a kind; every scan costs rfPerRequest (5,000 -> 4,900 -> 4,800 -> 4,700, printed in the reply); THE SCAN FILTER (the GUI's search box) keeps '@minecraft' listing both kinds, narrows 'dirt' to minecraft:dirt 32 and 'stone' to minecraft:stone 64, answers 'empty' for '@nomatch' with the price still paid (4,600), and leaves the unfiltered scan listing both (4,500) as the differential; THE SORT CHOICE orders the list (Amount- = dirt 64, stone 32, Amount+ = stone 32, dirt 64, Name = dirt 64, stone 32, Mod puts rftoolsstorage:storage_module0 after both minecraft kinds), and the saved AMOUNT_DESCENDING mode orders the list after a relaunch too (stone 64, dirt 32); an emptied buffer refuses with no power and reads nothing, and charging it again makes the scan answer again; scan and radius refuse a chest and the modular storage (not a storage scanner); the radius and the buffer survive a relaunch; THE INJECT TICK pushes 64 stone from the in-slot into the empty chest two blocks away so its Items[0] reads minecraft:stone x64 while the buffer drops by exactly one rfPerInsert (5000 -> 4980), and with the buffer at 0 the next stack stays in the slot and the chest keeps its 64, recharging proving the stall was the power (chest 128); the auto slot routes too (chest 160); items the game puts in the container are routed for one rfPerInsert; a stack nothing accepts is still charged one attempt (20 FE pays one, 19 pays none) and stays in the slot; the walk passes a full barrel for the chest behind it; the tick never writes the out slot; THE REQUEST PATH pulls 32 stone from a chest of 128 into the player's inventory (the reference's straight-to-inventory default), leaving the chest at 96 and the buffer at 4,990 for an explicit count, refuses a kind nothing holds and charges nothing, refuses below the price with the price line (costs 10, have 9), sends a ctl command's 16-stone request straight to the player too, and refuses a request whose out slot holds another kind without pulling or charging; THE INSERT PATH pushes the player's 64 stone into the chest two blocks away so its Items[0] reads minecraft:stone x64 and empties the player's stack for exactly one rfPerInsert (5000 -> 4980), inserts an explicit 10 of 64 for one more rfPerInsert, takes only the 32 a nearly-full barrel has room for and charges NOTHING for the partial insert, moves nothing and charges nothing against a full barrel (the player keeps 64), refuses a box with no routable inventory (there are no routable inventories, no charge) and refuses a chest (not a storage scanner); THE GIVE PATH hands the whole stack (64) straight to the player from a chest of 64, empties the chest and charges the FULL rfPerRequest (5000 -> 4900) with the out slot still holding the request counterfactual's dirt (it has no out slot), hands exactly one item with [single] (chest 3 -> 2, buffer 4900 -> 4800), refuses a kind nothing holds and charges nothing, and refuses below the full price with the price line (costs 100, have 99) without touching the chest, and a give on a chest is refused as not a storage scanner; the loot table drops the scanner on a survival break; the modular storage still answers 0/0 empty no module; a barrel answers not a storage scanner to the same commands; and THE ROUTABLE STAR LIST starts empty (a fresh scanner's tick routes its 64 stone nowhere and still pays one rfPerInsert), routes only the starred chest (B takes 64 while the un-starred A stays empty), still lists the un-starred A's 32 dirt in the scan (the star filters routing, not the list), restores the walk when A is re-starred (A 96), and survives a relaunch (the saved list names A, which then takes another stack, 160); THE EXPORT CHOICE (the GUI's 'Export to current container') sends the in-slot's 64 stone only to the selected inventory (B 64, A 0) for one rfPerInsert, follows the list's row (selecting A makes A 64 while B keeps its 64), saves the choice and the selection with the block so both survive a relaunch, does not fall through to the walk from a selected full barrel (the in-slot keeps its 64 and A and B stay at 64), and 'export first' restores the starred walk, which routes the same stack to B (128); THE ROW CONTENTS VIEW (the GUI list's per-row contents) answers 'contents B' with B's 128 stone and 'contents A' with A's 64 at buffer 0 and leaves the buffer at 0 (a listing is not a request), lists only the starred inventories in 'contents all' (un-starring B leaves A's 64 while B's own row still lists its 128, with nothing starred the all row is 'empty', and starring B alone lists 128), and refuses a cell outside the box, the scanner's own cell and an empty cell with 'is not an inventory in the scan box', also free; THE CRAFTING GRID (the jar's ghost pattern and its four craft buttons, carried as the grid and craft commands) reads a fresh block's pattern as empty and refuses a craft, matches the game's own recipe table (a cobblestone cell is no recipe and takes nothing, one plank is the shapeless oak button, two planks in the stick shape are minecraft:stick x4 and the saved grid key holds the two cells), crafts 4 sticks out of the storage for one set and takes exactly one plank a cell (8 -> 6, player 0 -> 4), takes one set for 4 (4 planks) and two for 8 (0 planks, 16 sticks), answers 'missing cell 1: 16 minecraft:oak_planks, cell 4: 16 minecraft:oak_planks' on an empty storage without taking anything and then crafts a whole stack (64 sticks) from 32 planks, takes the player's 2 planks before the storage's 8 (upstream's first-source order) and leaves the storage at 8, refuses an unlocked storage (missing 1 a cell, the storage keeps its 8) and refuses a cell write while unlocked ('lock the storage to change the pattern'), keeps the pattern and crafts again after a relaunch (4 more sticks), and clears with grid clear (the pattern is empty again)"
