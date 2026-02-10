use iced::{Alignment, Length};
use iced::widget::{container, text, Column, Row, text_input};

use crate::app::{AppState, Message, Route, BestiaryMessage, LocationsMessage, UniverseMessage, TimelineMessage};
use crate::{ui, pages::E};
use crate::state::DemoResetScope;

pub fn universe_detail<'a>(state: &'a AppState, t: ui::Tokens, universe_id: &'a str) -> E<'a> {
    let u = state.universes.iter().find(|x| x.id == universe_id);

    let (name, desc) = match u {
        Some(u) => (u.name.clone(), u.description.clone()),
        None => ("Unknown".to_string(), "".to_string()),
    };

    let header_left = Column::new()
        .spacing(4)
        .push(text(name.clone()).size(26).color(t.foreground))
        .push(text(desc).size(12).color(t.muted_fg))
        .push(text("Status: Active").size(12).color(t.muted_fg));

    let header_right = Row::new()
        .spacing(10)
        .push(ui::outline_button(t, "Back to universes".to_string(), Message::BackToUniverses))
        .push(ui::outline_button(t, "Go to PM Tools".to_string(), Message::Navigate(Route::PmList)))
        .push(ui::outline_button(t, "Toggle Debug Overlay".to_string(), Message::Universe(UniverseMessage::ToggleDebugOverlay)));

    let header = Row::new()
        .align_y(Alignment::Center)
        .push(container(header_left).width(Length::Fill))
        .push(header_right);

    let uid = universe_id.to_string();

    let tools = Column::new()
        .spacing(8)
        .push(text("Universe tools").size(12).color(t.muted_fg))
        .push(
            Row::new()
                .spacing(10)
                .push(ui::outline_button(
                    t,
                    "Bestiary".to_string(),
                    Message::Bestiary(BestiaryMessage::Open(uid.clone())),
                ))
                .push(ui::outline_button(
                    t,
                    "Locations".to_string(),
                    Message::Locations(LocationsMessage::Open(uid.clone())),
                ))
                .push(ui::outline_button(
                    t,
                    "Timeline".to_string(),
                    Message::Timeline(TimelineMessage::Open(uid.clone())),
                ))
        );

    let is_arhelis = universe_id == "u-arhelis-01";
    let busy = state.db_inflight.is_some();
    let mut body = Column::new()
        .spacing(20)
        .push(header)
        .push(ui::h_divider(t))
        .push(tools);

    if is_arhelis {
        let dev_header = Row::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(text("Developer Options (Arhelis only)").size(12).color(t.muted_fg))
            .push(ui::outline_button(
                t,
                if state.dev_panel_open { "Collapse".to_string() } else { "Expand".to_string() },
                Message::Universe(UniverseMessage::ToggleDeveloperPanel),
            ));

        body = body.push(ui::h_divider(t)).push(dev_header);

        if state.dev_panel_open {
            let qa = ui::card(
                t,
                Column::new()
                    .spacing(6)
                    .push(text("Status + QA").size(12).color(t.muted_fg))
                    .push(text(format!("Snapshots: {}", state.snapshots.len())).size(12).color(t.muted_fg))
                    .push(text(format!("Integrity issues: {}", state.integrity_issues.len())).size(12).color(t.muted_fg))
                    .into(),
            );

            let snap_input = text_input("Snapshot name", &state.snapshot_name)
                .on_input(|v| Message::Universe(UniverseMessage::SnapshotNameChanged(v)))
                .padding(10);

            // ✅ Removed "Refresh" button
            let snap_actions = Row::new()
                .spacing(10)
                .push(if busy {
                    ui::card(t, text("Create Snapshot (busy)").size(12).color(t.muted_fg).into())
                } else {
                    ui::primary_button(
                        t,
                        "Create Snapshot".to_string(),
                        Message::Universe(UniverseMessage::SnapshotCreate(universe_id.to_string())),
                    )
                })
                .push(if busy {
                    ui::card(t, text("Refresh (busy)").size(12).color(t.muted_fg).into())
                } else {
                    ui::outline_button(
                        t,
                        "Refresh".to_string(),
                        Message::Universe(UniverseMessage::SnapshotRefresh(universe_id.to_string())),
                    )
                })
                .push(if busy || state.integrity_busy {
                    ui::card(t, text("Validate (busy)").size(12).color(t.muted_fg).into())
                } else {
                    ui::outline_button(
                        t,
                        "Validate Integrity".to_string(),
                        Message::Universe(UniverseMessage::ValidateUniverse(universe_id.to_string())),
                    )
                });

            let mut snap_list = Column::new().spacing(6);
            if state.snapshots.is_empty() {
                snap_list = snap_list.push(text("No snapshots yet.").size(12).color(t.muted_fg));
            } else {
                for s in state.snapshots.iter().take(8) {
                    let sid = s.id.clone();

                    let row = Row::new()
                        .spacing(10)
                        .push(text(format!("{}  —  {}", s.name, s.created_at)).size(12).color(t.foreground))
                        .push(if busy {
                            ui::card(t, text("Restore (busy)").size(12).color(t.muted_fg).into())
                        } else {
                            ui::outline_button(
                                t,
                                "Restore".to_string(),
                                Message::Universe(UniverseMessage::SnapshotRestore(sid.clone())),
                            )
                        })
                        .push(if busy {
                            ui::card(t, text("Delete (busy)").size(12).color(t.muted_fg).into())
                        } else {
                            ui::danger_button(
                                t,
                                "Delete".to_string(),
                                Message::Universe(UniverseMessage::SnapshotDelete(sid)),
                            )
                        });

                    snap_list = snap_list.push(ui::card(t, row.into()));
                }
            }

            let snapshots_section = Column::new()
                .spacing(10)
                .push(text("Snapshots").size(12).color(t.muted_fg))
                .push(snap_input)
                .push(snap_actions)
                .push(snap_list);

            let inject_row = Row::new()
                .spacing(10)
                .push(if busy {
                    ui::card(t, text("Inject Demo Data (busy)").size(12).color(t.muted_fg).into())
                } else {
                    ui::primary_button(
                        t,
                        "Inject Demo Data".to_string(),
                        Message::Universe(UniverseMessage::InjectDemoData(universe_id.to_string())),
                    )
                });

            let reset_row = if busy {
                Column::new()
                    .spacing(6)
                    .push(text("Dev Tools locked (DB busy).").size(12).color(t.muted_fg))
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(ui::card(t, text("Reset ALL (busy)").size(12).color(t.muted_fg).into()))
                            .push(ui::card(t, text("Reset Timeline (busy)").size(12).color(t.muted_fg).into()))
                            .push(ui::card(t, text("Reset Locations (busy)").size(12).color(t.muted_fg).into()))
                            .push(ui::card(t, text("Reset Bestiary (busy)").size(12).color(t.muted_fg).into()))
                            .push(ui::card(t, text("Reset PM Tools (busy)").size(12).color(t.muted_fg).into()))
                    )
            } else {
                Column::new()
                    .spacing(6)
                    .push(
                        Row::new()
                            .spacing(10)
                            .push(ui::danger_button(
                                t,
                                "Reset ALL".to_string(),
                                Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::All)),
                            ))
                            .push(ui::outline_button(
                                t,
                                "Reset Timeline".to_string(),
                                Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::Timeline)),
                            ))
                            .push(ui::outline_button(
                                t,
                                "Reset Locations".to_string(),
                                Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::Locations)),
                            ))
                            .push(ui::outline_button(
                                t,
                                "Reset Bestiary".to_string(),
                                Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::Bestiary)),
                            ))
                            .push(ui::outline_button(
                                t,
                                "Reset PM Tools".to_string(),
                                Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::PmTools)),
                            ))
                    )
            };

            let dev_body = Column::new()
                .spacing(14)
                .push(qa)
                .push(snapshots_section)
                .push(text("Demo Data Tools").size(12).color(t.muted_fg))
                .push(inject_row)
                .push(reset_row);

            body = body.push(dev_body);
        }
    }

    ui::page_padding(body.width(Length::Fill).into())
}
