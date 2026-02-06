// ============================================
// UI: THE FORGE V4 (COMPLETE IMPLEMENTATION)
// ============================================
// Funcionalidades implementadas:
// 1. Botones +/- para expandir/contraer
// 2. NO selección en cascada
// 3. Rename input COMPLETAMENTE VISIBLE
// 4. Editor más claro que outline panel
// 5. Dividers alineados
// 6. Pills mismo tamaño (36px)
// ============================================

use iced::{Alignment, Background, Border, Color, Element, Length, Theme};
use iced::widget::{
    self, button, column, container, row, scrollable, text, text_editor, text_input, Column, Id, Row,
    Space,
};

use crate::app::{AppState, Message};
use crate::messages::TheForgeMessage;
use crate::model::{Chapter, Scene};
use crate::ui::{self, Tokens};

// --- CONSTANTS ---
const INDENT_CHAPTER: f32 = 20.0;
const INDENT_SCENE: f32 = 40.0;
const PILL_HEIGHT: f32 = 36.0;

// Premium alignment constants
const STATUS_COL_W: f32 = 64.0; // "Draft" column width
const METRIC_COL_W: f32 = 64.0; // Word count / metric column width (match Draft)

#[inline]
fn stable_key(tag: u64, id: &str) -> u64 {
    // FNV-1a 64-bit (determinístico, rápido, cero allocations)
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET ^ tag;
    for &b in id.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[inline]
fn stable_key_v(tag: u64, id: &str, version: u64) -> u64 {
    // Misma base FNV-1a, pero “salteada” con versión para forzar reconstrucción del widget
    // cuando el renderer/cache de texto en Windows se pone terco.
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET ^ tag;
    for &b in id.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    // Mezcla final con versión (sin heap, sin strings)
    hash ^= version;
    hash = hash.wrapping_mul(FNV_PRIME);
    hash
}

// --- HELPERS ---

fn divider(t: Tokens) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| ui::container_style(ui::alpha(t.border, 0.6), t.border))
        .into()
}

fn selection_bar(_t: Tokens, _is_active: bool) -> Element<'static, Message> {
    // Barra invisible (limpio/premium)
    Space::new()
        .width(Length::Fixed(3.0))
        .height(Length::Fill)
        .into()
}

fn outline_item_style(t: Tokens, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let mut s = button::Style::default();

        // Fondos sutiles
        let hover = ui::alpha(Color::WHITE, 0.028);
        let active_bg = ui::alpha(Color::WHITE, 0.050);
        let pressed = ui::alpha(Color::WHITE, 0.070);

        let bg = if active {
            match status {
                button::Status::Pressed => pressed,
                _ => active_bg,
            }
        } else {
            match status {
                button::Status::Hovered => hover,
                button::Status::Pressed => pressed,
                _ => Color::TRANSPARENT,
            }
        };

        s.background = Some(bg.into());
        s.text_color = t.foreground;

        // Sin borde, radio suave
        s.border = Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        };

        s
    }
}

fn icon_btn<'a>(t: Tokens, label: &'a str, on_press: Message) -> Element<'a, Message> {
    button(text(label).size(14))
        .padding([4, 8])
        .style(ui::ghost_button_style(t))
        .on_press(on_press)
        .into()
}

fn danger_icon_btn<'a>(t: Tokens, label: &'a str, on_press: Message) -> Element<'a, Message> {
    let t_danger = Tokens {
        accent: Color::from_rgb(0.95, 0.4, 0.4),
        ..t
    };

    button(text(label).size(14))
        .padding([4, 8])
        .style(ui::ghost_button_style(t_danger))
        .on_press(on_press)
        .into()
}

