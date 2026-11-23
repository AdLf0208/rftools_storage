//! The game glue of the modular storage block: its block entity, its container,
//! its menu (the screen, the padlock and the grid's slot rules), the player's
//! click and the readout command.
//!
//! In the SDK layout this is an ordinary module of the mod crate. The rules —
//! the module's slot count, one kind a slot, 64 a slot, the lock — live in
//! [`crate::runtime`], which has no `mod_api` types; this module only reads the
//! game, calls the runtime and writes the result back.
//!
//! The storage *is* the block entity's container slots: the pin saves a mod
//! block entity's `Items` with its chunk, so a relaunch keeps them. `module` and
//! `locked` ride the same saved compound, so the module and the lock survive a
//! relaunch too.
//!
//! A **player's break** that drops the block moves the container's stacks aside
//! into the same saved data before the removal (so the engine's own container
//! spill finds nothing), the block's drop carries them as vanilla's
//! `minecraft:block_entity_data` (`on_block_drops`), and placing the item writes
//! them back (`StorageItem`). A break the port cannot carry (creative, a refused
//! tool, a piston, water, an explosion) keeps the vanilla spill.

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use mod_api::item::containers::{ModBlockEntityFields, ModEnergyBuffer};
use mod_api::{
    self, BlockDropsEvent, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent,
    BlockEntityTickReply, BlockEntityTicksEvent, BlockKey, BlockPart, ClientHost, Command,
    CommandArgumentValue, CommandSource, Compound, Flow, Hand, Host, HostSide, HostTransaction,
    ITEMS, Identifier, InteractionResult, ItemKey, ItemPart, ItemResource, ItemStack, LevelHost,
    LevelRef, LevelSide, MenuButtonEvent, MenuContainer, MenuPart, MenuSlotRulesEvent,
    ModContainer, OnBlockPlacedByEvent, OnDestroyedByPlayerEvent, PlayerContainerOpenEvent,
    PlayerRef, RegisterCommandsEvent, Registrar, RegistrationContext, RightClickBlockEvent,
    RunModCommandEvent, Screen, ScreenClickEvent, ScreenDrawEvent, ScreenEvent, ServerHost, Tag,
    Text, Transaction,
};

use crate::runtime::{
    Found, GiveAmount, InsertCharge, Kind, LEVEL_ADMINS, LockPress, RequestCharge,
    SCANNER_AUTO_SLOT, SCANNER_IN_SLOT, SCANNER_MAX_ENERGY, SCANNER_OUT_SLOT,
    SCANNER_RECEIVE_PER_TICK, SCANNER_RF_PER_INSERT, SCANNER_SLOTS, SLOT_LIMIT, STORAGE_SCANNER,
    Slot, Storage, card_accepts, carried_slot_in, clamp_radius, gave_line, give_amount,
    give_nothing_line, give_price, inject_plan, injectable_at, insert_amount, insert_charge,
    insert_line, insert_nothing_line, insert_partial_line, insert_price, is_storage_scanner,
    lock_press, may_inspect, merge_counts, module_id, module_tier, no_power_line, no_routable_line,
    out_slot_room, request_amount, request_charge, request_line, request_no_power_line,
    request_nothing_line, request_out_slot_line, request_price, restored_count, scan_line,
    scan_positions, scanner_radius, slot_accepts, spend_power,
};

/// The block this slice implements, exactly as the content declares it.
const BLOCK: &str = crate::runtime::MODULAR_STORAGE;
/// The block-entity type this mod declares for that block.
const BE_TYPE: &str = "rftoolsstorage:storage_block_entity";
/// The scanner's block-entity type: the scanner's own saved compound (its FE
/// buffer and its scan radius) and nothing else.
const SCANNER_BE_TYPE: &str = "rftoolsstorage:storage_scanner_block_entity";
/// The saved compound key of the scanner's radius (`StorageScannerData.radius`).
const KEY_RADIUS: &str = "radius";
/// The saved compound key of a machine's FE buffer: the name the game's own
/// energy handler reads and writes (`ModEnergyBuffer`).
const KEY_ENERGY: &str = "energy";
/// The saved compound key of the installed module's tier. Only a block saved
/// before the card slot (0.2.2) has it: [`adopt_legacy_module`] moves it into the
/// slot, and every write clears it, so a removed module cannot come back.
const KEY_MODULE: &str = "module";
/// The saved compound key of the lock (a byte, as upstream's lockable entity).
const KEY_LOCKED: &str = "locked";
/// No module recorded in the saved compound. A stored sentinel rather than a
/// missing key: `merge_data` sets a key but cannot remove one, and the round trip
/// is internal — [`legacy_tier`] reads any value outside `0..=3` as no module.
const NO_MODULE: i32 = -1;
/// The container's storage slots: the tier-1 module's 100 (`TIER_SLOTS[0]`). A
/// block's container is declared once per block, not per instance, so only the
/// tier this slice carries is wired (notes.md, deviation 4); the menu declares
/// the same hundred plus the card slot and the screen places them in its grid.
const STORAGE_SLOTS: usize = 100;
/// The card slot, after the storage slots: upstream's `SLOT_STORAGE_MODULE`, which
/// holds the module item itself. Hoppers never reach it ([`register`] leaves it out
/// of the faces and of `inputs`/`outputs`, as upstream's card handler is not the
/// block's item capability).
const MODULE_SLOT: usize = STORAGE_SLOTS;
/// The container's slot count: the storage's hundred and the card slot.
const CONTAINER_SLOTS: usize = STORAGE_SLOTS + 1;
/// The action-bar prefix every readout carries.
const READOUT: &str = "Modular Storage: ";
/// The menu type the carried content declares
/// (`data/rftoolsstorage/horizon/menu/modular_storage.json`), named by the
/// block's container so that using the block opens it — upstream's GUI.
const MENU: &str = "rftoolsstorage:modular_storage";
/// The menu's one data slot: 1 while the storage is locked. It is not saved, so
/// it is filled when the menu opens and on every padlock press.
const DATA_LOCK: usize = 0;
/// The padlock button's id: `screen_click` answers it, the game sends the press
/// to `menu_button` (`AbstractContainerMenu.clickMenuButton`).
const BUTTON_LOCK: i32 = 0;
/// The panel: the tier-1 storage's 10 x 10 grid on the left and the player's
/// inventory to its right, so the whole screen is 210 px tall — under a vanilla
/// double chest's 222, the tallest screen the smallest vanilla GUI (240 px)
/// holds. Drawing slot by slot beats scrolling: the pin's screen layout is fixed
/// when the menu opens, so a scroll could not move a slot.
const SCREEN_WIDTH: f32 = 366.0;
const SCREEN_HEIGHT: f32 = 210.0;
const GRID_COLS: usize = 10;
const GRID_X: f32 = 8.0;
const GRID_Y: f32 = 26.0;
const SLOT_PITCH: f32 = 18.0;
const INVENTORY_X: f32 = 200.0;
const INVENTORY_Y: f32 = 26.0;
/// The card slot's top left, above the grid's first column (upstream's GUI keeps
/// it at the panel's left edge, `(5, 157)` in its own layout).
const MODULE_SLOT_AT: [f32; 2] = [GRID_X, 4.0];
/// The padlock button's rectangle, relative to the panel's top left: beside the
/// card slot, so the title clears both.
const LOCK_BUTTON: [f32; 4] = [32.0, 6.0, 46.0, 16.0];
/// The saved compound key a broken storage's contents wait under until its item
/// carries them: the container's stacks, in vanilla's `Items` shape, moved aside
/// before the removal so the engine's own container spill finds nothing to
/// scatter (`notes.md`, slice 4).
const KEY_CARRIED: &str = "carried";
/// The vanilla component that moves a block entity's saved data on its item
/// (`BlockEntity.applyComponentsFromItemStack`, `TypedEntityData.loadInto`).
const BLOCK_ENTITY_DATA: &str = "minecraft:block_entity_data";

/// The block key the click compares the loaded block against. A `BlockKey` only
/// names itself through a host, so it is asked for once, at registration.
static BLOCK_KEY: OnceLock<BlockKey> = OnceLock::new();
/// The scanner's block key, asked for once at registration for the same reason.
static SCANNER_KEY: OnceLock<BlockKey> = OnceLock::new();
/// The block's own item, resolved once at registration: `block_drops` runs with a
/// read-only level host, which cannot name an item, and only this block's own
/// drop takes the carried contents.
static ITEM_KEY: OnceLock<ItemKey> = OnceLock::new();

fn block_key() -> BlockKey {
    *BLOCK_KEY
        .get()
        .expect("registration runs before every hook")
}

fn scanner_key() -> BlockKey {
    *SCANNER_KEY
        .get()
        .expect("registration runs before every hook")
}

// --- registration ----------------------------------------------------------

/// Declare the block-entity type, attach its Part, declare the block's
/// container (100 slots, every face, inputs and outputs) and the menu it opens.
pub(crate) fn register(registrar: &mut dyn Registrar, _cx: &RegistrationContext) {
    let block = registrar.block_key(BLOCK);
    let _ = BLOCK_KEY.set(block);
    let kind = registrar
        .block_entity_type(BE_TYPE)
        .blocks(&[BLOCK])
        .build();
    registrar
        .block_entity_part(&kind, StorageBlock)
        .expect("one Part per block-entity type");
    registrar.menu_part(MENU, StorageMenu);
    // The block's own item: a placed storage takes back the contents a broken one
    // put on it (`minecraft:block_entity_data`), upstream's contents-in-the-block.
    let item = registrar.item_key(BLOCK);
    let _ = ITEM_KEY.set(item);
    registrar.item_part(item, StorageItem);
    let storage: Vec<usize> = (0..STORAGE_SLOTS).collect();
    // Upstream's `CapType.ITEMS` handler answers every side with the storage slots
    // only (its cards are a separate handler the capability never exposes), and a
    // hopper may both fill and empty it.
    // `..Default::default()` keeps a field a later game build adds from breaking
    // this literal (`cargo horizon-mod check` asks for it).
    #[expect(
        clippy::needless_update,
        reason = "the update is for a field a later game build may add"
    )]
    let container = ModContainer {
        slots: CONTAINER_SLOTS,
        faces: std::array::from_fn(|_| storage.clone()),
        inputs: storage.clone(),
        outputs: storage,
        // The block's use opens this menu, as a chest's does: the mod's own
        // surface instead of the slice-1 right-click substitute.
        menu: Some(MENU.to_owned()),
        ..ModContainer::default()
    };
    registrar.block_container(&block, container);
    register_scanner(registrar);
}

/// The scanner's declarations: its block-entity type and Part, its FE buffer
/// (`StorageScannerConfiguration`'s 50,000 / 500 a tick) and its container — the
/// GUI's in and out slots and the automation slot the tick routes
/// (`StorageScannerContainer`). The menu itself stays a gap: the pin's screen is a
/// fixed slot layout and the list widget is the next slice's, so the slots' consumer
/// here is the tick, not a screen.
fn register_scanner(registrar: &mut dyn Registrar) {
    let scanner = registrar.block_key(STORAGE_SCANNER);
    let _ = SCANNER_KEY.set(scanner);
    let kind = registrar
        .block_entity_type(SCANNER_BE_TYPE)
        .blocks(&[STORAGE_SCANNER])
        .build();
    registrar
        .block_entity_part(&kind, ScannerBlock)
        .expect("one Part per block-entity type");
    // `GenericEnergyStorage(this, true, MAXENERGY, RECEIVEPERTICK)`: filled from
    // any side up to 500 a move, never drained from outside (the machine spends
    // its own buffer), and the amount is the saved compound's `energy`.
    #[expect(
        clippy::needless_update,
        reason = "the update is for a field a later game build may add"
    )]
    let energy = ModEnergyBuffer {
        capacity: 50_000,
        max_insert: i32::try_from(SCANNER_RECEIVE_PER_TICK).unwrap_or(i32::MAX),
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
        ..ModEnergyBuffer::default()
    };
    registrar.block_energy(&scanner, energy);
    registrar.block_container(&scanner, scanner_container());
}

