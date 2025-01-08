#[macro_export]
macro_rules! bitboard {
    ($square:expr) => {
        1u64 << $square
    };
}

#[macro_export]
macro_rules! get_bit {
    ($bitboard:expr, $square:expr) => {
        ($bitboard >> $square) & 1 != 0
    };
}

#[macro_export]
macro_rules! set_bit {
    ($bitboard:expr, $square:expr) => {
        $bitboard |= 1 << $square
    };
}

#[macro_export]
macro_rules! clear_bit {
    ($bitboard:expr, $square:expr) => {
        $bitboard &= !(1 << $square)
    };
}

#[macro_export]
macro_rules! count_bits {
    ($bitboard:expr) => {
        $bitboard.count_ones()
    };
}

#[macro_export]
macro_rules! get_lsb {
    ($bitboard:expr) => {
        $bitboard.trailing_zeros()
    };
}

#[macro_export]
macro_rules! clear_lsb {
    ($bitboard:expr) => {
        $bitboard &= ($bitboard - 1)
    };
}

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
