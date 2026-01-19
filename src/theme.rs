//! small wrapper around `termimad::MadSkin` so you can pick an accent and move on.

use termimad::crossterm::style::Color;
use termimad::{Alignment, CompoundStyle, MadSkin};

/// accent-aware theme used by the renderer.
#[derive(Debug, Clone)]
pub struct HelpTheme {
    pub accent: Color,
    pub skin: MadSkin,
}

impl HelpTheme {
    /// apply an accent to an existing `MadSkin`.
    pub fn new(mut skin: MadSkin, accent: Color) -> Self {
        apply_accent(&mut skin, accent);
        Self { accent, skin }
    }

    /// light preset respecting the accent.
    pub fn light(accent: Color) -> Self {
        Self::new(MadSkin::default_light(), accent)
    }

    /// dark preset respecting the accent.
    pub fn dark(accent: Color) -> Self {
        Self::new(MadSkin::default_dark(), accent)
    }

    /// chooses light/dark based on terminal luminance.
    pub fn default(accent: Color) -> Self {
        if terminal_light::luma().map_or(false, |luma| luma > 0.6) {
            Self::light(accent)
        } else {
            Self::dark(accent)
        }
    }
}

/// tweak a `MadSkin` in-place to carry the accent through headers, bold, strikeouts, etc.
pub fn apply_accent(skin: &mut MadSkin, accent: Color) {
    skin.set_headers_fg(accent);
    skin.headers[0].align = Alignment::Center;
    skin.bold.set_fg(accent);
    skin.italic.set_fg(Color::DarkGrey);
    skin.strikeout = CompoundStyle::with_fg(accent);
    skin.scrollbar.thumb.set_fg(accent);
    skin.table_border_chars = termimad::ROUNDED_TABLE_BORDER_CHARS;
}
