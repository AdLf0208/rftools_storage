// RFTools Storage for Horizon — the modular storage engine (pure).
//
// Ported from RFToolsStorage 1.21-6.0.5 (`McJtyMods/RFToolsStorage`, MIT),
// Minecraft Java 1.21.1 NeoForge. The upstream source was read for behaviour
// only; this is a clean-room Rust rewrite.
//
// Slice 1: `rftoolsstorage:modular_storage` with a tier-1 storage module.
// Upstream's capacity is `StorageModuleItem.MAXSIZE` slots (100 for
// `storage_module0`), each holding one item kind up to
// `GlobalStorageItemWrapper.getSlotLimit()` = 64 (and never more than the
// item's own stack size). The block stores nothing until a module is in its
// slot *and* it is locked: `insertItem`/`extractItem` both start with
// `if (!isLocked())` and the module slot's `mayPlace` is `if (isLocked())`.
//
// The block entity's container slots are the storage (the pin saves a mod
// block entity's `Items` with its chunk), so the glue keeps the `ItemStack`s
// and this module keeps the rules: how many fit, in which slot, in which
// order, and what the readout says.

/// The block this slice implements.
pub const MODULAR_STORAGE: &str = "rftoolsstorage:modular_storage";

/// `StorageModuleItem.MAXSIZE`: the slots each tier's module gives
/// (`{100, 200, 300, 500, 0, 0, -1}`; the unused and remote entries are not
/// carried).
pub const TIER_SLOTS: [u32; 4] = [100, 200, 300, 500];

/// `GlobalStorageItemWrapper.getSlotLimit()`: the most one slot holds, before
/// the item's own stack size caps it (`getStackLimit`).
pub const SLOT_LIMIT: u32 = 64;

/// Whether `block` is the block this slice implements.
#[must_use]
pub fn is_modular_storage(block: &str) -> bool {
    block == MODULAR_STORAGE
}

/// The tier a storage module item installs, from its id
/// (`rftoolsstorage:storage_module0` is tier 1 → index 0).
#[must_use]
pub fn module_tier(item: &str) -> Option<u8> {
    match item.strip_prefix("rftoolsstorage:storage_module")? {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        _ => None,
    }
}

/// The id of a tier's storage module item (`StorageModuleItem`'s registry
/// names). A block that recorded its tier before the card slot held the item
/// (`notes.md`, slice 3) gets this item written into the slot.
#[must_use]
pub fn module_id(tier: u8) -> String {
    format!("rftoolsstorage:storage_module{tier}")
}

/// Whether a menu **storage slot** holding `stored` takes a stack of `incoming`:
/// upstream's `Slot.mayPlace` on a storage slot, which is
/// `if (!isLocked()) return false;` and then
/// `getItemHandler().isItemValid(slot, stack)` — a slot is empty or already
/// holds that kind (`ItemStack.isSameItemSameComponents`).
#[must_use]
pub fn slot_accepts(locked: bool, stored: Option<&str>, incoming: &str) -> bool {
    locked && stored.is_none_or(|held| held == incoming)
}

/// Whether the menu's **card slot** takes `item`: upstream's `mayPlace` on
/// `ModularStorageContainer.SLOT_STORAGE_MODULE` — a storage module while the
/// block is unlocked, which `!= occupied` covers because a locked storage keeps
/// its module — and, once it holds one, anything: the engine consults the card
/// slot's answer while it plans a click on **another** slot, and a refusal there
/// blocks that click (`notes.md`, "the card slot's answer must not block another
/// slot"). The modules are `stacksTo(1)`, so the engine's cap keeps a second one
/// out.
#[must_use]
pub fn card_accepts(occupied: bool, item: &str) -> bool {
    occupied || module_tier(item).is_some()
}

/// How many items a store of `slots` slots holds when each holds a stack of
/// `max_stack` (`min(64, the item's stack size)`, upstream's `getStackLimit`).
#[must_use]
pub fn capacity(slots: usize, max_stack: u32) -> u32 {
    let per_slot = max_stack.clamp(1, SLOT_LIMIT);
    u32::try_from(slots)
        .unwrap_or(u32::MAX)
        .saturating_mul(per_slot)
}

/// An item kind: `key` is its identity (item id, damage and components, as the
/// glue builds it — upstream's `ItemStack.isSameItemSameComponents`), `label`
/// the item id a readout shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind {
    pub key: String,
    pub label: String,
}

/// One slot's contents: which kind, and how many.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub kind: Kind,
    pub count: u32,
}

/// A stack taken out of the store, by slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taken {
    pub slot: usize,
    pub kind: Kind,
    pub count: u32,
}

/// The modular storage's rules and contents. `slots` is empty while no module
/// is installed (capacity 0).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Storage {
    pub slots: Vec<Option<Slot>>,
    pub tier: Option<u8>,
    pub locked: bool,
}

impl Storage {
    /// A storage with no module: it holds nothing.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// A storage with `tier`'s module installed and no contents.
    #[must_use]
    pub fn with_tier(tier: u8) -> Self {
        Self {
            slots: vec![None; slots_of(tier)],
            tier: Some(tier),
            locked: false,
        }
    }

    /// A storage as its block holds it: `tier`'s module in the card slot or `None`,
    /// the lock, and the block's own `slots` (contents, module or not).
    #[must_use]
    pub fn from_block(tier: Option<u8>, locked: bool, slots: Vec<Option<Slot>>) -> Self {
        Self {
            slots,
            tier,
            locked,
        }
    }

    /// Installs a module: only while unlocked, and a module already in place is
    /// not replaced. Resizes to the new tier, keeping what fits
    /// (`StorageEntry.resize`: a smaller module may lose the tail).
    pub fn install(&mut self, tier: u8) -> bool {
        if self.locked || self.tier.is_some() {
            return false;
        }
        self.slots.resize(slots_of(tier), None);
        self.tier = Some(tier);
        true
    }

    /// The module came out of the card slot: the block keeps its contents
    /// (upstream's world-level entry, `notes.md` deviation 1), so the storage
    /// shows and accepts nothing until a module is back, and then finds them. The
    /// view drops the lock with it — a storage with no module has nothing to guard,
    /// and the pin's menu rules only gate placement (no `may_pickup`), so a lock
    /// the view kept would leave no way to put the module back. The block's own
    /// byte stays where it is and the lock returns with the module.
    pub fn detach_module(&mut self) -> Option<u8> {
        let tier = self.tier.take()?;
        self.locked = false;
        Some(tier)
    }

    /// Flips the lock. Needs a module; false when there is none.
    pub fn toggle_lock(&mut self) -> bool {
        if self.tier.is_none() {
            return false;
        }
        self.locked = !self.locked;
        true
    }

    /// Whether the store takes and gives items: a module is in and the block is
    /// locked.
    #[must_use]
    pub fn accepts(&self) -> bool {
        self.locked && self.tier.is_some()
    }

    /// The slot count the module gives: none without a module, whatever the
    /// block's slots still hold.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        if self.tier.is_some() {
            self.slots.len()
        } else {
            0
        }
    }

    /// How many items are stored, all kinds together: none without a module (the
    /// block keeps them, the storage does not show them).
    #[must_use]
    pub fn total(&self) -> u32 {
        if self.tier.is_none() {
            return 0;
        }
        self.slots
            .iter()
            .flatten()
            .fold(0, |sum, slot| sum.saturating_add(slot.count))
    }

    /// How many slots hold something (upstream's `getNumStacks`, the module
    /// tooltip's "Contents: n/m"); none without a module.
    #[must_use]
    pub fn stacks_used(&self) -> u32 {
        if self.tier.is_none() {
            return 0;
        }
        u32::try_from(self.slots.iter().flatten().count()).unwrap_or(u32::MAX)
    }

    /// The first stored kind, in slot order; none without a module.
    #[must_use]
    pub fn first_kind(&self) -> Option<&Kind> {
        // No module, no visible contents.
        self.tier?;
        self.slots.iter().flatten().next().map(|slot| &slot.kind)
    }

    /// How many items the store holds when full, for an item of `max_stack`; 0
    /// without a module.
    #[must_use]
    pub fn capacity(&self, max_stack: u32) -> u32 {
        if self.tier.is_none() {
            return 0;
        }
        capacity(self.slots.len(), max_stack)
    }

    /// Inserts up to `count` of `kind`, one slot at a time in slot order (what
    /// `ItemHandlerHelper.insertItem` walks): a slot holding another kind is
    /// skipped, a slot holding this kind is topped up, an empty one filled.
    /// Returns how many were taken; the rest stays with the caller.
    pub fn insert(&mut self, kind: &Kind, count: u32, max_stack: u32) -> u32 {
        if !self.accepts() || count == 0 {
            return 0;
        }
        let per_slot = max_stack.clamp(1, SLOT_LIMIT);
        let mut left = count;
        for slot in &mut self.slots {
            if left == 0 {
                break;
            }
            match slot {
                Some(held) if held.kind.key == kind.key => {
                    let room = per_slot.saturating_sub(held.count);
                    let take = room.min(left);
                    held.count += take;
                    left -= take;
                }
                Some(_) => {}
                None => {
                    let take = per_slot.min(left);
                    *slot = Some(Slot {
                        kind: kind.clone(),
                        count: take,
                    });
                    left -= take;
                }
            }
        }
        count - left
    }

    /// Takes up to `max_stack` (never more than 64) out of the first stored
    /// slot, in slot order.
    pub fn extract_first(&mut self, max_stack: u32) -> Option<Taken> {
        if !self.accepts() {
            return None;
        }
        let per_slot = max_stack.clamp(1, SLOT_LIMIT);
        let index = self.slots.iter().position(Option::is_some)?;
        let slot = self.slots[index].take()?;
        let take = slot.count.min(per_slot);
        if slot.count > take {
            self.slots[index] = Some(Slot {
                count: slot.count - take,
                ..slot.clone()
            });
        }
        Some(Taken {
            slot: index,
            kind: slot.kind,
            count: take,
        })
    }

    /// Whether the store refuses every further item of `kind` (full for it):
    /// every slot is full, or holds another kind. Without a module it refuses
    /// everything.
    #[must_use]
    pub fn is_full_for(&self, kind: &Kind, max_stack: u32) -> bool {
        if self.tier.is_none() || self.slots.is_empty() {
            return true;
        }
        let per_slot = max_stack.clamp(1, SLOT_LIMIT);
        !self.slots.iter().any(|slot| match slot {
            None => true,
            Some(held) => held.kind.key == kind.key && held.count < per_slot,
        })
    }

    /// The state word the readout and `/rftoolsstorage info` print.
    #[must_use]
    pub fn state_word(&self) -> &'static str {
        if self.tier.is_none() {
            "no module"
        } else if self.locked {
            "locked"
        } else {
            "unlocked"
        }
    }

    /// `"<stored>/<capacity> <item>"`, or `"0/<capacity> empty"`; a storage with
    /// no module reads `"0/0 empty"` whatever its block holds.
    #[must_use]
    pub fn summary(&self, max_stack: u32) -> String {
        if self.tier.is_none() {
            return "0/0 empty".to_owned();
        }
        let capacity = self.capacity(max_stack);
        match self.first_kind() {
            Some(kind) => format!("{}/{} {}", self.total(), capacity, kind.label),
            None => format!("0/{capacity} empty"),
        }
    }
}

/// `TIER_SLOTS[tier]` as a slot count; 0 for a tier that is not carried.
fn slots_of(tier: u8) -> usize {
    TIER_SLOTS
        .get(usize::from(tier))
        .map_or(0, |n| usize::try_from(*n).unwrap_or(0))
}

/// What the padlock button answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockPress {
    /// No storage module: there is nothing to lock (upstream's `isLocked()` only
    /// means something with a module in the card slot).
    NoModule,
    /// It now stores nothing until it is unlocked again.
    Locked,
    /// It now stores and gives items.
    Unlocked,
}

/// The padlock's whole rule: flip the lock of a storage that has a module, and
/// say what happened.
pub fn lock_press(storage: &mut Storage) -> LockPress {
    if !storage.toggle_lock() {
        return LockPress::NoModule;
    }
    if storage.locked {
        LockPress::Locked
    } else {
        LockPress::Unlocked
    }
}

/// Whether a carried entry names a slot the block has: one of its `storage`
/// slots, or the card slot at `card`. Anything else (a slot past the container,
/// a doctored one) is dropped when the contents come back.
#[must_use]
pub fn carried_slot_in(storage: usize, card: usize, slot: u32) -> bool {
    let slot = usize::try_from(slot).unwrap_or(usize::MAX);
    slot < storage || slot == card
}

