use crate::engine::{moves, ASCII_PIECES};

/// Print the bitboard for debugging.
pub fn print_bitboard(bitboard: u64) {
    let divider = "-------------------";
    println!("{}", divider);
    for rank in 0..8 {
        print!("{} ", 8 - rank); // Print ranks in descending order (8 to 1)
        for file in 0..8 {
            let square = rank * 8 + file;
            let bit = if get_bit!(bitboard, square) {
                "1"
            } else {
                "•"
            };
            print!("{} ", bit);
        }
        println!();
    }
    println!("  a b c d e f g h"); // Print files
    println!("{}", divider);
    println!("Bitboard: {}", bitboard);
    println!("Hex: {:#X}", bitboard);
    println!("Binary: {:#b}", bitboard);
    println!("{}", divider);
}

pub fn print_move_list(moves: &[u32]) {
    let print_divider = || {
        println!("{}", "─".repeat(65));
    };
    let print_headers = || {
        println!(
            "{:>5} │ {:<6} │ {:^7} │ {:^7} │ {:^7} │ {:^7} │ {:^7}",
            "No.", "Move", "Piece", "Capt.", "Doub.", "En Pas.", "Castle"
        );
    };
    print_divider();
    println!("  Move list:");
    print_divider();
    print_headers();
    print_divider();

    moves.iter().enumerate().for_each(|(index, &move_)| {
        let (_, _, piece, _, (capture, double, en_passant, castle)) = decode_move!(move_);
        print!("{:>5} │ ", format!("{:>3}", index + 1));

        print!(
            "{:<6} │ {:^7} │ {:^7} │ {:^7} │ {:^7} │ {:^7}",
            moves::format(move_),
            ASCII_PIECES[piece as usize],
            if capture { "■■■" } else { "‧‧‧" },
            if double { "■■■" } else { "‧‧‧" },
            if en_passant { "■■■" } else { "‧‧‧" },
            if castle { "■■■" } else { "‧‧‧" }
        );
        println!();
    });
    print_divider();
    print_headers();
    print_divider();
    println!("  Total moves: {}", moves.len());
    print_divider();
}
