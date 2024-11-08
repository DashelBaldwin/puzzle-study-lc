// termcolors.rs

#[allow(dead_code)]
#[repr(usize)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan
}

pub fn color(text: &str, color: Color) -> String {
    let code: usize = color as usize + 31;
    format!("\x1b[{}m{}\x1b[0m", code, text)
}
