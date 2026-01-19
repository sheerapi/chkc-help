//! tiny clap help renderer with markdown output and a bit of color.
//!
//! the flow is simple:
//! - parse args with clap and send your match arm into `help_command` (or the `_docs` variant if
//!   you registered extra notes)
//! - `HelpPage` turns clap metadata into markdown-friendly structs
//! - `renderer` prints it with `termimad`, scrolling automatically if it doesn't fit
//! - `HelpTheme` keeps colors consistent and swappable
//!
//! everything is re-exported from here so you rarely need to dig into submodules.

mod doc_registry;
mod help_command;
mod help_page;
mod renderer;
mod theme;

pub use doc_registry::{CommandDoc, DocRegistry};
pub use help_command::{
    help_command, help_command_docs, help_command_program, help_command_program_docs, resolve_help, run_help_topic, HelpArgs,
    HelpTarget,
};
pub use help_page::HelpPage;
pub use renderer::{render_command_help, run_scrollable_help};
pub use theme::{apply_accent, HelpTheme};

pub use termimad::crossterm::style::Color;
pub use termimad::MadSkin;
