use std::{
    fmt::Display,
    sync::{Mutex, RwLock},
};

use once_cell::sync::Lazy;
use owo_colors::{style, OwoColorize, Rgb, Style, Styled};

pub static THEME: Lazy<RwLock<Box<dyn Theme>>> = Lazy::new(|| RwLock::new(Box::new(DefaultTheme)));

pub trait ThemedObject {
    fn red(&self) -> Styled<&Self>;
    fn green(&self) -> Styled<&Self>;
    fn yellow(&self) -> Styled<&Self>;
    fn blue(&self) -> Styled<&Self>;
    fn magenta(&self) -> Styled<&Self>;
    fn cyan(&self) -> Styled<&Self>;
    fn gray(&self) -> Styled<&Self>;
    fn white(&self) -> Styled<&Self>;
    fn text(&self) -> Styled<&Self>;
    fn info(&self) -> Styled<&Self>;
    fn warning(&self) -> Styled<&Self>;
    fn error(&self) -> Styled<&Self>;
    fn success(&self) -> Styled<&Self>;
    fn table_header(&self) -> Styled<&Self>;

    fn bold(&self) -> Styled<&Self>
    where
        Self: Sized,
    {
        OwoColorize::style(self, style().bold())
    }

    fn italic(&self) -> Styled<&Self>
    where
        Self: Sized,
    {
        OwoColorize::style(self, style().italic())
    }

    fn underline(&self) -> Styled<&Self>
    where
        Self: Sized,
    {
        OwoColorize::style(self, style().underline())
    }

    fn dimmed(&self) -> Styled<&Self>
    where
        Self: Sized,
    {
        OwoColorize::style(self, style().dimmed())
    }
}

impl<T: OwoColorize + Display> ThemedObject for T {
    fn red(&self) -> Styled<&Self> {
        self.style(Theme::red(&**THEME.read().unwrap()))
    }

    fn green(&self) -> Styled<&Self> {
        self.style(Theme::green(&**THEME.read().unwrap()))
    }

    fn yellow(&self) -> Styled<&Self> {
        self.style(Theme::yellow(&**THEME.read().unwrap()))
    }

    fn blue(&self) -> Styled<&Self> {
        self.style(Theme::blue(&**THEME.read().unwrap()))
    }

    fn magenta(&self) -> Styled<&Self> {
        self.style(Theme::magenta(&**THEME.read().unwrap()))
    }

    fn cyan(&self) -> Styled<&Self> {
        self.style(Theme::cyan(&**THEME.read().unwrap()))
    }

    fn gray(&self) -> Styled<&Self> {
        self.style(Theme::gray(&**THEME.read().unwrap()))
    }

    fn white(&self) -> Styled<&Self> {
        self.style(Theme::white(&**THEME.read().unwrap()))
    }

    fn text(&self) -> Styled<&Self> {
        self.style(Theme::text(&**THEME.read().unwrap()))
    }

    fn info(&self) -> Styled<&Self> {
        self.style(Theme::info(&**THEME.read().unwrap()))
    }

    fn warning(&self) -> Styled<&Self> {
        self.style(Theme::warning(&**THEME.read().unwrap()))
    }

    fn error(&self) -> Styled<&Self> {
        self.style(Theme::error(&**THEME.read().unwrap()))
    }

    fn success(&self) -> Styled<&Self> {
        self.style(Theme::success(&**THEME.read().unwrap()))
    }

    fn table_header(&self) -> Styled<&Self> {
        self.style(Theme::table_header(&**THEME.read().unwrap()))
    }
}

pub struct ColorPalette {
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
    pub gray: Rgb,
    pub white: Rgb,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            red: Rgb(204, 55, 46),
            green: Rgb(38, 164, 57),
            yellow: Rgb(205, 172, 8),
            blue: Rgb(8, 105, 203),
            magenta: Rgb(150, 71, 191),
            cyan: Rgb(71, 158, 194),
            gray: Rgb(152, 152, 157),
            white: Rgb(255, 255, 255),
        }
    }
}

pub trait Theme: Sync + Send {
    // Symbols
    fn success_symbol(&self) -> String {
        "✔"
            .to_owned()
            .style(THEME.read().unwrap().success().bold())
            .to_string()
    }

    fn error_symbol(&self) -> String {
        "✕"
            .to_owned()
            .style(THEME.read().unwrap().error().bold())
            .to_string()
    }

    fn warning_symbol(&self) -> String {
        "⚠"
            .to_owned()
            .style(THEME.read().unwrap().warning().bold())
            .to_string()
    }

    fn info_symbol(&self) -> String {
        "ℹ"
            .to_owned()
            .style(THEME.read().unwrap().info().bold())
            .to_string()
    }

    fn skipped_symbol(&self) -> String {
        "⊖"
            .to_owned()
            .style(THEME.read().unwrap().gray().bold())
            .to_string()
    }

    // Styling
    fn palette(&self) -> ColorPalette {
        ColorPalette::default()
    }
    fn red(&self) -> Style {
        style().color(self.palette().red)
    }
    fn green(&self) -> Style {
        style().color(self.palette().green)
    }

    fn yellow(&self) -> Style {
        style().color(self.palette().yellow)
    }
    fn blue(&self) -> Style {
        style().color(self.palette().blue)
    }
    fn magenta(&self) -> Style {
        style().color(self.palette().magenta)
    }
    fn cyan(&self) -> Style {
        style().color(self.palette().cyan)
    }
    fn gray(&self) -> Style {
        style().color(self.palette().gray)
    }
    fn white(&self) -> Style {
        style().color(self.palette().white)
    }
    fn text(&self) -> Style {
        style().remove_all_effects()
    }

    // Log-level styles
    fn info(&self) -> Style {
        style().fg_rgb::<8, 105, 203>()
    }
    fn warning(&self) -> Style {
        style().fg_rgb::<205, 172, 8>()
    }
    fn error(&self) -> Style {
        style().fg_rgb::<204, 55, 46>()
    }
    fn success(&self) -> Style {
        style().fg_rgb::<38, 164, 57>()
    }

    // Table styles
    fn table_header(&self) -> Style {
        style().color(self.palette().cyan).bold()
    }
}

pub struct DefaultTheme;

impl Theme for DefaultTheme {}