/// `StorageScannerContainer`: three slots — the GUI's input, the out slot a request
/// fills and the automation input — with the jar's own capability policy:
/// `GenericItemHandler…insertable(slot(SLOT_IN_AUTO))`, so automation inserts into
/// the auto slot only, and McJtyLib's `extractable` default `(slot, stack) -> true`,
/// so every slot may be pulled from. No menu: the GUI's list is a later slice, and
/// the slots' consumer here is the tick.
fn scanner_container() -> ModContainer {
    let all = (0..SCANNER_SLOTS).collect::<Vec<_>>();
    #[expect(
        clippy::needless_update,
        reason = "the update is for a field a later game build may add"
    )]
    ModContainer {
        slots: SCANNER_SLOTS,
        faces: std::array::from_fn(|_| all.clone()),
        inputs: vec![SCANNER_AUTO_SLOT],
        outputs: vec![SCANNER_IN_SLOT, SCANNER_OUT_SLOT, SCANNER_AUTO_SLOT],
        menu: None,
        ..ModContainer::default()
    }
}

/// The scanner's Part: its block entity (the saved compound the FE buffer and the
/// radius live in) and the tick that routes its two input slots into the
/// inventories its scan finds.
struct ScannerBlock;

#[mod_api::horizon_part]
impl BlockPart for ScannerBlock {}

#[mod_api::horizon_part]
impl BlockEntityPart for ScannerBlock {
    /// The scanner gets a block entity with empty saved data; `energy` and `radius`
    /// are written by the game's energy handler, by `/rftoolsstorage radius` and by
    /// the inject tick.
    fn block_entity(
        &self,
        _level: LevelSide<'_>,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        Some(Compound::new())
    }

    /// `EntityBlock.getTicker`: the scanner ticks (upstream's `TickingTileEntity`),
    /// which is what turns its input slots into pushes. Never clear a value an
    /// earlier mod set.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick (`StorageScannerTileEntity.tickServer`): the GUI input, then
    /// the automation input, each a `rfPerInsert` attempt into the scanned
    /// inventories. The block entity is out of the world during the call, so its own
    /// slots and the buffer go through the payload; the neighbours are written
    /// through their `ITEMS` capability.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        if event.entity.items.len() != SCANNER_SLOTS {
            event.entity.items.resize(SCANNER_SLOTS, None);
        }
        let mut saved = Tag::Compound(std::mem::take(&mut event.entity.nbt));
        let items = &mut event.entity.items;
        let changed = tick_scanner(host, event.pos, &mut saved, items);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        if !changed {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }
}

/// The Part of the mod's one block-entity type: the modular storage declares a
/// block entity, and nothing else about it is per-block.
struct StorageBlock;

#[mod_api::horizon_part]
impl BlockPart for StorageBlock {
    /// A player's break that will drop the block (`IBlockExtension.onDestroyedByPlayer`,
    /// asked before the removal): the contents move aside into the block entity's saved
    /// data, so the engine's container spill finds an empty container, and
    /// [`carried_payload`] puts them on the block's own drop.
    ///
    /// A break that drops nothing (a creative break, a tool the block refuses) keeps the
    /// vanilla spill: the contents are not put on an item that will not be there.
    fn on_destroyed_by_player(
        &self,
        host: &mut dyn ServerHost,
        event: &mut OnDestroyedByPlayerEvent<'_>,
    ) -> Option<bool> {
        if event.will_harvest {
            stash_carried(host, event.pos);
        }
        None
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for StorageBlock {
    /// The block gets a block entity with empty saved data; `Items` are the
    /// container's, and `module`/`locked` are written by the clicks.
    fn block_entity(
        &self,
        _level: LevelSide<'_>,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        Some(Compound::new())
    }

    /// The storage's menu layout (`native_screen`): the 100 slots of the grid,
    /// then the player's inventory beside it.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        Some(menu_screen())
    }

    /// The padlock, drawn over the panel; the grid and its frames are the
    /// engine's (`own_background` is false).
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        draw_screen(event);
    }

    /// A press on the padlock names the menu button the game sends to the
    /// server; a press anywhere else is none of ours.
    fn screen_click(
        &self,
        _host: &dyn ClientHost,
        event: &mut ScreenClickEvent<'_>,
    ) -> Option<i32> {
        let [x, y, width, height] = LOCK_BUTTON;
        let [at_x, at_y] = event.at;
        // A widget takes the left button only (`AbstractWidget.isValidClickButton`).
        (!event.right && (x..x + width).contains(&at_x) && (y..y + height).contains(&at_y))
            .then_some(BUTTON_LOCK)
    }

    /// The padlock's press, on the server (`clickMenuButton`).
    fn menu_button(&self, host: &mut dyn ServerHost, event: &mut MenuButtonEvent<'_>) {
        press_lock(host, event);
    }
}

/// The block's own item: a storage placed from a stack takes back the contents a
/// broken one carried on it (`minecraft:block_entity_data`), as upstream's block
/// finds the contents the module keyed.
pub(crate) struct StorageItem;

#[mod_api::horizon_part]
impl ItemPart for StorageItem {
    /// `BlockItemExtension.onBlockPlacedBy`, after the engine applied the item's
    /// `minecraft:block_entity_data` to the new block entity: write the carried
    /// stacks back into its slots, clamped, and clear the key so they travel once.
    fn on_block_placed_by(&self, host: &mut dyn ServerHost, event: &mut OnBlockPlacedByEvent<'_>) {
        let block = event.state.block().id(&*host);
        if block != BLOCK {
            return;
        }
        restore_carried(host, event.pos);
    }
}

/// The menu's layout: slot `index` of the grid at its column and row, then the
/// card slot above it, and the player's inventory to the right.
fn menu_screen() -> Screen {
    let mut slots: Vec<Option<[f32; 2]>> = (0..STORAGE_SLOTS)
        .map(|index| {
            let column = index % GRID_COLS;
            let row = index / GRID_COLS;
            Some([
                GRID_X + column as f32 * SLOT_PITCH,
                GRID_Y + row as f32 * SLOT_PITCH,
            ])
        })
        .collect();
    slots.push(Some(MODULE_SLOT_AT));
    Screen {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        slots,
        inventory_y: INVENTORY_Y,
        inventory_x: INVENTORY_X,
        // The panel's title, the inventory's label, the card slot's frame and the
        // padlock are drawn here, so the engine leaves its own labels out.
        own_labels: true,
        own_background: false,
        ..Screen::default()
    }
}

/// Draws the title, the inventory's label and the padlock (red while locked,
/// green while open). The padlock's state is the menu's one data slot.
fn draw_screen(event: &mut ScreenDrawEvent<'_>) {
    let locked = event.data.get(DATA_LOCK).copied().unwrap_or(0) != 0;
    let [x, y] = event.origin;
    let draw = &mut *event.draw;
    // The jar's own name for the block (`block.rftoolsstorage.modular_storage`);
    // the pin's text helper takes a literal, and the mod's lang file has it.
    draw.text(
        "Modular Storage",
        [x + LOCK_BUTTON[0] + LOCK_BUTTON[2] + 8.0, y + 10.0],
        0xff40_4040,
        false,
    );
    draw.text(
        "Inventory",
        [x + INVENTORY_X, y + INVENTORY_Y - 11.0],
        0xff40_4040,
        false,
    );
    let [bx, by, width, height] = LOCK_BUTTON;
    draw.fill(
        [x + bx, y + by, width, height],
        if locked { 0xff8b_0000 } else { 0xff00_6400 },
    );
    let label = if locked { "Unlock" } else { "Lock" };
    let text_width = draw.text_width(label);
    draw.text(
        label,
        [
            x + bx + (width - text_width) / 2.0,
            y + by + (height - 8.0) / 2.0,
        ],
        0xffff_ffff,
        false,
    );
}

/// The padlock press: flip the lock of a storage that has a module, push the
/// new state into the menu's data slot (so the button redraws) and say what
/// happened on the action bar.
fn press_lock(host: &mut dyn ServerHost, event: &mut MenuButtonEvent<'_>) {
    if event.context.id != BUTTON_LOCK {
        return;
    }
    let fields = &mut *event.entity;
    // A 0.2.2 block's module moves into the card slot here, so the padlock acts on
    // it (and the open menu shows it).
    adopt_legacy_fields(&*host, fields);
    let module = fields
        .items
        .get(MODULE_SLOT)
        .and_then(Option::as_ref)
        .cloned();
    let mut storage = read_storage(&*host, Some(&fields.nbt), &fields.items, module.as_ref());
    let line = match lock_press(&mut storage) {
        LockPress::NoModule => {
            // Nothing to lock: the block keeps its state, the button says why.
            fields.data = vec![0];
            "insert a storage module first"
        }
        LockPress::Locked => {
            lock_written(fields, &storage);
            "locked"
        }
        LockPress::Unlocked => {
            lock_written(fields, &storage);
            "unlocked"
        }
    };
    action_bar(host, event.context.player, format!("{READOUT}{line}"));
}

/// Saves the lock and hands it to the open menu's padlock (the one data slot).
fn lock_written(fields: &mut ModBlockEntityFields, storage: &Storage) {
    fields
        .nbt
        .insert(KEY_LOCKED.into(), Tag::Byte(i8::from(storage.locked)));
    fields.data = vec![i32::from(storage.locked)];
}

/// A player opened a menu: ours gets the storage's real lock state, because the
/// data slots are not saved and a reopened menu would otherwise show the state
/// of the last press.
pub(crate) fn on_player_container_open(
    host: &mut dyn ServerHost,
    event: &mut PlayerContainerOpenEvent<'_>,
) {
    if event.menu != MENU {
        return;
    }
    let Some(pos) = event.pos else {
        return;
    };
    // The hook runs with the player's state swapped in, so the level in place is
    // the menu's level (a block menu is only ever opened from the block's).
    let Some(level) = host.level(host.dimension()) else {
        return;
    };
    let Some(entity) = host.block_entity(level, pos) else {
        return;
    };
    // A block saved by 0.2.2 gets its module back as the slot's item before the
    // player sees the menu, so it can be taken out or swapped; a stash a refused
    // removal left in the saved data comes back too.
    adopt_legacy_module(host, entity);
    adopt_carried(host, pos);
    let data = entity.data(&*host);
    let module = module_at(&*host, entity);
    let locked = read_storage(&*host, data.as_ref(), &[], module.as_ref()).locked;
    let _ = entity.set_data_slots(host, vec![i32::from(locked)]);
}

/// The storage menu's slot rules (`native_menu_slot_rules`): a storage slot is
/// upstream's grid `mayPlace` (locked, and empty or the same kind), and the card
/// slot takes a storage module.
///
/// The card slot's answer is "anything, once it holds the module" on purpose: the
/// engine consults it while it plans a click on another slot and a refusal there
/// blocks that click (measured: with the card's rule strict, a click on a locked
/// storage's grid was refused). What it must not do is waste a click meant for a
/// storage slot, so only an *empty* card slot answers "module only" — which is
/// exactly when a non-module has no business being in the menu at all.
pub(crate) struct StorageMenu;

