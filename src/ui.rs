use iced::{
    border, gradient, Alignment, Background, Border, Color, ContentFit, Element, Length,
    Padding, Radians, Shadow, Theme, Vector,
};
use iced::widget::{button, container, text, Column, Row, Space};
use crate::app::{AppState, Message, Route, APP_NAME};
use crate::state::{Toast, ToastKind};

// --- [TOKENS Y ESTILOS VIEJOS SE MANTIENEN IGUALES] ---
pub type E<'a> = Element<'a, Message>;
const APP_SLOGAN: &str = "Where Reality Begins.";
#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub background: Color, pub foreground: Color, pub muted_fg: Color, pub accent: Color,
    pub shell_a: Color, pub shell_b: Color, pub card: Color, pub popover: Color, pub sidebar_bg: Color,
    pub border: Color, pub input_border: Color, pub hover_bg: Color, pub active_bg: Color,
    pub radius_xl: f32,
}
impl Tokens {
    pub fn nub_dark() -> Self {
        let white = Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0);
        Self {
            background: Color::from_rgba8(0x05, 0x05, 0x05, 1.0),
            foreground: Color::from_rgba8(0xF2, 0xF2, 0xF2, 1.0),
            muted_fg: Color::from_rgba8(0xA1, 0xA1, 0xAA, 1.0),
            accent: Color::from_rgba8(0x63, 0x66, 0xF1, 1.0),
            shell_a: Color::from_rgba8(0x18, 0x18, 0x1B, 1.0),
            shell_b: Color::from_rgba8(0x09, 0x09, 0x0B, 1.0),
            card: Color::from_rgba8(0x18, 0x18, 0x1B, 0.60),
            popover: Color::from_rgba8(0x1D, 0x1D, 0x1D, 0.94),
            sidebar_bg: Color::from_rgba8(0x09, 0x09, 0x0B, 1.0),
            border: alpha(white, 0.08), input_border: alpha(white, 0.12),
            hover_bg: alpha(white, 0.04), active_bg: alpha(white, 0.08),
            radius_xl: 16.0,
        }
    }
}
pub fn alpha(mut c: Color, a: f32) -> Color { c.a = a; c }
fn svg_icon(path: &str, color: Color) -> Element<'static, Message> {
    let svg_content = format!(r#"<svg viewBox="0 0 24 24" fill="none" stroke="rgba({},{},{},{})" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">{}</svg>"#, (color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8, color.a, path);
    iced::widget::svg(iced::widget::svg::Handle::from_memory(svg_content.into_bytes())).width(Length::Fixed(18.0)).height(Length::Fixed(18.0)).content_fit(ContentFit::Contain).into()
}
struct Icons;
impl Icons {
    const HOME: &'static str = r#"<path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline>"#;
    const GRID: &'static str = r#"<rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect>"#;
    const PLANET: &'static str = r#"<circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>"#;
    const FORGE: &'static str = r#"<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path>"#;
    const LIST: &'static str = r#"<line x1="8" y1="6" x2="21" y2="6"></line><line x1="8" y1="12" x2="21" y2="12"></line><line x1="8" y1="18" x2="21" y2="18"></line><line x1="3" y1="6" x2="3.01" y2="6"></line><line x1="3" y1="12" x2="3.01" y2="12"></line><line x1="3" y1="18" x2="3.01" y2="18"></line>"#;
    const ASSETS: &'static str = r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line>"#;
    const TRASH: &'static str = "M3 6h18M8 6V4a1 1 0 011-1h6a1 1 0 011 1v2m3 0v12a2 2 0 01-2 2H7a2 2 0 01-2-2V6h14z";
    const SETTINGS: &'static str = r#"<circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>"#;
}
pub fn container_style(bg: Color, fg: Color) -> iced::widget::container::Style {
    iced::widget::container::Style { text_color: Some(fg), background: Some(Background::Color(bg)), border: Border { color: Color::TRANSPARENT, width: 0.0, radius: border::Radius::from(0.0) }, shadow: Shadow::default(), snap: false }
}
pub fn shell_style(t: Tokens) -> iced::widget::container::Style {
    let grad = gradient::Linear::new(Radians::PI / 4.0).add_stop(0.0, t.shell_a).add_stop(1.0, t.shell_b);
    iced::widget::container::Style { text_color: Some(t.foreground), background: Some(grad.into()), border: Border { color: Color::TRANSPARENT, width: 0.0, radius: border::Radius::from(0.0) }, shadow: Shadow::default(), snap: false }
}
pub fn premium_card_style(t: Tokens) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| { let mut s = iced::widget::button::Style::default(); s.background = Some(Background::Color(t.card)); s.text_color = t.foreground; s.border = Border { color: t.border, width: 1.0, radius: border::Radius::from(t.radius_xl) }; s.shadow = Shadow { color: Color::from_rgba8(0,0,0,0.3), offset: Vector::new(0.0, 4.0), blur_radius: 12.0 }; if let iced::widget::button::Status::Hovered = status { s.border.color = alpha(t.accent, 0.5); s.background = Some(Background::Color(alpha(t.shell_a, 0.9))); s.shadow = Shadow { color: alpha(t.accent, 0.2), offset: Vector::new(0.0, 0.0), blur_radius: 20.0 }; } if let iced::widget::button::Status::Pressed = status { s.border.color = t.accent; s.background = Some(Background::Color(t.shell_b)); } s }
}
pub fn input_style(t: Tokens) -> impl Fn(&Theme, iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    move |_, status| { let base = iced::widget::text_input::Style { background: Background::Color(t.input_border), border: Border { color: t.border, width: 1.0, radius: 6.0.into() }, icon: t.muted_fg, placeholder: t.muted_fg, value: t.foreground, selection: t.accent }; match status { iced::widget::text_input::Status::Focused { .. } => iced::widget::text_input::Style { border: Border { color: t.accent, width: 1.0, radius: 6.0.into() }, ..base }, _ => base, } }
}
pub fn text_editor_style(t: Tokens) -> impl Fn(&Theme, iced::widget::text_editor::Status) -> iced::widget::text_editor::Style {
    move |_, status| { let base = iced::widget::text_editor::Style { background: Background::Color(t.input_border), border: Border { color: t.border, width: 1.0, radius: 6.0.into() }, placeholder: t.muted_fg, value: t.foreground, selection: t.accent }; match status { iced::widget::text_editor::Status::Focused { .. } => iced::widget::text_editor::Style { border: Border { color: t.accent, width: 1.0, radius: 6.0.into() }, ..base }, _ => base, } }
}
pub fn primary_button_style(t: Tokens) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_, status| { let mut s = iced::widget::button::Style::default(); let base = alpha(t.accent, 0.15); let hover = alpha(t.accent, 0.35); let press = alpha(t.accent, 0.10); let bg = match status { iced::widget::button::Status::Hovered => hover, iced::widget::button::Status::Pressed => press, _ => base }; s.background = Some(Background::Color(bg)); if let iced::widget::button::Status::Hovered = status { s.shadow = Shadow { color: alpha(t.accent, 0.3), offset: Vector::new(0.0, 0.0), blur_radius: 12.0 }; s.border = Border { color: alpha(t.accent, 0.6), width: 1.0, radius: border::Radius::from(6.0) }; } else { s.shadow = Shadow::default(); s.border = Border { color: Color::TRANSPARENT, width: 0.0, radius: border::Radius::from(6.0) }; } s.text_color = t.foreground; s }
}
pub fn ghost_button_style(t: Tokens) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_: &Theme, status| { let mut s = iced::widget::button::Style::default(); let bg = match status { iced::widget::button::Status::Hovered => t.hover_bg, iced::widget::button::Status::Pressed => t.active_bg, _ => Color::TRANSPARENT, }; s.background = Some(Background::Color(bg)); s.text_color = t.muted_fg; s.border = Border { radius: border::Radius::from(6.0), ..Border::default() }; s }
}
pub fn shell<'a>(t: Tokens, content: E<'a>) -> E<'a> { container(content).width(Length::Fill).height(Length::Fill).style(move |_| shell_style(t)).into() }
pub fn page_padding<'a>(content: E<'a>) -> E<'a> { container(content).padding(Padding { top: 24.0, right: 32.0, bottom: 32.0, left: 32.0 }).width(Length::Fill).into() }
pub fn section_title(t: Tokens, title: String, subtitle: Option<String>) -> Element<'static, Message> { let mut col = Column::new().spacing(6); col = col.push(text(title).size(32).color(t.foreground)); if let Some(s) = subtitle { col = col.push(text(s).size(14).color(t.muted_fg)); } col.into() }
pub fn card<'a>(t: Tokens, content: E<'a>) -> E<'a> { container(content).padding(20).width(Length::Fill).style(move |_: &Theme| { let mut s = container_style(t.card, t.foreground); s.border = Border { color: t.border, width: 1.0, radius: border::Radius::from(t.radius_xl) }; s.shadow = Shadow { color: Color::from_rgba8(0, 0, 0, 0.40), offset: Vector::new(0.0, 8.0), blur_radius: 24.0 }; s }).into() }
pub fn outline_button(t: Tokens, label: String, on_press: Message) -> Element<'static, Message> { button(text(label).size(13).color(t.foreground)).padding([8, 16]).style(move |_: &Theme, status| { let mut s = iced::widget::button::Style::default(); let bg = match status { iced::widget::button::Status::Hovered => t.hover_bg, iced::widget::button::Status::Pressed => t.active_bg, _ => Color::TRANSPARENT }; s.background = Some(Background::Color(bg)); s.border = Border { color: t.border, width: 1.0, radius: border::Radius::from(6.0) }; s.text_color = t.foreground; s }).on_press(on_press).into() }
pub fn ghost_button(t: Tokens, label: String, on_press: Message) -> Element<'static, Message> { button(text(label).size(13).color(t.muted_fg)).padding([6, 12]).style(ghost_button_style(t)).on_press(on_press).into() }
pub fn primary_button(t: Tokens, label: String, on_press: Message) -> Element<'static, Message> { button(text(label).size(13).color(t.foreground)).padding([8, 16]).style(primary_button_style(t)).on_press(on_press).into() }
pub fn danger_button(_t: Tokens, label: String, on_press: Message) -> Element<'static, Message> { let danger = Color::from_rgba8(0xEF, 0x44, 0x44, 1.0); button(text(label).size(13).color(danger)).padding([6, 12]).style(move |_: &Theme, status| { let mut s = iced::widget::button::Style::default(); let bg = match status { iced::widget::button::Status::Hovered => alpha(danger, 0.1), iced::widget::button::Status::Pressed => alpha(danger, 0.05), _ => alpha(danger, 0.0) }; s.background = Some(Background::Color(bg)); s.border = Border { color: alpha(danger, 0.2), width: 1.0, radius: border::Radius::from(6.0) }; s.text_color = danger; s }).on_press(on_press).into() }

