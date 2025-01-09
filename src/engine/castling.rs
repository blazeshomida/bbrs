pub mod flags {
    pub const WK: u8 = 1 << 0;
    pub const WQ: u8 = 1 << 1;
    pub const BK: u8 = 1 << 2;
    pub const BQ: u8 = 1 << 3;
}

#[rustfmt::skip]
pub const CASLTING_RIGHTS: [u8; 64] = [
     7, 15, 15, 15,  3, 15, 15, 11, 
    15, 15, 15, 15, 15, 15, 15, 15, 
    15, 15, 15, 15, 15, 15, 15, 15, 
    15, 15, 15, 15, 15, 15, 15, 15, 
    15, 15, 15, 15, 15, 15, 15, 15, 
    15, 15, 15, 15, 15, 15, 15, 15, 
    15, 15, 15, 15, 15, 15, 15, 15, 
    13, 15, 15, 15, 12, 15, 15, 14,
];

pub fn format(castling: u8) -> String {
    match castling {
        0 => "-".to_string(),
        _ => {
            let mut result = String::new();
            if castling & flags::WK != 0 {
                result.push('K');
            }
            if castling & flags::WQ != 0 {
                result.push('Q');
            }
            if castling & flags::BK != 0 {
                result.push('k');
            }
            if castling & flags::BQ != 0 {
                result.push('q');
            }
            result
        }
    }
}
