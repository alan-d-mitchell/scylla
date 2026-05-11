#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(pub u8);

impl Square {

    pub const fn new(sq: u8) -> Self {
        Self(sq)
    }
}