pub fn h_divider(t: Tokens) -> Element<'static, Message> { container(Space::new()).width(Length::Fill).height(Length::Fixed(1.0)).style(move |_: &Theme| container_style(alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.06), t.foreground)).into() }
#[derive(Debug, Clone, Copy)] enum NavKey { Overview, Workspaces, Universe, Forge, PmTools, Assets, Trash, Settings }
fn is_active(state: &AppState, key: NavKey) -> bool { match (key, &state.route) { (NavKey::Overview, Route::Overview) => true, (NavKey::Workspaces, _) => false, (NavKey::Universe, Route::UniverseList) | (NavKey::Universe, Route::UniverseDetail { .. }) | (NavKey::Universe, Route::Bestiary { .. }) | (NavKey::Universe, Route::Timeline { .. }) => true, (NavKey::Forge, Route::Forge) => true, (NavKey::PmTools, Route::PmList) | (NavKey::PmTools, Route::PmBoard { .. }) => true, (NavKey::Assets, Route::Assets) => true, (NavKey::Settings, Route::Account) => true, _ => false } }
fn icon_path_for(key: NavKey) -> &'static str {
    match key {
        NavKey::Overview => Icons::HOME,
        NavKey::Workspaces => Icons::GRID,
        NavKey::Universe => Icons::PLANET,
        NavKey::Forge => Icons::FORGE,
        NavKey::PmTools => Icons::LIST,
        NavKey::Assets => Icons::ASSETS,
        NavKey::Trash => Icons::TRASH,        // ← AGREGAR ESTA LÍNEA
        NavKey::Settings => Icons::SETTINGS
    }
}
fn group_label<'a>(label: &'a str, t: Tokens) -> E<'a> { container(text(label).size(11).color(alpha(t.muted_fg, 0.6))).width(Length::Fill).padding(Padding { top: 16.0, right: 16.0, bottom: 8.0, left: 16.0 }).into() }
fn nav_button_style(t: Tokens, active: bool, status: iced::widget::button::Status) -> iced::widget::button::Style { let mut s = iced::widget::button::Style::default(); if active { s.background = Some(Background::Color(alpha(t.accent, 0.10))); s.text_color = t.foreground; } else { let bg = match status { iced::widget::button::Status::Hovered => t.hover_bg, iced::widget::button::Status::Pressed => t.active_bg, _ => Color::TRANSPARENT }; s.background = Some(Background::Color(bg)); s.text_color = t.muted_fg; } s.border = Border { color: Color::TRANSPARENT, width: 0.0, radius: border::Radius::from(8.0) }; s }
fn nav_item<'a>(t: Tokens, label: &'a str, key: NavKey, on_press: Message, active: bool) -> E<'a> { let icon_color = if active { t.accent } else { alpha(t.muted_fg, 0.8) }; let icon = container(svg_icon(icon_path_for(key), icon_color)).width(Length::Fixed(20.0)).align_x(Alignment::Center); let label_widget = text(label).size(14).color(if active { t.foreground } else { t.muted_fg }); let inner = Row::new().spacing(12).align_y(Alignment::Center).push(icon).push(label_widget); Element::new( button(container(inner).width(Length::Fill).height(Length::Fill).align_y(Alignment::Center).padding([0, 12])).width(Length::Fill).height(Length::Fixed(40.0)).style(move |_: &Theme, status| nav_button_style(t, active, status)).on_press(on_press) ) }

