use std::time::Instant;
use crate::app::{AppState, BestiaryMessage, CreatureEditor};
use crate::state::{DbAction, ToastKind};


pub fn update(state: &mut AppState, message: BestiaryMessage) {
    match message {
        BestiaryMessage::Open(universe_id) => {
            state.creature_editor = None;
            state.route = crate::app::Route::Bestiary { universe_id };
        }
        BestiaryMessage::CardClicked(index) => {
            let now = Instant::now();
            if let Some((last_idx, last_time)) = state.last_bestiary_click {
                if last_idx == index && now.duration_since(last_time).as_millis() < 500 {
                    // Doble clic: Editar
                    // Nota: 'index' aquí refiere al filtrado visual actual, que puede ser confuso.
                    // Para robustez usamos búsqueda por ID en la versión "Pro", pero por ahora
                    // confiamos en que el grid se renderiza en orden.
                    // (Mejora bulldozer: usar ID siempre, pero mantenemos compatibilidad de índice por ahora)
                    if let Some(c) = state.creatures.get(index) {
                        state.creature_editor = Some(CreatureEditor::from_creature(index, c, &state.locations));
                    }
                    state.last_bestiary_click = None;
                    return;
                }
            }
            state.last_bestiary_click = Some((index, now));
        }
        BestiaryMessage::EditorOpenCreate => state.creature_editor = Some(CreatureEditor::create_new()),
        BestiaryMessage::EditorCancel => state.creature_editor = None,
        BestiaryMessage::EditorSave => {
            if let Some(editor) = state.creature_editor.take() {
                // Validar ruta ANTES de consumir el editor definitivamente
                let uid = match &state.route {
                    crate::app::Route::Bestiary { universe_id } => universe_id.clone(),
                    _ => {
                        state.creature_editor = Some(editor);
                        return;
                    }
                };

                if editor.name.trim().is_empty() {
                    state.creature_editor = Some(editor);
                    state.show_toast("Name cannot be empty", ToastKind::Error);
                    return;
                }

                let mut f = editor.into_creature();

                // ✅ REFACTOR A.3: O(1) lookup instead of O(n) iteration
                if let Some(existing) = state.find_creature_by_id(&f.id) {
                    f.archived = existing.archived;
                }

                state.queue(DbAction::SaveCreature(f, uid));
                state.show_toast("Saving creature...", ToastKind::Info);
            }
        }
        BestiaryMessage::NameChanged(v) => if let Some(e) = state.creature_editor.as_mut() { e.name = v },
        BestiaryMessage::KindChanged(v) => if let Some(e) = state.creature_editor.as_mut() { e.kind = v },
        BestiaryMessage::HabitatChanged(v) => if let Some(e) = state.creature_editor.as_mut() { e.habitat = v },
        BestiaryMessage::DescriptionChanged(action) => if let Some(e) = state.creature_editor.as_mut() { e.description.perform(action); },
        BestiaryMessage::DangerChanged(v) => if let Some(e) = state.creature_editor.as_mut() { e.danger = v },
        BestiaryMessage::LocationChanged(loc_id) => if let Some(e) = state.creature_editor.as_mut() { e.home_location_id = loc_id }, // ✅ C.1: ID only
        // NUEVAS ACCIONES QUE YA NO NAVEGAN
        BestiaryMessage::Delete(id) => {state.pending_confirm = Some(crate::state::ConfirmAction::DeleteCreature(id));},
        BestiaryMessage::Archive(id) => {
        state.queue(DbAction::ArchiveCreature(id, true));
        state.show_toast("Archiving creature...", ToastKind::Info); },
        BestiaryMessage::Restore(id) => {
        state.queue(DbAction::ArchiveCreature(id, false));
        state.show_toast("Restoring creature...", ToastKind::Info);},

    }
}