#[mod_api::horizon_part]
impl MenuPart for StorageMenu {
    fn menu_slot_rules(&self, host: &dyn Host, event: &mut MenuSlotRulesEvent<'_>) {
        if event.menu != MENU || event.container != MenuContainer::Main {
            return;
        }
        let Some(pos) = event.context.pos else {
            return;
        };
        let Some(locked) = lock_at(host, pos) else {
            return;
        };
        let incoming = kind_of(host, event.stack);
        if event.slot == MODULE_SLOT {
            let occupied = event
                .context
                .slot(MenuContainer::Main, MODULE_SLOT)
                .is_some();
            event.may_place = card_accepts(occupied, &incoming.label);
            return;
        }
        let stored = event
            .context
            .slot(MenuContainer::Main, event.slot)
            .map(|stack| kind_of(host, stack));
        event.may_place = slot_accepts(
            locked,
            stored.as_ref().map(|kind| kind.key.as_str()),
            &incoming.key,
        );
    }
}

/// Whether the storage at `pos` is locked; `None` when no storage stands there. A
/// storage with no module in its card slot is never locked: the block's byte may
/// outlive the module (deviation 9), and a lock that guards nothing must not let a
/// stack into the menu, where nothing would write it.
fn lock_at(host: &dyn Host, pos: [i32; 3]) -> Option<bool> {
    let entity = entity_at(host, pos)?;
    let data = entity.data(host);
    let module = module_at(host, entity).is_some();
    Some(module && matches!(field(data.as_ref(), KEY_LOCKED), Some(Tag::Byte(1))))
}

// --- the contents on the broken block's own item --------------------------

/// `block_drops`: a broken storage's own item carries its contents, so placing the
/// item puts them back (upstream's `ModularStorageTileEntity` keeps them for the
/// module; this port keeps them for the block).
///
/// The drop's stacks are already built and the block entity is out of the world. The
/// event's own saved data is not the break's: the engine snapshots it *before* the
/// removal, so it holds the container as it stood, with `Items`. What travels is the
/// stash [`stash_carried`] left for this break — its dimension and position, taken in
/// the same tick. An explosion's blocks are gone before the event, and a level-side
/// break (a piston, water, `/setblock destroy`) never runs the player's stash, so
/// those spill as vanilla does.
pub(crate) fn on_block_drops(level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
    if level.state_block(event.state) != Some(BLOCK) {
        return Flow::Continue;
    }
    // The saved data the event holds is the one read *before* the break's removal
    // (the engine snapshots it for `block_drops`), so the contents this break moved
    // aside are not in it: the stash `stash_carried` left beside it is.
    let pos = [event.pos.x, event.pos.y, event.pos.z];
    let Some(saved) = take_carried(event.dimension.key().id(), pos) else {
        return Flow::Continue;
    };
    let Some(patch) = carried_payload(&saved, BE_TYPE) else {
        return Flow::Continue;
    };
    let Some(item) = ITEM_KEY.get().copied() else {
        return Flow::Continue;
    };
    for drop in event.drops.iter_mut() {
        if drop.stack.item() == item {
            let _ = drop.stack.apply_components(&patch);
        }
    }
    Flow::Continue
}

/// The `minecraft:block_entity_data` patch a broken storage's item carries: the
/// block entity's saved data, tagged with its type id (vanilla loads a payload only
/// when the `id` names the block entity it is placed as), with the contents under
/// [`KEY_CARRIED`] and **never** `Items`.
///
/// `Items` must not travel: a mod container's stacks are the engine's own to spill
/// (`PreRemove::Contents`), so a payload that also carried them would put every
/// stack in the world twice. `None` when the break moved nothing aside — a plain
/// block item, which is what a creative break or a level-side break leaves.
#[must_use]
pub(crate) fn carried_payload(saved: &Compound, type_id: &str) -> Option<Compound> {
    let Some(Tag::List(entries)) = field(Some(saved), KEY_CARRIED) else {
        return None;
    };
    if entries.is_empty() {
        return None;
    }
    let mut payload = saved.clone();
    for key in ["Items", "id", "x", "y", "z"] {
        payload.remove(&key.into());
    }
    payload.insert("id".into(), Tag::String(type_id.into()));
    let mut patch = Compound::new();
    patch.insert(BLOCK_ENTITY_DATA.into(), Tag::Compound(payload));
    Some(patch)
}

/// A break's contents, waiting for the block's own drop: the block entity's saved
/// data as the break read it (with the contents under [`KEY_CARRIED`]), the level and
/// the position it came from, and when. `block_drops` takes it in the same tick; the
/// next push or lookup drops anything older than [`STASH_LIFETIME`], so a break whose
/// drop never runs cannot be attached to a later break at the same position.
struct Stash {
    dimension: &'static str,
    pos: [i32; 3],
    data: Compound,
    at: Instant,
}

/// How long an unconsumed break stash is kept.
const STASH_LIFETIME: Duration = Duration::from_secs(2);

/// The stashes of the breaks whose drops have not been built yet. Transient by
/// design: a break is answered in the same tick, and nothing survives a relaunch.
static STASHES: Mutex<Vec<Stash>> = Mutex::new(Vec::new());

/// The stash list, with a poisoned lock treated as a list (a panic in another mod
/// must not take this port's breaks with it).
fn stashes() -> std::sync::MutexGuard<'static, Vec<Stash>> {
    STASHES.lock().unwrap_or_else(|e| e.into_inner())
}

/// Keeps `data` for the drop of the break that just happened at `pos`.
fn remember(dimension: &'static str, pos: [i32; 3], data: Compound) {
    let mut guard = stashes();
    guard.retain(|stash| stash.at.elapsed() < STASH_LIFETIME);
    guard.push(Stash {
        dimension,
        pos,
        data,
        at: Instant::now(),
    });
}

/// The stash of the break at `pos` in `dimension`, consumed.
fn take_carried(dimension: &str, pos: [i32; 3]) -> Option<Compound> {
    let mut guard = stashes();
    guard.retain(|stash| stash.at.elapsed() < STASH_LIFETIME);
    let index = guard
        .iter()
        .position(|stash| stash.dimension == dimension && stash.pos == pos)?;
    Some(guard.remove(index).data)
}

/// Moves a break's contents aside and empties the container — so the engine's own
/// spill of a mod container finds nothing to scatter — and remembers the saved data
/// for [`on_block_drops`], which puts it on the block's own drop.
///
/// The stacks are written in vanilla's `Items` shape (`ItemStack::to_nbt` plus
/// `Slot`), which is what the engine's own list is. One transaction: a refused write
/// leaves the container as it was. The same compound also stays in the block
/// entity's saved data, so a removal the level refuses (the block stays) loses
/// nothing: [`adopt_carried`] puts the contents back on the next click or read.
fn stash_carried(host: &mut dyn ServerHost, pos: [i32; 3]) {
    let dimension = host.dimension();
    let Some(level) = host.level(dimension) else {
        return;
    };
    let Some(entity) = host.block_entity(level, pos) else {
        return;
    };
    let Some(saved) = entity.data(&*host) else {
        return;
    };
    let mut entries = Vec::new();
    let mut filled = Vec::new();
    for slot in 0..CONTAINER_SLOTS {
        let index = u32::try_from(slot).unwrap_or(u32::MAX);
        let Some(stack) = entity.item_slot(&*host, index).ok().flatten() else {
            continue;
        };
        let Some(mut entry) = stack.to_nbt(&*host) else {
            continue;
        };
        let Ok(slot) = i8::try_from(slot) else {
            continue;
        };
        entry.insert("Slot".into(), Tag::Byte(slot));
        entries.push(Tag::Compound(entry));
        filled.push(index);
    }
    if entries.is_empty() {
        return;
    }
    let mut carried = saved;
    carried.insert(KEY_CARRIED.into(), Tag::List(entries));
    let mut stash = Compound::new();
    if let Some(list) = field(Some(&carried), KEY_CARRIED) {
        stash.insert(KEY_CARRIED.into(), list.clone());
    }
    let written = host
        .transaction(|host, tx| -> Result<(), ()> {
            entity.merge_data(host, stash.clone()).map_err(|_| ())?;
            for slot in filled {
                entity.set_item_slot(host, tx, slot, None).map_err(|_| ())?;
            }
            let _ = entity.set_changed(host);
            Ok(())
        })
        .is_ok();
    if written {
        remember(dimension, pos, carried);
    }
}

/// Puts a carried payload's stacks back into the block's slots (the card slot
/// included) and clears the carried key, so the contents travel once.
///
/// `runtime::restored_count`/`carried_slot_in` clamp it: an item doctored through
/// `/give` may name any count in any slot, and the block keeps at most what its
/// container holds. Also runs from the click paths ([`adopt_carried`]) for a stash a
/// break left behind (a removal the level refused), so a stale stash is never lost.
fn restore_carried(host: &mut dyn ServerHost, pos: [i32; 3]) {
    let Some(level) = host.level(host.dimension()) else {
        return;
    };
    let Some(entity) = host.block_entity(level, pos) else {
        return;
    };
    let Some(data) = entity.data(&*host) else {
        return;
    };
    let Some(Tag::List(entries)) = field(Some(&data), KEY_CARRIED) else {
        return;
    };
    if entries.is_empty() {
        return;
    }
    let mut placed = Vec::new();
    for entry in entries {
        let Tag::Compound(entry) = entry else {
            continue;
        };
        let Some(slot) =
            number(field(Some(entry), "Slot")).and_then(|slot| u32::try_from(slot).ok())
        else {
            continue;
        };
        if !carried_slot_in(STORAGE_SLOTS, MODULE_SLOT, slot) {
            continue;
        }
        let Some(mut stack) = ItemStack::from_nbt(&*host, entry) else {
            continue;
        };
        let count = restored_count(stack.count(), max_stack_of(&*host, &stack));
        if count == 0 || stack.set_count(count).is_err() {
            continue;
        }
        placed.push((slot, stack));
    }
    let mut consumed = Compound::new();
    consumed.insert(KEY_CARRIED.into(), Tag::List(Vec::new()));
    let _ = host.transaction(|host, tx| -> Result<(), ()> {
        for (slot, stack) in placed {
            entity
                .set_item_slot(host, tx, slot, Some(stack))
                .map_err(|_| ())?;
        }
        entity.merge_data(host, consumed.clone()).map_err(|_| ())?;
        let _ = entity.set_changed(host);
        Ok(())
    });
}

/// [`restore_carried`] for the paths that read the block without placing it: a
/// stash a refused removal left behind comes back the next time the block is
/// opened, clicked or read, so nothing is stranded in its saved data.
fn adopt_carried(host: &mut dyn ServerHost, pos: [i32; 3]) {
    restore_carried(host, pos);
}

/// A numeric NBT tag as an integer: `Slot` and `count` are written as a byte or an
/// int depending on the writer (`save::encode_items` writes a byte slot).
fn number(tag: Option<&Tag>) -> Option<i64> {
    match tag? {
        Tag::Byte(n) => Some(i64::from(*n)),
        Tag::Short(n) => Some(i64::from(*n)),
        Tag::Int(n) => Some(i64::from(*n)),
        Tag::Long(n) => Some(*n),
        _ => None,
    }
}

// --- reading and writing the block entity ---------------------------------

/// A stack's identity: its item, damage and components (upstream's
/// `ItemStack.isSameItemSameComponents`).
fn kind_of(host: &dyn Host, stack: &ItemStack) -> Kind {
    let item = stack.item().id(host);
    let key = match stack.components() {
        Some(patch) => format!("{item}#{}#{:?}", stack.damage(), patch.as_nbt()),
        None => format!("{item}#{}#", stack.damage()),
    };
    Kind { key, label: item }
}