pub fn sidebar<'a>(state: &'a AppState, t: Tokens) -> E<'a> {
    let mut modules = Column::new().spacing(4).width(Length::Fill);
    modules = modules
        .push(group_label("MODULES", t))
        .push(nav_item(t, "Overview", NavKey::Overview, Message::Navigate(Route::Overview), is_active(state, NavKey::Overview)))
        .push(nav_item(t, "Workspaces", NavKey::Workspaces, Message::Navigate(Route::Workspaces), is_active(state, NavKey::Workspaces)))
        .push(nav_item(t, "Universe", NavKey::Universe, Message::Navigate(Route::UniverseList), is_active(state, NavKey::Universe)))
        // CAMBIO CRÍTICO: Enviamos "" para que el controller active el modo Standalone
        .push(nav_item(t, "The Forge", NavKey::Forge, Message::TheForge(crate::messages::TheForgeMessage::Open(None)), is_active(state, NavKey::Forge)))
        .push(nav_item(t, "PM Tools", NavKey::PmTools, Message::Navigate(Route::PmList), is_active(state, NavKey::PmTools)))
        .push(nav_item(t, "Assets", NavKey::Assets, Message::Navigate(Route::Assets), is_active(state, NavKey::Assets)))
        .push(nav_item(t, "Trash", NavKey::Trash, Message::Navigate(Route::Trash), is_active(state, NavKey::Trash)));

    let mut account = Column::new().spacing(4).width(Length::Fill);
    account = account.push(nav_item(t, "Settings", NavKey::Settings, Message::Navigate(Route::Account), is_active(state, NavKey::Settings)));

    let inner_content = Column::new().width(Length::Fill).height(Length::Fill)
        .push(modules)
        .push(container(Space::new()).height(Length::Fill))
        .push(account)
        .padding(Padding { top: 10.0, right: 12.0, bottom: 20.0, left: 12.0 })
        .spacing(10);

    Element::new(
        container(inner_content).width(Length::Fill).height(Length::Fill).style(move |_: &Theme| {
            let mut s = container_style(t.sidebar_bg, t.foreground);
            s.border = Border { color: Color::TRANSPARENT, width: 0.0, radius: border::Radius::from(0.0) };
            s
        })
    )
}

