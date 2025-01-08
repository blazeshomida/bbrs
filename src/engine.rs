use std::ops::Range;

use crate::{attacks::AttackTable, utils::index_to_algebraic};
use piece::pieces::*;

mod side {
    use std::ops::Range;

    use super::piece;

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
            WHITE => piece::range::WHITE,
            BLACK => piece::range::BLACK,
            _ => unreachable!(),
        }
    }
}

mod piece {
    pub mod types {
        pub const PAWN: u8 = 0;
        pub const KNIGHT: u8 = 1;
        pub const BISHOP: u8 = 2;
        pub const ROOK: u8 = 3;
        pub const QUEEN: u8 = 4;
        pub const KING: u8 = 5;
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
}

mod castling {
    pub const WK: u8 = 1 << 0;
    pub const WQ: u8 = 1 << 1;
    pub const BK: u8 = 1 << 2;
    pub const BQ: u8 = 1 << 3;

    pub fn format(castling: u8) -> String {
        match castling {
            0 => "-".to_string(),
            _ => {
                let mut result = String::new();
                if castling & WK != 0 {
                    result.push('K');
                }
                if castling & WQ != 0 {
                    result.push('Q');
                }
                if castling & BK != 0 {
                    result.push('k');
                }
                if castling & BQ != 0 {
                    result.push('q');
                }
                result
            }
        }
    }
}

mod fen {
    use super::{castling::*, piece::pieces::*, side::*, EngineState};
    use crate::utils::algebraic_to_index;
    fn parse_piece(fen: char) -> Option<u8> {
        match fen {
            'P' => Some(WHITE_PAWN),
            'N' => Some(WHITE_KNIGHT),
            'B' => Some(WHITE_BISHOP),
            'R' => Some(WHITE_ROOK),
            'Q' => Some(WHITE_QUEEN),
            'K' => Some(WHITE_KING),
            'p' => Some(BLACK_PAWN),
            'n' => Some(BLACK_KNIGHT),
            'b' => Some(BLACK_BISHOP),
            'r' => Some(BLACK_ROOK),
            'q' => Some(BLACK_QUEEN),
            'k' => Some(BLACK_KING),
            _ => None,
        }
    }

    /// Convert castling rights from a FEN string to a bitmask.
    fn parse_castle_rights(rights: &str) -> Result<u8, &str> {
        let mut mask = 0;
        for ch in rights.chars() {
            match ch {
                'K' => mask |= WK,
                'Q' => mask |= WQ,
                'k' => mask |= BK,
                'q' => mask |= BQ,
                '-' => (),
                _ => return Err("Invalid FEN: Unexpected character in castling rights"),
            }
        }
        Ok(mask)
    }

    /// Parse the en passant square from a FEN string.
    fn parse_en_passant(square: &str) -> Result<Option<u8>, &str> {
        if square == "-" {
            return Ok(None);
        }
        if square.len() != 2 {
            return Err("Invalid FEN: En passant square must be in algebraic notation");
        }
        Ok(Some(algebraic_to_index(square)))
    }

    pub fn parse(fen: &str) -> Result<EngineState, &str> {
        let sections: Vec<&str> = fen.split_whitespace().collect();

        if sections.len() != 6 {
            return Err("Invalid FEN: Incorrect number of sections");
        }

        let (piece_placement, side, castling, en_passant, half_moves, full_moves) = (
            sections[0],
            sections[1],
            sections[2],
            sections[3],
            sections[4]
                .parse::<u8>()
                .map_err(|_| "Invalid halfmove clock")?,
            sections[5]
                .parse::<u8>()
                .map_err(|_| "Invalid fullmove number")?,
        );

        // Reset the board state
        let mut bitboards = [0u64; 12];

        // Parse piece placement
        let mut index = 0;
        for ch in piece_placement.chars() {
            match ch {
                '/' => continue,
                ch if ch.is_ascii_digit() => {
                    index += ch.to_digit(10).unwrap() as u64;
                    continue;
                }
                _ => {
                    if let Some(piece) = parse_piece(ch) {
                        set_bit!(bitboards[piece as usize], index);
                        index += 1;
                    } else {
                        return Err("Invalid FEN: Unexpected character");
                    }
                }
            };
        }

        // Parse active color
        let side = match side {
            "w" => WHITE,
            "b" => BLACK,
            _ => return Err("Invalid FEN: Active color must be 'w' or 'b'"),
        };

        // Parse castling rights
        let castling = parse_castle_rights(castling)?;

        // Parse en passant square
        let en_passant = parse_en_passant(en_passant)?;

        Ok(EngineState {
            bitboards,
            side,
            castling,
            en_passant,
            half_moves,
            full_moves,
        })
    }
}

struct EngineState {
    bitboards: [u64; 12],
    side: u8,
    castling: u8,
    half_moves: u8,
    full_moves: u8,
    en_passant: Option<u8>,
}

pub struct Engine {
    attack_table: AttackTable,
    state: EngineState,
}

impl Engine {
    pub fn new(fen: &str) -> Result<Self, &str> {
        let state = fen::parse(fen)?;
        Ok(Engine {
            attack_table: AttackTable::init(),
            state,
        })
    }

