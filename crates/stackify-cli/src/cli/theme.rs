use lazy_static::lazy_static;
use owo_colors::{style, OwoColorize, Rgb, Style, Styled};

lazy_static! {
    static ref THEME: Theme = Theme::default();
}

pub fn theme() -> &'static Theme {
    &*THEME
}

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
}

impl<T: OwoColorize> ThemedObject for T {
    fn red(&self) -> Styled<&Self> {
        self.style(THEME.red)
    }

    fn green(&self) -> Styled<&Self> {
        self.style(THEME.green)
    }

    fn yellow(&self) -> Styled<&Self> {
        self.style(THEME.yellow)
    }

    fn blue(&self) -> Styled<&Self> {
        self.style(THEME.blue)
    }

    fn magenta(&self) -> Styled<&Self> {
        self.style(THEME.magenta)
    }

    fn cyan(&self) -> Styled<&Self> {
        self.style(THEME.cyan)
    }

    fn gray(&self) -> Styled<&Self> {
        self.style(THEME.gray)
    }

    fn white(&self) -> Styled<&Self> {
        self.style(THEME.white)
    }

    fn text(&self) -> Styled<&Self> {
        self.style(THEME.text)
    }

    fn info(&self) -> Styled<&Self> {
        self.style(THEME.info)
    }

    fn warning(&self) -> Styled<&Self> {
        self.style(THEME.warning)
    }

    fn error(&self) -> Styled<&Self> {
        self.style(THEME.error)
    }

    fn success(&self) -> Styled<&Self> {
        self.style(THEME.success)
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
    pub white: Rgb
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

pub struct Theme {
    pub palette: ColorPalette,
    pub red: Style,
    pub green: Style,
    pub yellow: Style,
    pub blue: Style,
    pub magenta: Style,
    pub cyan: Style,
    pub gray: Style,
    pub white: Style,
    pub text: Style,
    pub info: Style,
    pub warning: Style,
    pub error: Style,
    pub success: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::apple()
    }
}

impl Theme {
    pub fn palette(&self) -> &ColorPalette {
        &self.palette
    }
    pub fn apple() -> Self {
        let palette = ColorPalette::default();
        Self {
            red: style().color(palette.red),
            green: style().color(palette.green),
            yellow: style().color(palette.yellow),
            blue: style().color(palette.blue),
            magenta: style().color(palette.magenta),
            cyan: style().color(palette.cyan),
            gray: style().color(palette.gray),
            white: style().color(palette.white),
            text: style().remove_all_effects(),
            info: style().fg_rgb::<8, 105, 203>(),
            warning: style().fg_rgb::<205, 172, 8>(),
            error: style().fg_rgb::<204, 55, 46>(),
            success: style().fg_rgb::<38, 164, 57>(),
            palette,
        }
    }
}