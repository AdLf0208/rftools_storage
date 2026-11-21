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

/// Merges the kinds an inventory walk found into one entry a kind, sorted by
/// item id — upstream's `SortingMode.NAME`, the default.
#[must_use]
pub fn merge_counts(entries: &[(String, u32)]) -> Vec<Found> {
    let mut found: Vec<Found> = Vec::new();
    for (item, count) in entries {
        if *count == 0 {
            continue;
        }
        match found.binary_search_by(|entry| entry.item.as_str().cmp(item.as_str())) {
            Ok(index) => found[index].count = found[index].count.saturating_add(*count),
            Err(index) => found.insert(
                index,
                Found {
                    item: item.clone(),
                    count: *count,
                },
            ),
        }
    }
    found
}

/// The scan's answer after a paid scan: what the inventories hold, then the
/// buffer left.
#[must_use]
pub fn scan_line(after: u32, found: &[Found]) -> String {
    let items = if found.is_empty() {
        "empty".to_owned()
    } else {
        found
            .iter()
            .map(|entry| format!("{} {}", entry.item, entry.count))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!("{items} ({after}/{SCANNER_MAX_ENERGY} FE)")
}

/// The scan's answer when the buffer cannot pay for it.
#[must_use]
pub fn no_power_line(stored: u32) -> String {
    format!("no power ({stored}/{SCANNER_MAX_ENERGY} FE)")
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
        let found = merge_counts(&[
            ("minecraft:stone".to_owned(), 64),
            ("minecraft:dirt".to_owned(), 3),
            ("minecraft:stone".to_owned(), 32),
        ]);
        assert_eq!(found.len(), 2, "one entry a kind");
        assert_eq!(
            found[0].item, "minecraft:dirt",
            "sorted by item id (the GUI's NAME mode)"
        );
        assert_eq!(found[0].count, 3);
        assert_eq!(found[1].item, "minecraft:stone");
        assert_eq!(found[1].count, 96, "64 + 32, the whole count");
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
}
