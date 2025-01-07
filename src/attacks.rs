use std::{array, time::Instant, vec};

use crate::magic::MagicPRNG;

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

#[rustfmt::skip]
const BISHOP_RELEVANT_BITS: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 
    5, 5, 5, 5, 5, 5, 5, 5, 
    5, 5, 7, 7, 7, 7, 5, 5, 
    5, 5, 7, 9, 9, 7, 5, 5, 
    5, 5, 7, 9, 9, 7, 5, 5, 
    5, 5, 7, 7, 7, 7, 5, 5, 
    5, 5, 5, 5, 5, 5, 5, 5, 
    6, 5, 5, 5, 5, 5, 5, 6,
];

#[rustfmt::skip]
const ROOK_RELEVANT_BITS: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 
    12, 11, 11, 11, 11, 11, 11, 12,
];

const BISHOP_MAGICS: [u64; 64] = [
    0x40040844404084,
    0x2004208A004208,
    0x10190041080202,
    0x108060845042010,
    0x581104180800210,
    0x2112080446200010,
    0x1080820820060210,
    0x3C0808410220200,
    0x4050404440404,
    0x21001420088,
    0x24D0080801082102,
    0x1020A0A020400,
    0x40308200402,
    0x4011002100800,
    0x401484104104005,
    0x801010402020200,
    0x400210C3880100,
    0x404022024108200,
    0x810018200204102,
    0x4002801A02003,
    0x85040820080400,
    0x810102C808880400,
    0xE900410884800,
    0x8002020480840102,
    0x220200865090201,
    0x2010100A02021202,
    0x152048408022401,
    0x20080002081110,
    0x4001001021004000,
    0x800040400A011002,
    0xE4004081011002,
    0x1C004001012080,
    0x8004200962A00220,
    0x8422100208500202,
    0x2000402200300C08,
    0x8646020080080080,
    0x80020A0200100808,
    0x2010004880111000,
    0x623000A080011400,
    0x42008C0340209202,
    0x209188240001000,
    0x400408A884001800,
    0x110400A6080400,
    0x1840060A44020800,
    0x90080104000041,
    0x201011000808101,
    0x1A2208080504F080,
    0x8012020600211212,
    0x500861011240000,
    0x180806108200800,
    0x4000020E01040044,
    0x300000261044000A,
    0x802241102020002,
    0x20906061210001,
    0x5A84841004010310,
    0x4010801011C04,
    0xA010109502200,
    0x4A02012000,
    0x500201010098B028,
    0x8040002811040900,
    0x28000010020204,
    0x6000020202D0240,
    0x8918844842082200,
    0x4010011029020020,
];

const ROOK_MAGICS: [u64; 64] = [
    0x8A80104000800020,
    0x140002000100040,
    0x2801880A0017001,
    0x100081001000420,
    0x200020010080420,
    0x3001C0002010008,
    0x8480008002000100,
    0x2080088004402900,
    0x800098204000,
    0x2024401000200040,
    0x100802000801000,
    0x120800800801000,
    0x208808088000400,
    0x2802200800400,
    0x2200800100020080,
    0x801000060821100,
    0x80044006422000,
    0x100808020004000,
    0x12108A0010204200,
    0x140848010000802,
    0x481828014002800,
    0x8094004002004100,
    0x4010040010010802,
    0x20008806104,
    0x100400080208000,
    0x2040002120081000,
    0x21200680100081,
    0x20100080080080,
    0x2000A00200410,
    0x20080800400,
    0x80088400100102,
    0x80004600042881,
    0x4040008040800020,
    0x440003000200801,
    0x4200011004500,
    0x188020010100100,
    0x14800401802800,
    0x2080040080800200,
    0x124080204001001,
    0x200046502000484,
    0x480400080088020,
    0x1000422010034000,
    0x30200100110040,
    0x100021010009,
    0x2002080100110004,
    0x202008004008002,
    0x20020004010100,
    0x2048440040820001,
    0x101002200408200,
    0x40802000401080,
    0x4008142004410100,
    0x2060820C0120200,
    0x1001004080100,
    0x20C020080040080,
    0x2935610830022400,
    0x44440041009200,
    0x280001040802101,
    0x2100190040002085,
    0x80C0084100102001,
    0x4024081001000421,
    0x20030A0244872,
    0x12001008414402,
    0x2006104900A0804,
    0x1004081002402,
];

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

