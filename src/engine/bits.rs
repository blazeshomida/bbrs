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