/// How many of `stack`'s item one slot holds: the item's own stack size, never
/// past [`SLOT_LIMIT`].
fn max_stack_of(host: &dyn Host, stack: &ItemStack) -> u32 {
    host.max_stack_size(stack.item())
        .unwrap_or(SLOT_LIMIT)
        .clamp(1, SLOT_LIMIT)
}

/// The block at `pos` is the modular storage.
fn is_storage(host: &dyn Host, pos: [i32; 3]) -> bool {
    let Some(level) = host.level(host.dimension()) else {
        return false;
    };
    level.block(host, pos) == Some(block_key())
}

/// [`is_storage`] on either side's host: the click is heard on both.
fn storage_clicked(host: &HostSide<'_>, pos: [i32; 3]) -> bool {
    match host {
        HostSide::Server(host) => is_storage(&**host, pos),
        HostSide::Client(host) => is_storage(&**host, pos),
    }
}

/// The mod block entity at `pos`, when one stands there.
fn entity_at(host: &dyn Host, pos: [i32; 3]) -> Option<BlockEntityRef> {
    let level = host.level(host.dimension())?;
    host.block_entity(level, pos)
}

/// The id of the block at `pos`, for the commands' answer.
fn block_id_at(host: &dyn Host, pos: [i32; 3]) -> String {
    host.level(host.dimension())
        .and_then(|level| level.block(host, pos))
        .map_or_else(String::new, |block| block.id(host))
}

/// The container's storage slots, read one by one (a mod block entity's item
/// storage is per slot; a refused read is an empty slot). The card slot is not
/// one of them.
fn read_slots(host: &dyn Host, entity: BlockEntityRef) -> Vec<Option<ItemStack>> {
    (0..STORAGE_SLOTS)
        .map(|slot| {
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            entity.item_slot(host, slot).ok().flatten()
        })
        .collect()
}

/// The stack in the card slot, when one is in it: the storage module whose tier
/// the storage's capacity is.
fn module_at(host: &dyn Host, entity: BlockEntityRef) -> Option<ItemStack> {
    let slot = u32::try_from(MODULE_SLOT).ok()?;
    entity.item_slot(host, slot).ok().flatten()
}

/// The tier a block saved before the card slot (0.2.2) records in its compound,
/// where the module used to live. `None` once [`adopt_legacy_module`] has moved
/// it into the slot, or for any value outside the tiers.
fn legacy_tier(data: Option<&Compound>) -> Option<u8> {
    match field(data, KEY_MODULE) {
        Some(Tag::Int(n)) if (0..=3).contains(n) => Some(*n as u8),
        _ => None,
    }
}

/// `tier`'s module item as a stack, when this game's registry has it.
fn module_stack(host: &dyn Host, tier: u8) -> Option<ItemStack> {
    let id = Identifier::parse(&module_id(tier)).ok()?;
    let item = host.item_key(&id)?;
    Some(ItemStack::of(item))
}

/// A 0.2.2 block kept its module in the saved compound; the card slot is where it
/// lives now. Moves it there once (and clears the key), so such a world keeps its
/// storage: after this, a block with an empty card slot is a block with no module.
fn adopt_legacy_module(host: &mut dyn ServerHost, entity: BlockEntityRef) {
    if module_at(&*host, entity).is_some() {
        return;
    }
    let data = entity.data(&*host);
    let Some(tier) = legacy_tier(data.as_ref()) else {
        return;
    };
    let Some(stack) = module_stack(&*host, tier) else {
        return;
    };
    let slot = u32::try_from(MODULE_SLOT).unwrap_or(u32::MAX);
    let mut cleared = Compound::new();
    cleared.insert(KEY_MODULE.into(), Tag::Int(NO_MODULE));
    let _ = host.transaction(|host, tx| -> Result<(), ()> {
        entity
            .set_item_slot(host, tx, slot, Some(stack.clone()))
            .map_err(|_| ())?;
        entity.merge_data(host, cleared.clone()).map_err(|_| ())?;
        Ok(())
    });
}

/// [`adopt_legacy_module`] for a menu button's view of the block entity, which is
/// the field set a `menu_button` call may write.
fn adopt_legacy_fields(host: &dyn Host, fields: &mut ModBlockEntityFields) {
    if fields.items.get(MODULE_SLOT).is_some_and(Option::is_some) {
        return;
    }
    let Some(tier) = legacy_tier(Some(&fields.nbt)) else {
        return;
    };
    let Some(stack) = module_stack(host, tier) else {
        return;
    };
    if fields.items.len() <= MODULE_SLOT {
        fields.items.resize(CONTAINER_SLOTS, None);
    }
    fields.items[MODULE_SLOT] = Some(stack);
    fields.nbt.insert(KEY_MODULE.into(), Tag::Int(NO_MODULE));
}

/// The storage a block entity holds: its module from the card slot (or, before
/// [`adopt_legacy_module`], from the compound), its lock from the saved compound,
/// its contents from the storage slots. The lock only counts with a module in
/// place: a card slot emptied while locked takes the lock with it, because the pin
/// gates placement only and a lock with no module could never be opened again.
fn read_storage(
    host: &dyn Host,
    data: Option<&Compound>,
    slots: &[Option<ItemStack>],
    module: Option<&ItemStack>,
) -> Storage {
    let tier = module
        .and_then(|stack| module_tier(&stack.item().id(host)))
        .or_else(|| legacy_tier(data));
    let mut storage = Storage::from_block(tier, false, vec![None; slots.len()]);
    storage.locked = tier.is_some() && matches!(field(data, KEY_LOCKED), Some(Tag::Byte(1)));
    for (index, slot) in storage.slots.iter_mut().enumerate() {
        let Some(stack) = slots.get(index).and_then(Option::as_ref) else {
            continue;
        };
        if stack.count() == 0 {
            continue;
        }
        *slot = Some(Slot {
            kind: kind_of(host, stack),
            count: stack.count(),
        });
    }
    storage
}

/// One saved field by its Java key. The compound is keyed by Java's lossless
/// UTF-16 strings, which the API does not export by name, so the lookup spells
/// the key the way Java does and compares the stored units (as the quarry port
/// does): this keeps the package on the one SDK path dependency the launcher
/// resolves by itself.
fn field<'a>(data: Option<&'a Compound>, name: &str) -> Option<&'a Tag> {
    let units: Vec<u16> = name.encode_utf16().collect();
    data?
        .iter()
        .find(|(key, _)| key.0 == units)
        .map(|(_, tag)| tag)
}

/// The stack a slot keeps when it holds `kind`: the one it had (components
/// included), or else the stack the player used (`held`).
fn template(
    host: &dyn Host,
    before: &[Option<ItemStack>],
    held: Option<&ItemStack>,
    kind: &Kind,
) -> Option<ItemStack> {
    before
        .iter()
        .flatten()
        .chain(held)
        .find(|stack| kind_of(host, stack).key == kind.key)
        .cloned()
}

/// Whether two slots hold the same stack, so an unchanged slot is not written.
fn same_stack(left: Option<&ItemStack>, right: Option<&ItemStack>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            left.count() == right.count()
                && left.item() == right.item()
                && left.damage() == right.damage()
                && left.same_components(right)
        }
        (Some(_), None) | (None, Some(_)) => false,
    }
}

/// Writes `storage` back: the lock into the saved compound and the contents into
/// the storage slots. A slot that holds a kind keeps the stack it had; a slot that
/// just filled takes the stack the player used (`held`). The card slot is never
/// written here — its module is the game's own item, moved by the player's clicks —
/// and a storage with no module writes no storage slot, so the block keeps what it
/// holds until a module is back.
fn write_storage(
    host: &mut dyn ServerHost,
    tx: &Transaction,
    entity: BlockEntityRef,
    storage: &Storage,
    before: &[Option<ItemStack>],
    held: Option<&ItemStack>,
) -> Result<(), ()> {
    let mut data = Compound::new();
    // The module is the card slot's item now; the key only carries a 0.2.2 block's
    // tier into `adopt_legacy_module`, and clearing it keeps a removed module from
    // coming back.
    data.insert(KEY_MODULE.into(), Tag::Int(NO_MODULE));
    data.insert(KEY_LOCKED.into(), Tag::Byte(i8::from(storage.locked)));
    entity.merge_data(host, data).map_err(|_| ())?;
    if storage.tier.is_none() {
        return Ok(());
    }
    let wanted: Vec<Option<ItemStack>> = (0..STORAGE_SLOTS)
        .map(|index| {
            let slot = storage.slots.get(index)?.as_ref()?;
            let mut stack = template(host, before, held, &slot.kind)?;
            stack.set_count(slot.count).ok()?;
            Some(stack)
        })
        .collect();
    for (index, wanted) in wanted.iter().enumerate() {
        if same_stack(before.get(index).and_then(Option::as_ref), wanted.as_ref()) {
            continue;
        }
        let slot = u32::try_from(index).unwrap_or(u32::MAX);
        entity
            .set_item_slot(host, tx, slot, wanted.clone())
            .map_err(|_| ())?;
    }
    // The slots changed: mark the chunk and notify comparators, as upstream's
    // `setChanged` does (the raw item write is silent by design).
    let _ = entity.set_changed(host);
    Ok(())
}

/// Reads the storage at `pos`, applies `change`, writes it back, unless the
/// change refuses (`None`): then nothing is written, so a mutation that did not
/// happen cannot erase a container. A block that is not this mod's modular
/// storage is refused before any mutable handle is taken — a foreign mod
/// container's slots are never read into this storage nor written back.
fn mutate_checked<R>(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    held: Option<&ItemStack>,
    change: impl FnOnce(&mut Storage, &[Option<ItemStack>]) -> Option<R>,
) -> Option<R> {
    if !is_storage(&*host, pos) {
        return None;
    }
    let entity = entity_at(host, pos)?;
    // One representation: a 0.2.2 block's recorded module becomes the card slot's
    // item before the change reads it, and a stash a refused removal left behind
    // goes back into the slots it came from.
    adopt_legacy_module(host, entity);
    adopt_carried(host, pos);
    let before = read_slots(host, entity);
    let data = entity.data(host);
    let module = module_at(&*host, entity);
    let mut storage = read_storage(&*host, data.as_ref(), &before, module.as_ref());
    let result = change(&mut storage, &before)?;
    host.transaction(|host, tx| write_storage(host, tx, entity, &storage, &before, held))
        .ok()?;
    Some(result)
}

/// [`mutate_checked`] for a change that always commits.
fn mutate<R>(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    held: Option<&ItemStack>,
    change: impl FnOnce(&mut Storage, &[Option<ItemStack>]) -> R,
) -> Option<R> {
    mutate_checked(host, pos, held, |storage, before| {
        Some(change(storage, before))
    })
}

/// Whether this command source may read or lock the storage at `pos`: an
/// operator (permission ≥ 2), or a player in the same level within
/// [`INSPECT_RANGE`](crate::runtime::INSPECT_RANGE) of the block.
fn may_touch(
    host: &dyn ServerHost,
    source: &CommandSource,
    level: LevelRef,
    pos: [i32; 3],
) -> bool {
    let op = source.permission().level() >= LEVEL_ADMINS;
    match source {
        CommandSource::Player { player, .. } => {
            let same_level = player.level(host) == Some(level);
            let distance = player
                .entity()
                .position(host)
                .map(|at| {
                    let block = [
                        f64::from(pos[0]) + 0.5,
                        f64::from(pos[1]) + 0.5,
                        f64::from(pos[2]) + 0.5,
                    ];
                    (0..3)
                        .map(|axis| (at[axis] - block[axis]).powi(2))
                        .sum::<f64>()
                        .sqrt()
                })
                .unwrap_or(f64::MAX);
            may_inspect(op, same_level, distance)
        }
        _ => op,
    }
}

