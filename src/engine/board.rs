#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Square {
    a8, b8, c8, d8, e8, f8, g8, h8, 
    a7, b7, c7, d7, e7, f7, g7, h7, 
    a6, b6, c6, d6, e6, f6, g6, h6, 
    a5, b5, c5, d5, e5, f5, g5, h5, 
    a4, b4, c4, d4, e4, f4, g4, h4, 
    a3, b3, c3, d3, e3, f3, g3, h3, 
    a2, b2, c2, d2, e2, f2, g2, h2, 
    a1, b1, c1, d1, e1, f1, g1, h1,
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