fn workspace_pill(t: Tokens, label: String) -> Element<'static, Message> { container(text(label).size(12).color(t.foreground)).padding([6, 12]).style(move |_: &Theme| { let mut s = container_style(alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.04), t.foreground); s.border = Border { color: alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.1), width: 1.0, radius: border::Radius::from(999.0) }; s }).into() }
pub fn header<'a>(state: &'a AppState, t: Tokens) -> E<'a> { let ws_name = state.active_project.as_ref().map(|p| p.name.clone()).unwrap_or("Launcher".to_string()); let brand = Column::new().spacing(0).push(text(APP_NAME).size(15).color(t.foreground)).push(text(APP_SLOGAN).size(12).color(alpha(t.muted_fg, 0.7))); let right = Row::new().spacing(12).align_y(Alignment::Center).push(workspace_pill(t, ws_name)); let bar = Row::new().align_y(Alignment::Center).push(container(brand).width(Length::Fill)).push(right).padding(Padding { top: 16.0, right: 32.0, bottom: 16.0, left: 32.0 }); Element::new( container(bar).width(Length::Fill).style(move |_: &Theme| container_style(Color::TRANSPARENT, t.foreground)) ) }
pub fn recents_card_universe<'a>(t: Tokens) -> E<'a> { card(t, Column::new().push(text("Arhelis (Universe)").size(14).color(t.foreground)).into()) }
pub fn recents_card_forge<'a>(t: Tokens) -> E<'a> { card(t, Column::new().push(text("Chapter 1: The Awakening").size(14).color(t.foreground)).into()) }

