// SPDX-License-Identifier: MIT OR Apache-2.0
//! SigmaOS-theme-y — Mint-Y GTK theme engine.
//!
//! Generates GTK3 CSS for the Mint-Y color palette family (Aqua, Blue, Green,
//! Grey, Orange, Pink, Red, Teal) with dark/light mode switching, window
//! decoration colours, icon theme mapping, and Catppuccin Mocha accent overrides
//! inspired by Omarchy's Wallust-driven theming pipeline.
//!
//! Principles: deterministic CSS output, fail-fast validation, zero `unsafe`.

#![forbid(unsafe_code)]

use std::fmt;

// ── Errors ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeError {
    InvalidHex(String),
    EmptyName,
    InvalidOpacity(String),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHex(h) => write!(f, "invalid hex colour: {h}"),
            Self::EmptyName => write!(f, "theme name must not be empty"),
            Self::InvalidOpacity(v) => write!(f, "opacity must be 0.0..=1.0, got {v}"),
        }
    }
}

impl std::error::Error for ThemeError {}

// ── Colour ──────────────────────────────────────────────────────────────

/// An RGB colour value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex string like `#aabbcc` or `aabbcc`.
    pub fn from_hex(s: &str) -> Result<Self, ThemeError> {
        let s = s.trim().trim_start_matches('#');
        if s.len() != 6 {
            return Err(ThemeError::InvalidHex(s.to_string()));
        }
        let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ThemeError::InvalidHex(s.to_string()))?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ThemeError::InvalidHex(s.to_string()))?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ThemeError::InvalidHex(s.to_string()))?;
        Ok(Self { r, g, b })
    }

    /// Render as `#rrggbb`.
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Render as `rgba(r, g, b, a)`.
    pub fn to_rgba(&self, alpha: f64) -> String {
        format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, alpha)
    }

    /// Lighten by a factor (0.0 = no change, 1.0 = white).
    pub fn lighten(&self, factor: f64) -> Self {
        let f = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f64 + (255.0 - self.r as f64) * f) as u8,
            g: (self.g as f64 + (255.0 - self.g as f64) * f) as u8,
            b: (self.b as f64 + (255.0 - self.b as f64) * f) as u8,
        }
    }

    /// Darken by a factor (0.0 = no change, 1.0 = black).
    pub fn darken(&self, factor: f64) -> Self {
        let f = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f64 * (1.0 - f)) as u8,
            g: (self.g as f64 * (1.0 - f)) as u8,
            b: (self.b as f64 * (1.0 - f)) as u8,
        }
    }
}

// ── Mode ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

// ── Mint-Y Variants ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintYVariant {
    Aqua,
    Blue,
    Green,
    Grey,
    Orange,
    Pink,
    Red,
    Teal,
}

impl MintYVariant {
    /// Accent colour for this variant.
    pub fn accent(&self) -> Colour {
        match self {
            Self::Aqua   => Colour::new(0x1f, 0x9e, 0xce),
            Self::Blue   => Colour::new(0x0c, 0x75, 0xde),
            Self::Green  => Colour::new(0x92, 0xb3, 0x72),
            Self::Grey   => Colour::new(0x8c, 0x8c, 0x8c),
            Self::Orange => Colour::new(0xe5, 0x96, 0x40),
            Self::Pink   => Colour::new(0xe0, 0x6e, 0x9e),
            Self::Red    => Colour::new(0xc1, 0x48, 0x48),
            Self::Teal   => Colour::new(0x2b, 0x9e, 0x93),
        }
    }

    /// Variant name as lowercase string.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Aqua => "aqua", Self::Blue => "blue", Self::Green => "green",
            Self::Grey => "grey", Self::Orange => "orange", Self::Pink => "pink",
            Self::Red => "red", Self::Teal => "teal",
        }
    }

    /// Suggested icon theme name.
    pub fn icon_theme(&self) -> String {
        format!("Mint-Y-{}", match self {
            Self::Aqua => "Aqua", Self::Blue => "Blue", Self::Green => "Green",
            Self::Grey => "Grey", Self::Orange => "Orange", Self::Pink => "Pink",
            Self::Red => "Red", Self::Teal => "Teal",
        })
    }
}

// ── Theme Config ────────────────────────────────────────────────────────

/// Full theme configuration.
#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub name: String,
    pub variant: MintYVariant,
    pub mode: Mode,
    pub border_radius: u32,
    pub font_family: String,
    pub font_size: u32,
    /// Window opacity for compositors (0.0..=1.0).
    pub window_opacity: f64,
}