/// How many of a carried slot's items a placed block keeps: at most the item's
/// own stack size, never past [`SLOT_LIMIT`]. A `/give`n item may name any count
/// in any slot, and a block must never hold more than its container can.
#[must_use]
pub fn restored_count(count: u32, max_stack: u32) -> u32 {
    count.min(max_stack.clamp(1, SLOT_LIMIT))
}

/// How far a player without operator permission may be from a storage to read
/// or lock it.
pub const INSPECT_RANGE: f64 = 8.0;

/// `Commands.LEVEL_ADMINS`: the permission level that bypasses the reach check.
pub const LEVEL_ADMINS: u8 = 2;

/// Whether a caller may read or lock a storage: an operator, or a player in the
/// same level within [`INSPECT_RANGE`] of the block.
pub fn may_inspect(op: bool, same_level: bool, distance: f64) -> bool {
    op || (same_level && distance <= INSPECT_RANGE)
}

// --- the storage scanner (slice 5) ----------------------------------------

/// The block the scanner slice implements: `rftoolsstorage:storage_scanner`.
pub const STORAGE_SCANNER: &str = "rftoolsstorage:storage_scanner";

/// Whether `block` is the storage scanner.
#[must_use]
pub fn is_storage_scanner(block: &str) -> bool {
    block == STORAGE_SCANNER
}

/// `StorageScannerConfiguration.MAXENERGY`: how much FE a scanner holds.
pub const SCANNER_MAX_ENERGY: u32 = 50_000;

/// `StorageScannerConfiguration.RECEIVEPERTICK`: the most one fill moves.
pub const SCANNER_RECEIVE_PER_TICK: u32 = 500;

/// `StorageScannerConfiguration.rfPerRequest`: what one readout of what the
/// scanned inventories hold costs (upstream charges it per item request; the
/// port's readout is the request's price, `notes.md`).
pub const SCANNER_RF_PER_REQUEST: u32 = 100;

/// `StorageScannerConfiguration.rfPerInsert`: upstream's price for pushing one
/// stack into a scanned inventory (carried for the automation slice).
pub const SCANNER_RF_PER_INSERT: u32 = 20;

/// `StorageScannerData.DEFAULT.radius`: one, a 3 x 3 x 3 box around the block.
pub const SCANNER_DEFAULT_RADIUS: i32 = 1;

/// `GuiStorageScanner`'s radius slider: `.realMinimum(RFToolsStorage.setup.xnet ?
/// 0 : 1)`, `.realMaximum(20)` — 1 is the minimum without XNet.
pub const SCANNER_MIN_RADIUS: i32 = 1;

/// The GUI slider's upper end, and the radius a saved value is clamped to.
pub const SCANNER_MAX_RADIUS: i32 = 20;

/// A radius saved in the block's compound, clamped to the GUI's slider; a block
/// with no saved radius scans at [`SCANNER_DEFAULT_RADIUS`], as upstream's
/// attachment's default does.
#[must_use]
pub fn scanner_radius(saved: Option<i32>) -> i32 {
    saved.map_or(SCANNER_DEFAULT_RADIUS, clamp_radius)
}

/// `radius` inside the GUI's slider range.
#[must_use]
pub fn clamp_radius(radius: i32) -> i32 {
    radius.clamp(SCANNER_MIN_RADIUS, SCANNER_MAX_RADIUS)
}

/// Every block position in the scanner's box: `centre - radius ..= centre +
/// radius` on each axis, upstream's `inRange` (`findInventories`' three nested
/// loops). It is a cube, not a sphere.
#[must_use]
pub fn scan_positions(centre: [i32; 3], radius: i32) -> Vec<[i32; 3]> {
    let radius = radius.max(0);
    let side = usize::try_from(radius.saturating_mul(2).saturating_add(1)).unwrap_or(0);
    let mut positions = Vec::with_capacity(side.saturating_pow(3));
    for x in centre[0].saturating_sub(radius)..=centre[0].saturating_add(radius) {
        for y in centre[1].saturating_sub(radius)..=centre[1].saturating_add(radius) {
            for z in centre[2].saturating_sub(radius)..=centre[2].saturating_add(radius) {
                positions.push([x, y, z]);
            }
        }
    }
    positions
}

/// What a scan costs off `stored`: the buffer after it, or none when it cannot
/// pay [`SCANNER_RF_PER_REQUEST`].
#[must_use]
pub fn spend_power(stored: u32) -> Option<u32> {
    stored.checked_sub(SCANNER_RF_PER_REQUEST)
}

/// One kind the scan found, with the count of it in every inventory together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The item id (`minecraft:stone`).
    pub item: String,
    /// How many of it the scanned inventories hold.
    pub count: u32,
}

/// Merges the kinds an inventory walk found into one entry a kind, in the order
/// the walk first met them — the base the stable amount sort keeps for equal
/// counts (`GuiStorageScanner.updateContentsList`).
#[must_use]
pub fn merge_counts(entries: &[(String, u32)]) -> Vec<Found> {
    let mut found: Vec<Found> = Vec::new();
    for (item, count) in entries {
        if *count == 0 {
            continue;
        }
        if let Some(entry) = found.iter_mut().find(|entry| entry.item == *item) {
            entry.count = entry.count.saturating_add(*count);
        } else {
            found.push(Found {
                item: item.clone(),
                count: *count,
            });
        }
    }
    found
}

/// The kinds a scan or a list row shows, in the readout's voice: one
/// `item count` an entry, comma separated, or `empty` when nothing matched.
#[must_use]
pub fn found_list(found: &[Found]) -> String {
    if found.is_empty() {
        "empty".to_owned()
    } else {
        found
            .iter()
            .map(|entry| format!("{} {}", entry.item, entry.count))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// The scan's answer after a paid scan: what the inventories hold, then the
/// buffer left.
#[must_use]
pub fn scan_line(after: u32, found: &[Found]) -> String {
    let items = found_list(found);
    format!("{items} ({after}/{SCANNER_MAX_ENERGY} FE)")
}

/// The scan's answer when the buffer cannot pay for it.
#[must_use]
pub fn no_power_line(stored: u32) -> String {
    format!("no power ({stored}/{SCANNER_MAX_ENERGY} FE)")
}

// --- the scan's search filter (slice 10) ------------------------------------

/// Whether one kind passes the GUI list's search (`StorageScannerTileEntity
/// .getMatcher` / `makeSearchPredicate`): the search is split on whitespace and
/// **every** word must match (upstream ANDs the predicates); a word that starts
/// with `@` is the rest of it matched against the item's namespace by prefix
/// (upstream's `startsWith`), any other word is a case-insensitive contains on
/// the item's name. An empty search matches every kind (`getMatcher`'s
/// `s -> true`).
#[must_use]
pub fn matches_search(search: &str, namespace: &str, name: &str) -> bool {
    let namespace = namespace.to_lowercase();
    let name = name.to_lowercase();
    search.split_whitespace().all(|word| {
        let word = word.to_lowercase();
        match word.strip_prefix('@') {
            Some(modid) => namespace.starts_with(modid),
            None => name.contains(&word),
        }
    })
}

/// The namespace a search's `@modid` word is tested against: the part of an item
/// id before its `:`, `minecraft` for an id with no namespace.
#[must_use]
pub fn search_namespace(id: &str) -> &str {
    id.split_once(':')
        .map_or("minecraft", |(namespace, _)| namespace)
}

/// The name a plain search word is tested against: the item id's path with its
/// underscores as spaces (`minecraft:oak_log` -> `oak log`). Upstream tests
/// `ItemStack.getHoverName()`, which the SDK answers only on the client, so the
/// id path is the server-side reading of the name (`notes.md`, slice 10's
/// deviation 1).
#[must_use]
pub fn search_name(id: &str) -> String {
    id.split_once(':')
        .map_or(id, |(_, path)| path)
        .replace('_', " ")
}

// --- the scan list's sort choice (slice 11) --------------------------------

/// Upstream's `SortingMode`: the scanner GUI list's five orders. `Name` is the
/// default (`StorageScannerData.DEFAULT.sortingMode`), which is the order every
/// slice before this one printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortingMode {
    /// `AMOUNT_ASCENDING` — the GUI's "Amount+".
    AmountAscending,
    /// `AMOUNT_DESCENDING` — the GUI's "Amount-".
    AmountDescending,
    /// `MOD` — the GUI's "Mod".
    Mod,
    /// `TAG` — the GUI's "Tag"; blocked at this pin, see [`SortingMode::available`].
    Tag,
    /// `NAME` — the GUI's "Name", the default.
    Name,
}

impl SortingMode {
    /// The GUI choice's label (`SortingMode.description`), which `sort` takes.
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::AmountAscending => "Amount+",
            Self::AmountDescending => "Amount-",
            Self::Mod => "Mod",
            Self::Tag => "Tag",
            Self::Name => "Name",
        }
    }

    /// `StringRepresentable.getSerializedName()`: the value the saved compound
    /// holds under `sortMode`, the key upstream's `Value.createEnum` uses.
    #[must_use]
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::AmountAscending => "AMOUNT_ASCENDING",
            Self::AmountDescending => "AMOUNT_DESCENDING",
            Self::Mod => "MOD",
            Self::Tag => "TAG",
            Self::Name => "NAME",
        }
    }

    /// `SortingMode.byDescription`: the GUI's label back to the mode, matched
    /// exactly as the choice is written (`s.equals(mode.description)`).
    #[must_use]
    pub fn from_description(text: &str) -> Option<Self> {
        [
            Self::AmountAscending,
            Self::AmountDescending,
            Self::Mod,
            Self::Tag,
            Self::Name,
        ]
        .into_iter()
        .find(|mode| mode.description() == text)
    }

    /// A saved serialized name back to the mode. A missing or unknown value is
    /// the default (`StorageScannerData.DEFAULT.sortingMode` is `NAME`); a saved
    /// `TAG` also reads as `NAME`, because this pin cannot compute that order
    /// (see [`SortingMode::available`]).
    #[must_use]
    pub fn from_saved(saved: Option<&str>) -> Self {
        saved.and_then(Self::from_serialized).unwrap_or(Self::Name)
    }

    /// A serialized name back to the mode, for the three orders this pin can
    /// compute (`TAG` is deliberately absent, see [`SortingMode::available`]).
    fn from_serialized(name: &str) -> Option<Self> {
        [Self::AmountAscending, Self::AmountDescending, Self::Mod]
            .into_iter()
            .find(|mode| mode.serialized_name() == name)
    }

    /// Whether this order can be computed at this pin. `TAG` sorts by the
    /// "largest" tag an item is in (`GuiStorageScanner.findLargestTag`), which
    /// needs the set of tags an item belongs to. The SDK's `Tags` service
    /// (`horizon_mod_vocab::Tags`) answers only whether an item is in a **named**
    /// tag and lists a tag's members, with no way to list an item's tags or every
    /// tag, so `TAG` is `blocked(no item-tag enumeration)`.
    #[must_use]
    pub fn available(self) -> bool {
        !matches!(self, Self::Tag)
    }
}

/// Applies the GUI list's order to the merged counts (`GuiStorageScanner
/// .updateContentsList`). `Name` is the item id's order — the server-side
/// reading of the display name (slice 10's deviation 1); `Mod` groups the
/// namespace then the item id (upstream compares the namespace and breaks a tie
/// on the hover name); `Amount+`/`Amount-` sort by the merged count and are
/// **stable**, so equal counts keep the order the walk first met them (upstream
/// calls `List.sort`, which is stable and has no secondary key). `Tag` cannot be
/// computed at this pin (see [`SortingMode::available`]) and falls back to
/// `Name`.
#[must_use]
pub fn sort_found(mode: SortingMode, found: &[Found]) -> Vec<Found> {
    let mut sorted = found.to_vec();
    match mode {
        SortingMode::AmountAscending => sorted.sort_by_key(|entry| entry.count),
        SortingMode::AmountDescending => {
            sorted.sort_by_key(|entry| std::cmp::Reverse(entry.count));
        }
        SortingMode::Mod => sorted.sort_by(|a, b| {
            (search_namespace(&a.item), a.item.as_str())
                .cmp(&(search_namespace(&b.item), b.item.as_str()))
        }),
        SortingMode::Name | SortingMode::Tag => sorted.sort_by(|a, b| a.item.cmp(&b.item)),
    }
    sorted
}

// --- the scanner's item slots and its inject tick (slice 6) ----------------

/// `StorageScannerContainer.SLOT_IN`: the GUI's input slot, which a player fills
/// and the tick pushes into the scanned inventories.
pub const SCANNER_IN_SLOT: usize = 0;

/// `StorageScannerContainer.SLOT_OUT`: where a GUI request puts what it pulled.
/// The tick never writes it.
pub const SCANNER_OUT_SLOT: usize = 1;

