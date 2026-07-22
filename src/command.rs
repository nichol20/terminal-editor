#[allow(dead_code)]
pub enum Direction {
    Position { x: usize, y: usize },
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Top,
    Bottom,
    PageUp,
    PageDown,
    LineEnd,
    LineStart,
    None,
}

pub enum Action {
    Move(Direction),
    Resize,
}
