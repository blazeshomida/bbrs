use crate::engine::{board::index_to_algebraic, ASCII_PIECES};

/// Encodes a chess move into a 32-bit integer.
/// - `encode_move!(source, target, piece, promotion, flags)`
/// - `encode_move!(source, target, piece)` (defaults promotion and flags to 0)
/// - `encode_move!(source, target, piece, flags)` (defaults promotion to 0)
#[macro_export]
macro_rules! encode_move {
    ($source:expr, $target:expr, $piece:expr, $promotion:expr, $flags:expr) => {
        ($source | ($target << 6) | ($piece << 12) | ($promotion << 16) | ($flags << 20)) as u32
    };
    ($source:expr, $target:expr, $piece:expr) => {
        encode_move!($source, $target, $piece, 0, 0)
    };
    ($source:expr, $target:expr, $piece:expr, $flags:expr) => {
        encode_move!($source, $target, $piece, 0, $flags)
    };
}

/// Decodes a 32-bit chess move into a tuple (source, target, piece, promotion, (capture, double pawn push, en passant, castle)).
#[macro_export]
macro_rules! decode_move {
    ($move:expr) => {
        (
            ($move & 0x3F) as u8,        // source square (0-63)
            (($move >> 6) & 0x3F) as u8, // target square (0-63)
            (($move >> 12) & 0xF) as u8, // piece moved (0-15)
            (($move >> 16) & 0xF) as u8, // promotion piece (0-15)
            (
                ($move & (1 << 20)) != 0, // capture
                ($move & (1 << 21)) != 0, // double pawn push
                ($move & (1 << 22)) != 0, // en passant
                ($move & (1 << 23)) != 0, // castle
            ),
        )
    };
}

pub mod flags {
    pub const CAPTURE: u8 = 1 << 0;
    pub const DOUBLE: u8 = 1 << 1;
    pub const EN_PASSANT: u8 = 1 << 2;
    pub const CASTLE: u8 = 1 << 3;
}

pub fn format(move_: u32) -> String {
    let (source, target, _, promotion, _) = decode_move!(move_);
    let suffix = if promotion != 0 {
        format!("{}", ASCII_PIECES[promotion as usize])
    } else {
        String::new()
    };

    format!(
        "{}{}{}",
        index_to_algebraic(source as usize),
        index_to_algebraic(target as usize),
        suffix
    )
}