// Estilo para rename input con máxima visibilidad
fn rename_input_style(
    t: Tokens,
) -> impl Fn(&Theme, iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    move |_, status| {
        use iced::widget::text_input;

        let focused = matches!(status, text_input::Status::Focused { .. });

        let bg = ui::alpha(Color::WHITE, if focused { 0.06 } else { 0.03 });
        let border = if focused {
            ui::alpha(t.accent, 0.35)
        } else {
            ui::alpha(Color::WHITE, 0.10)
        };

        text_input::Style {
            background: bg.into(),
            border: Border {
                color: border,
                width: 1.0,
                radius: 6.0.into(),
            },
            icon: ui::alpha(t.muted_fg, 0.7),

            placeholder: ui::alpha(t.muted_fg, 0.55),
            value: t.foreground,
            selection: ui::alpha(t.accent, 0.70),
        }
    }
}

fn rename_input<'a>(
    t: Tokens,
    id: Id,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_submit: Message,
) -> Element<'a, Message> {
    text_input("", value)
        .id(id)
        .on_input(on_input)
        .on_submit(on_submit)
        .size(13)
        .padding([0, 2]) // compacto
        .width(Length::Fill)
        .style(rename_input_style(t))
        .into()
}

// --- TREE ITEMS ---

fn novel_row<'a>(
    t: Tokens,
    title: &'a str,
    novel_id: String,
    is_active: bool,
    is_expanded: bool,
    is_renaming: bool,
) -> Element<'a, Message> {
    let bar = selection_bar(t, is_active);

    let expand_btn = icon_btn(
        t,
        if is_expanded { "−" } else { "+" },
        Message::TheForge(TheForgeMessage::ToggleNovel(novel_id.clone())),
    );

    let title_widget: Element<Message> = if is_renaming {
        rename_input(
            t,
            Id::new("forge_novel_rename"),
            title,
            |s| Message::TheForge(TheForgeMessage::NovelTitleChanged(s)),
            Message::TheForge(TheForgeMessage::EndRename),
        )
    } else {
        // String propio para evitar caches raros con &str prestado
        text(format!("{title}"))
            .size(13)
            .color(if is_active {
                t.foreground
            } else {
                ui::alpha(t.muted_fg, 0.85)
            })
            .width(Length::Fill)
            .into()
    };

    let content_row = row![bar, expand_btn, title_widget]
        .spacing(10)
        .align_y(Alignment::Center);

    let main: Element<Message> = if is_renaming {
        container(content_row)
            .padding([6, 10])
            .width(Length::Fill)
            .height(Length::Fixed(PILL_HEIGHT))
            .style(move |_| ui::container_style(ui::alpha(Color::WHITE, 0.04), t.foreground))
            .into()
    } else {
        button(content_row)
            .width(Length::Fill)
            .height(Length::Fixed(PILL_HEIGHT))
            .padding([6, 10])
            .style(outline_item_style(t, is_active))
            .on_press(Message::TheForge(TheForgeMessage::SelectNovel(novel_id.clone())))
            .into()
    };

    // Minimiza clones: usamos uno para delete sin duplicar lógica
    let nid = novel_id.clone();

    let buttons_row = row![
        icon_btn(
            t,
            "+",
            Message::TheForge(TheForgeMessage::CreateChapter(novel_id)),
        ),
        danger_icon_btn(t, "×", Message::TheForge(TheForgeMessage::DeleteNovel(nid))),
    ]
        .spacing(6)
        .align_y(Alignment::Center);

    row![main, buttons_row]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