/// A storage as it stands, with the stacks behind it.
struct Read {
    storage: Storage,
    slots: Vec<Option<ItemStack>>,
}

/// The storage at `pos` as it stands, without changing it.
fn read_at(host: &dyn Host, pos: [i32; 3]) -> Option<Read> {
    if !is_storage(host, pos) {
        return None;
    }
    let entity = entity_at(host, pos)?;
    let slots = read_slots(host, entity);
    let data = entity.data(host);
    let module = module_at(host, entity);
    Some(Read {
        storage: read_storage(host, data.as_ref(), &slots, module.as_ref()),
        slots,
    })
}

/// The stack size the readout's capacity is computed with: the first stored
/// kind's, or 64 for an empty storage (what the old `native_give_stack` path
/// used, and what upstream's `getStackLimit` clamps to anyway).
fn read_max_stack(host: &dyn Host, read: &Read) -> u32 {
    read.slots
        .iter()
        .flatten()
        .next()
        .map_or(SLOT_LIMIT, |stack| max_stack_of(host, stack))
}

/// The readout for a storage's state, as the action bar shows it.
fn readout(storage: &Storage, max_stack: u32) -> String {
    if storage.tier.is_none() {
        return format!("{READOUT}no storage module");
    }
    if !storage.locked {
        return format!("{READOUT}unlocked — press the padlock to lock");
    }
    format!("{READOUT}{}", storage.summary(max_stack))
}

/// Answers in the acting player's action bar.
fn action_bar(host: &mut dyn ServerHost, uuid: u128, line: String) {
    let Some(player) = host.player(uuid) else {
        return;
    };
    let _ = player.action_bar(host, Text::literal(line));
}

/// Takes `taken` items off the acting player's held stack.
fn take_from_hand(host: &mut dyn ServerHost, uuid: u128, held: &ItemStack, taken: u32) {
    let Some(player) = host.player(uuid) else {
        return;
    };
    let Some(slot) = player.selected_slot(host) else {
        return;
    };
    let remainder = if held.count() > taken {
        let mut rest = held.clone();
        let _ = rest.set_count(held.count() - taken);
        Some(rest)
    } else {
        None
    };
    let _ = player.set_inventory_slot(host, u32::try_from(slot).unwrap_or(u32::MAX), remainder);
}

// --- the player's right click ---------------------------------------------

/// Right click: a **plain** click is the block's own use, which opens the
/// storage's menu on the server — upstream's GUI on any face. While **sneaking**
/// the click is the quick gesture instead: a storage module in hand installs, a
/// held stack inserts, and an empty hand seals an unlocked block or takes a stack
/// out of a locked one.
///
/// The hook is heard on the clicking client **and** on the server (once per
/// hand). Either path consumes the click on both sides — `Success` ends it as
/// used, so neither the block's own use nor the held item's runs after it (the
/// default `Pass` would let a block in hand be placed behind the insert) — but
/// only the server touches the world: it opens the menu, or runs the gesture.
pub(crate) fn on_right_click_block(
    host: HostSide<'_>,
    event: &mut RightClickBlockEvent<'_>,
) -> Flow {
    let click = event.click;
    if !storage_clicked(&host, click.pos) {
        return Flow::Continue;
    }
    // The hook fires once per hand and the off hand follows the main hand in
    // the same click: this block's gestures are the main hand's, so the second
    // call does nothing (and is not cancelled, so the off hand's own use is
    // untouched).
    if click.hand != Hand::Main {
        return Flow::Continue;
    }
    if !sneaking(&host, click.player) {
        return open_storage(host, event, click);
    }
    if let HostSide::Server(host) = host {
        match click.stack {
            None => empty_hand(host, click.pos, click.player),
            Some(stack) if module_tier(&stack.item().id(host)).is_some() => {
                install(host, click.pos, click.player, stack);
            }
            Some(stack) => insert(host, click.pos, click.player, stack),
        }
    }
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
}

/// A plain click opens the storage's menu (`Player.openMenu` against its block
/// entity): the block's template has no use of its own, so the mod asks for the
/// menu itself, as a block's `getMenuProvider` would. Both sides consume the
/// click so the held item's use is not predicted behind it; only the server
/// opens anything, and a refused open leaves the click to the game.
fn open_storage(
    host: HostSide<'_>,
    event: &mut RightClickBlockEvent<'_>,
    click: &mod_api::BlockClick<'_>,
) -> Flow {
    let HostSide::Server(server) = host else {
        *event.cancellation_result = InteractionResult::Success;
        return Flow::Cancel;
    };
    let Some(player) = server.player(click.player) else {
        return Flow::Continue;
    };
    let Some(entity) = entity_at(&*server, click.pos) else {
        return Flow::Continue;
    };
    let opened = player.open_menu(server, entity).unwrap_or(false);
    if !opened {
        return Flow::Continue;
    }
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
}

/// Whether the clicking player holds sneak (`Player.isSecondaryUseActive`), on
/// either side's host.
fn sneaking(host: &HostSide<'_>, uuid: u128) -> bool {
    match host {
        HostSide::Server(server) => player_sneaks(&**server, uuid),
        HostSide::Client(client) => player_sneaks(*client, uuid),
    }
}

/// `Player.isSecondaryUseActive()` for the player `uuid`, read on `host`.
fn player_sneaks(host: &dyn Host, uuid: u128) -> bool {
    host.player(uuid)
        .and_then(|player| player.is_secondary_use_active(host))
        .unwrap_or(false)
}

/// An empty hand is the padlock's quick path while sneaking: it seals a
/// module-bearing unlocked storage (the state the menu's button sets), and from
/// a locked one it takes a stack out.
fn empty_hand(host: &mut dyn ServerHost, pos: [i32; 3], uuid: u128) {
    let unlocked =
        read_at(host, pos).is_some_and(|read| read.storage.tier.is_some() && !read.storage.locked);
    if unlocked {
        let toggled = mutate(host, pos, None, |storage, _| storage.toggle_lock()).unwrap_or(false);
        let line = match (toggled, read_at(host, pos)) {
            (true, Some(read)) => format!("{READOUT}{}", read.storage.state_word()),
            (false, _) => format!("{READOUT}insert a storage module first"),
            (true, None) => format!("{READOUT}not a modular storage"),
        };
        action_bar(host, uuid, line);
        return;
    }
    withdraw(host, pos, uuid);
}

/// A storage module in hand goes into the block's **card slot** — upstream's
/// `SLOT_STORAGE_MODULE` — only while unlocked and empty (its `mayPlace`). The
/// item stays there as the storage's module, so the menu can take it back out.
fn install(host: &mut dyn ServerHost, pos: [i32; 3], uuid: u128, held: &ItemStack) {
    let state = read_at(host, pos).map(|read| read.storage);
    let placed = match state {
        None => false,
        Some(storage) if storage.locked => false,
        Some(storage) if storage.tier.is_some() => false,
        Some(_) => put_card(host, pos, held),
    };
    let line = if placed {
        take_from_hand(host, uuid, held, 1);
        match read_at(host, pos) {
            Some(read) => format!(
                "{READOUT}module installed, {} slots",
                read.storage.slot_count()
            ),
            None => format!("{READOUT}module installed"),
        }
    } else {
        match read_at(host, pos) {
            Some(read) if read.storage.locked => format!("{READOUT}locked — unlock it first"),
            Some(_) => format!("{READOUT}a module is already in"),
            None => format!("{READOUT}not a modular storage"),
        }
    };
    action_bar(host, uuid, line);
}

/// Puts one of `held` into the card slot (the card holds a whole module: the
/// items are `stacksTo(1)`), and clears a 0.2.2 block's recorded tier so the two
/// cannot disagree.
fn put_card(host: &mut dyn ServerHost, pos: [i32; 3], held: &ItemStack) -> bool {
    let Some(entity) = entity_at(host, pos) else {
        return false;
    };
    let mut one = held.clone();
    if one.set_count(1).is_err() {
        return false;
    }
    let slot = u32::try_from(MODULE_SLOT).unwrap_or(u32::MAX);
    let mut data = Compound::new();
    data.insert(KEY_MODULE.into(), Tag::Int(NO_MODULE));
    host.transaction(|host, tx| -> Result<(), ()> {
        entity
            .set_item_slot(host, tx, slot, Some(one.clone()))
            .map_err(|_| ())?;
        entity.merge_data(host, data.clone()).map_err(|_| ())?;
        let _ = entity.set_changed(host);
        Ok(())
    })
    .is_ok()
}

/// The held stack goes in, as much as fits.
fn insert(host: &mut dyn ServerHost, pos: [i32; 3], uuid: u128, held: &ItemStack) {
    let kind = kind_of(host, held);
    let max_stack = max_stack_of(host, held);
    let inserted = mutate(host, pos, Some(held), |storage, _| {
        storage.insert(&kind, held.count(), max_stack)
    })
    .unwrap_or(0);
    if inserted > 0 {
        take_from_hand(host, uuid, held, inserted);
    }
    let line = match read_at(host, pos) {
        None => format!("{READOUT}not a modular storage"),
        Some(read) if read.storage.tier.is_none() => format!("{READOUT}no storage module"),
        Some(read) if !read.storage.locked => {
            format!("{READOUT}unlocked — press the padlock to lock")
        }
        Some(read) if inserted == 0 && read.storage.is_full_for(&kind, max_stack) => {
            format!(
                "{READOUT}full, nothing fits ({})",
                read.storage.summary(max_stack)
            )
        }
        Some(read) if inserted == 0 => {
            format!(
                "{READOUT}no room for it ({})",
                read.storage.summary(max_stack)
            )
        }
        Some(read) => readout(&read.storage, max_stack),
    };
    action_bar(host, uuid, line);
}

/// An empty-hand click takes a stack of the first stored kind, into the
/// player's inventory; a refused give leaves it in the block.
fn withdraw(host: &mut dyn ServerHost, pos: [i32; 3], uuid: u128) {
    let max_stack = read_at(host, pos).map_or(SLOT_LIMIT, |read| read_max_stack(host, &read));
    let taken = mutate(host, pos, None, |storage, before| {
        let taken = storage.extract_first(max_stack)?;
        let mut stack = before.get(taken.slot).and_then(Option::clone)?;
        stack.set_count(taken.count).ok()?;
        Some(stack)
    })
    .flatten();
    let Some(stack) = taken else {
        let line = match read_at(host, pos) {
            None => format!("{READOUT}not a modular storage"),
            Some(read) if read.storage.tier.is_none() => format!("{READOUT}no storage module"),
            Some(read) if !read.storage.locked => {
                format!("{READOUT}unlocked — press the padlock to lock")
            }
            Some(_) => format!("{READOUT}empty"),
        };
        action_bar(host, uuid, line);
        return;
    };
    let item = stack.item().id(host);
    let given = stack.count();
    let player = host.player(uuid);
    let given_ok = match player {
        Some(player) => player.give_stack(host, stack.clone()).is_ok(),
        None => false,
    };
    if !given_ok {
        // Nothing reached the player: put the stack back where it came from.
        let kind = kind_of(host, &stack);
        mutate(host, pos, Some(&stack), |storage, _| {
            storage.insert(&kind, given, max_stack);
        });
    }
    let line = match read_at(host, pos) {
        Some(read) => format!(
            "{READOUT}took {} {} ({})",
            if given_ok { given } else { 0 },
            item,
            read.storage.summary(max_stack)
        ),
        None => format!("{READOUT}took {item}"),
    };
    action_bar(host, uuid, line);
}

// --- the storage scanner ---------------------------------------------------

