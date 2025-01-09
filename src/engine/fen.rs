use super::{
    board::algebraic_to_index,
    castling,
    piece::{pieces::*, side},
    EngineState,
};

pub fn parse_piece(fen: char) -> Option<u8> {
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
            'K' => mask |= castling::flags::WK,
            'Q' => mask |= castling::flags::WQ,
            'k' => mask |= castling::flags::BK,
            'q' => mask |= castling::flags::BQ,
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
        "w" => side::WHITE,
        "b" => side::BLACK,
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
