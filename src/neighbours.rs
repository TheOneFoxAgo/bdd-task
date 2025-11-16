//! Фукнции поика соседей.
//! Моя сетка соседских отношений имеет вид:
//! _ _ l
//! _ 0 _
//! _ r _
//!
//! Клетки пронумерованы следующим образом (N = 9):
//! 0 1 2
//! 3 4 5
//! 6 7 8
use super::N;
const WIDTH: usize = 3;
const HEIGHT: usize = 3;
// Если N больше количества клеток в квадрате
// То программа явно написана неправильно.
const _: () = assert!(N == WIDTH * HEIGHT);
const ORIGIN: Position = Position { x: 1, y: 1 };
const LEFT: Position = Position { x: 2, y: 0 };
const RIGHT: Position = Position { x: 1, y: 2 };
#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}
impl Direction {
    fn get_position(self) -> Position {
        match self {
            Direction::Left => LEFT,
            Direction::Right => RIGHT,
        }
    }
}
/// Отдаёт, может ли в этом положении быть сосед
pub fn validate(i: usize, direction: Direction, wrapping: bool) -> bool {
    let neighbour_position = direction.get_position();
    let shift = Position::decompose(i).calculate_shift(neighbour_position);
    ORIGIN.shift_by(shift, wrapping).is_some()
}
/// Отдаёт номер, где должен располагаться сосед, если такое возможно.
pub fn neighbour(i: usize, direction: Direction, wrapping: bool) -> Option<usize> {
    let neighbour_position = direction.get_position();
    let shift = Position::decompose(i).calculate_shift(ORIGIN);
    neighbour_position
        .shift_by(shift, wrapping)
        .map(Position::compose)
}
#[derive(Clone, Copy)]
struct Position {
    x: isize,
    y: isize,
}
impl Position {
    fn decompose(i: usize) -> Self {
        let x = i.rem_euclid(WIDTH) as isize;
        let y = i.div_euclid(HEIGHT) as isize;
        assert!(y < HEIGHT as isize);
        Self { x, y }
    }
    fn compose(self) -> usize {
        (self.x + self.y * WIDTH as isize) as usize
    }
    fn calculate_shift(self, origin: Self) -> Self {
        Self {
            x: self.x - origin.x,
            y: self.y - origin.y,
        }
    }
    fn shift_by(mut self, shift: Self, wrapping: bool) -> Option<Self> {
        self.y += shift.y;
        if !(0..HEIGHT as isize).contains(&self.y) {
            return None;
        }
        self.x += shift.x;
        if wrapping {
            self.x = self.x.rem_euclid(WIDTH as isize);
            Some(self)
        } else if (0..WIDTH as isize).contains(&self.x) {
            Some(self)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn left_shift_some() {
        let t = tester_some(Direction::Left, false);
        t(3, 1);
        t(4, 2);
        t(6, 4);
        t(7, 5);
    }
    #[test]
    fn right_shift_some() {
        let t = tester_some(Direction::Right, false);
        t(3, 6);
        t(4, 7);
        t(5, 8);
        t(0, 3);
        t(1, 4);
        t(2, 5);
    }
    #[test]
    fn left_shift_some_wrapping() {
        let t = tester_some(Direction::Left, true);
        t(3, 1);
        t(4, 2);
        t(6, 4);
        t(7, 5);

        t(5, 0);
        t(8, 3);
    }
    #[test]
    fn left_shift_none() {
        let t = tester_none(Direction::Left, false);
        t(2);
        t(1);
        t(0);
        t(5);
        t(8);
    }
    #[test]
    fn right_shift_none() {
        let t = tester_none(Direction::Right, false);
        t(6);
        t(7);
        t(8);
    }
    #[test]
    fn left_shift_none_wrapping() {
        let t = tester_none(Direction::Left, true);
        t(2);
        t(1);
        t(0);
    }
    fn tester_some(direction: Direction, wrapping: bool) -> impl Fn(usize, usize) {
        move |input, output| {
            assert!(neighbour(input, direction, wrapping).is_some_and(|s| s == output));
            assert!(validate(output, direction, wrapping));
        }
    }
    fn tester_none(direction: Direction, wrapping: bool) -> impl Fn(usize) {
        move |input| {
            assert!(neighbour(input, direction, wrapping).is_none());
        }
    }
}