fn chapter_row<'a>(
    t: Tokens,
    title: &'a str,
    status: &'a str,
    chapter_id: String,
    is_active: bool,
    is_expanded: bool,
    is_renaming: bool,
    outline_nonce: u32, // CANARIO + NUDGE: fuerza cambio real en shaping/cache
) -> Element<'a, Message> {
    let bar = selection_bar(t, is_active);

    let expand_btn = icon_btn(
        t,
        if is_expanded { "−" } else { "+" },
        Message::TheForge(TheForgeMessage::ToggleChapter(chapter_id.clone())),
    );

    let title_widget: Element<Message> = if is_renaming {
        rename_input(
            t,
            Id::new("forge_chapter_rename"),
            title,
            |s| Message::TheForge(TheForgeMessage::ChapterTitleChanged(s)),
            Message::TheForge(TheForgeMessage::EndRename),
        )
    } else {
        // 🧨 CANARIO VISUAL + LAYOUT NUDGE (industrial):
        // Alterna un caracter invisible para forzar “text shaping” distinto.
        let nudge = if (outline_nonce & 1) == 0 {
            '\u{200B}'
        } else {
            '\u{200C}'
        };

        // Canario: útil para verificar si el frame se dibuja realmente.
        let label = format!("{title}{nudge}  ·{}", outline_nonce);

        text(label)
            .size(13)
            .color(if is_active {
                t.foreground
            } else {
                ui::alpha(t.muted_fg, 0.85)
            })
            .width(Length::Fill)
            .into()
    };

    let status_text: Element<Message> = container(
        text(status)
            .size(11)
            .color(ui::alpha(t.muted_fg, 0.55)),
    )
        .padding([0, 8])
        .width(Length::Fixed(STATUS_COL_W))
        .align_x(Alignment::End)
        .into();

    let content_row = row![bar, expand_btn, title_widget, status_text]
        .spacing(10)
        .align_y(Alignment::Center);

    // MAIN siempre button (mismo tipo de widget)
    let mut main_btn = button(content_row)
        .width(Length::Fill)
        .padding([6, 10])
        .style(outline_item_style(t, is_active));

    if !is_renaming {
        main_btn = main_btn.on_press(Message::TheForge(TheForgeMessage::SelectChapter(
            chapter_id.clone(),
        )));
    }

    let main: Element<Message> = main_btn.into();

    let create_btn: Element<Message> = if is_renaming {
        button(text("+").size(14))
            .padding([4, 8])
            .style(ui::ghost_button_style(t))
            .into()
    } else {
        icon_btn(
            t,
            "+",
            Message::TheForge(TheForgeMessage::CreateScene(chapter_id.clone())),
        )
    };

    let delete_btn: Element<Message> = if is_renaming {
        button(text("×").size(14))
            .padding([4, 8])
            .style(ui::ghost_button_style(t))
            .into()
    } else {
        danger_icon_btn(
            t,
            "×",
            Message::TheForge(TheForgeMessage::DeleteChapter(chapter_id.clone())),
        )
    };

    let buttons_row = row![create_btn, delete_btn]
        .spacing(6)
        .align_y(Alignment::Center);

    container(
        row![
            Space::new().width(Length::Fixed(INDENT_CHAPTER)),
            main,
            buttons_row
        ]
            .spacing(8)
            .align_y(Alignment::Center),
    )
        .width(Length::Fill)
        .into()
}

fn scene_row<'a>(
    t: Tokens,
    title: &'a str,
    word_count: i64,
    scene_id: String,
    is_active: bool,
    is_renaming: bool,
) -> Element<'a, Message> {
    let bar = selection_bar(t, is_active);

    let title_widget: Element<Message> = if is_renaming {
        rename_input(
            t,
            Id::new("forge_scene_rename"),
            title,
            |s| Message::TheForge(TheForgeMessage::SceneTitleChanged(s)),
            Message::TheForge(TheForgeMessage::EndRename),
        )
    } else {
        text(format!("{title}"))
            .size(13)
            .color(if is_active {
                t.foreground
            } else {
                ui::alpha(t.muted_fg, 0.85)
            })
            .width(Length::Fill)
            .into()
    };

    // Word count como texto simple (no pill)
    let wc_text: Element<Message> = text(format!("{word_count}"))
        .size(11)
        .color(ui::alpha(t.muted_fg, 0.55))
        .width(Length::Fixed(METRIC_COL_W))
        .into();

    let content_row = if is_renaming {
        row![bar, title_widget].spacing(10).align_y(Alignment::Center)
    } else {
        row![bar, title_widget, wc_text]
            .spacing(10)
            .align_y(Alignment::Center)
    };

    let main: Element<Message> = if is_renaming {
        container(content_row)
            .padding([6, 10])
            .width(Length::Fill)
            .style(move |_| ui::container_style(ui::alpha(Color::WHITE, 0.04), t.foreground))
            .into()
    } else {
        button(content_row)
            .width(Length::Fill)
            .padding([6, 10])
            .style(outline_item_style(t, is_active))
            .on_press(Message::TheForge(TheForgeMessage::SelectScene(scene_id.clone())))
            .into()
    };

    let sid = scene_id;

    let buttons_row = row![
        // Scenes son hoja: solo delete
        danger_icon_btn(t, "×", Message::TheForge(TheForgeMessage::DeleteScene(sid))),
    ]
        .spacing(6)
        .align_y(Alignment::Center);

    container(
        row![
            Space::new().width(Length::Fixed(INDENT_SCENE)),
            main,
            buttons_row
        ]
            .spacing(8)
            .align_y(Alignment::Center),
    )
        .width(Length::Fill)
        .into()
}