/// Whether the storage scanner stands at `pos`.
fn is_scanner(host: &dyn Host, pos: [i32; 3]) -> bool {
    host.level(host.dimension())
        .and_then(|level| level.block(host, pos))
        == Some(scanner_key())
}

/// The scanner's block entity at `pos`, when the scanner stands there.
fn scanner_entity(host: &dyn Host, pos: [i32; 3]) -> Option<BlockEntityRef> {
    if !is_scanner(host, pos) {
        return None;
    }
    entity_at(host, pos)
}

/// A saved non-negative count (`energy`), 0 when the key is missing or is not one.
fn saved_count(data: Option<&Compound>, key: &str) -> u32 {
    number(field(data, key))
        .filter(|value| *value > 0)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0)
}

/// The scanner's saved radius: `StorageScannerData.radius`, 1 when the block has
/// never been given one, always inside the GUI slider's range.
fn saved_radius(data: Option<&Compound>) -> i32 {
    scanner_radius(number(field(data, KEY_RADIUS)).and_then(|value| i32::try_from(value).ok()))
}

/// `/rftoolsstorage scan <pos>`: what the inventories in the scanner's box hold.
///
/// Upstream's GUI lists them for free and charges `rfPerRequest` when the player
/// pulls an item out; this port has no list widget (a gap, `notes.md`), so the
/// readout is the whole player surface and it costs the request's price. With
/// less than that in the buffer the scan is refused with `no power` and nothing
/// is read or spent.
fn scan_readout(host: &mut dyn ServerHost, pos: [i32; 3]) -> String {
    let block = block_id_at(&*host, pos);
    if !is_storage_scanner(&block) {
        return format!("{block}: not a storage scanner");
    }
    let Some(entity) = entity_at(&*host, pos) else {
        return format!("{block}: not a storage scanner");
    };
    let data = entity.data(&*host);
    let stored = saved_count(data.as_ref(), KEY_ENERGY);
    let Some(after) = spend_power(stored) else {
        return format!("{block}: {}", no_power_line(stored));
    };
    let found = scan_found(host, pos, saved_radius(data.as_ref()));
    if let Ok(after) = i32::try_from(after) {
        let mut spent = Compound::new();
        spent.insert(KEY_ENERGY.into(), Tag::Int(after));
        let _ = entity.merge_data(host, spent);
    }
    format!("{block}: {}", scan_line(after, &found))
}

/// The kinds every inventory in the scanner's box holds, one entry a kind: a
/// neighbour's item capability (the game's own answer for a chest, a barrel or a
/// mod container), read slot by slot and summed. The scanner's own cell and any
/// other storage scanner are skipped, as upstream skips its own tile entity; a
/// position with no inventory, in an unloaded chunk or in a level that is not in
/// place answers nothing.
fn scan_found(host: &mut dyn ServerHost, pos: [i32; 3], radius: i32) -> Vec<Found> {
    let Some(level) = host.level(host.dimension()) else {
        return Vec::new();
    };
    let mut entries: Vec<(String, u32)> = Vec::new();
    for at in scan_positions(pos, radius) {
        if at == pos || level.block(&*host, at) == Some(scanner_key()) {
            continue;
        }
        let Some(handler) = level.capability(&mut *host, &ITEMS, at, None) else {
            continue;
        };
        let size = handler.size(host);
        for index in 0..size {
            let Some(resource) = handler.resource(host, index) else {
                continue;
            };
            let count = handler.amount(host, index);
            if count > 0 {
                entries.push((resource.item().id(&*host), count));
            }
        }
    }
    merge_counts(&entries)
}

/// `/rftoolsstorage request <pos> <item> [<count>]`: the GUI's request path
/// (`StorageScannerTileEntity.requestStack`) reached through a command.
///
/// The reference's order is kept: the price first (an explicit count is a tenth
/// of `rfPerRequest`, a whole stack the full price), the buffer check before the
/// walk, the **out slot's guard** before anything is pulled, then the scanned
/// inventories, and the charge only when something came out. With the
/// reference's `requestStraightToInventory` default (true) a player's request
/// goes straight into their inventory (`give_stack`, as `/give`), and the out
/// slot is written only when there is no player to give it to or the give is
/// refused — the reference's own fallback.
fn request_readout(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    requested: &ItemStack,
    explicit: Option<u32>,
) -> String {
    let block = block_id_at(&*host, pos);
    if !is_storage_scanner(&block) {
        return format!("{block}: not a storage scanner");
    }
    let Some(entity) = scanner_entity(&*host, pos) else {
        return format!("{block}: not a storage scanner");
    };
    let data = entity.data(&*host);
    let stored = saved_count(data.as_ref(), KEY_ENERGY);
    let radius = saved_radius(data.as_ref());
    let max_stack = max_stack_of(&*host, requested);
    let item = requested.item().id(&*host);
    let Some(wanted) = request_amount(explicit, max_stack) else {
        return format!("{block}: a request of 0 pulls nothing ({stored}/{SCANNER_MAX_ENERGY} FE)");
    };
    let price = request_price(explicit.is_some());
    if stored < price {
        return format!(
            "{block}: {}",
            request_no_power_line(stored, price, wanted, &item)
        );
    }
    let resource = ItemResource::of(requested);
    let out_slot = u32::try_from(SCANNER_OUT_SLOT).unwrap_or(u32::MAX);
    let out_stack = entity.item_slot(&*host, out_slot).ok().flatten();
    let out_count = out_stack.as_ref().map_or(0, ItemStack::count);
    let same_kind = out_stack
        .as_ref()
        .is_some_and(|held| resource.matches(held));
    let Some(room) = out_slot_room(
        out_stack.as_ref().map(|_| (same_kind, out_count)),
        max_stack,
    ) else {
        let held = out_stack
            .as_ref()
            .map_or_else(String::new, |held| held.item().id(&*host));
        return format!("{block}: {}", request_out_slot_line(stored, &held));
    };
    let wanted = wanted.min(room);
    let pulled = pull_request(host, pos, radius, &resource, wanted);
    let RequestCharge::Paid(after) = request_charge(stored, price, pulled) else {
        // The buffer was checked above; a walk that pulled nothing is the only
        // other refusal, and it spends nothing.
        return format!("{block}: {}", request_nothing_line(stored, &item));
    };
    let mut spent = Compound::new();
    spent.insert(
        KEY_ENERGY.into(),
        Tag::Int(i32::try_from(after).unwrap_or(i32::MAX)),
    );
    let _ = entity.merge_data(&mut *host, spent);
    let Some(stack) = resource.to_stack(out_count.saturating_add(pulled)) else {
        return format!("{block}: {}", request_line(pulled, &item, after));
    };
    // The reference's `requestStraightToInventory` default: into the player, and
    // the out slot stays empty. A console's request (or a refused give) keeps the
    // stack in the out slot instead.
    let given = match source {
        CommandSource::Player { player, .. } => {
            player.give_stack(&mut *host, stack.clone()).is_ok()
        }
        _ => false,
    };
    let out_after = if given { None } else { Some(stack) };
    let _ = host.transaction(|host, tx| entity.set_item_slot(host, tx, out_slot, out_after));
    format!("{block}: {}", request_line(pulled, &item, after))
}

/// Pulls up to `amount` of `resource` out of the scanned inventories, in scan
/// order, the own cell and another scanner skipped as the inject walk skips them.
/// One transaction: a handler that refuses part-way leaves the inventories as
/// they were and nothing is pulled.
fn pull_request(
    host: &mut dyn ServerHost,
    own: [i32; 3],
    radius: i32,
    resource: &ItemResource,
    amount: u32,
) -> u32 {
    if amount == 0 {
        return 0;
    }
    let Some(level) = host.level(host.dimension()) else {
        return 0;
    };
    let targets = scan_targets(&*host, level, own, radius);
    host.transaction(|host, tx| -> Result<u32, ()> {
        let mut pulled = 0_u32;
        for at in &targets {
            if pulled >= amount {
                break;
            }
            let Some(handler) = level.capability(host, &ITEMS, *at, None) else {
                continue;
            };
            pulled =
                pulled.saturating_add(handler.extract_any(host, tx, resource, amount - pulled));
        }
        Ok(pulled)
    })
    .unwrap_or(0)
}

/// The positions a pull may take from: the scanner's box in scan order, its own cell
/// and another storage scanner skipped (`injectable_at`, upstream's
/// `findInventories`).
fn scan_targets(host: &dyn Host, level: LevelRef, own: [i32; 3], radius: i32) -> Vec<[i32; 3]> {
    scan_positions(own, radius)
        .into_iter()
        .filter(|at| injectable_at(own, *at, scanner_stands(host, level, *at)))
        .collect()
}

/// `/rftoolsstorage give <pos> <item> [single]`: the storage control screen module's
/// click on a listed stack (`StorageControlScreenModule` →
/// `StorageScannerTileEntity.giveToPlayerFromScreen`) reached through a command.
///
/// The reference's order is kept: the **full** price (`rfPerRequest`, even for a
/// single item) before anything is touched, the scan walk's pull, and the charge only
/// when something came out. There is **no out slot** on this path at all — no guard
/// and no fallback: the pulled stack goes straight into the player's inventory (as
/// `/give`: what does not fit is thrown from their eyes), so a source with no player
/// is refused before a single item leaves an inventory.
fn give_readout(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    requested: &ItemStack,
    mode: GiveAmount,
) -> String {
    let block = block_id_at(&*host, pos);
    if !is_storage_scanner(&block) {
        return format!("{block}: not a storage scanner");
    }
    let Some(entity) = scanner_entity(&*host, pos) else {
        return format!("{block}: not a storage scanner");
    };
    let CommandSource::Player { player, .. } = source else {
        return format!(
            "{block}: a give hands the stack to the player; there is no console source"
        );
    };
    let data = entity.data(&*host);
    let stored = saved_count(data.as_ref(), KEY_ENERGY);
    let radius = saved_radius(data.as_ref());
    let max_stack = max_stack_of(&*host, requested);
    let item = requested.item().id(&*host);
    let wanted = give_amount(mode, max_stack);
    let price = give_price();
    if stored < price {
        return format!(
            "{block}: {}",
            request_no_power_line(stored, price, wanted, &item)
        );
    }
    let resource = ItemResource::of(requested);
    let Some(pulled) = pull_to_player(host, pos, radius, &resource, wanted, *player) else {
        return format!("{block}: {}", give_nothing_line(stored, &item));
    };
    let RequestCharge::Paid(after) = request_charge(stored, price, pulled) else {
        return format!("{block}: {}", give_nothing_line(stored, &item));
    };
    let mut spent = Compound::new();
    spent.insert(
        KEY_ENERGY.into(),
        Tag::Int(i32::try_from(after).unwrap_or(i32::MAX)),
    );
    let _ = entity.merge_data(&mut *host, spent);
    format!("{block}: {}", gave_line(pulled, &item, after))
}

/// Pulls up to `amount` of `resource` out of the scanned inventories straight into
/// `player`, in **one** transaction: a give the player refuses aborts the whole walk,
/// so no inventory loses a stack that never reached them. `None` when the level is
/// gone, or when the walk pulled nothing.
fn pull_to_player(
    host: &mut dyn ServerHost,
    own: [i32; 3],
    radius: i32,
    resource: &ItemResource,
    amount: u32,
    player: PlayerRef,
) -> Option<u32> {
    if amount == 0 {
        return None;
    }
    let level = host.level(host.dimension())?;
    let targets = scan_targets(&*host, level, own, radius);
    host.transaction(|host, tx| -> Result<u32, ()> {
        let mut pulled = 0_u32;
        for at in &targets {
            if pulled >= amount {
                break;
            }
            let Some(handler) = level.capability(host, &ITEMS, *at, None) else {
                continue;
            };
            pulled =
                pulled.saturating_add(handler.extract_any(host, tx, resource, amount - pulled));
        }
        if pulled == 0 {
            return Ok(0);
        }
        // A give only pulls what it can hand over: a stack the game cannot build
        // refunds the walk rather than dropping what the inventories gave up.
        let stack = resource.to_stack(pulled).ok_or(())?;
        player.give_stack(host, stack).map_err(|_| ())?;
        Ok(pulled)
    })
    .ok()
}

