use std::{
    io::{self, Read},
    thread,
    time::Duration,
};

use crate::engine::piece::pieces::ASCII_PIECES;

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

pub fn format_move(move_: u32) -> String {
    let (source, target, _, promotion, _) = decode_move!(move_);
    let suffix = if promotion != 0 {
        format!("={}", ASCII_PIECES[promotion as usize])
    } else {
        String::new()
    };

    format!(
        "{}{}{}",
        index_to_algebraic(source as usize),
        index_to_algebraic(target as usize),
        suffix
    )
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
            format_move(move_),
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

/// Pauses execution until any key is pressed.
pub fn pause() {
    println!("Press any key to continue...");

    // Create a buffer to hold one byte
    let mut buffer = [0; 1];

    // Read one byte from standard input to pause execution
    io::stdin().read_exact(&mut buffer).unwrap();
}

/// Sleeps for a specified number of milliseconds.
pub fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
