#[derive(Copy, Clone, Default, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn new(r : u8, g : u8, b : u8) -> Color {
        Color(
            if r > 31 { 31 } else { r },
            if g > 63 { 63 } else { g },
            if b > 31 { 31 } else { b },
        )
    }
}

#[allow(dead_code)]
pub mod colors {
    use super::Color;
    pub const RED          : Color = Color(31, 0, 0);
    pub const LIGHT_RED    : Color = Color(31, 0, 0);
    pub const GREEN        : Color = Color(0, 31, 0);
    pub const LIGHT_GREEN  : Color = Color(0, 8, 0);
    pub const BLUE         : Color = Color(0, 0, 31);
    pub const LIGHT_BLUE   : Color = Color(0, 0, 8);
    pub const YELLOW       : Color = Color(31, 63, 0);
    pub const CYAN         : Color = Color(0, 63, 24);
    pub const BLACK        : Color = Color(0, 0, 0);
    pub const PURPLE       : Color = Color(31, 0, 31);
    pub const WHITE        : Color = Color(31, 63, 31);
}