/// What a screen insert moved: how many items, and whether that emptied the stack
/// the player handed over (the only moment it is charged).
struct Inserted {
    moved: u32,
    emptied: bool,
}

/// A screen insert resolved by the readout: the scan's box, the item the player
/// hands over and how many of it to move.
struct Insertion<'a> {
    own: [i32; 3],
    radius: i32,
    resource: &'a ItemResource,
    amount: u32,
}

/// `/rftoolsstorage insert <pos> <item> [count]`: the storage control screen
/// module's insert button (`StorageScannerTileEntity.injectStackFromScreen`)
/// reached through a command. The player hands a stack over, the walk pushes it
/// into the scanned inventories, and the remainder stays in that inventory slot.
///
/// The reference's order is kept: `rfPerInsert` in the buffer first, then
/// `checkForRoutableInventories`, then the walk — and the price is charged **only
/// when the handed stack emptied** (`if (stack.isEmpty()) consumeEnergy(…);`), so a
/// partial insert is free. There is no out slot: this path is the player's
/// inventory, not the scanner's container.
fn insert_readout(
    host: &mut dyn ServerHost,
    source: &CommandSource,
    pos: [i32; 3],
    requested: &ItemStack,
    explicit: Option<u32>,
) -> String {
    let block = block_id_at(&*host, pos);
    if !is_storage_scanner(&block) {
        return format!("{block}: not a storage scanner");
    }
    let Some(entity) = scanner_entity(&*host, pos) else {
        return format!("{block}: not a storage scanner");
    };
    let CommandSource::Player { player, .. } = source else {
        return format!(
            "{block}: an insert hands the player's stack over; there is no console source"
        );
    };
    let data = entity.data(&*host);
    let stored = saved_count(data.as_ref(), KEY_ENERGY);
    let radius = saved_radius(data.as_ref());
    let resource = ItemResource::of(requested);
    let item = requested.item().id(&*host);
    let Some((slot, stack)) = player_stack(&*host, *player, &resource) else {
        return format!("{block}: {}", insert_nothing_line(stored, &item));
    };
    let Some(wanted) = insert_amount(explicit, stack.count()) else {
        return format!("{block}: a count of 0 inserts nothing ({stored}/{SCANNER_MAX_ENERGY} FE)");
    };
    let price = insert_price();
    if stored < price {
        return format!(
            "{block}: {}",
            request_no_power_line(stored, price, wanted, &item)
        );
    }
    if !has_routable(host, pos, radius) {
        return format!("{block}: {}", no_routable_line(stored));
    }
    let Some(inserted) = insert_to_inventories(
        host,
        *player,
        slot,
        &stack,
        &Insertion {
            own: pos,
            radius,
            resource: &resource,
            amount: wanted,
        },
    ) else {
        return format!("{block}: {}", insert_nothing_line(stored, &item));
    };
    match insert_charge(stored, price, inserted.emptied) {
        InsertCharge::NoPower => format!(
            "{block}: {}",
            request_no_power_line(stored, price, wanted, &item)
        ),
        InsertCharge::Free(after) => format!(
            "{block}: {}",
            insert_partial_line(inserted.moved, wanted, &item, after)
        ),
        InsertCharge::Paid(after) => {
            let mut spent = Compound::new();
            spent.insert(
                KEY_ENERGY.into(),
                Tag::Int(i32::try_from(after).unwrap_or(i32::MAX)),
            );
            let _ = entity.merge_data(&mut *host, spent);
            format!("{block}: {}", insert_line(inserted.moved, &item, after))
        }
    }
}

/// The player's stack of `resource` an insert takes: the selected hotbar slot when
/// it holds one (the GUI's click is on the stack in hand), else the lowest matching
/// slot. Item and components match, as the request and give paths do.
fn player_stack(
    host: &dyn Host,
    player: PlayerRef,
    resource: &ItemResource,
) -> Option<(u32, ItemStack)> {
    let stacks = player.inventory_slots(host)?;
    let holds = |stack: &ItemStack| stack.count() > 0 && resource.matches(stack);
    if let Some(slot) = player
        .selected_slot(host)
        .and_then(|at| u32::try_from(at).ok())
        && let Some((_, stack)) = stacks
            .iter()
            .find(|(at, stack)| *at == slot && holds(stack))
    {
        return Some((slot, stack.clone()));
    }
    stacks.into_iter().find(|(_, stack)| holds(stack))
}

/// Upstream's `checkForRoutableInventories`: at least one scanned cell has an
/// inventory. The port reads "routable" as every scanned inventory (slice 6's
/// deviation 1 — there is no star list), and `scan_targets` already skips its own
/// cell and another storage scanner.
fn has_routable(host: &mut dyn ServerHost, own: [i32; 3], radius: i32) -> bool {
    let Some(level) = host.level(host.dimension()) else {
        return false;
    };
    let targets = scan_targets(&*host, level, own, radius);
    targets
        .iter()
        .any(|at| level.capability(host, &ITEMS, *at, None).is_some())
}

/// Hands `amount` of `resource` from the player's slot `slot` to the scanned
/// inventories in scan order, in **one** transaction: the walk's writes and the
/// remainder written back into the player's slot commit together, so a refused
/// write leaves every inventory and the player's stack as they were. `None` when
/// the level is gone or the amount is nothing.
fn insert_to_inventories(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    slot: u32,
    stack: &ItemStack,
    insertion: &Insertion<'_>,
) -> Option<Inserted> {
    let Insertion {
        own,
        radius,
        resource,
        amount,
    } = *insertion;
    if amount == 0 {
        return None;
    }
    let level = host.level(host.dimension())?;
    let targets = scan_targets(&*host, level, own, radius);
    host.transaction(|host, tx| -> Result<Inserted, ()> {
        let mut moved = 0_u32;
        for at in &targets {
            if moved >= amount {
                break;
            }
            let Some(handler) = level.capability(host, &ITEMS, *at, None) else {
                continue;
            };
            moved = moved.saturating_add(handler.insert_any(host, tx, resource, amount - moved));
        }
        if moved == 0 {
            // Nothing accepted it: the player keeps the whole stack.
            return Ok(Inserted {
                moved: 0,
                emptied: false,
            });
        }
        let left = stack.count().saturating_sub(moved);
        let remainder = if left == 0 {
            None
        } else {
            let mut rest = stack.clone();
            rest.set_count(left).map_err(|_| ())?;
            Some(rest)
        };
        player
            .set_inventory_slot(host, slot, remainder)
            .map_err(|_| ())?;
        Ok(Inserted {
            moved,
            emptied: moved == amount,
        })
    })
    .ok()
}

/// `/rftoolsstorage radius <pos> <radius>`: the GUI's radius slider, which this
/// slice has no screen for. Saves the clamped value in the scanner's compound and
/// answers with it.
fn set_radius(host: &mut dyn ServerHost, pos: [i32; 3], wanted: i32) -> Option<i32> {
    let entity = scanner_entity(&*host, pos)?;
    let radius = clamp_radius(wanted);
    let mut data = Compound::new();
    data.insert(KEY_RADIUS.into(), Tag::Int(radius));
    entity.merge_data(host, data).ok()?;
    Some(radius)
}

/// The level that is ticking: a block entity's tick runs with its own level in
/// place, which is not necessarily the dimension a command would act in.
fn ticking_level(host: &dyn ServerHost) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|level| level.in_place(host))
}

/// Whether another storage scanner stands at `at` in `level`: the inject walk skips
/// it, as upstream's `findInventories` keeps no `StorageScannerTileEntity`.
fn scanner_stands(host: &dyn Host, level: LevelRef, at: [i32; 3]) -> bool {
    level.block(host, at) == Some(scanner_key())
}

/// One scanner tick (`StorageScannerTileEntity.tickServer`): [`inject_plan`] decides
/// which of the two input slots this tick drains, each attempt walks the scan in
/// scan order and pushes the stack into the inventories that take it, and every
/// attempt costs `rfPerInsert` — charged even when nothing accepted it, as upstream
/// charges. True when the tick touched the buffer or a slot.
fn tick_scanner(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    saved: &mut Tag,
    items: &mut [Option<ItemStack>],
) -> bool {
    let data = match &*saved {
        Tag::Compound(nbt) => Some(nbt),
        _ => None,
    };
    let stored = saved_count(data, KEY_ENERGY);
    let radius = saved_radius(data);
    let in_filled = items.get(SCANNER_IN_SLOT).is_some_and(Option::is_some);
    let auto_filled = items.get(SCANNER_AUTO_SLOT).is_some_and(Option::is_some);
    let plan = inject_plan(stored, in_filled, auto_filled);
    if plan.is_empty() {
        return false;
    }
    let Some(level) = ticking_level(&*host) else {
        return false;
    };
    let mut left = stored;
    for step in plan {
        let Some(cell) = items.get_mut(step.index()) else {
            continue;
        };
        let Some(stack) = cell.take() else {
            continue;
        };
        if stack.count() == 0 {
            continue;
        }
        *cell = inject_stack(host, level, pos, radius, stack);
        left = left.saturating_sub(SCANNER_RF_PER_INSERT);
    }
    if let Tag::Compound(nbt) = saved
        && let Ok(left) = i32::try_from(left)
    {
        nbt.insert(KEY_ENERGY.into(), Tag::Int(left));
    }
    true
}

/// Pushes one stack into the scanned inventories, in scan order, and answers what
/// stayed: the cell keeps the remainder, `None` when the whole stack went in.
/// Upstream's `injectStackInternal` walk — the own cell and another scanner are
/// skipped, each inventory takes what it will and the next one gets the rest — with
/// every neighbour's write in one transaction, so a refused write leaves each
/// inventory as it was and the stack whole.
fn inject_stack(
    host: &mut dyn ServerHost,
    level: LevelRef,
    own: [i32; 3],
    radius: i32,
    stack: ItemStack,
) -> Option<ItemStack> {
    let resource = ItemResource::of(&stack);
    let total = stack.count();
    let targets = scan_targets(&*host, level, own, radius);
    let outcome = host.transaction(|host, tx| -> Result<Option<ItemStack>, ()> {
        let mut moved = 0_u32;
        for at in &targets {
            if moved >= total {
                break;
            }
            let Some(handler) = level.capability(host, &ITEMS, *at, None) else {
                continue;
            };
            moved = moved.saturating_add(handler.insert_any(host, tx, &resource, total - moved));
        }
        if moved == 0 {
            // Nothing accepted it: the slot keeps the stack it holds.
            return Ok(Some(stack.clone()));
        }
        if moved >= total {
            return Ok(None);
        }
        // A partial remainder must be buildable, or the whole move is undone rather
        // than silently dropping what the inventories did take.
        resource.to_stack(total - moved).map(Some).ok_or(())
    });
    match outcome {
        Ok(rest) => rest,
        Err(()) => Some(stack),
    }
}

// --- the readout commands --------------------------------------------------

