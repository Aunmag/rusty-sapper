use termwiz::color::AnsiColor;
use termwiz::color::ColorAttribute;

pub struct Cell {
    pub mines_around: Option<u8>,
}

impl Cell {
    pub fn new() -> Self {
        return Cell { mines_around: None };
    }

    pub fn is_discovered(&self) -> bool {
        return self.mines_around.is_some();
    }

    pub fn get_mark(&self, is_marked: bool, is_mined: bool) -> CellMark {
        let symbol;
        let mut foreground = ColorAttribute::Default;
        let mut background = ColorAttribute::Default;

        if is_marked {
            symbol = '!';
            background = AnsiColor::Maroon.into();
        } else if is_mined {
            symbol = '#';
            background = AnsiColor::Maroon.into();
        } else {
            if let Some(mines_around) = self.mines_around {
                if mines_around == 0 {
                    symbol = ' ';
                } else {
                    symbol = std::char::from_digit(mines_around as u32, 10).unwrap_or('?');
                }

                foreground = match mines_around {
                    0 => foreground,
                    1 => AnsiColor::Blue.into(),
                    2 => AnsiColor::Green.into(),
                    3 => AnsiColor::Red.into(),
                    4 => AnsiColor::Navy.into(),
                    5 => AnsiColor::Maroon.into(),
                    6 => AnsiColor::Aqua.into(),
                    _ => AnsiColor::Purple.into(),
                };
            } else {
                symbol = '.';
            }
        }

        return CellMark {
            symbol,
            foreground,
            background,
        };
    }
}

pub struct CellMark {
    pub symbol: char,
    pub foreground: ColorAttribute,
    pub background: ColorAttribute,
}
