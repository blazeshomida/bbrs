use std::ops::Range;

use crate::{
    attacks::AttackTable,
    utils::{algebraic_to_index, index_to_algebraic},
};

const WHITE: u8 = 0;
const BLACK: u8 = 1;

const PAWN: u8 = 0;
const KNIGHT: u8 = 1;
const BISHOP: u8 = 2;
const ROOK: u8 = 3;
const QUEEN: u8 = 4;
const KING: u8 = 5;

const WHITE_PAWN: u8 = 0;
const WHITE_KNIGHT: u8 = 1;
const WHITE_BISHOP: u8 = 2;
const WHITE_ROOK: u8 = 3;
const WHITE_QUEEN: u8 = 4;
const WHITE_KING: u8 = 5;
const BLACK_PAWN: u8 = 6;
const BLACK_KNIGHT: u8 = 7;
const BLACK_BISHOP: u8 = 8;
const BLACK_ROOK: u8 = 9;
const BLACK_QUEEN: u8 = 10;
const BLACK_KING: u8 = 11;

const ASCII_PIECES: [char; 12] = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];
const WHITE_RANGE: Range<usize> = 0..6;
const BLACK_RANGE: Range<usize> = 6..12;
const ALL_RANGE: Range<usize> = 0..12;

fn fen_to_piece(fen: char) -> Option<u8> {
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
fn castling_rights_to_mask(rights: &str) -> Result<u8, &str> {
    let mut mask = 0;
    for ch in rights.chars() {
        match ch {
            'K' => mask |= 1,
            'Q' => mask |= 2,
            'k' => mask |= 4,
            'q' => mask |= 8,
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

fn parse_fen(fen: &str) -> Result<EngineState, &str> {
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
                if let Some(piece) = fen_to_piece(ch) {
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
    let castling = castling_rights_to_mask(castling)?;

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
        let state = parse_fen(fen)?;
        Ok(Engine {
            attack_table: AttackTable::init(),
            state,
        })
    }

    pub fn set_position<'a>(&mut self, fen: &'a str) -> Result<(), &'a str> {
        self.state = parse_fen(fen)?;
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
        println!(
            "Side: {}",
            match side {
                0 => "white",
                1 => "black",
                _ => unreachable!(),
            }
        );
        println!(
            "Castling: {}",
            match castling {
                0 => "-".to_string(),
                _ => {
                    format!(
                        "{}{}{}{}",
                        if castling & 1 != 0 { "K" } else { "" },
                        if castling & 2 != 0 { "Q" } else { "" },
                        if castling & 4 != 0 { "k" } else { "" },
                        if castling & 8 != 0 { "q" } else { "" }
                    )
                }
            }
        );
        println!(
            "Enpassant: {}",
            en_passant.map_or_else(|| "-".to_string(), |sq| { index_to_algebraic(sq as usize) })
        );
        println!("Halfmove: {}", half_moves);
        println!("Fullmove: {}", full_moves);
    }
}
