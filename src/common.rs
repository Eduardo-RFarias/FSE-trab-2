#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Elevator {
    One = 0x00,
    Two = 0x01,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Floor {
    Ground,
    First,
    Second,
    Third,
}