/// Generates slider attacks using the Hyperbola Quintessence formula:
/// (o - 2s) ^ reverse_bits( reverse_bits(o) - 2 * reverse_bits(s) ).
fn generate_slider_attacks(square: u8, slider_mask: u64, occupancy: u64) -> u64 {
    let s = bitboard!(square);

    let mut forward = occupancy & slider_mask;
    let mut reverse = forward.reverse_bits();

    forward = forward.wrapping_sub(s << 1);
    reverse = reverse.wrapping_sub(s.reverse_bits() << 1);

    forward ^= reverse.reverse_bits();
    forward & slider_mask
}

fn mask_slider_attacks(square: u8, slider_mask: u64) -> u64 {
    generate_slider_attacks(square, slider_mask, 0)
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
    generate_slider_attacks(
        square,
        DIAGONAL_MASKS[(7 - rank + file) as usize],
        occupancy,
    ) | generate_slider_attacks(
        square,
        ANTI_DIAGONAL_MASKS[(rank + file) as usize],
        occupancy,
    )
}

/// Generates rook attacks by combining rank and file lines.
fn generate_rook_attacks(square: u8, occupancy: u64) -> u64 {
    // Use the same line-attack helper for rank and file
    generate_slider_attacks(square, RANK_MASKS[(square >> 3) as usize], occupancy)
        | generate_slider_attacks(square, FILE_MASKS[(square & 7) as usize], occupancy)
}

fn create_occupancy(index: usize, mask: u64, bits: u8) -> u64 {
    let mut copy = mask;
    (0..bits).fold(0, |mut occupancy, count| {
        let square = get_lsb!(copy);
        clear_lsb!(copy);
        if index & 1 << count != 0 {
            set_bit!(occupancy, square);
        }
        occupancy
    })
}

fn init_slider_attacks(masks: [u64; 64], is_bishop: bool) -> [Box<[u64]>; 64] {
    array::from_fn(|square| {
        let mask = masks[square];
        let (magic, bits) = if is_bishop {
            (BISHOP_MAGICS[square], BISHOP_RELEVANT_BITS[square])
        } else {
            (ROOK_MAGICS[square], ROOK_RELEVANT_BITS[square])
        };
        let variations = 1 << bits;
        let mut attacks = vec![0; variations];
        (0..variations).for_each(|index| {
            let occupancy = create_occupancy(index, mask, bits);
            let magic_index = ((occupancy.wrapping_mul(magic)) >> (64 - bits)) as usize;
            attacks[magic_index] = if is_bishop {
                generate_bishop_attacks(square as u8, occupancy)
            } else {
                generate_rook_attacks(square as u8, occupancy)
            };
        });
        attacks.into()
    })
}

#[allow(dead_code)]
fn find_magic_number(rng: &mut MagicPRNG, square: u8, is_bishop: bool) -> Result<u64, &str> {
    let (mask, bits) = if is_bishop {
        (
            mask_bishop_attacks(square),
            BISHOP_RELEVANT_BITS[square as usize],
        )
    } else {
        (
            mask_rook_attacks(square),
            ROOK_RELEVANT_BITS[square as usize],
        )
    };
    let variations = 1 << bits;
    let mut occupancies = vec![0; variations];
    let mut attacks = vec![0; variations];
    (0..variations).for_each(|index| {
        occupancies[index] = create_occupancy(index, mask, bits);
        attacks[index] = if is_bishop {
            generate_bishop_attacks(square, occupancies[index])
        } else {
            generate_rook_attacks(square, occupancies[index])
        };
    });
    for _ in 0..1_000_000_000 {
        let magic = rng.rand_magic();

        if count_bits!((mask.wrapping_mul(magic)) & 0xFF00000000000000) < 6 {
            continue;
        };

        let mut used = vec![0; variations];

        let mut fail = false;
        for index in 0..variations {
            let magic_index = ((occupancies[index].wrapping_mul(magic)) >> (64 - bits)) as usize;
            if used[magic_index] == 0 {
                used[magic_index] = attacks[index];
            }
            if used[magic_index] != attacks[index] {
                fail = true;
                break;
            }
        }
        if !fail {
            println!("{:#X},", magic);
            return Ok(magic);
        }
    }

    Err("failed to find magic number")
}