/// The `item` argument of a `/rftoolsstorage` usage, from the dispatcher's parsed
/// value.
///
/// It is a macro because `uses.json` is a source-text scan: a bare
/// `CommandArgumentValue::Item(..)` pattern is recorded as a build of the SDK's
/// *content declaration* type `Item` — a type this port never touches — and
/// `fits` then refuses every build whose `Item` differs. The scan does not
/// expand macros, so this match keeps the compatibility table true (the
/// precedent is `ports/toolbelt`, and `ports/artifacts` renamed a local `Config`
/// for the same collision).
macro_rules! item_argument {
    ($value:expr) => {
        match $value {
            Some(mod_api::CommandArgumentValue::Item(stack)) => Some(stack),
            _ => None,
        }
    };
}

/// `/rftoolsstorage info`, `lock`, `scan`, `radius`, `request`, `give` and
/// `insert` (`register_commands`).
pub(crate) fn on_register_commands(event: &mut RegisterCommandsEvent<'_>) {
    event
        .commands
        .push(Command::new("rftoolsstorage info <pos:block_pos>"));
    event
        .commands
        .push(Command::new("rftoolsstorage lock <pos:block_pos>"));
    event
        .commands
        .push(Command::new("rftoolsstorage scan <pos:block_pos>"));
    event.commands.push(Command::new(
        "rftoolsstorage radius <pos:block_pos> <radius:integer>",
    ));
    event.commands.push(Command::new(
        "rftoolsstorage request <pos:block_pos> <item:item> [count:integer]",
    ));
    event.commands.push(Command::new(
        "rftoolsstorage give <pos:block_pos> <item:item> [single:word]",
    ));
    event.commands.push(Command::new(
        "rftoolsstorage insert <pos:block_pos> <item:item> [count:integer]",
    ));
}

/// The `/rftoolsstorage` commands' bodies (`run_mod_command`).
pub(crate) fn on_run_mod_command(
    host: &mut dyn ServerHost,
    event: &mut RunModCommandEvent<'_>,
) -> Flow {
    let [root, sub] = event.command.literals.as_slice() else {
        return Flow::Continue;
    };
    if root != "rftoolsstorage" {
        return Flow::Continue;
    }
    let Some(CommandArgumentValue::BlockPos(pos)) = event.command.argument("pos") else {
        feedback(
            host,
            "usage: /rftoolsstorage info|lock|scan|radius|request|give|insert <x> <y> <z> …",
        );
        return Flow::Cancel;
    };
    let pos = *pos;
    let Some(level) = host.level(host.dimension()) else {
        return Flow::Continue;
    };
    // The readout and the lock are player commands, not administrative ones:
    // gate them like a reach, so another player's storage is not read or
    // changed from across the world.
    if !may_touch(&*host, event.source, level, pos) {
        feedback(
            host,
            &format!(
                "You must be an operator or within {:.0} blocks of the storage.",
                crate::runtime::INSPECT_RANGE
            ),
        );
        return Flow::Cancel;
    }
    let block = block_id_at(host, pos);
    let line = match sub.as_str() {
        "lock" => {
            // A storage with no module refuses the toggle; then nothing is
            // written (the old path wrote the empty decode back and wiped a
            // foreign container's slots).
            let toggled = mutate_checked(host, pos, None, |storage, _| {
                storage.toggle_lock().then_some(())
            })
            .is_some();
            match (toggled, read_at(host, pos)) {
                (false, Some(read)) if read.storage.tier.is_none() => {
                    format!("{block}: no module to lock")
                }
                (false, _) => format!("{block}: not a modular storage"),
                (true, Some(read)) => {
                    // An open menu's padlock reads the data slot: keep it true.
                    if let Some(entity) = entity_at(host, pos) {
                        let _ = entity.set_data_slots(host, vec![i32::from(read.storage.locked)]);
                    }
                    format!("{block}: {}", read.storage.state_word())
                }
                (true, None) => format!("{block}: not a modular storage"),
            }
        }
        "info" => {
            // A 0.2.2 block's module moves into the card slot on the first read,
            // so the readout and the menu agree from then on; a carried stash a
            // refused removal left behind comes back the same way.
            if let Some(entity) = entity_at(&*host, pos) {
                adopt_legacy_module(host, entity);
            }
            adopt_carried(host, pos);
            match read_at(&*host, pos) {
                Some(read) => {
                    let max_stack = read_max_stack(&*host, &read);
                    format!(
                        "{block}: {} {}",
                        read.storage.summary(max_stack),
                        read.storage.state_word()
                    )
                }
                None => format!("{block}: not a modular storage"),
            }
        }
        "scan" => scan_readout(host, pos),
        "request" => {
            let Some(requested) = item_argument!(event.command.argument("item")) else {
                feedback(
                    host,
                    "usage: /rftoolsstorage request <x> <y> <z> <item> [<count>]",
                );
                return Flow::Cancel;
            };
            let explicit = match event.command.argument("count") {
                None => None,
                Some(CommandArgumentValue::Integer(count)) => match u32::try_from(*count) {
                    Ok(count) => Some(count),
                    Err(_) => {
                        feedback(host, "the count must be a whole number of 0 or more");
                        return Flow::Cancel;
                    }
                },
                Some(_) => None,
            };
            request_readout(host, event.source, pos, requested, explicit)
        }
        "give" => {
            let Some(requested) = item_argument!(event.command.argument("item")) else {
                feedback(
                    host,
                    "usage: /rftoolsstorage give <x> <y> <z> <item> [single]",
                );
                return Flow::Cancel;
            };
            let mode = match event.command.argument("single") {
                None => GiveAmount::Stack,
                Some(CommandArgumentValue::String(flag)) if flag == "single" => GiveAmount::Single,
                Some(_) => {
                    feedback(
                        host,
                        "usage: /rftoolsstorage give <x> <y> <z> <item> [single]",
                    );
                    return Flow::Cancel;
                }
            };
            give_readout(host, event.source, pos, requested, mode)
        }
        "insert" => {
            let Some(requested) = item_argument!(event.command.argument("item")) else {
                feedback(
                    host,
                    "usage: /rftoolsstorage insert <x> <y> <z> <item> [<count>]",
                );
                return Flow::Cancel;
            };
            let explicit = match event.command.argument("count") {
                None => None,
                Some(CommandArgumentValue::Integer(count)) => match u32::try_from(*count) {
                    Ok(count) => Some(count),
                    Err(_) => {
                        feedback(host, "the count must be a whole number of 0 or more");
                        return Flow::Cancel;
                    }
                },
                Some(_) => None,
            };
            insert_readout(host, event.source, pos, requested, explicit)
        }
        "radius" => {
            let Some(CommandArgumentValue::Integer(wanted)) = event.command.argument("radius")
            else {
                feedback(host, "usage: /rftoolsstorage radius <x> <y> <z> <radius>");
                return Flow::Cancel;
            };
            match set_radius(host, pos, *wanted) {
                Some(radius) => format!("{block}: radius {radius}"),
                None => format!("{block}: not a storage scanner"),
            }
        }
        _ => return Flow::Continue,
    };
    feedback(host, &line);
    Flow::Cancel
}

/// The command's answer, in the player's chat.
fn feedback(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One carried slot in the shape `stash_carried` writes: vanilla's item escape
    /// plus the slot, which is what `save::encode_items` writes for `Items`.
    fn carried_entry(slot: i8, item: &str, count: i8) -> Tag {
        Tag::compound([
            ("Slot", Tag::Byte(slot)),
            ("id", Tag::String(item.into())),
            ("count", Tag::Byte(count)),
        ])
    }

    #[test]
    fn the_payload_carries_the_contents_and_never_items() {
        // The break moved 64 stone (slot 0) and the module (slot 100) aside; a
        // stray `Items` list is stripped: the engine spills a container's own
        // stacks, so a payload that carried them too would put them in the world
        // twice.
        let saved = Compound::from_iter([
            ("locked".into(), Tag::Byte(1)),
            (
                KEY_CARRIED.into(),
                Tag::List(vec![
                    carried_entry(0, "minecraft:stone", 64),
                    carried_entry(100, "rftoolsstorage:storage_module0", 1),
                ]),
            ),
            (
                "Items".into(),
                Tag::List(vec![carried_entry(0, "minecraft:stone", 64)]),
            ),
            (
                "id".into(),
                Tag::String("rftoolsstorage:storage_block_entity".into()),
            ),
        ]);
        let patch = carried_payload(&saved, BE_TYPE).expect("a carried break travels");
        let Some(Tag::Compound(payload)) = field(Some(&patch), BLOCK_ENTITY_DATA) else {
            panic!("the patch holds no {BLOCK_ENTITY_DATA}");
        };
        assert!(
            field(Some(payload), "Items").is_none(),
            "the payload carries the container's stacks, which the engine spills itself"
        );
        assert!(
            matches!(
                field(Some(payload), "id"),
                Some(Tag::String(id)) if id.to_string_lossy() == BE_TYPE
            ),
            "the payload is not tagged with the block entity type"
        );
        assert!(
            matches!(field(Some(payload), "locked"), Some(Tag::Byte(1))),
            "the lock did not travel with the contents"
        );
        let Some(Tag::List(entries)) = field(Some(payload), KEY_CARRIED) else {
            panic!("the payload lost the carried contents");
        };
        assert_eq!(entries.len(), 2, "the contents and the card slot's module");
    }

    #[test]
    fn a_break_that_moved_nothing_aside_carries_nothing() {
        // A creative break, a refused tool, a piston, water, an explosion: no stash,
        // so the item is plain and the contents take the vanilla spill.
        let plain = Compound::from_iter([("locked".into(), Tag::Byte(0))]);
        assert!(carried_payload(&plain, BE_TYPE).is_none());
        let emptied = Compound::from_iter([(KEY_CARRIED.into(), Tag::List(Vec::new()))]);
        assert!(carried_payload(&emptied, BE_TYPE).is_none());
    }

    #[test]
    fn the_scanners_container_is_the_jars_three_slots() {
        // `StorageScannerContainer`: SLOT_IN 0, SLOT_OUT 1, SLOT_IN_AUTO 2;
        // `insertable(slot(SLOT_IN_AUTO))` and McJtyLib's extractable default true.
        let container = scanner_container();
        assert_eq!(container.slots, SCANNER_SLOTS);
        assert_eq!(container.slots, 3);
        assert_eq!(
            container.inputs,
            vec![SCANNER_AUTO_SLOT],
            "automation inserts into the auto slot only"
        );
        assert_eq!(
            container.outputs,
            vec![SCANNER_IN_SLOT, SCANNER_OUT_SLOT, SCANNER_AUTO_SLOT],
            "every slot may be pulled from"
        );
        let all = vec![SCANNER_IN_SLOT, SCANNER_OUT_SLOT, SCANNER_AUTO_SLOT];
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| all.clone()),
            "every face reaches the three slots"
        );
        assert!(
            container.menu.is_none(),
            "the GUI's list is a later slice; the slots' consumer is the tick"
        );
    }

    #[test]
    fn the_readout_says_what_the_player_must_do_first() {
        let mut storage = Storage::with_tier(0);
        assert_eq!(
            readout(&storage, 64),
            "Modular Storage: unlocked — press the padlock to lock"
        );
        assert!(storage.toggle_lock());
        assert_eq!(readout(&storage, 64), "Modular Storage: 0/6400 empty");
        storage.insert(
            &Kind {
                key: "minecraft:stone".to_owned(),
                label: "minecraft:stone".to_owned(),
            },
            64,
            64,
        );
        assert_eq!(
            readout(&storage, 64),
            "Modular Storage: 64/6400 minecraft:stone"
        );
        assert_eq!(
            readout(&Storage::empty(), 64),
            "Modular Storage: no storage module"
        );
    }
}
