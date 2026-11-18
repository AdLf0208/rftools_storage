//! The game glue of the modular storage block: its block entity, its container,
//! the player's click and the readout command.
//!
//! In the SDK layout this is an ordinary module of the mod crate. The rules —
//! the module's slot count, one kind a slot, 64 a slot, the lock — live in
//! [`crate::runtime`], which has no `mod_api` types; this module only reads the
//! game, calls the runtime and writes the result back.
//!
//! The storage *is* the block entity's container slots: the pin saves a mod
//! block entity's `Items` with its chunk, so a relaunch keeps them, and a broken
//! block spills them as loose drops. `module` and `locked` ride the same saved
//! compound, so the module and the lock survive a relaunch too.

use std::sync::OnceLock;

use horizon_core::nbt::JavaString;
use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityRef, BlockKey, BlockPart, Command,
    CommandArgumentValue, Compound, Flow, Hand, Host, HostSide, HostTransaction, InteractionResult,
    ItemStack, LevelSide, ModContainer, RegisterCommandsEvent, Registrar, RegistrationContext,
    RightClickBlockEvent, RunModCommandEvent, ServerHost, Tag, Text, Transaction,
};

use crate::runtime::{Kind, SLOT_LIMIT, Slot, Storage, module_tier};

/// The block this slice implements, exactly as the content declares it.
const BLOCK: &str = crate::runtime::MODULAR_STORAGE;
/// The block-entity type this mod declares for that block.
const BE_TYPE: &str = "rftoolsstorage:storage_block_entity";
/// The saved compound key of the installed module's tier.
const KEY_MODULE: &str = "module";
/// The saved compound key of the lock (a byte, as upstream's lockable entity).
const KEY_LOCKED: &str = "locked";
/// No module in the saved compound. A stored sentinel rather than a missing
/// key: `merge_data` sets a key but cannot remove one, and the round trip is
/// internal — [`read_storage`] reads any value outside `0..=3` as no module.
const NO_MODULE: i32 = -1;
/// The container's slot count: the tier-1 module's 100 (`TIER_SLOTS[0]`). A
/// block's container is declared once per block, not per instance, so only the
/// tier this slice carries is wired (notes.md, deviation 4).
const CONTAINER_SLOTS: usize = 100;
/// The action-bar prefix every readout carries.
const READOUT: &str = "Modular Storage: ";

/// The block key the click compares the loaded block against. A `BlockKey` only
/// names itself through a host, so it is asked for once, at registration.
static BLOCK_KEY: OnceLock<BlockKey> = OnceLock::new();

fn block_key() -> BlockKey {
    *BLOCK_KEY
        .get()
        .expect("registration runs before every hook")
}

// --- registration ----------------------------------------------------------

/// Declare the block-entity type, attach its Part and declare the block's
/// container (100 slots, every face, inputs and outputs, no menu).
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
    let all: Vec<usize> = (0..CONTAINER_SLOTS).collect();
    // Upstream's `CapType.ITEMS` handler answers every side, and a hopper may
    // both fill and empty it.
    // `..Default::default()` keeps a field a later game build adds from breaking
    // this literal (`cargo horizon-mod check` asks for it).
    #[expect(
        clippy::needless_update,
        reason = "the update is for a field a later game build may add"
    )]
    let container = ModContainer {
        slots: CONTAINER_SLOTS,
        faces: std::array::from_fn(|_| all.clone()),
        inputs: all.clone(),
        outputs: all,
        menu: None,
        ..ModContainer::default()
    };
    registrar.block_container(&block, container);
}

/// The Part of the mod's one block-entity type: the modular storage declares a
/// block entity, and nothing else about it is per-block.
struct StorageBlock;

#[mod_api::horizon_part]
impl BlockPart for StorageBlock {}

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

/// The container's slots, read one by one (a mod block entity's item storage is
/// per slot; a refused read is an empty slot).
fn read_slots(host: &dyn Host, entity: BlockEntityRef) -> Vec<Option<ItemStack>> {
    (0..CONTAINER_SLOTS)
        .map(|slot| {
            let slot = u32::try_from(slot).unwrap_or(u32::MAX);
            entity.item_slot(host, slot).ok().flatten()
        })
        .collect()
}

