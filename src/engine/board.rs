#[allow(dead_code)]
#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8, 
    A7, B7, C7, D7, E7, F7, G7, H7, 
    A6, B6, C6, D6, E6, F6, G6, H6, 
    A5, B5, C5, D5, E5, F5, G5, H5, 
    A4, B4, C4, D4, E4, F4, G4, H4, 
    A3, B3, C3, D3, E3, F3, G3, H3, 
    A2, B2, C2, D2, E2, F2, G2, H2, 
    A1, B1, C1, D1, E1, F1, G1, H1,
}

/// Convert an algebraic square (e.g., "a8") to a bitboard index (0-63).
pub fn algebraic_to_index(square: &str) -> u8 {
    let mut chars = square.chars();
    let file = chars.next().unwrap() as u8 - b'a';
    let rank = 8 - chars.next().unwrap().to_digit(10).unwrap() as u8;
    rank * 8 + file
}

/// Convert a bitboard index (0-63) to an algebraic square (e.g., 0 to "a8").
pub fn index_to_algebraic(index: usize) -> String {
    let file = (index % 8) as u8 + b'a';
    let rank = 8 - (index / 8);
    format!("{}{}", file as char, rank)
}
