use iced::{Color, Element, Length, Theme, Vector};
use iced::widget::{container, scrollable, text, Column, Row, Stack};

use crate::{pages, ui};
use super::{AppState, Message, Route, PmState};
use crate::state::ConfirmAction;

// --- DEBUG OVERLAY (A) ---
fn debug_overlay(state: &AppState, t: ui::Tokens) -> Element<'_, Message> {
    let now = std::time::Instant::now();

    let inflight = match &state.db_inflight {
        None => "None".to_string(),
        Some(a) => format!("{:?}", a),
    };

    let schema = match state.debug_schema_version {
        None => "…".to_string(),
        Some(v) => v.to_string(),
    };

    let route = format!("{:?}", state.route);

    // ==========================
    // FASE 13: Contrato visible
    // ==========================
    let selection = format!(
        "active_novel={:?} active_chapter={:?} active_scene={:?}",
        state.active_novel_id, state.active_chapter_id, state.active_scene_id
    );

    let renames = format!(
        "renaming: novel={:?} chapter={:?} scene={:?} | temp: n={} c={} s={}",
        state.forge_renaming_novel_id,
        state.forge_renaming_chapter_id,
        state.forge_renaming_scene_id,
        state.forge_renaming_novel_temp.is_some(),
        state.forge_renaming_chapter_temp.is_some(),
        state.forge_renaming_scene_temp.is_some()
    );

    let confirm = match &state.pending_confirm {
        None => "confirm: None".to_string(),
        Some(a) => format!("confirm: {:?}", a),
    };

    let gating = format!(
        "gating: forge_in_progress={} core_in_progress={}",
        state.forge_loading_in_progress.len(),
        state.core_loading_in_progress.len()
    );

    let last_invalidation = match &state.debug_last_invalidation {
        None => "last_invalidation: None".to_string(),
        Some(inv) => format!(
            "last_invalidation: {} key={} age={}ms reason={}",
            inv.scope,
            inv.key,
            now.saturating_duration_since(inv.at).as_millis(),
            inv.reason
        ),
    };

    let last_ignored = match &state.debug_last_ignored {
        None => "last_ignored: None".to_string(),
        Some(ig) => format!(
            "last_ignored: age={}ms reason={}",
            now.saturating_duration_since(ig.at).as_millis(),
            ig.reason
        ),
    };

    // Conteos generales (mantengo los tuyos)
    let counts = format!(
        "Universes={} Creatures={} Locations={} Eras={} Events={} Snapshots={} Issues={}",
        state.universes.len(),
        state.creatures.len(),
        state.locations.len(),
        state.timeline_eras.len(),
        state.timeline_events.len(),
        state.snapshots.len(),
        state.integrity_issues.len(),
    );

    // Eventos recientes (últimos 14)
    let mut events_col = Column::new().spacing(4);
    let tail = state.debug_events.iter().rev().take(14).collect::<Vec<_>>();

    if tail.is_empty() {
        events_col = events_col.push(
            text("No debug events yet (abre overlay y ejecutá rename/delete).")
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        );
    } else {
        for ev in tail.into_iter().rev() {
            let age_ms = now.saturating_duration_since(ev.at).as_millis();
            let prefix = match ev.kind {
                crate::state::DebugEventKind::Info => "ℹ️",
                crate::state::DebugEventKind::Warn => "⚠️",
                crate::state::DebugEventKind::Error => "🧨",
            };

            events_col = events_col.push(
                text(format!("{} +{}ms — {}", prefix, age_ms, ev.msg))
                    .size(12)
                    .style(move |_| iced::widget::text::Style { color: Some(t.foreground) }),
            );
        }
    }

    // Integrity issues (mantengo)
    let mut issues_col = Column::new().spacing(4);
    if state.integrity_issues.is_empty() {
        issues_col = issues_col.push(
            text("No integrity issues detected.")
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        );
    } else {
        for (i, issue) in state.integrity_issues.iter().take(6).enumerate() {
            issues_col = issues_col.push(
                text(format!("{}. {}", i + 1, issue))
                    .size(12)
                    .style(move |_| iced::widget::text::Style { color: Some(t.foreground) }),
            );
        }
        if state.integrity_issues.len() > 6 {
            issues_col = issues_col.push(
                text(format!("…and {} more", state.integrity_issues.len() - 6))
                    .size(12)
                    .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
            );
        }
    }

    let content = Column::new()
        .spacing(10)
        .push(
            Row::new()
                .spacing(12)
                .push(
                    text("Debug Overlay — FASE 13 (PRO)")
                        .size(16)
                        .style(move |_| iced::widget::text::Style { color: Some(t.foreground) }),
                )
                .push(
                    text("(Contrato: inflight + gating + invalidation + out-of-order)")
                        .size(12)
                        .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
                ),
        )
        .push(
            text(format!("schema_version={}", schema))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(format!("db_inflight={}", inflight))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(format!("route={}", route))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(selection)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(renames)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(confirm)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(gating)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(last_invalidation)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(last_ignored)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(counts)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(ui::h_divider(t))
        .push(
            text("Recent debug events (latest 14):")
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(events_col)
        .push(ui::h_divider(t))
        .push(
            text("Integrity issues (top 6):")
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(issues_col);

    let panel = container(content)
        .padding(14)
        .width(Length::Fixed(920.0))
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(t.shell_b, 0.98), t.foreground);
            s.border.color = t.accent;
            s.border.width = 2.0;
            s.border.radius = 12.0.into();
            s.shadow = iced::Shadow {
                color: Color::BLACK,
                offset: Vector::new(0.0, 12.0),
                blur_radius: 24.0,
            };
            s
        });

    container(panel)
        .padding(iced::Padding {
            top: 20.0,
            left: 260.0,
            right: 20.0,
            bottom: 20.0,
        })
        .into()
}