/// `StorageScannerContainer.SLOT_IN_AUTO`: the automation input, which the GUI
/// does not show and a hopper reaches (the jar's capability is
/// `insertable(slot(SLOT_IN_AUTO))`).
pub const SCANNER_AUTO_SLOT: usize = 2;

/// `StorageScannerContainer.SLOTS`: the three slots above.
pub const SCANNER_SLOTS: usize = 3;

/// Which of the scanner's two input slots a tick's inject step drains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InjectSlot {
    /// `SLOT_IN`: the GUI's input, which a player fills.
    In,
    /// `SLOT_IN_AUTO`: the automation slot, which a hopper reaches.
    Auto,
}

impl InjectSlot {
    /// The container slot this step takes its stack from.
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::In => SCANNER_IN_SLOT,
            Self::Auto => SCANNER_AUTO_SLOT,
        }
    }
}

/// The slots one tick tries, in upstream's order and with its early stop:
/// `StorageScannerTileEntity.tickServer` takes the GUI's input first and the
/// automation input second, each one `rfPerInsert` attempt, and a buffer that
/// cannot pay for a step `return`s from the whole tick — so a poor buffer holds
/// the automation slot back too rather than borrowing from the next tick's price.
/// `stored` is the buffer at the start of the tick.
#[must_use]
pub fn inject_plan(stored: u32, in_filled: bool, auto_filled: bool) -> Vec<InjectSlot> {
    let mut plan = Vec::new();
    let mut budget = stored;
    for (slot, filled) in [(InjectSlot::In, in_filled), (InjectSlot::Auto, auto_filled)] {
        if !filled {
            continue;
        }
        if budget < SCANNER_RF_PER_INSERT {
            break;
        }
        budget -= SCANNER_RF_PER_INSERT;
        plan.push(slot);
    }
    plan
}

/// Whether the scanner's inject walk may push into a scanned cell: never its own
/// cell, and never another storage scanner — upstream's
/// `!p.equals(getBlockPos())` and `!(te instanceof StorageScannerTileEntity)`.
#[must_use]
pub fn injectable_at(centre: [i32; 3], at: [i32; 3], scanner_there: bool) -> bool {
    at != centre && !scanner_there
}

/// Whether `at` is in the scanner's saved routable set: upstream's
/// `isRoutable(p)`, the GUI's per-inventory star. A scanner that has never starred
/// anything has an **empty** set — `StorageScannerTileEntity` declares
/// `routable = new HashSet<>()` and nothing seeds it — so a fresh scanner routes
/// nothing until the player stars an inventory.
#[must_use]
pub fn is_routable(saved: &[[i32; 3]], at: [i32; 3]) -> bool {
    saved.contains(&at)
}

/// The routable set a star press leaves: `at` added (`on`) or removed, and
/// nothing else changed. Upstream's `toggleRoutable` is a set membership, so the
/// port's `on`/`off` are idempotent; the order of the other cells is kept.
#[must_use]
pub fn star(saved: &[[i32; 3]], at: [i32; 3], on: bool) -> Vec<[i32; 3]> {
    let mut next: Vec<[i32; 3]> = saved.iter().copied().filter(|cell| *cell != at).collect();
    if on {
        next.push(at);
    }
    next
}

/// Whether the inject/request/give/insert walk may use `at`: upstream's
/// `injectStackInternal` filter is `!p.equals(getBlockPos()) && isRoutable(p)`
/// (plus the access and matcher tests the glue applies), so an inventory the
/// player has not starred is skipped even when the scan found it.
#[must_use]
pub fn routing_target(centre: [i32; 3], at: [i32; 3], scanner_there: bool, starred: bool) -> bool {
    injectable_at(centre, at, scanner_there) && starred
}

/// The reply to `/rftoolsstorage routable <pos> <target> <on|off>`: the cell and
/// the state it now has, in the readout's voice.
#[must_use]
pub fn routable_line(at: [i32; 3], on: bool) -> String {
    let [x, y, z] = at;
    format!("routable {x} {y} {z} {}", if on { "on" } else { "off" })
}

// --- the scanner's export choice (slice 13) -------------------------------

/// The GUI's export choice: `GuiStorageScanner.exportToStarred`, the
/// `ImageChoiceLabel` whose two choices read "Export to current container" and
/// "Export to first routable container" and whose value is
/// `StorageScannerData.exportC` — the saved `exportc` flag, `false` by default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportMode {
    /// `exportC` true: a tick's GUI-slot stack goes to the inventory the player
    /// **selected** in the list (`getInventoryForBlock`'s `lastSelectedInventory`).
    Current,
    /// `exportC` false (`StorageScannerData.DEFAULT`): the stack goes to the
    /// first routable container — the starred walk every slice since 6 uses.
    First,
}

impl ExportMode {
    /// The command's word: `current` for `exportC` true, `first` for the walk.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::First => "first",
        }
    }

    /// The command's word back to the choice; anything else is refused.
    #[must_use]
    pub fn from_word(text: &str) -> Option<Self> {
        [Self::Current, Self::First]
            .into_iter()
            .find(|mode| mode.word() == text)
    }

    /// Upstream's saved flag (`StorageScannerData.exportC`, the `exportc` key).
    #[must_use]
    pub fn saved(self) -> bool {
        matches!(self, Self::Current)
    }

    /// The saved flag back to the choice: a block with no flag is the default.
    #[must_use]
    pub fn from_saved(flag: Option<bool>) -> Self {
        match flag {
            Some(true) => Self::Current,
            _ => Self::First,
        }
    }
}

/// Where one tick's GUI-slot attempt pushes its stack: upstream's
/// `injectStackInternal(stack, toSelected, …)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportRoute {
    /// The starred walk (`exportC` false, or no inventory selected).
    Walk,
    /// Only this inventory. Upstream **returns** from the selected attempt with
    /// whatever stayed, so a selected container that refuses leaves the stack in
    /// the slot: it never falls through to the walk.
    Only([i32; 3]),
}

/// The route one tick's GUI-slot stack takes: `Only(selected)` under the
/// current-container choice, `Walk` otherwise. Upstream's selected attempt does
/// not consult the star or the radius — the selection came from the list, which
/// shows every inventory in the box — so only the mode and the selection decide
/// here.
#[must_use]
pub fn export_route(mode: ExportMode, selected: Option<[i32; 3]>) -> ExportRoute {
    match (mode, selected) {
        (ExportMode::Current, Some(at)) => ExportRoute::Only(at),
        _ => ExportRoute::Walk,
    }
}

/// The reply to `/rftoolsstorage export <pos> <current|first>`.
#[must_use]
pub fn export_line(mode: ExportMode) -> String {
    format!("export {}", mode.word())
}

/// The reply to `/rftoolsstorage select <pos> <target>`: the cell the GUI's
/// list selected, which the current-container choice exports into.
#[must_use]
pub fn selection_line(at: [i32; 3]) -> String {
    let [x, y, z] = at;
    format!("select {x} {y} {z}")
}

// --- the list's row contents view (slice 14) ------------------------------

/// Whether `at` is inside the scanner's box on every axis (`pos - radius ..=
/// pos + radius`), upstream's `inRange` — the cube `findInventories` walks, not a
/// sphere.
#[must_use]
pub fn in_scan_box(centre: [i32; 3], radius: i32, at: [i32; 3]) -> bool {
    let radius = i64::from(radius.max(0));
    (0..3).all(|axis| (i64::from(at[axis]) - i64::from(centre[axis])).abs() <= radius)
}

/// Whether a cell can be one row of the scanner's list: inside the box, not the
/// scanner's own cell, not another storage scanner, and an inventory (its item
/// capability answers). Upstream's rows are the cells `findInventories` recorded,
/// which is exactly that test — a cell the scan did not find has no row, so
/// `getInventoryForBlock` cannot list it.
///
/// The star is deliberately absent: `getInventoryForBlock`'s single-row branch has
/// no `isRoutable` test, so an unstarred row still lists its own contents.
#[must_use]
pub fn contents_row(
    centre: [i32; 3],
    radius: i32,
    at: [i32; 3],
    scanner_there: bool,
    has_inventory: bool,
) -> bool {
    in_scan_box(centre, radius, at) && injectable_at(centre, at, scanner_there) && has_inventory
}

/// The reply to `/rftoolsstorage contents <pos> <target|all>`: the row's own kinds,
/// or the list's first "All routable" row (upstream's `(-1, -1, -1)` sentinel,
/// which lists every starred inventory), in the readout's voice. A row listing is
/// not a request: the line carries no buffer, because `getInventoryForBlock`
/// charges nothing.
#[must_use]
pub fn contents_line(row: Option<[i32; 3]>, found: &[Found]) -> String {
    let listed = found_list(found);
    match row {
        Some([x, y, z]) => format!("contents {x} {y} {z}: {listed}"),
        None => format!("contents all: {listed}"),
    }
}

/// The refusal a cell that is not a row gets: the GUI can only select an inventory
/// its list shows.
#[must_use]
pub fn contents_refused_line(at: [i32; 3]) -> String {
    let [x, y, z] = at;
    format!("{x} {y} {z} is not an inventory in the scan box")
}

// --- the scanner's request path (slice 7) ---------------------------------

/// The price of a request (`StorageScannerTileEntity.requestStack`): an explicit
/// count is a tenth of `rfPerRequest` ("Less RF usage for requesting less
/// items"), while a request with no count (upstream's `amount == -1`, a whole
/// stack) is the full price.
pub fn request_price(explicit: bool) -> u32 {
    if explicit {
        SCANNER_RF_PER_REQUEST / 10
    } else {
        SCANNER_RF_PER_REQUEST
    }
}

/// How many items a request wants: the item's own stack size when the command
/// gives no count (`amount == -1` becomes `getMaxStackSize()`), an explicit count
/// capped at that size (a request is at most one stack — the GUI offers only 1
/// and a whole stack), or none for a count of 0 (upstream pulls nothing and
/// charges nothing).
pub fn request_amount(explicit: Option<u32>, max_stack: u32) -> Option<u32> {
    let max = max_stack.clamp(1, SLOT_LIMIT);
    match explicit {
        Some(0) => None,
        Some(count) => Some(count.min(max)),
        None => Some(max),
    }
}

/// The out slot's answer to a request, before anything is pulled: upstream's
/// guard — an empty slot has a whole stack of room, a slot of the same kind has
/// its free room, and a slot of another kind or a full one refuses the request.
/// `None` is the refusal. `out` is `(same item and components, its count)`.
pub fn out_slot_room(out: Option<(bool, u32)>, max_stack: u32) -> Option<u32> {
    let max = max_stack.clamp(1, SLOT_LIMIT);
    match out {
        None => Some(max),
        Some((false, _)) => None,
        Some((true, count)) => (count < max).then(|| max - count),
    }
}

/// What a request did with the buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestCharge {
    /// The buffer is below the price: nothing is pulled and nothing is spent.
    NoPower,
    /// Nothing was pulled: a request charges only when something came out.
    Nothing,
    /// The buffer after a paid request.
    Paid(u32),
}

/// The request's charge rule: the buffer must cover the price before the walk
/// (`if (getStoredPower() < rf) return;`), and a walk that pulled nothing
/// charges nothing (`if (todo[0] == finalAmount) return;` before `consumeEnergy`).
pub fn request_charge(stored: u32, price: u32, pulled: u32) -> RequestCharge {
    if stored < price {
        return RequestCharge::NoPower;
    }
    if pulled == 0 {
        return RequestCharge::Nothing;
    }
    RequestCharge::Paid(stored - price)
}

/// `pulled <n> <item> (<after>/<max> FE)`.
pub fn request_line(pulled: u32, item: &str, after: u32) -> String {
    format!("pulled {pulled} {item} ({after}/{SCANNER_MAX_ENERGY} FE)")
}

/// The refusal when the buffer cannot pay the request's price, with the price.
pub fn request_no_power_line(stored: u32, price: u32, amount: u32, item: &str) -> String {
    format!(
        "not enough power for {amount} {item}: costs {price}, have {stored}/{SCANNER_MAX_ENERGY} FE"
    )
}

/// The refusal when the scanned inventories hold none of the requested kind.
pub fn request_nothing_line(stored: u32, item: &str) -> String {
    format!("nothing to pull for {item} ({stored}/{SCANNER_MAX_ENERGY} FE)")
}

/// The refusal when the out slot holds another kind (or is full).
pub fn request_out_slot_line(stored: u32, held: &str) -> String {
    format!("the out slot holds {held}; request refused ({stored}/{SCANNER_MAX_ENERGY} FE)")
}

