//! RFTools Storage for Horizon: the modular storage block (slice 1).
//!
//! One block, `rftoolsstorage:modular_storage`, with a block entity: a storage
//! module in its slot sets the capacity (tier 1 holds 100 slots, one item kind a
//! slot, up to 64 a slot), and the block stores nothing until it is **locked**.
//! Upstream opens a GUI from the block; this slice's player surface is the
//! right click — a module in hand installs, a held stack inserts, an empty hand
//! seals an unlocked block or takes a stack out of a locked one — plus
//! `/rftoolsstorage info|lock`, which stand in for the GUI's readout and padlock.
//!
//! [`runtime`] is the pure engine (the module's slot count, one kind a slot, the
//! lock, the readout text) and [`support`] is the game glue. See `notes.md` for
//! the slice boundary, the deliberate deviations and the gaps.
pub mod runtime;
mod support;

use mod_api::{
    self, Flow, Hooks, HostSide, RegisterCommandsEvent, Registrar, RegistrationContext,
    RightClickBlockEvent, RunModCommandEvent, ServerHost,
};

/// The mod holds no state of its own: the storage is the block entity's saved
/// compound and its container slots, and everything else is derived per click.
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the block-entity type, its Part, and the block's container.
    fn register(&self, registrar: &mut dyn Registrar, cx: &RegistrationContext) {
        support::register(registrar, cx);
    }

    /// The block's whole player surface: install, insert, seal, withdraw.
    fn right_click_block(&self, host: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        support::on_right_click_block(host, event)
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
}