// --- CONFIRM MODAL ---
fn confirm_modal(action: &ConfirmAction, t: ui::Tokens) -> Element<'_, Message> {
    let (title, message) = match action {
        ConfirmAction::DeleteUniverse(_) => (
            "Delete Universe?",
            "This will permanently delete the universe and all its content."
        ),
        ConfirmAction::DeleteBoard(_) => (
            "Delete Board?",
            "This will permanently delete the board and all its cards."
        ),
        ConfirmAction::DeleteNovel(_) => (
            "Delete Novel?",
            "This will permanently delete the novel and all its chapters and scenes."
        ),
        ConfirmAction::DeleteChapter(_) => (
            "Delete Chapter?",
            "This will permanently delete the chapter and all its scenes."
        ),
        ConfirmAction::DeleteScene(_) => (
            "Delete Scene?",
            "This will permanently delete the scene and its content."
        ),
        ConfirmAction::DeleteCreature(_) => (
            "Delete Creature?",
            "This creature will be moved to trash. You can restore it later.",
        ),
        ConfirmAction::DeleteLocation(_) => (
            "Delete Location?",
            "This location will be moved to trash. You can restore it later.",
        ),
        ConfirmAction::DeleteEvent(_) => (
            "Delete Event?",
            "This timeline event will be moved to trash. You can restore it later.",
        ),
        ConfirmAction::DeleteEra(_) => (
            "Delete Era?",
            "This timeline era will be moved to trash. You can restore it later.",
        ),
    };

    let content = Column::new()
        .spacing(20)
        .push(
            text(title)
                .size(20)
                .style(move |_| iced::widget::text::Style { color: Some(t.foreground) })
        )
        .push(
            text(message)
                .size(14)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) })
        )
        .push(
            Row::new()
                .spacing(12)
                .push(
                    ui::ghost_button(t, "Cancel".to_string(), Message::CancelConfirm)
                )
                .push(
                    ui::danger_button(t, "Delete".to_string(), Message::ConfirmDelete)
                )
        );

    let panel = container(content)
        .padding(24)
        .width(Length::Fixed(400.0))
        .style(move |_: &Theme| {
            let mut s = ui::container_style(t.shell_b, t.foreground);
            s.border.color = Color::from_rgb(0.9, 0.2, 0.2);
            s.border.width = 2.0;
            s.border.radius = 12.0.into();
            s.shadow = iced::Shadow {
                color: Color::BLACK,
                offset: Vector::new(0.0, 12.0),
                blur_radius: 24.0,
            };
            s
        });

    // Overlay background
    let overlay_bg = container(
        container(panel)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| {
            ui::container_style(ui::alpha(Color::BLACK, 0.6), t.foreground)
        });

    overlay_bg.into()
}