// --- the scanner's give-from-screen path (slice 8) ------------------------

/// What a give hands over: the storage control screen module's click gives one item
/// while the player sneaks and a whole stack otherwise
/// (`GuiStorageScanner`-style `player.isShiftKeyDown()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GiveAmount {
    /// One item (`single`).
    Single,
    /// As many as the item stacks to.
    Stack,
}

/// What a give costs: `rfPerRequest`, the whole price, whether it hands over one item
/// or a stack. `giveToPlayerFromScreen` spends exactly one `rfPerRequest` after a give
/// that took something; the tenth-price mode belongs to `requestStack` alone.
pub fn give_price() -> u32 {
    SCANNER_RF_PER_REQUEST
}

/// How many items a give asks for: `cnt = single ? 1 : stack.getMaxStackSize()`,
/// clamped the way every request is (one stack, never past [`SLOT_LIMIT`]).
pub fn give_amount(mode: GiveAmount, max_stack: u32) -> u32 {
    match mode {
        GiveAmount::Single => 1,
        GiveAmount::Stack => max_stack.clamp(1, SLOT_LIMIT),
    }
}

/// `gave <n> <item> (<after>/<max> FE)`.
pub fn gave_line(given: u32, item: &str, after: u32) -> String {
    format!("gave {given} {item} ({after}/{SCANNER_MAX_ENERGY} FE)")
}

/// The refusal when the scanned inventories hold none of the given kind: upstream's
/// give says nothing at all, and a command must answer.
pub fn give_nothing_line(stored: u32, item: &str) -> String {
    format!("nothing to give for {item} ({stored}/{SCANNER_MAX_ENERGY} FE)")
}

// --- the scanner's insert-from-screen path (slice 9) ----------------------

/// What a screen insert costs: one `rfPerInsert`, and **only when the stack the
/// player handed over went in whole**. `injectStackFromScreen` charges after
/// `stack.isEmpty()`; a partial insert is free, its remainder staying with the
/// player.
pub fn insert_price() -> u32 {
    SCANNER_RF_PER_INSERT
}

/// How many items the screen insert asks for: the stack the player hands over, or
/// the command's explicit count capped at that stack. A count of 0 inserts
/// nothing, as a request of 0 pulls nothing.
pub fn insert_amount(explicit: Option<u32>, held: u32) -> Option<u32> {
    match explicit {
        Some(0) => None,
        Some(count) => Some(count.min(held)),
        None => Some(held),
    }
}

/// What an insert did with the buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertCharge {
    /// The buffer is below `rfPerInsert`: nothing leaves the player and nothing is
    /// spent.
    NoPower,
    /// The stack did not empty: free.
    Free(u32),
    /// The whole stack emptied: one `rfPerInsert`, the buffer after it.
    Paid(u32),
}

/// The insert's charge rule: the buffer must cover `rfPerInsert` before the walk,
/// and only a walk that emptied the stack spends it.
pub fn insert_charge(stored: u32, price: u32, emptied: bool) -> InsertCharge {
    if stored < price {
        return InsertCharge::NoPower;
    }
    if emptied {
        return InsertCharge::Paid(stored - price);
    }
    InsertCharge::Free(stored)
}

/// `inserted <n> <item> (<after>/<max> FE)`.
pub fn insert_line(inserted: u32, item: &str, after: u32) -> String {
    format!("inserted {inserted} {item} ({after}/{SCANNER_MAX_ENERGY} FE)")
}

/// The answer for a partial insert: what went in, of what was handed over, and
/// that the remainder is still the player's (and free).
pub fn insert_partial_line(inserted: u32, wanted: u32, item: &str, stored: u32) -> String {
    format!(
        "inserted {inserted} of {wanted} {item}; the rest stays with you, no charge ({stored}/{SCANNER_MAX_ENERGY} FE)"
    )
}

/// The refusal when the player holds none of the item the command names.
pub fn insert_nothing_line(stored: u32, item: &str) -> String {
    format!("nothing to insert for {item} ({stored}/{SCANNER_MAX_ENERGY} FE)")
}

/// The refusal when the scanner's box holds no routable inventory: upstream's
/// `checkForRoutableInventories` ("There are no routable inventories!").
pub fn no_routable_line(stored: u32) -> String {
    format!("there are no routable inventories ({stored}/{SCANNER_MAX_ENERGY} FE)")
}

// --- the crafting grid's pattern and craft-from-storage (slice 15) --------

/// The pattern's cells: upstream's nine ghost inputs
/// (`CraftingGridInventory.SLOT_GHOSTINPUT + 0..9`), row by row, cell 1 the top
/// left. The ghost output (`SLOT_GHOSTOUTPUT`) is not stored: it is what the
/// engine makes of the cells, computed when the pattern is read, as
/// `GuiCraftingGrid.testRecipe` computes it on the client.
pub const PATTERN_CELLS: usize = 9;

/// One of the GUI's four craft buttons (`GuiCraftingGrid`: `1`, `4`, `8` and
/// `*`, the last upstream's `craftItems(player, -1, …)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Batch {
    One,
    Four,
    Eight,
    /// The `*` button: a whole stack of results (upstream's `-1`).
    Stack,
}

impl Batch {
    /// The button the command's word names; `None` for anything else.
    #[must_use]
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "1" => Some(Self::One),
            "4" => Some(Self::Four),
            "8" => Some(Self::Eight),
            "stack" => Some(Self::Stack),
            _ => None,
        }
    }

    /// The count the button asks for; `None` is upstream's `-1`, the result's own
    /// stack size.
    #[must_use]
    pub fn wanted(self) -> Option<u32> {
        match self {
            Self::One => Some(1),
            Self::Four => Some(4),
            Self::Eight => Some(8),
            Self::Stack => None,
        }
    }

    /// The button's face, as the command takes it.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Four => "4",
            Self::Eight => "8",
            Self::Stack => "stack",
        }
    }
}

/// How many times one craft runs (`StorageCraftingTools.craftItems`): `wanted` is
/// the button's count (`None` the whole result stack), `result_count` what one set
/// makes and `max_stack` the result item's own stack size. Upstream rounds up to a
/// whole set and then drops one when the product would not fit one stack.
pub fn craft_sets(wanted: Option<u32>, result_count: u32, max_stack: u32) -> u32 {
    if result_count == 0 || max_stack == 0 {
        return 0;
    }
    let mut sets = wanted.unwrap_or(max_stack);
    let remainder = sets % result_count;
    sets /= result_count;
    if remainder != 0 {
        sets += 1;
    }
    if sets.saturating_mul(result_count) > max_stack {
        sets = sets.saturating_sub(1);
    }
    sets
}

/// What one pattern cell asks for: the cell (1..=9) and how many of its kind the
/// batch needs (one a set, because a crafting grid's cell is one item a craft).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Need {
    pub cell: usize,
    pub kind: Kind,
    pub count: u32,
}

/// One stack a craft may take from: a slot of a source. Upstream's
/// `InventoriesItemSource` walks the player's inventory first and the storage
/// second, and inside a source in slot order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    pub slot: usize,
    pub kind: Kind,
    pub count: u32,
}

/// One take: which source and slot, and how many.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draw {
    pub source: usize,
    pub slot: usize,
    pub count: u32,
}

/// A cell the sources cannot fill, and by how much (`tryRecipe`'s answer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Short {
    pub cell: usize,
    pub kind: Kind,
    pub short: u32,
}

/// The plan a batch's needs make over the sources: either every take, or the
/// cells that are short — and then nothing is taken at all, as upstream undoes
/// the walk when an ingredient is missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CraftPlan {
    Ready(Vec<Draw>),
    Missing(Vec<Short>),
}

/// The batch's asks: every filled cell asks for `sets` of its kind.
#[must_use]
pub fn pattern_needs(cells: &[Option<Kind>], sets: u32) -> Vec<Need> {
    cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            cell.as_ref().map(|kind| Need {
                cell: index + 1,
                kind: kind.clone(),
                count: sets,
            })
        })
        .collect()
}

/// Where the batch's items come from: the sources in upstream's order, each slot
/// giving what it has. A short cell fails the whole plan, so a craft never takes
/// half a set.
#[must_use]
pub fn craft_plan(needs: &[Need], sources: &[Vec<Held>]) -> CraftPlan {
    // What each slot still holds, so two cells of one kind cannot spend the same
    // stack twice (upstream's `findMatchingItems` extracts as it walks).
    let mut available: Vec<Vec<u32>> = sources
        .iter()
        .map(|source| source.iter().map(|held| held.count).collect())
        .collect();
    let mut draws: Vec<Draw> = Vec::new();
    let mut shorts = Vec::new();
    for need in needs {
        let mut remaining = need.count;
        for (index, source) in sources.iter().enumerate() {
            if remaining == 0 {
                break;
            }
            for (at, held) in source.iter().enumerate() {
                if remaining == 0 {
                    break;
                }
                if held.kind.key != need.kind.key {
                    continue;
                }
                let left = available[index][at];
                if left == 0 {
                    continue;
                }
                let take = remaining.min(left);
                available[index][at] = left - take;
                remaining -= take;
                match draws.last_mut() {
                    Some(last) if last.source == index && last.slot == held.slot => {
                        last.count += take;
                    }
                    _ => draws.push(Draw {
                        source: index,
                        slot: held.slot,
                        count: take,
                    }),
                }
            }
        }
        if remaining > 0 {
            shorts.push(Short {
                cell: need.cell,
                kind: need.kind.clone(),
                short: remaining,
            });
        }
    }
    if shorts.is_empty() {
        CraftPlan::Ready(draws)
    } else {
        CraftPlan::Missing(shorts)
    }
}

/// The `/rftoolsstorage grid <pos>` readback: the cells that hold something, then
/// what the engine makes of them (upstream's ghost output cell, computed instead
/// of stored).
#[must_use]
pub fn pattern_line(cells: &[Option<Kind>], result: Option<(&Kind, u32)>) -> String {
    let filled: Vec<String> = cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            cell.as_ref()
                .map(|kind| format!("{}={}", index + 1, kind.label))
        })
        .collect();
    if filled.is_empty() {
        return "pattern empty".to_owned();
    }
    let cells = filled.join(", ");
    match result {
        Some((kind, count)) => format!("pattern {cells} -> {} x{count}", kind.label),
        None => format!("pattern {cells} -> no recipe"),
    }
}

/// The `/rftoolsstorage craft` answer for a batch that was made.
#[must_use]
pub fn crafted_line(count: u32, kind: &Kind) -> String {
    format!("crafted {count} {}", kind.label)
}

/// What a batch could not find (`tryRecipe`'s per-cell answer), as one line.
#[must_use]
pub fn missing_line(shorts: &[Short]) -> String {
    let listed: Vec<String> = shorts
        .iter()
        .map(|short| format!("cell {}: {} {}", short.cell, short.short, short.kind.label))
        .collect();
    format!("missing {}", listed.join(", "))
}

/// A pattern no recipe matches (`getCachedRecipe` empty: the jar's GUI then shows
/// nothing, and nothing can be crafted).
#[must_use]
pub fn no_recipe_line() -> &'static str {
    "the pattern crafts nothing"
}

/// A pattern with no cell filled.
#[must_use]
pub fn empty_pattern_line() -> &'static str {
    "the pattern is empty"
}

/// A batch the game refused to complete (a write or a give the world turned down):
/// every take is rolled back, so nothing left the sources.
#[must_use]
pub fn craft_failed_line() -> &'static str {
    "the craft could not be completed; nothing was taken"
}

