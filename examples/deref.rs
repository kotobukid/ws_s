use std::fmt::Display;
use std::ops::{Deref, DerefMut};

struct Position {
    x: i32,
    y: i32,
    z: i32,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Movable for Position {
    fn move_to(&mut self, position: Position) {
        self.x = position.x;
        self.y = position.y;
        self.z = position.z;
    }
}

struct Character {
    name: String,
    position: Position,
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.position)
    }
}

impl Movable for Character {
    fn move_to(&mut self, position: Position) {
        self.position.move_to(position);
    }
}

trait Movable {
    fn move_to(&mut self, position: Position);
}

impl Deref for Character {
    type Target = Position;

    fn deref(&self) -> &Position {
        &self.position
    }
}

impl DerefMut for Character {
    fn deref_mut(&mut self) -> &mut Position {
        &mut self.position
    }
}

fn walk(actor: &mut impl Movable, position: Position) {
    actor.move_to(position);
}

fn main() {
    let mut c = Character {
        name: String::from("Rust"),
        position: Position { x: 1, y: 2, z: 3 },
    };
    walk(
        &mut c,
        Position {
            x: 10,
            y: 20,
            z: 30,
        },
    );
    println!("{}", c);

    let mut p = Position { x: 1, y: 2, z: 3 };
    walk(
        &mut p,
        Position {
            x: 30,
            y: 20,
            z: 10,
        },
    );
    println!("{}", p);
}
