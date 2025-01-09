use bbrs::engine::Engine;
use std::io::{self, BufRead};
extern crate bbrs;
use std::process::{self, Command};

enum UCICommand<'a> {
    Uci,
    IsReady,
    Position {
        fen: Option<String>,
        moves: Vec<&'a str>,
    },
    Go {
        depth: Option<u32>,
    },
    Perft {
        depth: Option<u32>,
    },
    UciNewGame,
    Clear,
    Quit,
    Unknown(String),
}

const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -  0 1";

fn parse_position(input: &str) -> UCICommand {
    let mut tokens = input.split_whitespace().skip(1);
    let subcommand = tokens.next();
    let fen = match subcommand {
        Some("startpos") => Some(START_POSITION.to_string()),
        Some("kiwipete") => Some(KIWIPETE_POSITION.to_string()),
        Some("fen") => Some(tokens.by_ref().take(6).collect::<Vec<&str>>().join(" ")),
        _ => return UCICommand::Unknown(input.to_string()),
    };

    let moves = if tokens.next() == Some("moves") {
        tokens.collect()
    } else {
        vec![]
    };

    UCICommand::Position { fen, moves }
}

fn parse_go(input: &str) -> UCICommand {
    let mut tokens = input.split_whitespace().skip(1);
    let depth = tokens
        .next()
        .filter(|&s| s == "depth")
        .and_then(|_| tokens.next())
        .and_then(|d| d.parse::<u32>().ok());
    UCICommand::Go { depth }
}

fn parse_perft(input: &str) -> UCICommand {
    let mut tokens = input.split_whitespace().skip(1);
    let depth = tokens.next().and_then(|d| d.parse::<u32>().ok());
    UCICommand::Perft { depth }
}

fn parse_uci_command(input: &str) -> UCICommand {
    let command = input.split_whitespace().next().unwrap_or("");
    match command {
        "uci" => UCICommand::Uci,
        "isready" => UCICommand::IsReady,
        "position" => parse_position(input),
        "go" => parse_go(input),
        "perft" => parse_perft(input),
        "ucinewgame" => UCICommand::UciNewGame,
        "clear" => UCICommand::Clear,
        "quit" => UCICommand::Quit,
        _ => UCICommand::Unknown(input.to_string()),
    }
}

fn main() {
    let stdin = io::stdin();
    let handle = stdin.lock();
    let reader = io::BufReader::new(handle);
    let mut engine = Engine::new(START_POSITION).unwrap();

    for line in reader.lines().map_while(Result::ok) {
        match parse_uci_command(&line) {
            UCICommand::Uci => {
                println!("id name bbrs");
                println!("id author Blaze Shomida");
                println!("uciok");
            }
            UCICommand::IsReady => println!("readyok"),
            UCICommand::Position { fen, moves } => {
                engine
                    .set_position(fen.unwrap_or(START_POSITION.to_string()).as_str())
                    .unwrap();
                engine.load_moves(moves);
            }
            UCICommand::Go { depth } => {
                engine.search_position(depth.unwrap_or(6) as u8);
                println!()
            }
            UCICommand::Perft { depth } => {
                engine.perft(depth.unwrap_or(1) as u8);
            }
            UCICommand::UciNewGame => {
                engine.set_position(START_POSITION).unwrap();
            }
            UCICommand::Clear => {
                Command::new("clear").status().unwrap();
            }
            UCICommand::Quit => process::exit(0),
            UCICommand::Unknown(command) => println!("Unknown command: {}\n", command),
        };
    }
}