impl ThemeConfig {
    pub fn new(name: impl Into<String>, variant: MintYVariant, mode: Mode) -> Result<Self, ThemeError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ThemeError::EmptyName);
        }
        Ok(Self {
            name,
            variant,
            mode,
            border_radius: 8,
            font_family: "FiraCode Nerd Font".into(),
            font_size: 10,
            window_opacity: 1.0,
        })
    }

    pub fn with_opacity(mut self, opacity: f64) -> Result<Self, ThemeError> {
        if !(0.0..=1.0).contains(&opacity) {
            return Err(ThemeError::InvalidOpacity(format!("{opacity}")));
        }
        self.window_opacity = opacity;
        Ok(self)
    }

    /// Background colour based on mode.
    pub fn bg(&self) -> Colour {
        match self.mode {
            Mode::Dark => Colour::new(0x1e, 0x1e, 0x2e),   // Catppuccin Mocha base
            Mode::Light => Colour::new(0xef, 0xf1, 0xf5),  // Catppuccin Latte base
        }
    }

    /// Foreground colour based on mode.
    pub fn fg(&self) -> Colour {
        match self.mode {
            Mode::Dark => Colour::new(0xcd, 0xd6, 0xf4),
            Mode::Light => Colour::new(0x4c, 0x4f, 0x69),
        }
    }

    /// Generate GTK3 CSS.
    pub fn to_gtk3_css(&self) -> String {
        let accent = self.variant.accent();
        let bg = self.bg();
        let fg = self.fg();
        let hover = accent.lighten(0.15);
        let border = match self.mode {
            Mode::Dark => bg.lighten(0.12),
            Mode::Light => bg.darken(0.12),
        };

        format!(
            r#"/* Mint-Y {variant} ({mode}) — Generated by SigmaOS-theme-y */
@define-color theme_bg_color {bg};
@define-color theme_fg_color {fg};
@define-color theme_selected_bg_color {accent};
@define-color theme_selected_fg_color #ffffff;
@define-color borders {border};

window {{
    background-color: {bg};
    color: {fg};
    border-radius: {radius}px;
    font-family: "{font}";
    font-size: {size}pt;
}}

button {{
    background-color: {accent};
    color: #ffffff;
    border-radius: {radius}px;
    border: 1px solid {border};
    padding: 6px 16px;
}}

button:hover {{
    background-color: {hover};
}}

headerbar {{
    background-color: {hdr_bg};
    color: {fg};
    border-bottom: 1px solid {border};
}}

.titlebar {{
    background-color: {hdr_bg};
}}
"#,
            variant = self.variant.name(),
            mode = match self.mode { Mode::Dark => "Dark", Mode::Light => "Light" },
            bg = bg.to_hex(),
            fg = fg.to_hex(),
            accent = accent.to_hex(),
            border = border.to_hex(),
            hover = hover.to_hex(),
            hdr_bg = match self.mode {
                Mode::Dark => bg.lighten(0.05).to_hex(),
                Mode::Light => bg.darken(0.05).to_hex(),
            },
            radius = self.border_radius,
            font = self.font_family,
            size = self.font_size,
        )
    }

    /// Generate a Waybar CSS colour snippet for Omarchy bar integration.
    pub fn to_waybar_colors(&self) -> String {
        let accent = self.variant.accent();
        let bg = self.bg();
        let fg = self.fg();
        format!(
            "@define-color bar-bg {};\n@define-color bar-fg {};\n@define-color bar-accent {};\n",
            bg.to_hex(), fg.to_hex(), accent.to_hex()
        )
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colour_from_hex() {
        let c = Colour::from_hex("#1f9ece").unwrap();
        assert_eq!(c, Colour::new(0x1f, 0x9e, 0xce));
    }

    #[test]
    fn test_colour_to_hex_roundtrip() {
        let c = Colour::new(0xab, 0xcd, 0xef);
        assert_eq!(c.to_hex(), "#abcdef");
        assert_eq!(Colour::from_hex(&c.to_hex()).unwrap(), c);
    }

    #[test]
    fn test_invalid_hex_rejected() {
        assert!(Colour::from_hex("zzzzzz").is_err());
        assert!(Colour::from_hex("#abc").is_err());
    }

    #[test]
    fn test_lighten_darken() {
        let c = Colour::new(100, 100, 100);
        let lighter = c.lighten(0.5);
        assert!(lighter.r > c.r);
        let darker = c.darken(0.5);
        assert!(darker.r < c.r);
    }

    #[test]
    fn test_theme_config_creation() {
        let t = ThemeConfig::new("My Theme", MintYVariant::Aqua, Mode::Dark).unwrap();
        assert_eq!(t.variant, MintYVariant::Aqua);
        assert_eq!(t.mode, Mode::Dark);
    }

    #[test]
    fn test_empty_name_rejected() {
        assert_eq!(
            ThemeConfig::new("", MintYVariant::Green, Mode::Light).unwrap_err(),
            ThemeError::EmptyName
        );
    }

    #[test]
    fn test_invalid_opacity() {
        let t = ThemeConfig::new("T", MintYVariant::Blue, Mode::Dark).unwrap();
        assert!(t.with_opacity(1.5).is_err());
    }

    #[test]
    fn test_gtk3_css_generation() {
        let t = ThemeConfig::new("Mint-Y-Aqua-Dark", MintYVariant::Aqua, Mode::Dark).unwrap();
        let css = t.to_gtk3_css();
        assert!(css.contains("@define-color theme_bg_color"));
        assert!(css.contains("Mint-Y aqua (Dark)"));
        assert!(css.contains("#1f9ece")); // aqua accent
    }

    #[test]
    fn test_variant_accent_colours() {
        assert_eq!(MintYVariant::Green.accent(), Colour::new(0x92, 0xb3, 0x72));
        assert_eq!(MintYVariant::Red.accent(), Colour::new(0xc1, 0x48, 0x48));
    }

    #[test]
    fn test_icon_theme_name() {
        assert_eq!(MintYVariant::Pink.icon_theme(), "Mint-Y-Pink");
    }

    #[test]
    fn test_waybar_colors() {
        let t = ThemeConfig::new("T", MintYVariant::Teal, Mode::Dark).unwrap();
        let css = t.to_waybar_colors();
        assert!(css.contains("bar-accent"));
        assert!(css.contains("#2b9e93"));
    }

    #[test]
    fn test_light_mode_colours() {
        let t = ThemeConfig::new("T", MintYVariant::Blue, Mode::Light).unwrap();
        let bg = t.bg();
        // Light mode should have bright background
        assert!(bg.r > 200);
    }
}