// --- VIEW DISPATCHER ---
pub fn view(state: &AppState) -> Element<'_, Message> {
    let t = ui::Tokens::nub_dark();

    // 1) LAUNCHER MODE
    if state.active_project.is_none() {
        return pages::launcher::launcher_view(state, t);
    }

    // 2) STUDIO MODE
    let sidebar = ui::sidebar(state, t);
    let header = ui::header(state, t);

    let page: Element<'_, Message> = match &state.route {
        Route::Overview => pages::overview(state, t),
        Route::Workspaces => pages::workspaces::workspaces_page(state, t),
        Route::UniverseList => pages::universe_list(state, t),
        Route::UniverseDetail { universe_id } => pages::universe_detail(state, t, universe_id),
        Route::Bestiary { universe_id } => pages::bestiary(state, t, universe_id),
        Route::Locations { universe_id } => pages::locations::locations(state, t, universe_id),
        Route::Timeline { universe_id } => pages::timeline::timeline(state, t, universe_id),
        Route::PmList => pages::pm_list::pm_list(state, t),
        Route::PmBoard { .. } => pages::pm_board::pm_board(state, t, &state.pm_data),
        Route::Trash => pages::trash_page(state, t),

        // The Forge
        Route::Forge => pages::the_forge(state, t),

        Route::Assets => pages::assets_stub(state, t),
        Route::Account => pages::account_stub(state, t),
    };

    // IMPORTANT:
    // Some full-height pages (like Forge) have their own internal scrollables and rely on Fill layout.
    // Wrapping them in a global vertical Scrollable can collapse their height to 0.
    let page_host: Element<'_, Message> = match &state.route {
        Route::Forge => page,
        _ => scrollable(page).width(Length::Fill).height(Length::Fill).into(),
    };

    let right = Column::new()
        .spacing(14)
        .push(header)
        .push(page_host)
        .width(Length::Fill)
        .height(Length::Fill);

    let root = Row::new()
        .spacing(0)
        .push(container(sidebar).width(Length::Fixed(240.0)).height(Length::Fill))
        .push(right)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut stack = Stack::new().push(ui::shell(t, root.into()));

    // OVERLAYS (Modals)
    if let PmState::Editing {
        title,
        description,
        priority,
        card_id,
        ..
    } = &state.pm_state
    {
        let is_new = card_id.is_none();
        stack = stack.push(crate::pages::pm_board::render_modal(
            t,
            title,
            description,
            priority,
            is_new,
        ));
    }
    if let Some(editor) = &state.creature_editor {
        stack = stack.push(pages::bestiary::render_creature_modal(t, editor, &state.locations));
    }
    if let Some(editor) = &state.location_editor {
        stack = stack.push(pages::locations::render_location_modal(t, editor));
    }
    if let Some(editor) = &state.event_editor {
        stack = stack.push(pages::timeline::render_event_modal(t, editor, &state.locations));
    }
    if let Some(editor) = &state.era_editor {
        stack = stack.push(pages::timeline::render_era_modal(t, editor));
    }

    // ✅ OPTIMIZED Dragging Ghost - Zero lookups per frame
    if let PmState::Dragging {
        card_title,       // <-- Use cached title (O(1))
        current_cursor,
        active,
        ..
    } = &state.pm_state
    {
        if *active {
            // ✅ PERFORMANCE: Direct reference, no HashMap lookup needed
            // Before: O(n) search through all cards × 60 FPS = 6000+ iterations/sec
            // After: O(1) reference × 60 FPS = 60 references/sec
            let title: &str = card_title.as_str();

            let ghost = container(
                text(title)
                    .size(14)
                    .style(move |_| iced::widget::text::Style {
                        color: Some(t.foreground),
                    }),
            )
                .padding(12)
                .width(Length::Fixed(280.0))
                .style(move |_: &Theme| {
                    let mut s = ui::container_style(t.shell_b, t.foreground);
                    s.border.color = t.accent;
                    s.border.width = 2.0;
                    s.border.radius = 6.0.into();
                    s.background = Some(ui::alpha(t.background, 0.9).into());
                    s.shadow = iced::Shadow {
                        color: Color::BLACK,
                        offset: Vector::new(0.0, 10.0),
                        blur_radius: 20.0,
                    };
                    s
                });

            stack = stack.push(container(ghost).padding(iced::Padding {
                top: current_cursor.y + 10.0,
                left: current_cursor.x + 10.0,
                bottom: 0.0,
                right: 0.0,
            }));
        }
    }
    // Confirm Modal (debe estar ANTES del debug overlay para aparecer encima)
    if let Some(ref action) = state.pending_confirm {
        stack = stack.push(confirm_modal(action, t));
    }

    // Debug Overlay (above modals)
    if state.debug_overlay_open {
        stack = stack.push(debug_overlay(state, t));
    }

    // Toasts
    if !state.toasts.is_empty() {
        stack = stack.push(ui::toasts_overlay(t, &state.toasts));
    }

    stack.into()
}
