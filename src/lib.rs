//! RFTools Storage for Horizon: the modular storage block (slice 2).
//!
//! One block, `rftoolsstorage:modular_storage`, with a block entity and its own
//! menu: a storage module in its slot sets the capacity (tier 1 holds 100 slots,
//! one item kind a slot, up to 64 a slot), the block stores nothing until it is
//! **locked**, and a right click opens the storage's grid — upstream's GUI —
//! with the padlock on its panel. While sneaking, a click is the quick path
//! instead: a module in hand installs, a held stack inserts, an empty hand seals
//! an unlocked block or takes a stack out of a locked one. `/rftoolsstorage
//! info|lock` remain the readout and the operator's lock.
//!
//! A broken storage's **contents travel on its own item** (slice 4): the break
//! moves them aside into the block entity's saved data before the removal, the
//! block's drop carries them as vanilla's `minecraft:block_entity_data`, and
//! placing the item puts them back — the lock and the card slot's module with them.
//!
//! [`runtime`] is the pure engine (the module's slot count, one kind a slot, the
//! lock, the menu's slot and padlock rules, the carried payload's clamp, the
//! readout text) and [`support`] is the game glue. See `notes.md` for the slice
//! boundary, the deliberate deviations and the gaps.
pub mod runtime;
mod support;

use mod_api::{
    self, BlockDropsEvent, Flow, Hooks, HostSide, LevelHost, PlayerContainerOpenEvent,
    RegisterCommandsEvent, Registrar, RegistrationContext, RightClickBlockEvent,
    RunModCommandEvent, ServerHost,
};

/// The mod holds no state of its own: the storage is the block entity's saved
/// compound and its container slots, and everything else is derived per click.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the block-entity type, its Part, the block's container and its
    /// menu, and the menu's own slot rules.
    fn register(&self, registrar: &mut dyn Registrar, cx: &RegistrationContext) {
        support::register(registrar, cx);
    }

    /// A plain click opens the menu; a sneaking one is the quick gesture.
    fn right_click_block(&self, host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        support::on_right_click_block(host, event)
    }

    /// A broken storage's own item carries its contents (`BlockDropsEvent`): the
    /// break moved them aside before the removal, so the engine's container spill
    /// found an empty container and they are not on the ground twice.
    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        support::on_block_drops(level, event)
    }

    /// `/rftoolsstorage info` and `/rftoolsstorage lock`.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        support::on_register_commands(event);
    }

    /// The bodies of `/rftoolsstorage info|lock`.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        support::on_run_mod_command(host, event)
    }

    /// The menu's padlock reads a data slot, which is not saved: fill it with
    /// the block's real state when the menu opens.
    fn player_container_open(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerContainerOpenEvent<'_>,
    ) {
        support::on_player_container_open(host, event);
    }
}
