/// FILE_MASKS represents the 8 files (columns) on an 8x8 chessboard.
///
/// Direction: ↑↓ (a-file to h-file, up and down)
///
/// Files: `a` to `h`
const FILE_MASKS: [u64; 8] = [
    0x101010101010101,  // a-file
    0x202020202020202,  // b-file
    0x404040404040404,  // c-file
    0x808080808080808,  // d-file
    0x1010101010101010, // e-file
    0x2020202020202020, // f-file
    0x4040404040404040, // g-file
    0x8080808080808080, // h-file
];

/// RANK_MASKS represents the 8 ranks (rows) on an 8x8 chessboard.
///
/// Direction: ←→ (rank 8 to rank 1, left to right)
///
/// Ranks: `8` to `1`
const RANK_MASKS: [u64; 8] = [
    0xFF,               // rank 8
    0xFF00,             // rank 7
    0xFF0000,           // rank 6
    0xFF000000,         // rank 5
    0xFF00000000,       // rank 4
    0xFF0000000000,     // rank 3
    0xFF000000000000,   // rank 2
    0xFF00000000000000, // rank 1
];

/// DIAGONAL_MASKS represents the 15 diagonals on an 8x8 chessboard.
///
/// Direction: ↖↘ (a8 to h1, top-left to bottom-right)
///
/// Formula: `diag_index = 7 - rank + file`
const DIAGONAL_MASKS: [u64; 15] = [
    0x100000000000000,
    0x201000000000000,
    0x402010000000000,
    0x804020100000000,
    0x1008040201000000,
    0x2010080402010000,
    0x4020100804020100,
    0x8040201008040201,
    0x80402010080402,
    0x804020100804,
    0x8040201008,
    0x80402010,
    0x804020,
    0x8040,
    0x80,
];

/// ANTI_DIAGONAL_MASKS represents the 15 anti-diagonals on an 8x8 chessboard.
///
/// Direction: ↗↙ (a1 to h8, top-right to bottom-left)
///
/// Formula: `anti_diag_index = file + rank`
const ANTI_DIAGONAL_MASKS: [u64; 15] = [
    0x1,
    0x102,
    0x10204,
    0x1020408,
    0x102040810,
    0x10204081020,
    0x1020408102040,
    0x102040810204080,
    0x204081020408000,
    0x408102040800000,
    0x810204080000000,
    0x1020408000000000,
    0x2040800000000000,
    0x4080000000000000,
    0x8000000000000000,
];

const FILE_A: u64 = FILE_MASKS[0];
const FILE_B: u64 = FILE_MASKS[1];
const FILE_G: u64 = FILE_MASKS[6];
const FILE_H: u64 = FILE_MASKS[7];
const FILE_AB: u64 = FILE_A | FILE_B;
const FILE_GH: u64 = FILE_G | FILE_H;
const RANK_1: u64 = RANK_MASKS[0];
const RANK_8: u64 = RANK_MASKS[7];

const VBORDER_MASK: u64 = FILE_A | FILE_H;
const HBORDER_MASK: u64 = RANK_1 | RANK_8;
const BORDER_MASK: u64 = VBORDER_MASK | HBORDER_MASK;

#[rustfmt::skip]
const PAWN_OFFSETS: [[(i8, u64); 2]; 2] = [
    [(-7, !FILE_A), (-9, !FILE_H)],
    [(9, !FILE_A), (7, !FILE_H)],
];

#[rustfmt::skip]
const KNIGHT_OFFSETS: [(i8, u64); 8] = [
    ( 6, !FILE_GH), ( 10, !FILE_AB), ( 15, !FILE_H), ( 17, !FILE_A),
    (-6, !FILE_AB), (-10, !FILE_GH), (-15, !FILE_A), (-17, !FILE_H)
];

#[rustfmt::skip]
const KING_OFFSETS: [(i8, u64); 8] = [
    ( 1, !FILE_A), ( 7, !FILE_H), ( 8, !0), ( 9, !FILE_A),
    (-1, !FILE_H), (-7, !FILE_A), (-8, !0), (-9, !FILE_H),
];

const BISHOP_OFFSETS: [i8; 4] = [-9, -7, 7, 9];
const ROOK_OFFSETS: [i8; 4] = [-1, -8, 1, 8];

fn mask_leaper_attacks(square: u8, offsets: &[(i8, u64)]) -> u64 {
    let bitboard = bitboard!(square);
    offsets.iter().fold(0, |mut attacks, &(offset, mask)| {
        let shifted = if offset > 0 {
            bitboard << offset
        } else {
            bitboard >> -offset
        };
        if shifted & mask != 0 {
            attacks |= shifted;
        }
        attacks
    })
}

fn mask_pawn_attacks(square: u8, side: u8) -> u64 {
    mask_leaper_attacks(square, &PAWN_OFFSETS[side as usize])
}

fn mask_knight_attacks(square: u8) -> u64 {
    mask_leaper_attacks(square, &KNIGHT_OFFSETS)
}

fn mask_king_attacks(square: u8) -> u64 {
    mask_leaper_attacks(square, &KING_OFFSETS)
}