/// The cells are upstream's ghost ones: they are only editable on a locked storage
/// (`GhostSlot.mayPlace`/`mayPickup` refuse while unlocked).
#[must_use]
pub fn pattern_locked_line() -> &'static str {
    "lock the storage to change the pattern"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A kind whose identity and label are the same item id.
    fn plain(item: &str) -> Kind {
        Kind {
            key: item.to_owned(),
            label: item.to_owned(),
        }
    }

    fn stone() -> Kind {
        plain("minecraft:stone")
    }

    fn dirt() -> Kind {
        plain("minecraft:dirt")
    }

    fn locked(tier: u8) -> Storage {
        let mut storage = Storage::with_tier(tier);
        assert!(storage.toggle_lock(), "a module lets the storage lock");
        storage
    }

    #[test]
    fn the_tier_table_is_the_jars_maxsize() {
        // StorageModuleItem.MAXSIZE = {100, 200, 300, 500, 0, 0, -1}.
        assert_eq!(TIER_SLOTS, [100, 200, 300, 500]);
        assert_eq!(module_tier("rftoolsstorage:storage_module0"), Some(0));
        assert_eq!(module_tier("rftoolsstorage:storage_module3"), Some(3));
        assert_eq!(module_tier("rftoolsstorage:storage_module6"), None);
        assert_eq!(module_tier("minecraft:chest"), None);
    }

    #[test]
    fn a_tier_one_storage_holds_six_thousand_four_hundred_stone() {
        // 100 slots x min(64, the item's stack size).
        assert_eq!(capacity(100, 64), 6_400);
        assert_eq!(capacity(100, 16), 1_600);
        assert_eq!(capacity(100, 1), 100);
        assert_eq!(capacity(0, 64), 0);
    }

    #[test]
    fn a_fresh_storage_has_no_module_and_stores_nothing() {
        let mut storage = Storage::empty();
        assert!(!storage.accepts());
        assert_eq!(storage.slot_count(), 0);
        assert_eq!(storage.insert(&stone(), 64, 64), 0);
        assert_eq!(storage.total(), 0);
        assert_eq!(storage.state_word(), "no module");
    }

    #[test]
    fn a_module_goes_in_only_while_unlocked_and_only_once() {
        let mut storage = Storage::empty();
        assert!(!storage.toggle_lock(), "no module, no lock");
        assert!(storage.install(0));
        assert_eq!(storage.slot_count(), 100);
        assert_eq!(storage.state_word(), "unlocked");
        assert!(!storage.accepts(), "an unlocked storage is inert");
        assert!(!storage.install(1), "a module is already in place");
        assert!(storage.toggle_lock());
        assert_eq!(storage.state_word(), "locked");
        assert!(storage.accepts());
        assert!(!storage.install(1), "a locked storage takes no module");
        assert!(
            card_accepts(true, "rftoolsstorage:storage_module0"),
            "a card slot holding the module does not block a click elsewhere"
        );
    }

    #[test]
    fn an_unlocked_storage_refuses_insert_and_extract() {
        let mut storage = Storage::with_tier(0);
        assert_eq!(storage.insert(&stone(), 64, 64), 0);
        assert_eq!(storage.extract_first(64), None);
        assert!(storage.toggle_lock());
        assert_eq!(storage.insert(&stone(), 64, 64), 64);
        assert_eq!(storage.extract_first(64).map(|taken| taken.count), Some(64));
    }

    #[test]
    fn a_full_slot_opens_the_next_one_and_one_kind_stays_together() {
        let mut storage = locked(0);
        assert_eq!(storage.insert(&stone(), 64, 64), 64);
        assert_eq!(storage.insert(&stone(), 64, 64), 64);
        assert_eq!(storage.total(), 128);
        assert_eq!(storage.stacks_used(), 2, "two slots of one kind");
        assert_eq!(storage.slots[0].as_ref().map(|s| s.count), Some(64));
        assert_eq!(storage.slots[1].as_ref().map(|s| s.count), Some(64));
        assert_eq!(storage.insert(&dirt(), 64, 64), 64);
        assert_eq!(storage.stacks_used(), 3, "dirt takes the next slot");
        assert_eq!(storage.insert(&stone(), 64, 64), 64, "stone tops up");
        assert_eq!(
            storage.slots[3].as_ref().map(|s| s.kind.label.as_str()),
            Some("minecraft:stone")
        );
        assert_eq!(
            storage.slots[2].as_ref().map(|s| s.kind.label.as_str()),
            Some("minecraft:dirt")
        );
    }

    #[test]
    fn a_slot_never_holds_more_than_the_items_own_stack_size() {
        let mut storage = locked(0);
        assert_eq!(storage.insert(&stone(), 64, 16), 64);
        assert_eq!(storage.stacks_used(), 4, "64 of a 16-stack item: 4 slots");
        assert_eq!(storage.slots[3].as_ref().map(|s| s.count), Some(16));
        assert_eq!(storage.insert(&stone(), 1, 16), 1);
        assert_eq!(storage.stacks_used(), 5);
    }

    #[test]
    fn two_kinds_with_the_same_item_id_but_other_components_stay_apart() {
        let mut storage = locked(0);
        let plain = plain("minecraft:diamond_sword");
        let named = Kind {
            key: "minecraft:diamond_sword|0|Some(compound)".to_owned(),
            label: "minecraft:diamond_sword".to_owned(),
        };
        assert_eq!(storage.insert(&plain, 1, 1), 1);
        assert_eq!(storage.insert(&named, 1, 1), 1);
        assert_eq!(storage.stacks_used(), 2, "different components, two slots");
        assert_eq!(
            storage.insert(&plain, 1, 1),
            1,
            "the same kind opens another slot"
        );
        assert_eq!(storage.stacks_used(), 3);
    }

    #[test]
    fn the_store_is_full_at_the_tier_capacity_and_refuses_more() {
        let mut storage = locked(0);
        for _ in 0..100 {
            assert_eq!(storage.insert(&stone(), 64, 64), 64);
        }
        assert_eq!(storage.total(), 6_400);
        assert_eq!(storage.capacity(64), 6_400);
        assert!(storage.is_full_for(&stone(), 64));
        assert_eq!(
            storage.insert(&stone(), 64, 64),
            0,
            "a full block refuses more"
        );
        assert!(
            storage.is_full_for(&dirt(), 64),
            "no empty slot for a new kind"
        );
        assert_eq!(storage.stacks_used(), 100);
    }

    #[test]
    fn a_part_full_store_takes_the_remainder_and_reports_it() {
        let mut storage = locked(0);
        for _ in 0..99 {
            storage.insert(&stone(), 64, 64);
        }
        storage.insert(&stone(), 32, 64);
        assert_eq!(storage.total(), 6_368);
        assert!(!storage.is_full_for(&stone(), 64));
        assert_eq!(storage.insert(&stone(), 64, 64), 32, "only 32 fit");
        assert_eq!(storage.total(), 6_400);
        assert_eq!(storage.insert(&stone(), 64, 64), 0);
    }

    #[test]
    fn extract_takes_a_stack_from_the_first_slot_in_order() {
        let mut storage = locked(0);
        storage.insert(&stone(), 64, 64);
        storage.insert(&dirt(), 32, 64);
        assert_eq!(
            storage.extract_first(64),
            Some(Taken {
                slot: 0,
                kind: stone(),
                count: 64
            })
        );
        assert_eq!(storage.stacks_used(), 1, "the emptied slot is free again");
        assert_eq!(
            storage.extract_first(64),
            Some(Taken {
                slot: 1,
                kind: dirt(),
                count: 32
            })
        );
        assert_eq!(storage.extract_first(64), None);
        assert_eq!(storage.total(), 0);
    }

    #[test]
    fn the_readout_names_the_amount_the_capacity_and_the_item() {
        let mut storage = locked(0);
        assert_eq!(storage.summary(64), "0/6400 empty");
        storage.insert(&stone(), 64, 64);
        assert_eq!(storage.summary(64), "64/6400 minecraft:stone");
        assert_eq!(storage.summary(16), "64/1600 minecraft:stone");
    }

    #[test]
    fn a_bigger_module_keeps_the_contents_and_a_smaller_one_truncates() {
        let mut storage = locked(1);
        assert_eq!(storage.slot_count(), 200);
        for _ in 0..150 {
            storage.insert(&stone(), 64, 64);
        }
        assert_eq!(storage.stacks_used(), 150);
        assert!(storage.toggle_lock());
        assert_eq!(storage.detach_module(), Some(1));
        assert_eq!(storage.slot_count(), 0, "no module, no capacity");
        assert_eq!(storage.total(), 0);
        // The contents are the block's, so a smaller module finds what fits.
        assert!(storage.install(0));
        assert_eq!(storage.slot_count(), 100);
        assert!(storage.toggle_lock());
        for _ in 0..100 {
            storage.insert(&stone(), 64, 64);
        }
        assert_eq!(storage.total(), 6_400);
    }

    #[test]
    fn the_padlock_needs_a_module_and_then_flips_the_lock() {
        let mut empty = Storage::empty();
        assert_eq!(lock_press(&mut empty), LockPress::NoModule);
        assert!(!empty.locked, "a module-less press changes nothing");
        let mut storage = Storage::with_tier(0);
        assert_eq!(lock_press(&mut storage), LockPress::Locked);
        assert!(storage.locked);
        assert!(storage.accepts(), "a locked storage with a module stores");
        assert_eq!(lock_press(&mut storage), LockPress::Unlocked);
        assert!(!storage.locked);
        assert!(!storage.accepts());
    }

    #[test]
    fn a_readout_or_lock_needs_an_operator_reach_or_the_same_level() {
        assert!(may_inspect(true, false, f64::MAX), "an operator");
        assert!(may_inspect(false, true, 5.0), "a player in reach");
        assert!(!may_inspect(false, true, 8.5), "out of reach");
        assert!(!may_inspect(false, false, 1.0), "another level");
    }

    #[test]
    fn a_carried_slot_comes_back_clamped_and_only_where_the_block_has_one() {
        // A break moves the container's stacks aside and the item carries them;
        // placing it back writes them into the block's own slots, the card slot
        // included, clamped the way the engine clamps any write (slice 4).
        assert!(carried_slot_in(100, 100, 0));
        assert!(carried_slot_in(100, 100, 99));
        assert!(
            carried_slot_in(100, 100, 100),
            "the card slot's module travels with the contents"
        );
        assert!(
            !carried_slot_in(100, 100, 101),
            "nothing past the container comes back"
        );
        assert_eq!(restored_count(64, 64), 64);
        assert_eq!(
            restored_count(200, 64),
            64,
            "a doctored count cannot overflow a slot"
        );
        assert_eq!(
            restored_count(64, 16),
            16,
            "a slot holds at most the item's own stack size"
        );
        assert_eq!(restored_count(200, 1), 1, "a module stays one");
        assert_eq!(
            restored_count(1, 0),
            1,
            "a nonsense stack size still keeps one"
        );
    }

    #[test]
    fn the_grid_takes_a_stack_only_when_locked_and_in_a_slot_of_its_own() {
        // Upstream's `Slot.mayPlace` on the storage grid: `!isLocked()` first,
        // then the handler's `isItemValid` (empty, or the same item and components).
        assert!(
            slot_accepts(true, None, "minecraft:stone"),
            "a locked, empty slot takes a stack"
        );
        assert!(
            !slot_accepts(false, None, "minecraft:stone"),
            "an unlocked storage stores nothing, whatever the slot holds"
        );
        assert!(
            !slot_accepts(true, Some("minecraft:dirt"), "minecraft:stone"),
            "one kind a slot: a slot holding dirt takes no stone"
        );
        assert!(
            slot_accepts(true, Some("minecraft:stone"), "minecraft:stone"),
            "a slot tops up its own kind"
        );
    }

    #[test]
    fn the_card_slot_takes_a_module_and_never_blocks_another_slot() {
        // Upstream's card-slot `mayPlace`: a StorageModuleItem, while unlocked. The
        // occupied case answers true so the engine's plan for another slot's click
        // is not blocked by it (`notes.md`).
        assert!(card_accepts(false, "rftoolsstorage:storage_module0"));
        assert!(
            !card_accepts(false, "minecraft:stone"),
            "no stack in the card slot"
        );
        assert!(
            !card_accepts(false, "rftoolsstorage:dump_module"),
            "a screen module is not a storage module"
        );
        assert!(
            card_accepts(true, "minecraft:stone"),
            "an occupied card slot answers yes to anything"
        );
    }

    #[test]
    fn a_module_id_round_trips_through_its_tier() {
        assert_eq!(module_id(0), "rftoolsstorage:storage_module0");
        for tier in 0..=3 {
            assert_eq!(module_tier(&module_id(tier)), Some(tier), "tier {tier}");
        }
    }

    #[test]
    fn a_removed_module_hides_the_contents_and_its_return_finds_them() {
        let mut storage = locked(0);
        assert_eq!(storage.insert(&stone(), 64, 64), 64);
        assert!(storage.toggle_lock());
        assert_eq!(storage.detach_module(), Some(0));
        assert!(storage.tier.is_none());
        assert!(!storage.locked, "a storage with no module is never locked");
        assert!(!storage.accepts());
        assert_eq!(storage.slot_count(), 0, "no module, no capacity");
        assert_eq!(storage.total(), 0, "the contents wait for a module");
        assert_eq!(storage.summary(64), "0/0 empty");
        assert_eq!(
            storage.insert(&dirt(), 64, 64),
            0,
            "no module stores nothing"
        );
        assert!(storage.install(0), "no lock stops the module coming back");
        assert_eq!(storage.slot_count(), 100);
        assert!(storage.toggle_lock());
        assert_eq!(storage.total(), 64, "the module finds the contents it left");
        assert_eq!(storage.summary(64), "64/6400 minecraft:stone");
    }

    #[test]
    fn a_moduleless_storage_reads_as_empty_whatever_the_block_holds() {
        let contents = vec![
            Some(Slot {
                kind: stone(),
                count: 64,
            }),
            None,
        ];
        let storage = Storage::from_block(None, false, contents);
        assert_eq!(storage.slots.len(), 2, "the block still holds them");
        assert_eq!(storage.slot_count(), 0);
        assert_eq!(storage.capacity(64), 0);
        assert_eq!(storage.total(), 0);
        assert_eq!(storage.stacks_used(), 0);
        assert_eq!(storage.summary(64), "0/0 empty");
        assert_eq!(storage.state_word(), "no module");
    }

    #[test]
    fn the_scanner_scans_a_cube_of_its_radius_and_nothing_outside_it() {
        // Upstream's `inRange`/`findInventories`: `pos - radius ..= pos + radius`
        // on every axis, a cube, not a sphere.
        let centre = [10, 64, -3];
        let near = scan_positions(centre, 1);
        assert_eq!(near.len(), 27, "a 3 x 3 x 3 box at the default radius");
        assert!(
            near.contains(&centre),
            "the scanner's own cell is in the walk"
        );
        assert!(near.contains(&[11, 65, -2]), "a corner of the box is in");
        assert!(
            !near.contains(&[12, 64, -3]),
            "two blocks away is outside radius 1"
        );
        let wider = scan_positions(centre, 2);
        assert_eq!(wider.len(), 125, "a 5 x 5 x 5 box");
        assert!(wider.contains(&[12, 66, -1]), "the box is a cube");
        assert!(
            !wider.contains(&[13, 64, -3]),
            "three away is outside radius 2"
        );
    }

    #[test]
    fn the_scanners_radius_is_the_gui_sliders_one_to_twenty() {
        // `StorageScannerData.DEFAULT.radius` = 1; the GUI's slider is 1..20
        // (0 only with XNet, which this pin does not have).
        assert_eq!(SCANNER_DEFAULT_RADIUS, 1);
        assert_eq!(SCANNER_MIN_RADIUS, 1);
        assert_eq!(SCANNER_MAX_RADIUS, 20);
        assert_eq!(clamp_radius(0), 1, "below the slider's minimum");
        assert_eq!(clamp_radius(-3), 1);
        assert_eq!(clamp_radius(1), 1);
        assert_eq!(clamp_radius(20), 20);
        assert_eq!(clamp_radius(21), 20, "past the slider's maximum");
        assert_eq!(
            scanner_radius(None),
            SCANNER_DEFAULT_RADIUS,
            "an unscanned block scans its neighbours"
        );
        assert_eq!(scanner_radius(Some(3)), 3);
        assert_eq!(scanner_radius(Some(99)), 20);
    }

    #[test]
    fn a_scan_costs_the_request_price_and_refuses_below_it() {
        // The jar's config defaults: scannerMaxRF 50,000, scannerRFPerTick 500,
        // rfPerRequest 100, rfPerInsert 20.
        assert_eq!(SCANNER_MAX_ENERGY, 50_000);
        assert_eq!(SCANNER_RECEIVE_PER_TICK, 500);
        assert_eq!(SCANNER_RF_PER_REQUEST, 100);
        assert_eq!(SCANNER_RF_PER_INSERT, 20);
        assert_eq!(spend_power(0), None, "an empty buffer scans nothing");
        assert_eq!(spend_power(99), None, "one short is still no scan");
        assert_eq!(spend_power(100), Some(0), "the last 100 buy one scan");
        assert_eq!(spend_power(5_000), Some(4_900), "a scan takes its price");
    }

    #[test]
    fn the_scan_counts_each_kind_once_across_the_inventories_it_found() {
        let merged = merge_counts(&[
            ("minecraft:stone".to_owned(), 64),
            ("minecraft:dirt".to_owned(), 3),
            ("minecraft:stone".to_owned(), 32),
        ]);
        assert_eq!(merged.len(), 2, "one entry a kind");
        assert_eq!(
            merged[0].item, "minecraft:stone",
            "the walk's first-seen order is the base the amount sort's stability keeps"
        );
        assert_eq!(merged[0].count, 96, "64 + 32, the whole count");
        assert_eq!(merged[1].item, "minecraft:dirt");
        let found = sort_found(SortingMode::Name, &merged);
        assert_eq!(
            found[0].item, "minecraft:dirt",
            "the default NAME order is the item id's"
        );
        assert_eq!(found[1].item, "minecraft:stone");
        assert!(
            merge_counts(&[]).is_empty(),
            "nothing found, nothing listed"
        );
    }

    #[test]
    fn the_scan_reply_names_the_items_and_the_energy_left_or_the_missing_power() {
        let found = merge_counts(&[("minecraft:stone".to_owned(), 64)]);
        assert_eq!(
            scan_line(4_900, &found),
            "minecraft:stone 64 (4900/50000 FE)"
        );
        assert_eq!(
            scan_line(4_900, &[]),
            "empty (4900/50000 FE)",
            "an empty scan still says what it cost"
        );
        assert_eq!(no_power_line(99), "no power (99/50000 FE)");
    }

    #[test]
    fn a_tick_drains_the_gui_slot_first_and_the_automation_slot_second_at_a_price_each() {
        // `StorageScannerTileEntity.tickServer`: the GUI input first, then the auto
        // input; each non-empty slot is one `rfPerInsert` attempt, and a buffer that
        // cannot pay for a step returns from the whole tick.
        assert_eq!(SCANNER_SLOTS, 3);
        assert_eq!(
            (SCANNER_IN_SLOT, SCANNER_OUT_SLOT, SCANNER_AUTO_SLOT),
            (0, 1, 2),
            "the jar's SLOT_IN, SLOT_OUT and SLOT_IN_AUTO"
        );
        assert_eq!(InjectSlot::In.index(), SCANNER_IN_SLOT);
        assert_eq!(InjectSlot::Auto.index(), SCANNER_AUTO_SLOT);
        assert_eq!(SCANNER_RF_PER_INSERT, 20, "one attempt costs rfPerInsert");
        assert_eq!(
            inject_plan(5_000, false, false),
            Vec::new(),
            "nothing to route costs nothing"
        );
        assert_eq!(
            inject_plan(5_000, true, false),
            vec![InjectSlot::In],
            "the GUI slot is the tick's first step"
        );
        assert_eq!(
            inject_plan(5_000, false, true),
            vec![InjectSlot::Auto],
            "with the GUI slot empty the auto slot gets the first price"
        );
        assert_eq!(
            inject_plan(40, true, true),
            vec![InjectSlot::In, InjectSlot::Auto],
            "two filled slots are two attempts in one tick"
        );
        assert_eq!(
            inject_plan(20, true, true),
            vec![InjectSlot::In],
            "one price buys one attempt: the auto slot waits for the next tick"
        );
        assert_eq!(
            inject_plan(19, true, true),
            Vec::new(),
            "one short is no attempt at all"
        );
        assert_eq!(
            inject_plan(0, true, false),
            Vec::new(),
            "an empty buffer routes nothing"
        );
        assert_eq!(
            inject_plan(19, false, true),
            Vec::new(),
            "a buffer that cannot pay stops the tick before the auto slot"
        );
    }

    #[test]
    fn the_inject_walk_skips_its_own_cell_and_another_scanner() {
        // Upstream's `injectStackInternal` filters `!p.equals(getBlockPos())`, and its
        // `inventories` list holds no `StorageScannerTileEntity`.
        let centre = [10, 64, -3];
        assert!(
            !injectable_at(centre, centre, false),
            "the scanner never pushes into its own container"
        );
        assert!(
            injectable_at(centre, [12, 64, -3], false),
            "a chest two away is a target"
        );
        assert!(
            !injectable_at(centre, [11, 64, -3], true),
            "another storage scanner is skipped"
        );
        assert!(
            injectable_at(centre, [11, 64, -3], false),
            "the same cell is a target once no scanner stands there"
        );
    }

    #[test]
    fn a_request_with_a_count_costs_a_tenth_and_the_default_asks_a_whole_stack() {
        // `requestStack`: `rf = rfPerRequest; if (amount >= 0) rf /= 10; if (amount
        // == -1) amount = requested.getMaxStackSize();`
        assert_eq!(request_price(true), 10, "an explicit count is cheaper");
        assert_eq!(
            request_price(false),
            100,
            "a whole-stack request is the full price"
        );
        assert_eq!(
            request_amount(None, 64),
            Some(64),
            "no count asks a whole stack"
        );
        assert_eq!(request_amount(Some(32), 64), Some(32));
        assert_eq!(
            request_amount(Some(0), 64),
            None,
            "a count of 0 pulls nothing"
        );
        assert_eq!(
            request_amount(Some(99), 64),
            Some(64),
            "a request is at most one stack"
        );
        assert_eq!(
            request_amount(None, 16),
            Some(16),
            "an item's own stack size caps it"
        );
    }

    #[test]
    fn a_request_refuses_an_out_slot_of_another_kind_or_a_full_one() {
        // Upstream's out-slot guard runs before the walk: `!isSameItemSameComponents`
        // and `getCount() >= getMaxStackSize()` both `return`.
        assert_eq!(
            out_slot_room(None, 64),
            Some(64),
            "an empty out slot has a stack of room"
        );
        assert_eq!(
            out_slot_room(Some((true, 32)), 64),
            Some(32),
            "the same kind leaves its free room"
        );
        assert_eq!(
            out_slot_room(Some((true, 64)), 64),
            None,
            "a full out slot refuses"
        );
        assert_eq!(
            out_slot_room(Some((false, 1)), 64),
            None,
            "another kind refuses the request"
        );
    }

    #[test]
    fn a_request_charges_only_what_it_pulled_and_only_with_the_power_to_pay() {
        assert_eq!(request_charge(5_000, 10, 32), RequestCharge::Paid(4_990));
        assert_eq!(
            request_charge(9, 10, 32),
            RequestCharge::NoPower,
            "one short of the price is no request"
        );
        assert_eq!(
            request_charge(5_000, 10, 0),
            RequestCharge::Nothing,
            "nothing pulled, nothing charged"
        );
        assert_eq!(
            request_charge(10, 10, 1),
            RequestCharge::Paid(0),
            "the last ten buy a request"
        );
        assert_eq!(
            request_line(32, "minecraft:stone", 4_990),
            "pulled 32 minecraft:stone (4990/50000 FE)"
        );
        assert_eq!(
            request_no_power_line(9, 10, 32, "minecraft:stone"),
            "not enough power for 32 minecraft:stone: costs 10, have 9/50000 FE"
        );
        assert_eq!(
            request_nothing_line(5_000, "minecraft:dirt"),
            "nothing to pull for minecraft:dirt (5000/50000 FE)"
        );
        assert_eq!(
            request_out_slot_line(5_000, "minecraft:dirt"),
            "the out slot holds minecraft:dirt; request refused (5000/50000 FE)"
        );
    }

    #[test]
    fn a_give_is_the_full_price_whether_it_hands_over_one_item_or_a_stack() {
        // `giveToPlayerFromScreen`: one `consumeEnergy(rfPerRequest)` after a give that
        // took something, whatever `single` is; `cnt = {single ? 1 : the stack size}`.
        assert_eq!(give_price(), 100, "a give is rfPerRequest");
        assert_eq!(
            give_price(),
            request_price(false),
            "a give costs what a whole-stack request costs"
        );
        assert_ne!(
            give_price(),
            request_price(true),
            "a give has no tenth-price mode"
        );
        assert_eq!(give_amount(GiveAmount::Single, 64), 1, "single is one item");
        assert_eq!(give_amount(GiveAmount::Stack, 64), 64, "a whole stack");
        assert_eq!(
            give_amount(GiveAmount::Stack, 16),
            16,
            "the item's own stack size"
        );
        assert_eq!(
            give_amount(GiveAmount::Stack, 999),
            64,
            "never past SLOT_LIMIT"
        );
        assert_eq!(
            request_charge(5_000, give_price(), 64),
            RequestCharge::Paid(4_900),
            "a give of 64 off 5,000 leaves 4,900"
        );
        assert_eq!(
            request_charge(99, give_price(), 64),
            RequestCharge::NoPower,
            "the price is checked before anything is pulled"
        );
    }

    #[test]
    fn the_give_reply_names_what_was_given_or_refuses_an_empty_walk() {
        assert_eq!(
            gave_line(64, "minecraft:stone", 4_900),
            "gave 64 minecraft:stone (4900/50000 FE)"
        );
        assert_eq!(
            give_nothing_line(4_800, "minecraft:dirt"),
            "nothing to give for minecraft:dirt (4800/50000 FE)"
        );
    }

    #[test]
    fn a_screen_insert_costs_one_rf_per_insert_and_only_when_the_stack_emptied() {
        // `injectStackFromScreen`: `if (getStoredPower() < rfPerInsert) return stack;`
        // then `stack = injectStackInternal(...); if (stack.isEmpty())
        // consumeEnergy(rfPerInsert);` — a partial insert spends nothing.
        assert_eq!(insert_price(), 20, "an insert is rfPerInsert");
        assert_eq!(
            insert_charge(5_000, insert_price(), true),
            InsertCharge::Paid(4_980),
            "the whole stack emptied -> one rfPerInsert"
        );
        assert_eq!(
            insert_charge(5_000, insert_price(), false),
            InsertCharge::Free(5_000),
            "a partial insert is free"
        );
        assert_eq!(
            insert_charge(19, insert_price(), true),
            InsertCharge::NoPower,
            "19 FE cannot pay one insert, and the check is before the walk"
        );
        assert_eq!(
            insert_charge(20, insert_price(), true),
            InsertCharge::Paid(0),
            "20 FE pays exactly one insert"
        );
        assert_eq!(
            insert_amount(None, 64),
            Some(64),
            "no count inserts the stack the player hands over"
        );
        assert_eq!(
            insert_amount(Some(10), 64),
            Some(10),
            "an explicit count inserts that many"
        );
        assert_eq!(
            insert_amount(Some(200), 64),
            Some(64),
            "never more than the stack the player holds"
        );
        assert_eq!(
            insert_amount(Some(0), 64),
            None,
            "a count of 0 inserts nothing"
        );
    }

    #[test]
    fn the_insert_reply_names_what_went_in_and_what_did_not() {
        assert_eq!(
            insert_line(64, "minecraft:stone", 4_980),
            "inserted 64 minecraft:stone (4980/50000 FE)"
        );
        assert_eq!(
            insert_partial_line(0, 64, "minecraft:stone", 5_000),
            "inserted 0 of 64 minecraft:stone; the rest stays with you, no charge (5000/50000 FE)"
        );
        assert_eq!(
            insert_nothing_line(5_000, "minecraft:stone"),
            "nothing to insert for minecraft:stone (5000/50000 FE)"
        );
        assert_eq!(
            no_routable_line(5_000),
            "there are no routable inventories (5000/50000 FE)"
        );
    }

    #[test]
    fn a_search_word_matches_the_name_case_insensitively() {
        assert!(
            matches_search("dirt", "minecraft", "Dirt"),
            "the hover name is compared lower-cased"
        );
        assert!(
            matches_search("STONE", "minecraft", "Stone"),
            "the search is lower-cased too"
        );
        assert!(
            matches_search("ak lo", "minecraft", "Oak Log"),
            "a word is a contains, not a whole-name match"
        );
        assert!(
            !matches_search("dirt", "minecraft", "Stone"),
            "a word no name holds matches nothing"
        );
    }

    #[test]
    fn a_search_that_starts_with_at_matches_the_namespace_by_prefix() {
        assert!(
            matches_search("@minecraft", "minecraft", "Stone"),
            "@modid tests the namespace"
        );
        assert!(
            matches_search("@mine", "minecraft", "Stone"),
            "upstream uses startsWith, so a namespace prefix matches"
        );
        assert!(
            !matches_search("@craft", "minecraft", "Stone"),
            "a namespace substring that is not a prefix does not match"
        );
        assert!(
            !matches_search("@nomatch", "minecraft", "Stone"),
            "an unknown namespace matches nothing"
        );
    }

    #[test]
    fn every_search_word_must_match_and_an_empty_search_matches_everything() {
        assert!(
            matches_search("", "minecraft", "Stone"),
            "an empty search is getMatcher's s -> true"
        );
        assert!(
            matches_search("   ", "minecraft", "Stone"),
            "whitespace only is an empty search"
        );
        assert!(
            matches_search("@minecraft stone", "minecraft", "Stone"),
            "two words that both match pass"
        );
        assert!(
            !matches_search("@minecraft dirt", "minecraft", "Stone"),
            "one word that does not match fails the whole search"
        );
        assert!(
            !matches_search("stone dirt", "minecraft", "Stone"),
            "every word is ANDed, not ORed"
        );
    }

    #[test]
    fn the_search_name_comes_from_the_item_ids_path() {
        assert_eq!(
            search_name("minecraft:oak_log"),
            "oak log",
            "the path's underscores are spaces"
        );
        assert_eq!(search_name("minecraft:stone"), "stone");
        assert_eq!(
            search_namespace("rftoolsstorage:storage_scanner"),
            "rftoolsstorage"
        );
        assert_eq!(
            search_namespace("minecraft:stone"),
            "minecraft",
            "@minecraft is the namespace of every vanilla item"
        );
        assert_eq!(
            search_namespace("stone"),
            "minecraft",
            "an id without a namespace is the default one"
        );
        assert_eq!(search_name("stone"), "stone");
    }

    #[test]
    fn the_list_modes_are_the_gui_s_five_choices_and_default_to_name() {
        // SortingMode, verbatim: the labels are the GUI's choice text and the
        // saved value is the enum's serialized name (StorageScannerData.CODEC).
        assert_eq!(SortingMode::AmountAscending.description(), "Amount+");
        assert_eq!(SortingMode::AmountDescending.description(), "Amount-");
        assert_eq!(SortingMode::Mod.description(), "Mod");
        assert_eq!(SortingMode::Tag.description(), "Tag");
        assert_eq!(SortingMode::Name.description(), "Name");
        assert_eq!(
            SortingMode::AmountDescending.serialized_name(),
            "AMOUNT_DESCENDING"
        );
        assert_eq!(SortingMode::Mod.serialized_name(), "MOD");
        assert_eq!(
            SortingMode::from_description("Amount-"),
            Some(SortingMode::AmountDescending),
            "the command takes the GUI's label"
        );
        assert_eq!(
            SortingMode::from_description("Name"),
            Some(SortingMode::Name)
        );
        assert_eq!(
            SortingMode::from_description("name"),
            None,
            "byDescription is exact, not case-insensitive"
        );
        assert_eq!(SortingMode::from_description("Bogus"), None);
        assert_eq!(
            SortingMode::from_saved(None),
            SortingMode::Name,
            "StorageScannerData.DEFAULT.sortingMode is NAME"
        );
        assert_eq!(SortingMode::from_saved(Some("MOD")), SortingMode::Mod);
        assert_eq!(
            SortingMode::from_saved(Some("NOPE")),
            SortingMode::Name,
            "an unknown saved value is the default"
        );
        assert_eq!(
            SortingMode::from_saved(Some("TAG")),
            SortingMode::Name,
            "TAG cannot be honoured at this pin, so it reads as the default"
        );
        assert!(SortingMode::Name.available());
        assert!(SortingMode::AmountAscending.available());
        assert!(SortingMode::Mod.available());
        assert!(
            !SortingMode::Tag.available(),
            "the SDK has no item-tag enumeration; upstream's findLargestTag needs one"
        );
    }

    #[test]
    fn the_scan_list_sorts_by_the_chosen_mode() {
        let walk = merge_counts(&[
            ("minecraft:stone".to_owned(), 32),
            ("minecraft:dirt".to_owned(), 64),
            ("rftoolsstorage:storage_module0".to_owned(), 1),
        ]);
        assert_eq!(
            walk.iter()
                .map(|entry| entry.item.as_str())
                .collect::<Vec<_>>(),
            [
                "minecraft:stone",
                "minecraft:dirt",
                "rftoolsstorage:storage_module0"
            ],
            "merge_counts keeps the walk's first-seen order"
        );
        let items = |mode: SortingMode| {
            sort_found(mode, &walk)
                .into_iter()
                .map(|entry| (entry.item, entry.count))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            items(SortingMode::AmountAscending),
            vec![
                ("rftoolsstorage:storage_module0".to_owned(), 1),
                ("minecraft:stone".to_owned(), 32),
                ("minecraft:dirt".to_owned(), 64),
            ],
            "Amount+ is the count ascending"
        );
        assert_eq!(
            items(SortingMode::AmountDescending),
            vec![
                ("minecraft:dirt".to_owned(), 64),
                ("minecraft:stone".to_owned(), 32),
                ("rftoolsstorage:storage_module0".to_owned(), 1),
            ],
            "Amount- is the count descending"
        );
        assert_eq!(
            items(SortingMode::Name),
            vec![
                ("minecraft:dirt".to_owned(), 64),
                ("minecraft:stone".to_owned(), 32),
                ("rftoolsstorage:storage_module0".to_owned(), 1),
            ],
            "Name is the item id's order"
        );
        assert_eq!(
            items(SortingMode::Mod),
            vec![
                ("minecraft:dirt".to_owned(), 64),
                ("minecraft:stone".to_owned(), 32),
                ("rftoolsstorage:storage_module0".to_owned(), 1),
            ],
            "Mod groups minecraft before rftoolsstorage"
        );
    }

    #[test]
    fn a_fresh_scanners_star_list_is_empty_and_a_star_switches_one_cell() {
        let fresh: [[i32; 3]; 0] = [];
        assert!(
            !is_routable(&fresh, [8, 102, 3]),
            "StorageScannerTileEntity's routable set is a fresh HashSet: nothing is starred"
        );
        let starred = star(&fresh, [8, 102, 3], true);
        assert_eq!(starred, vec![[8, 102, 3]], "a press adds the cell");
        assert!(is_routable(&starred, [8, 102, 3]));
        assert!(!is_routable(&starred, [1, 2, 3]), "only the pressed cell");
        assert_eq!(
            star(&starred, [8, 102, 3], true),
            starred,
            "starring a starred cell changes nothing"
        );
        let other = star(&starred, [1, 2, 3], true);
        assert_eq!(
            other,
            vec![[8, 102, 3], [1, 2, 3]],
            "a second star keeps the first"
        );
        let unstarred = star(&other, [8, 102, 3], false);
        assert_eq!(unstarred, vec![[1, 2, 3]], "an off removes just that cell");
        assert_eq!(
            star(&fresh, [8, 102, 3], false),
            fresh,
            "un-starring an unstarred cell changes nothing"
        );
    }

    #[test]
    fn the_walk_skips_a_cell_that_is_not_starred() {
        let centre = [8, 102, 3];
        assert!(
            routing_target(centre, [10, 102, 3], false, true),
            "a starred inventory in the box is routable"
        );
        assert!(
            !routing_target(centre, [10, 102, 3], false, false),
            "an unstarred inventory is not routable even though the scan lists it"
        );
        assert!(
            !routing_target(centre, centre, false, true),
            "the scanner's own cell is never routable"
        );
        assert!(
            !routing_target(centre, [9, 102, 3], true, true),
            "another storage scanner is never routable"
        );
    }

    #[test]
    fn the_star_reply_names_the_cell_and_its_state() {
        assert_eq!(routable_line([8, 102, 3], true), "routable 8 102 3 on");
        assert_eq!(routable_line([8, 102, 3], false), "routable 8 102 3 off");
    }

    #[test]
    fn the_export_choice_is_upstreams_exportc_flag_and_defaults_to_the_walk() {
        assert_eq!(ExportMode::from_word("current"), Some(ExportMode::Current));
        assert_eq!(ExportMode::from_word("first"), Some(ExportMode::First));
        assert_eq!(
            ExportMode::from_word("Current"),
            None,
            "the command's words are exact"
        );
        assert_eq!(ExportMode::from_word(""), None, "no word is no choice");
        assert_eq!(ExportMode::Current.word(), "current");
        assert_eq!(ExportMode::First.word(), "first");
        assert!(
            ExportMode::Current.saved(),
            "exportC true is the current-container choice"
        );
        assert!(
            !ExportMode::First.saved(),
            "StorageScannerData.DEFAULT is exportC false, the first routable container"
        );
        assert_eq!(ExportMode::from_saved(Some(true)), ExportMode::Current);
        assert_eq!(ExportMode::from_saved(Some(false)), ExportMode::First);
        assert_eq!(
            ExportMode::from_saved(None),
            ExportMode::First,
            "a block that was never given the flag is the default"
        );
    }

    #[test]
    fn the_current_choice_sends_the_stack_only_to_the_selected_inventory() {
        let selected = [12, 102, 3];
        assert_eq!(
            export_route(ExportMode::Current, Some(selected)),
            ExportRoute::Only(selected),
            "the current-container choice exports into the selected inventory"
        );
        assert_eq!(
            export_route(ExportMode::Current, None),
            ExportRoute::Walk,
            "with nothing selected the current-container choice has nothing to export into"
        );
        assert_eq!(
            export_route(ExportMode::First, Some(selected)),
            ExportRoute::Walk,
            "the first-routable choice ignores the selection"
        );
        assert_eq!(
            export_route(ExportMode::First, None),
            ExportRoute::Walk,
            "the default is the starred walk"
        );
    }

    #[test]
    fn the_export_and_selection_replies_name_the_choice_and_the_cell() {
        assert_eq!(export_line(ExportMode::Current), "export current");
        assert_eq!(export_line(ExportMode::First), "export first");
        assert_eq!(selection_line([12, 102, 3]), "select 12 102 3");
    }

    #[test]
    fn amount_sorting_is_stable_so_equal_counts_keep_the_walk_order() {
        let walk = merge_counts(&[
            ("minecraft:dirt".to_owned(), 64),
            ("minecraft:stone".to_owned(), 32),
            ("minecraft:sand".to_owned(), 64),
        ]);
        assert_eq!(
            sort_found(SortingMode::AmountDescending, &walk)
                .into_iter()
                .map(|entry| entry.item)
                .collect::<Vec<_>>(),
            ["minecraft:dirt", "minecraft:sand", "minecraft:stone"],
            "equal counts keep the order the walk first met them (a stable sort)"
        );
        assert_eq!(
            sort_found(SortingMode::AmountAscending, &walk)
                .into_iter()
                .map(|entry| entry.item)
                .collect::<Vec<_>>(),
            ["minecraft:stone", "minecraft:dirt", "minecraft:sand"],
            "the ascending direction is stable too"
        );
    }

    #[test]
    fn a_contents_row_is_an_inventory_the_scan_lists() {
        // getInventoryForBlock(cpos) can only name a cell findInventories recorded:
        // in the box, not the scanner itself, not another scanner, and an inventory.
        let centre = [16, 102, 10];
        assert!(
            contents_row(centre, 2, [18, 102, 12], false, true),
            "a chest in the box is a row"
        );
        assert!(
            contents_row(centre, 2, [14, 102, 10], false, true),
            "the box reaches its lower corner"
        );
        assert!(
            !contents_row(centre, 2, [19, 102, 10], false, true),
            "three blocks away is outside a radius-2 box"
        );
        assert!(
            !contents_row(centre, 2, [18, 105, 10], false, true),
            "the box is a cube: y is bounded too"
        );
        assert!(
            !contents_row(centre, 2, centre, false, true),
            "the scanner's own cell is never a row"
        );
        assert!(
            !contents_row(centre, 2, [17, 102, 10], true, true),
            "another storage scanner is never a row"
        );
        assert!(
            !contents_row(centre, 2, [17, 102, 10], false, false),
            "a cell with no item capability is not an inventory"
        );
        assert!(in_scan_box(centre, 2, [18, 102, 12]));
        assert!(!in_scan_box(centre, 2, [19, 102, 10]));
    }

    #[test]
    fn the_contents_reply_names_the_row_or_the_all_row_and_its_kinds() {
        let stone = Found {
            item: "minecraft:stone".to_owned(),
            count: 128,
        };
        assert_eq!(
            contents_line(Some([16, 102, 12]), std::slice::from_ref(&stone)),
            "contents 16 102 12: minecraft:stone 128"
        );
        assert_eq!(
            contents_line(
                None,
                &[Found {
                    item: "minecraft:stone".to_owned(),
                    count: 64,
                }],
            ),
            "contents all: minecraft:stone 64"
        );
        assert_eq!(
            contents_line(Some([18, 102, 10]), &[]),
            "contents 18 102 10: empty",
            "a row with nothing in it still names the row"
        );
        assert_eq!(
            contents_line(
                None,
                &[
                    stone,
                    Found {
                        item: "minecraft:dirt".to_owned(),
                        count: 32,
                    },
                ],
            ),
            "contents all: minecraft:stone 128, minecraft:dirt 32",
            "several kinds keep the scan's one-entry-a-kind voice"
        );
    }

    #[test]
    fn a_cell_the_scan_did_not_list_is_refused_with_the_box_reason() {
        assert_eq!(
            contents_refused_line([19, 102, 10]),
            "19 102 10 is not an inventory in the scan box"
        );
    }

    #[test]
    fn the_four_craft_buttons_are_the_guis_and_the_count_is_upstreams_maths() {
        // GuiCraftingGrid: craft1Button/craft4Button/craft8Button/craftSButton.
        assert_eq!(Batch::from_word("1"), Some(Batch::One));
        assert_eq!(Batch::from_word("4"), Some(Batch::Four));
        assert_eq!(Batch::from_word("8"), Some(Batch::Eight));
        assert_eq!(Batch::from_word("stack"), Some(Batch::Stack));
        assert_eq!(
            Batch::from_word("16"),
            None,
            "the GUI has four buttons, not a free count"
        );
        assert_eq!(
            Batch::from_word("Stack"),
            None,
            "the command's words are exact"
        );
        assert_eq!(Batch::from_word(""), None);
        assert_eq!(Batch::One.word(), "1");
        assert_eq!(Batch::Four.word(), "4");
        assert_eq!(Batch::Eight.word(), "8");
        assert_eq!(Batch::Stack.word(), "stack");
        assert_eq!(Batch::One.wanted(), Some(1));
        assert_eq!(
            Batch::Stack.wanted(),
            None,
            "the * button is craftItems' amount -1"
        );

        // craftItems: n = -1 → the result's max stack size; sets = ceil(n /
        // resultCount); one set fewer when sets * resultCount would not fit a stack.
        assert_eq!(
            craft_sets(Some(1), 4, 64),
            1,
            "one craft of a 4-stick recipe is one set"
        );
        assert_eq!(craft_sets(Some(4), 4, 64), 1, "four sticks are one set");
        assert_eq!(craft_sets(Some(8), 4, 64), 2, "eight sticks are two sets");
        assert_eq!(
            craft_sets(None, 4, 64),
            16,
            "the * button asks for a whole stack: 64 sticks, 16 sets"
        );
        assert_eq!(
            craft_sets(Some(64), 1, 64),
            64,
            "a 1-result recipe: 64 sets"
        );
        assert_eq!(
            craft_sets(Some(65), 1, 64),
            64,
            "past a stack the product is dropped back to one stack"
        );
        assert_eq!(
            craft_sets(None, 3, 8),
            2,
            "a 3-result recipe in an 8 stack: 2"
        );
        assert_eq!(
            craft_sets(None, 1, 16),
            16,
            "a 16-stack item's * button is 16 sets"
        );
        assert_eq!(craft_sets(Some(0), 4, 64), 0, "no count, nothing to craft");
        assert_eq!(craft_sets(Some(8), 0, 64), 0, "a recipe with no result");
        assert_eq!(craft_sets(Some(8), 4, 0), 0, "an item no game stack holds");
    }

    #[test]
    fn the_pattern_asks_one_of_each_filled_cell_a_set() {
        let planks = plain("minecraft:oak_planks");
        let mut cells: Vec<Option<Kind>> = vec![None; PATTERN_CELLS];
        cells[0] = Some(planks.clone());
        cells[3] = Some(planks.clone());
        let needs = pattern_needs(&cells, 1);
        assert_eq!(
            needs,
            vec![
                Need {
                    cell: 1,
                    kind: planks.clone(),
                    count: 1
                },
                Need {
                    cell: 4,
                    kind: planks,
                    count: 1
                },
            ],
            "the cells are upstream's ghost inputs: cell 1 is the top left"
        );
        assert_eq!(
            pattern_needs(&cells, 16)
                .iter()
                .map(|need| need.count)
                .collect::<Vec<_>>(),
            [16, 16],
            "a batch of sixteen sets asks sixteen of every filled cell"
        );
        assert!(
            pattern_needs(&vec![None; PATTERN_CELLS], 1).is_empty(),
            "an empty pattern asks nothing"
        );
    }

    #[test]
    fn the_sources_are_walked_in_order_and_a_short_cell_fails_the_whole_plan() {
        let planks = plain("minecraft:oak_planks");
        let sticks = plain("minecraft:stick");
        let needs = vec![
            Need {
                cell: 1,
                kind: planks.clone(),
                count: 2,
            },
            Need {
                cell: 4,
                kind: planks.clone(),
                count: 2,
            },
        ];
        // The player's inventory first (upstream's `InvWrapper(player)`), then the
        // storage: one plank in the player's slot 5, three in the storage's slot 0.
        let player = vec![Held {
            slot: 5,
            kind: planks.clone(),
            count: 1,
        }];
        let storage = vec![
            Held {
                slot: 0,
                kind: sticks,
                count: 64,
            },
            Held {
                slot: 1,
                kind: planks.clone(),
                count: 3,
            },
        ];
        assert_eq!(
            craft_plan(&needs, &[player.clone(), storage.clone()]),
            CraftPlan::Ready(vec![
                Draw {
                    source: 0,
                    slot: 5,
                    count: 1
                },
                Draw {
                    source: 1,
                    slot: 1,
                    count: 3
                },
            ]),
            "the player's plank goes first, the storage's three fill the rest"
        );
        // A cell short by one fails everything: nothing is taken at all.
        assert_eq!(
            craft_plan(
                &needs,
                &[
                    vec![Held {
                        slot: 5,
                        kind: planks.clone(),
                        count: 1
                    }],
                    vec![Held {
                        slot: 1,
                        kind: planks.clone(),
                        count: 2
                    }],
                ],
            ),
            CraftPlan::Missing(vec![Short {
                cell: 4,
                kind: planks.clone(),
                short: 1
            }]),
            "a cell one short fails the whole plan, as upstream undoes its walk"
        );
        assert_eq!(
            craft_plan(&needs, &[Vec::new(), Vec::new()]),
            CraftPlan::Missing(vec![
                Short {
                    cell: 1,
                    kind: planks.clone(),
                    short: 2
                },
                Short {
                    cell: 4,
                    kind: planks,
                    short: 2
                },
            ]),
            "every short cell is named, in cell order"
        );
        // Two cells of the same kind take from one slot without doubling it.
        assert_eq!(
            craft_plan(
                &needs,
                &[vec![Held {
                    slot: 0,
                    kind: plain("minecraft:oak_planks"),
                    count: 3
                }]],
            ),
            CraftPlan::Missing(vec![Short {
                cell: 4,
                kind: plain("minecraft:oak_planks"),
                short: 1
            }]),
            "a slot's four planks are not counted twice for two cells"
        );
        assert_eq!(
            craft_plan(
                &[Need {
                    cell: 1,
                    kind: plain("minecraft:oak_planks"),
                    count: 1
                }],
                &[vec![Held {
                    slot: 0,
                    kind: plain("minecraft:oak_planks"),
                    count: 3
                }]],
            ),
            CraftPlan::Ready(vec![Draw {
                source: 0,
                slot: 0,
                count: 1
            }]),
            "a slot gives only what the cell asks for"
        );
    }

    #[test]
    fn the_pattern_craft_and_missing_replies_name_what_happened() {
        let planks = plain("minecraft:oak_planks");
        let sticks = plain("minecraft:stick");
        let mut cells: Vec<Option<Kind>> = vec![None; PATTERN_CELLS];
        cells[0] = Some(planks.clone());
        cells[3] = Some(planks.clone());
        assert_eq!(
            pattern_line(&cells, Some((&sticks, 4))),
            "pattern 1=minecraft:oak_planks, 4=minecraft:oak_planks -> minecraft:stick x4"
        );
        assert_eq!(
            pattern_line(&cells, None),
            "pattern 1=minecraft:oak_planks, 4=minecraft:oak_planks -> no recipe"
        );
        assert_eq!(
            pattern_line(&vec![None; PATTERN_CELLS], None),
            "pattern empty"
        );
        assert_eq!(
            crafted_line(4, &sticks),
            "crafted 4 minecraft:stick",
            "the result reaches the player's inventory"
        );
        assert_eq!(
            missing_line(&[
                Short {
                    cell: 1,
                    kind: planks.clone(),
                    short: 2
                },
                Short {
                    cell: 4,
                    kind: sticks.clone(),
                    short: 1
                },
            ]),
            "missing cell 1: 2 minecraft:oak_planks, cell 4: 1 minecraft:stick"
        );
        assert_eq!(no_recipe_line(), "the pattern crafts nothing");
        assert_eq!(empty_pattern_line(), "the pattern is empty");
        assert_eq!(
            craft_failed_line(),
            "the craft could not be completed; nothing was taken",
            "a refused batch is rolled back, so nothing left the sources"
        );
        assert_eq!(
            pattern_locked_line(),
            "lock the storage to change the pattern",
            "upstream's ghost cells refuse a place while the storage is unlocked"
        );
    }
}