#[allow(dead_code)]
fn find_magic_numbers() {
    let mut rng = MagicPRNG::new();
    let now = Instant::now();
    println!("Rook magics:");
    (0..64).for_each(|square| {
        find_magic_number(&mut rng, square, false).unwrap();
    });
    println!();
    println!("Bishop magics:");
    (0..64).for_each(|square| {
        find_magic_number(&mut rng, square, true).unwrap();
    });
    println!("Total time: {:?}", now.elapsed());
}

pub struct AttackTable {
    pawns: [[u64; 64]; 2],
    knights: [u64; 64],
    kings: [u64; 64],
    bishops: [Box<[u64]>; 64],
    rooks: [Box<[u64]>; 64],

    bishop_masks: [u64; 64],
    rook_masks: [u64; 64],
}

impl AttackTable {
    pub fn init() -> Self {
        let mut pawns = [[0; 64]; 2];
        let mut knights = [0; 64];
        let mut kings = [0; 64];
        let mut bishop_masks = [0; 64];
        let mut rook_masks = [0; 64];

        // Initialize attack masks
        (0..64).for_each(|square| {
            pawns[0][square] = mask_pawn_attacks(square as u8, 0);
            pawns[1][square] = mask_pawn_attacks(square as u8, 1);
            knights[square] = mask_knight_attacks(square as u8);
            kings[square] = mask_king_attacks(square as u8);
            bishop_masks[square] = mask_bishop_attacks(square as u8);
            rook_masks[square] = mask_rook_attacks(square as u8);
        });

        // Initialize bishop and rook attack tables
        let bishops: [Box<[u64]>; 64] = init_slider_attacks(bishop_masks, true);
        let rooks: [Box<[u64]>; 64] = init_slider_attacks(rook_masks, false);

        AttackTable {
            pawns,
            knights,
            kings,
            bishops,
            rooks,
            bishop_masks,
            rook_masks,
        }
    }

    fn get_slider_attacks(&self, square: usize, occupancy: u64, is_bishop: bool) -> u64 {
        let (mask, magic, bits) = if is_bishop {
            (
                self.bishop_masks[square],
                BISHOP_MAGICS[square],
                BISHOP_RELEVANT_BITS[square],
            )
        } else {
            (
                self.rook_masks[square],
                ROOK_MAGICS[square],
                ROOK_RELEVANT_BITS[square],
            )
        };
        let magic_index = ((occupancy & mask).wrapping_mul(magic) >> (64 - bits)) as usize;
        if is_bishop {
            self.bishops[square][magic_index]
        } else {
            self.rooks[square][magic_index]
        }
    }

    pub fn get_pawn_attacks(&self, side: u8, square: usize) -> u64 {
        self.pawns[side as usize][square]
    }
    pub fn get_knight_attacks(&self, square: usize) -> u64 {
        self.knights[square]
    }
    pub fn get_king_attacks(&self, square: usize) -> u64 {
        self.kings[square]
    }
    pub fn get_bishop_attacks(&self, square: usize, occupancy: u64) -> u64 {
        self.get_slider_attacks(square, occupancy, true)
    }
    pub fn get_rook_attacks(&self, square: usize, occupancy: u64) -> u64 {
        self.get_slider_attacks(square, occupancy, false)
    }
    pub fn get_queen_attacks(&self, square: usize, occupancy: u64) -> u64 {
        self.get_bishop_attacks(square, occupancy) | self.get_rook_attacks(square, occupancy)
    }
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