// --- MAIN VIEW ---

pub fn the_forge<'a>(state: &'a AppState, t: Tokens) -> Element<'a, Message> {
    // ✅ FIX: Forzar que Iced detecte cambios en el outline (lectura intencional)
    let _ = state.forge_outline_version;

    let is_renaming_any = state.forge_renaming_novel_id.is_some()
        || state.forge_renaming_chapter_id.is_some()
        || state.forge_renaming_scene_id.is_some();

    // Construimos la lista de hijos del outline
    let mut outline_children: Vec<(u64, Element<'a, Message>)> = Vec::new();

    // Renderizar Novels
    for novel in &state.novels {
        let has_scene_selected = state.active_scene_id.is_some();
        let has_chapter_selected = state.active_chapter_id.is_some();

        let is_current_novel = state.active_novel_id.as_ref() == Some(&novel.id);

        let is_active_novel = is_current_novel && !has_chapter_selected && !has_scene_selected;

        let is_expanded = state.expanded_novels.contains(&novel.id);
        let is_renaming_novel = state.forge_renaming_novel_id.as_ref() == Some(&novel.id);

        // ✅ Menos clones: clonamos el id una vez por novela
        let novel_id = novel.id.clone();

        outline_children.push((
            stable_key(1, &novel_id),
            novel_row(
                t,
                &novel.title,
                novel_id,
                is_active_novel,
                is_expanded,
                is_renaming_novel,
            ),
        ));

        if is_expanded {
            let chapters: &[Chapter] = state
                .chapters_by_novel_id
                .get(&novel.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            for chapter in chapters {
                // ✅ Menos clones: clonamos una vez por capítulo (y lo reutilizamos)
                let chapter_id = chapter.id.clone();

                let is_active_chapter = state.active_chapter_id.as_ref() == Some(&chapter_id);
                let is_chapter_expanded = state.expanded_chapters.contains(&chapter_id);
                let is_renaming_chapter =
                    state.forge_renaming_chapter_id.as_ref() == Some(&chapter_id);

                // DEBUG: ver qué título se está renderizando
                crate::logger::info(&format!(
                    "   🎨 RENDER chapter {} title='{}'",
                    chapter_id, chapter.title
                ));

                outline_children.push((
                    // Key versionada: si outline_version sube, este row se reconstruye sí o sí
                    stable_key_v(2, &chapter_id, state.forge_outline_version as u64),
                    chapter_row(
                        t,
                        &chapter.title,
                        &chapter.status,
                        chapter_id.clone(),
                        is_active_chapter,
                        is_chapter_expanded,
                        is_renaming_chapter,
                        state.forge_outline_version,
                    ),
                ));

                if is_chapter_expanded {
                    let scenes: &[Scene] = state
                        .scenes_by_chapter_id
                        .get(&chapter_id)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);

                    for scene in scenes {
                        // ✅ Menos clones: clonamos una vez por escena
                        let scene_id = scene.id.clone();

                        let is_active_scene = state.active_scene_id.as_ref() == Some(&scene_id);
                        let is_renaming_scene =
                            state.forge_renaming_scene_id.as_ref() == Some(&scene_id);

                        outline_children.push((
                            stable_key(3, &scene_id),
                            scene_row(
                                t,
                                &scene.title,
                                scene.word_count,
                                scene_id,
                                is_active_scene,
                                is_renaming_scene,
                            ),
                        ));
                    }
                }
            }
        }
    }

    // Click en espacio vacío del outline = guardar rename (EndRename)
    if is_renaming_any {
        let spacer: Element<'a, Message> = iced::widget::Button::new(
            Space::new()
                .height(Length::Fixed(200.0))
                .width(Length::Fill),
        )
            .style(|_, _| iced::widget::button::Style {
                background: None,
                text_color: Color::TRANSPARENT,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                shadow: iced::Shadow::default(),
                snap: false,
            })
            .on_press(Message::TheForge(TheForgeMessage::EndRename))
            .into();

        outline_children.push((stable_key(9, "outline_rename_spacer"), spacer));
    }

    // 🔥 BULLDOZER:
    // Column normal para evitar reuse/caching por keys en Windows (wgpu).
    let mut outline = Column::new().spacing(2);
    for (_k, child) in outline_children {
        outline = outline.push(child);
    }

    // 🧨 INVALIDACIÓN DE VIEWPORT:
    // Toggle del Id cuando cambia outline_version => fuerza rebuild del scrollable
    let outline_scroll_id = if (state.forge_outline_version & 1) == 0 {
        "forge_outline_scroll_a"
    } else {
        "forge_outline_scroll_b"
    };

    let outline_section = Column::new()
        .spacing(8)
        .push(
            Row::new()
                .align_y(Alignment::Center)
                .push(text("Novels").size(12).color(t.muted_fg).width(Length::Fill))
                .push(icon_btn(t, "+", Message::TheForge(TheForgeMessage::CreateNovel))),
        )
        .push(divider(t))
        .push(scrollable(outline).id(Id::new(outline_scroll_id)).height(Length::Fill));

    let outline_panel = container(outline_section)
        .padding(16)
        .width(Length::Fixed(320.0))
        .height(Length::Fill)
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(t.shell_a, 0.5), t.foreground);
            s.border.width = 1.0;
            s.border.color = t.border;
            s.border.radius = 12.0.into();
            s
        });

    // Editor
    let editor_content = if state.active_scene_id.is_some() {
        column![
            Row::new()
                .align_y(Alignment::Center)
                .push(text("Editor").size(12).color(t.muted_fg).width(Length::Fill)),
            divider(t),
            text_editor(&state.forge_content)
                .on_action(|a| Message::TheForge(TheForgeMessage::SceneBodyChanged(a)))
                .padding(16)
                .height(Length::Fill)
                .style(move |theme: &Theme, status| {
                    let mut s = ui::text_editor_style(t)(theme, status);
                    // Área de texto más clara
                    s.background = Background::Color(ui::alpha(t.shell_a, 0.7));
                    s
                })
        ]
            .spacing(8)
    } else {
        column![
            Row::new()
                .align_y(Alignment::Center)
                .push(text("Editor").size(12).color(t.muted_fg).width(Length::Fill)),
            divider(t),
            container(
                text("Select a scene to start writing")
                    .size(14)
                    .color(ui::alpha(t.muted_fg, 0.6))
            )
            .padding(32)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        ]
            .spacing(8)
    };

    let editor_panel = container(editor_content)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(t.shell_a, 0.5), t.foreground);
            s.border.width = 1.0;
            s.border.color = t.border;
            s.border.radius = 12.0.into();
            s
        });

    let main_row = Row::new()
        .spacing(16)
        .push(outline_panel)
        .push(editor_panel);

    container(main_row)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
