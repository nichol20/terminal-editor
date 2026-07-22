#[allow(dead_code)]
#[derive(Debug)]
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

#[derive(Debug)]
pub enum Action {
    Move(Direction),
    Resize,
}