pub fn toasts_overlay<'a>(t: Tokens, toasts: &'a [Toast]) -> E<'a> {
    let mut col = Column::new().spacing(10);
    for toast in toasts {
        let (bg, icon) = match toast.kind {
            ToastKind::Info => (t.shell_b, "ℹ"),
            ToastKind::Success => (Color::from_rgb8(22, 101, 52), "✓"),
            ToastKind::Error => (Color::from_rgb8(153, 27, 27), "!"),
        };
        let content = Row::new().align_y(Alignment::Center).spacing(12)
            .push(text(icon).size(16).color(t.foreground))
            .push(text(&toast.message).size(14).color(t.foreground).width(Length::Fill))
            .push(button(text("×").size(16).color(t.muted_fg)).style(ghost_button_style(t)).on_press(Message::ToastDismiss(toast.id)));
        let card = container(content).width(Length::Fixed(320.0)).padding(12).style(move |_| { let mut s = container_style(bg, t.foreground); s.border = Border { color: t.border, width: 1.0, radius: border::Radius::from(8.0) }; s.shadow = Shadow { color: Color::BLACK, offset: Vector::new(0.0, 4.0), blur_radius: 12.0 }; s });
        col = col.push(card);
    }
    container(col).padding(20).width(Length::Fill).height(Length::Fill).align_x(iced::alignment::Horizontal::Right).align_y(iced::alignment::Vertical::Bottom).into()
}
