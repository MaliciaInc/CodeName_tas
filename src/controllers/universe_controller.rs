use crate::app::{AppState, UniverseMessage};
use crate::state::{DbAction, ToastKind};
use uuid::Uuid;
use crate::state::ConfirmAction;

pub fn update(state: &mut AppState, message: UniverseMessage) {
    match message {
        UniverseMessage::NameChanged(v) => state.new_universe_name = v,
        UniverseMessage::DescChanged(v) => state.new_universe_desc = v,

        UniverseMessage::Create => {
            if !state.new_universe_name.trim().is_empty() {
                let id = format!("u-{}", Uuid::new_v4());
                let name = state.new_universe_name.trim().to_string();
                let desc = state.new_universe_desc.trim().to_string();

                state.queue(DbAction::CreateUniverse { id, name, desc });
                state.new_universe_name.clear();
                state.new_universe_desc.clear();
                state.show_toast("Creating universe...", ToastKind::Info);
            }
        }

        UniverseMessage::Delete(id) => {
            state.pending_confirm = Some(ConfirmAction::DeleteUniverse(id));
            state.show_toast("Universe deleted", ToastKind::Info);
        }

        UniverseMessage::Open(id) => {
            state.route = crate::app::Route::UniverseDetail { universe_id: id };
        }

        UniverseMessage::InjectDemoData(id) => {
            state.queue(DbAction::InjectDemoData(id));
            state.show_toast("Injecting demo data...", ToastKind::Info);
        }

        UniverseMessage::ResetDemoPrompt(uid, scope) => {
            state.queue(DbAction::ResetDemoDataScoped(uid, scope));
            state.show_toast("Resetting demo data...", ToastKind::Info);
        }

        UniverseMessage::ToggleDeveloperPanel => {
            state.dev_panel_open = !state.dev_panel_open;
        }

        UniverseMessage::ToggleDebugOverlay => {
            state.debug_overlay_open = !state.debug_overlay_open;
            // Force refresh next render tick
            state.debug_schema_version = None;
        }

        UniverseMessage::SnapshotNameChanged(v) => state.snapshot_name = v,

        UniverseMessage::SnapshotCreate(universe_id) => {
            let name = state.snapshot_name.trim().to_string();
            if !name.is_empty() {
                // ✅ FASE 10: invalidación real del contrato Core (loaded_for + in_progress + flag legacy)
                state.loaded_snapshots_universe = None;
                state.core_snapshots_loaded_for.remove(&universe_id);
                state
                    .core_loading_in_progress
                    .remove(&crate::state::CoreLoadKey::Snapshots {
                        universe_id: universe_id.clone(),
                    });

                state.queue(DbAction::SnapshotCreate { universe_id, name });
                state.snapshot_name.clear();
                state.show_toast("Creating snapshot...", ToastKind::Info);
            }
        }

        UniverseMessage::SnapshotRefresh(universe_id) => {
            // ✅ FASE 10: refresh debe abrir compuerta sí o sí
            state.loaded_snapshots_universe = None;
            state.core_snapshots_loaded_for.remove(&universe_id);
            state
                .core_loading_in_progress
                .remove(&crate::state::CoreLoadKey::Snapshots {
                    universe_id: universe_id.clone(),
                });

            state.show_toast("Refreshing snapshots...", ToastKind::Info);
            state.route = crate::app::Route::UniverseDetail { universe_id };
        }

        UniverseMessage::SnapshotRestore(snapshot_id) => {
            // Restore affects universe data; mark caches dirty so UI refreshes.
            state.loaded_creatures_universe = None;
            state.loaded_locations_universe = None;
            state.loaded_timeline_universe = None;
            state.loaded_snapshots_universe = None;

            // Nota: acá no tenemos universe_id para invalidar scoped CoreLoadKey::Snapshots.
            // La limpieza fuerte ya se hace en ActionDone/RestoreFromTrash/etc. (y el fetch guardado evita out-of-order).
            state.queue(DbAction::SnapshotRestore { snapshot_id });
            state.show_toast("Restoring snapshot...", ToastKind::Info);
        }

        UniverseMessage::SnapshotDelete(snapshot_id) => {
            // Optimistic UI: remove immediately; DB action will confirm.
            let sid = snapshot_id.clone();
            state.snapshots.retain(|s| s.id != sid);

            // ✅ FASE 10: sin universe_id no podemos limpiar CoreLoadKey::Snapshots scoped.
            // Igual marcamos stale el flag legacy; la invalidación fuerte se hace cuando DB confirma (ActionDone).
            state.loaded_snapshots_universe = None;

            state.queue(DbAction::SnapshotDelete { snapshot_id });
            state.show_toast("Deleting snapshot...", ToastKind::Info);
        }

        UniverseMessage::ValidateUniverse(_universe_id) => {
            // We will fetch issues via root_controller task (not queued) to avoid breaking inflight clearing.
            state.integrity_busy = true;
        }
    }
}
