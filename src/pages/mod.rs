pub mod overview;
pub mod workspaces;
pub mod universe_list;
pub mod universe_detail;
pub mod bestiary;
pub mod locations;
pub mod timeline;
pub mod pm_list;
pub mod pm_board;
pub mod launcher;
pub mod stubs;        // ✅ requerido por ui_shell.rs
pub mod the_forge;
pub mod trash;

// --- RE-EXPORTS ---
pub use overview::overview;
pub use universe_list::universe_list;
pub use universe_detail::universe_detail;
pub use bestiary::bestiary;
pub use the_forge::the_forge;

pub use stubs::{account_stub, assets_stub}; // ✅ requerido por ui_shell.rs

pub use trash::trash_page;

pub type E<'a> = iced::Element<'a, crate::messages::Message>;