/// Generates line attacks using the Hyperbola Quintessence formula:
/// (o - 2s) ^ reverse_bits( reverse_bits(o) - 2 * reverse_bits(s) ).
fn generate_line_attacks(square: u8, line_mask: u64, occupancy: u64) -> u64 {
    let s = bitboard!(square);

    let mut forward = occupancy & line_mask;
    let mut reverse = forward.reverse_bits();

    forward = forward.wrapping_sub(s << 1);
    reverse = reverse.wrapping_sub(s.reverse_bits() << 1);

    forward ^= reverse.reverse_bits();
    forward & line_mask
}

fn mask_slider_attacks(square: u8, line_mask: u64) -> u64 {
    generate_line_attacks(square, line_mask, 0)
}

fn mask_bishop_attacks(square: u8) -> u64 {
    let (rank, file) = (square >> 3, square & 7);

    mask_slider_attacks(
        square,
        DIAGONAL_MASKS[(7 - rank + file) as usize] & !BORDER_MASK,
    ) | mask_slider_attacks(
        square,
        ANTI_DIAGONAL_MASKS[(rank + file) as usize] & !BORDER_MASK,
    )
}

fn mask_rook_attacks(square: u8) -> u64 {
    // Use the same line-attack helper for rank and file
    mask_slider_attacks(square, RANK_MASKS[(square >> 3) as usize] & !VBORDER_MASK)
        | mask_slider_attacks(square, FILE_MASKS[(square & 7) as usize] & !HBORDER_MASK)
}

/// Generates bishop attacks by combining diagonal and anti-diagonal lines.
fn generate_bishop_attacks(square: u8, occupancy: u64) -> u64 {
    let (rank, file) = (square >> 3, square & 7);

    // Just call the line-attack helper for each relevant mask
    generate_line_attacks(
        square,
        DIAGONAL_MASKS[(7 - rank + file) as usize],
        occupancy,
    ) | generate_line_attacks(
        square,
        ANTI_DIAGONAL_MASKS[(rank + file) as usize],
        occupancy,
    )
}

/// Generates rook attacks by combining rank and file lines.
fn generate_rook_attacks(square: u8, occupancy: u64) -> u64 {
    // Use the same line-attack helper for rank and file
    generate_line_attacks(square, RANK_MASKS[(square >> 3) as usize], occupancy)
        | generate_line_attacks(square, FILE_MASKS[(square & 7) as usize], occupancy)
}

pub struct AttackTable {
    pawns: [[u64; 64]; 2],
    knights: [u64; 64],
    kings: [u64; 64],
    bishops: [Box<[u64]>; 64],
    rooks: [Box<[u64]>; 64],
}

#[cfg(test)]
mod tests {
    use crate::consts::Square;

    use super::*;

    #[test]
    fn test_file_masks() {
        assert_eq!(FILE_A, 0x101010101010101);
        assert_eq!(FILE_H, 0x8080808080808080);
    }

    #[test]
    fn test_rank_masks() {
        assert_eq!(RANK_1, 0xFF);
        assert_eq!(RANK_8, 0xFF00000000000000);
    }

    #[test]
    fn test_mask_pawn_attacks() {
        // White pawn on e5 (square 28)
        assert_eq!(mask_pawn_attacks(Square::e5 as u8, 0), 0x280000);

        // Black pawn on e4 (square 36)
        assert_eq!(mask_pawn_attacks(Square::e4 as u8, 1), 0x280000000000);
    }

    #[test]
    fn test_mask_knight_attacks() {
        // Knight on b8 (square 1)
        assert_eq!(mask_knight_attacks(Square::b8 as u8), 0x50800);
    }

    #[test]
    fn test_mask_king_attacks() {
        // King on e5 (square 28)
        assert_eq!(mask_king_attacks(Square::e5 as u8), 0x3828380000);
    }

    #[test]
    fn test_mask_bishop_attacks() {
        // Bishop on d5 (square 27)
        assert_eq!(mask_bishop_attacks(Square::d5 as u8), 0x40221400142200);
    }

    #[test]
    fn test_mask_rook_attacks() {
        // Rook on d5 (square 27)
        assert_eq!(mask_rook_attacks(Square::d5 as u8), 0x8080876080800);
    }

    #[test]
    fn test_generate_bishop_attacks() {
        // Bishop on a8 (square 0)
        assert_eq!(
            generate_bishop_attacks(Square::a8 as u8, 0),
            0x8040201008040200
        );

        let occupancy = bitboard!(Square::e4 as u8);
        // Bishop on a8 (square 0) with occupancy of e4 (square 36)
        assert_eq!(
            generate_bishop_attacks(Square::a8 as u8, occupancy),
            0x1008040200
        );
    }

    #[test]
    fn test_generate_rook_attacks() {
        // Rook on a8 (square 0)
        assert_eq!(
            generate_rook_attacks(Square::a8 as u8, 0),
            0x1010101010101FE
        );

        let occupancy = bitboard!(Square::a3 as u8);
        // Rook on a8 (square 0) with occupancy of a3 (square 40)
        assert_eq!(
            generate_rook_attacks(Square::a8 as u8, occupancy),
            0x101010101FE
        );
    }
}