    pub fn set_position<'a>(&mut self, fen: &'a str) -> Result<(), &'a str> {
        self.state = fen::parse(fen)?;
        Ok(())
    }

    fn get_occupancy(&self, range: Range<usize>) -> u64 {
        self.state.bitboards[range]
            .iter()
            .fold(0, |mut acc, bitboard| {
                acc |= bitboard;
                acc
            })
    }

    pub fn is_square_attacked(&self, square: usize, side: u8) -> bool {
        let EngineState { bitboards, .. } = self.state;
        let enemy = side ^ 1;

        // Select the appropriate piece types for the enemy
        let (pawn, knight, bishop, rook, queen, king) = if enemy == side::WHITE {
            (
                WHITE_PAWN,
                WHITE_KNIGHT,
                WHITE_BISHOP,
                WHITE_ROOK,
                WHITE_QUEEN,
                WHITE_KING,
            )
        } else {
            (
                BLACK_PAWN,
                BLACK_KNIGHT,
                BLACK_BISHOP,
                BLACK_ROOK,
                BLACK_QUEEN,
                BLACK_KING,
            )
        };

        // Check non-sliding pieces (pawn, knight, king)
        if self.attack_table.get_pawn_attacks(side, square) & bitboards[pawn as usize] != 0
            || self.attack_table.get_knight_attacks(square) & bitboards[knight as usize] != 0
            || self.attack_table.get_king_attacks(square) & bitboards[king as usize] != 0
        {
            return true;
        }

        // Occupancy is only needed for sliding pieces
        let occupancy = self.get_occupancy(piece::range::ALL);

        // Check sliding pieces (bishop, rook, queen)
        if self.attack_table.get_bishop_attacks(square, occupancy) & bitboards[bishop as usize] != 0
            || self.attack_table.get_rook_attacks(square, occupancy) & bitboards[rook as usize] != 0
            || self.attack_table.get_queen_attacks(square, occupancy) & bitboards[queen as usize]
                != 0
        {
            return true;
        }

        false
    }

    pub fn print_attacked_squares(&self, side: u8) {
        for rank in 0..8 {
            print!("{} ", 8 - rank);
            for file in 0..8 {
                let square = rank * 8 + file;
                if self.is_square_attacked(square, side) {
                    print!("X ");
                } else {
                    print!("• ");
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
    }

    pub fn print(&self) {
        let EngineState {
            bitboards,
            side,
            castling,
            en_passant,
            half_moves,
            full_moves,
        } = self.state;
        for rank in 0..8 {
            print!("{} ", 8 - rank);
            for file in 0..8 {
                let square = rank * 8 + file;
                let mut found = false;
                bitboards.iter().enumerate().for_each(|(index, &bitboard)| {
                    if get_bit!(bitboard, square) {
                        print!("{} ", ASCII_PIECES[index]);
                        found = true
                    }
                });
                if !found {
                    print!("• ");
                }
            }
            println!();
        }
        println!("  a b c d e f g h");

        println!();
        println!("Side: {}", side::format(side));
        println!("Castling: {}", castling::format(castling));
        println!(
            "Enpassant: {}",
            en_passant.map_or_else(|| "-".to_string(), |sq| { index_to_algebraic(sq as usize) })
        );
        println!("Halfmove: {}", half_moves);
        println!("Fullmove: {}", full_moves);
    }
}