/// The storage a block entity holds: its module and lock from the saved
/// compound, its contents from the container slots.
fn read_storage(host: &dyn Host, data: Option<&Compound>, slots: &[Option<ItemStack>]) -> Storage {
    let tier = data.and_then(|data| match data.get(&JavaString::from(KEY_MODULE)) {
        Some(Tag::Int(n)) if (0..=3).contains(n) => Some(*n as u8),
        _ => None,
    });
    let mut storage = match tier {
        Some(tier) => Storage::with_tier(tier),
        None => Storage::empty(),
    };
    storage.locked = matches!(
        data.and_then(|data| data.get(&JavaString::from(KEY_LOCKED))),
        Some(Tag::Byte(1))
    );
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

/// Writes `storage` back: the module and lock into the saved compound, the
/// slots into the container's `Items`. A slot that holds a kind keeps the stack
/// it had; a slot that just filled takes the stack the player used (`held`).
fn write_storage(
    host: &mut dyn ServerHost,
    tx: &Transaction,
    entity: BlockEntityRef,
    storage: &Storage,
    before: &[Option<ItemStack>],
    held: Option<&ItemStack>,
) -> Result<(), ()> {
    let mut data = Compound::new();
    data.insert(
        JavaString::from(KEY_MODULE),
        Tag::Int(storage.tier.map_or(NO_MODULE, i32::from)),
    );
    data.insert(
        JavaString::from(KEY_LOCKED),
        Tag::Byte(i8::from(storage.locked)),
    );
    entity.merge_data(host, data).map_err(|_| ())?;
    let wanted: Vec<Option<ItemStack>> = (0..CONTAINER_SLOTS)
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

/// Reads the storage at `pos`, applies `change`, writes it back. The closure
/// also sees the slots as they were, to build the stack it takes out.
fn mutate<R>(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    held: Option<&ItemStack>,
    change: impl FnOnce(&mut Storage, &[Option<ItemStack>]) -> R,
) -> Option<R> {
    let entity = entity_at(host, pos)?;
    let before = read_slots(host, entity);
    let data = entity.data(host);
    let mut storage = read_storage(host, data.as_ref(), &before);
    let result = change(&mut storage, &before);
    host.transaction(|host, tx| write_storage(host, tx, entity, &storage, &before, held))
        .ok()?;
    Some(result)
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
    Some(Read {
        storage: read_storage(host, data.as_ref(), &slots),
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
        return format!("{READOUT}unlocked — right-click with an empty hand to lock");
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

/// Right click: the block's whole player surface for this slice. A storage
/// module in hand installs, a held stack inserts, and an empty hand either
/// seals an unlocked block or takes a stack out of a locked one.
///
/// The hook is heard on the clicking client **and** on the server (once per
/// hand). Both consume the click — `Success` ends it as used, so neither the
/// block's own use nor the held item's runs after it (the default `Pass` would
/// let a block in hand be placed behind the insert) — but only the server
/// touches the world.
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
    if let HostSide::Server(host) = host {
        match click.stack {
            None => empty_hand(host, click.pos, click.player),
            Some(stack) => match module_tier(&stack.item().id(host)) {
                Some(tier) => install(host, click.pos, click.player, stack, tier),
                None => insert(host, click.pos, click.player, stack),
            },
        }
    }
    *event.cancellation_result = InteractionResult::Success;
    Flow::Cancel
}

/// An empty hand is the padlock: it seals a module-bearing unlocked storage
/// (the state the GUI's button sets), and withdraws from a locked one.
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

/// A storage module in hand goes into the block's module slot (only while
/// unlocked and empty, as upstream's `mayPlace`).
fn install(host: &mut dyn ServerHost, pos: [i32; 3], uuid: u128, held: &ItemStack, tier: u8) {
    let installed = mutate(host, pos, None, |storage, _| storage.install(tier)).unwrap_or(false);
    let line = if installed {
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
            format!("{READOUT}unlocked — right-click with an empty hand to lock")
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
                format!("{READOUT}unlocked — right-click with an empty hand to lock")
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

// --- the readout commands --------------------------------------------------

/// `/rftoolsstorage info` and `/rftoolsstorage lock` (`register_commands`).
pub(crate) fn on_register_commands(event: &mut RegisterCommandsEvent<'_>) {
    event
        .commands
        .push(Command::new("rftoolsstorage info <pos:block_pos>"));
    event
        .commands
        .push(Command::new("rftoolsstorage lock <pos:block_pos>"));
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
        feedback(host, "usage: /rftoolsstorage info|lock <x> <y> <z>");
        return Flow::Cancel;
    };
    let pos = *pos;
    let block = block_id_at(host, pos);
    let line = match sub.as_str() {
        "lock" => {
            let toggled =
                mutate(host, pos, None, |storage, _| storage.toggle_lock()).unwrap_or(false);
            match (toggled, read_at(host, pos)) {
                (false, Some(read)) if read.storage.tier.is_none() => {
                    format!("{block}: no module to lock")
                }
                (false, _) => format!("{block}: not a modular storage"),
                (true, Some(read)) => format!("{block}: {}", read.storage.state_word()),
                (true, None) => format!("{block}: not a modular storage"),
            }
        }
        "info" => match read_at(host, pos) {
            Some(read) => {
                let max_stack = read_max_stack(host, &read);
                format!(
                    "{block}: {} {}",
                    read.storage.summary(max_stack),
                    read.storage.state_word()
                )
            }
            None => format!("{block}: not a modular storage"),
        },
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

    #[test]
    fn the_readout_says_what_the_player_must_do_first() {
        let mut storage = Storage::with_tier(0);
        assert_eq!(
            readout(&storage, 64),
            "Modular Storage: unlocked — right-click with an empty hand to lock"
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
