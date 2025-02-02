// Type Convert

use ratatui::style::{
    Style as RStyle,
    Color as RColor,
    Modifier
};

use syntect::highlighting::{
    Style as HStyle,
    Color as HColor,
    FontStyle
};

pub trait StyleConvert {
    fn to_rstyle(self) -> RStyle;
}

pub trait ColorConvert {
    fn to_rcolor(self) -> RColor;
}

impl StyleConvert for HStyle {
    fn to_rstyle(self) -> RStyle {
        let mut temp = RStyle::default()
            .fg(self.foreground.to_rcolor());

        if self.font_style.contains(FontStyle::BOLD) {
            temp = temp.add_modifier(Modifier::BOLD);
        }

        if self.font_style.contains(FontStyle::ITALIC) {
            temp = temp.add_modifier(Modifier::ITALIC);
        }

        if self.font_style.contains(FontStyle::UNDERLINE) {
            temp = temp.add_modifier(Modifier::UNDERLINED);
        }
        
        temp
    }
}

impl ColorConvert for HColor {
    fn to_rcolor(self) -> RColor {
        RColor::Rgb(self.r, self.g, self.b)
    }
}
