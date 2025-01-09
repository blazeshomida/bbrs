pub mod side {
    use super::range;
    use std::ops::Range;

    pub const WHITE: u8 = 0;
    pub const BLACK: u8 = 1;

    pub fn format<'a>(side: u8) -> &'a str {
        match side {
            WHITE => "white",
            BLACK => "black",
            _ => unreachable!(),
        }
    }

    pub fn range(side: u8) -> Range<usize> {
        match side {
            WHITE => range::WHITE,
            BLACK => range::BLACK,
            _ => unreachable!(),
        }
    }
}

pub mod types {
    pub const PAWN: u8 = 0;
    pub const KNIGHT: u8 = 1;
    pub const BISHOP: u8 = 2;
    pub const ROOK: u8 = 3;
    pub const QUEEN: u8 = 4;
    pub const KING: u8 = 5;

    pub const PROMOTION_PIECES: [u8; 4] = [QUEEN, ROOK, BISHOP, KNIGHT];
}

pub mod pieces {
    pub const WHITE_PAWN: u8 = 0;
    pub const WHITE_KNIGHT: u8 = 1;
    pub const WHITE_BISHOP: u8 = 2;
    pub const WHITE_ROOK: u8 = 3;
    pub const WHITE_QUEEN: u8 = 4;
    pub const WHITE_KING: u8 = 5;
    pub const BLACK_PAWN: u8 = 6;
    pub const BLACK_KNIGHT: u8 = 7;
    pub const BLACK_BISHOP: u8 = 8;
    pub const BLACK_ROOK: u8 = 9;
    pub const BLACK_QUEEN: u8 = 10;
    pub const BLACK_KING: u8 = 11;
    pub const ASCII_PIECES: [char; 12] =
        ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];
}

pub mod range {
    use std::ops::Range;
    pub const WHITE: Range<usize> = 0..6;
    pub const BLACK: Range<usize> = 6..12;
    pub const ALL: Range<usize> = 0..12;
}
