use std::{ops::Range, time::Instant};

use attacks::{masks, AttackTable};
use board::{algebraic_to_index, index_to_algebraic, Square};
use piece::{pieces::*, side};

#[macro_use]
mod bits;
#[macro_use]
pub mod moves;

mod attacks;
mod board;
mod castling;
mod debug;
mod evaluate;
mod fen;
mod magics;
mod piece;

#[derive(Debug)]
pub struct HistoryItem {
    move_: u32,
    captured: u8,
    side: u8,
    castling: u8,
    en_passant: Option<u8>,
}

#[derive(Debug)]
pub struct EngineState {
    bitboards: [u64; 12],
    side: u8,
    castling: u8,
    half_moves: u8,
    full_moves: u8,
    en_passant: Option<u8>,
}

pub struct Engine {
    attack_table: AttackTable,
    pub state: EngineState,
    pub history: Vec<HistoryItem>,
    search_ply: u8,
    search_nodes: u64,
    killer_moves: [[u32; 64]; 2],
    history_moves: [[u32; 64]; 12],
    pv_length: [u32; 64],
    pv_table: [[u32; 64]; 64],
}

impl Engine {
    pub fn new(fen: &str) -> Result<Self, &str> {
        let state = fen::parse(fen)?;
        Ok(Engine {
            attack_table: AttackTable::init(),
            state,
            history: vec![],
            search_ply: 0,
            search_nodes: 0,
            killer_moves: [[0; 64]; 2],
            history_moves: [[0; 64]; 12],
            pv_length: [0; 64],
            pv_table: [[0; 64]; 64],
        })
    }

    pub fn set_position<'a>(&mut self, fen: &'a str) -> Result<(), &'a str> {
        self.history.clear();
        self.state = fen::parse(fen)?;
        self.print();
        println!();
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

    pub fn generate_moves(&self) -> Vec<u32> {
        let mut moves: Vec<u32> = Vec::new();

        let EngineState {
            bitboards,
            side,
            en_passant,
            ..
        } = self.state;
        let all_pieces = self.get_occupancy(piece::range::ALL);
        let friendly_pieces = self.get_occupancy(side::range(side));
        let enemy_pieces = self.get_occupancy(side::range(side ^ 1));

        bitboards[side::range(side)]
            .iter()
            .enumerate()
            .for_each(|(piece_type, &bitboard)| {
                let mut bitboard = bitboard;
                let piece_type = piece_type as u8;
                let piece = (piece_type + side * 6) as usize;
                if piece_type == piece::types::PAWN {
                    let (start_rank, end_rank, promotion_rank, push) = if side == side::WHITE {
                        (masks::RANK_2, masks::RANK_8, masks::RANK_7, -8)
                    } else {
                        (masks::RANK_7, masks::RANK_1, masks::RANK_2, 8)
                    };
                    while bitboard != 0 {
                        let source = get_lsb!(bitboard) as usize;
                        let source_bitboard = bitboard!(source);
                        if source_bitboard & end_rank != 0 {
                            break;
                        }
                        // Quiet moves
                        let target = source.wrapping_add_signed(push);
                        if !get_bit!(all_pieces, target) {
                            if source_bitboard & promotion_rank != 0 {
                                // Promotions
                                piece::types::PROMOTION_PIECES
                                    .iter()
                                    .for_each(|&promotion| {
                                        let promotion_piece = promotion + self.state.side * 6;
                                        moves.push(encode_move!(
                                            source,
                                            target,
                                            piece,
                                            promotion_piece as usize,
                                            0
                                        ));
                                    });
                            } else {
                                // Single push
                                moves.push(encode_move!(source, target, piece));
                            }

                            // Double push
                            if source_bitboard & start_rank != 0 {
                                let double = target.wrapping_add_signed(push);
                                if !get_bit!(all_pieces, double) {
                                    moves.push(encode_move!(
                                        source,
                                        double,
                                        piece,
                                        moves::flags::DOUBLE as usize
                                    ));
                                }
                            }
                        }

                        // Attacks
                        let mut attacks = self.attack_table.get_pawn_attacks(side, source);

                        while attacks != 0 {
                            let target = get_lsb!(attacks) as usize;
                            let target_bitboard = bitboard!(target);

                            // Captures
                            if target_bitboard & enemy_pieces != 0 {
                                if source_bitboard & promotion_rank != 0 {
                                    // Promotions
                                    piece::types::PROMOTION_PIECES
                                        .iter()
                                        .for_each(|&promotion| {
                                            let promotion_piece = promotion + self.state.side * 6;
                                            moves.push(encode_move!(
                                                source,
                                                target,
                                                piece,
                                                promotion_piece as usize,
                                                moves::flags::CAPTURE as usize
                                            ));
                                        });
                                } else {
                                    moves.push(encode_move!(
                                        source,
                                        target,
                                        piece,
                                        moves::flags::CAPTURE as usize
                                    ));
                                }
                            }

                            // En passant
                            if let Some(en_passant) = en_passant {
                                if target_bitboard & bitboard!(en_passant) != 0 {
                                    moves.push(encode_move!(
                                        source,
                                        target,
                                        piece,
                                        (moves::flags::CAPTURE | moves::flags::EN_PASSANT) as usize
                                    ));
                                }
                            }
                            clear_lsb!(attacks);
                        }

                        clear_lsb!(bitboard);
                    }
                    return;
                }
                if piece_type == piece::types::KING {
                    // Castling
                    let (
                        king_square,
                        king_target,
                        queen_target,
                        king_empty,
                        queen_empty,
                        king_mask,
                        queen_mask,
                    ) = if side == side::WHITE {
                        (
                            Square::E1,
                            Square::G1,
                            Square::C1,
                            [Square::F1, Square::G1],
                            [Square::D1, Square::C1, Square::B1],
                            castling::flags::WK,
                            castling::flags::WQ,
                        )
                    } else {
                        (
                            Square::E8,
                            Square::G8,
                            Square::C8,
                            [Square::F8, Square::G8],
                            [Square::D8, Square::C8, Square::B8],
                            castling::flags::BK,
                            castling::flags::BQ,
                        )
                    };
                    if self.can_castle(king_mask)
                        && king_empty
                            .iter()
                            .all(|&square| !get_bit!(all_pieces, square as u8))
                        && !self.is_square_attacked(king_square as usize, side)
                        && !self.is_square_attacked(king_empty[0] as usize, side)
                    {
                        moves.push(encode_move!(
                            king_square as usize,
                            king_target as usize,
                            piece,
                            moves::flags::CASTLE as usize
                        ));
                    }
                    if self.can_castle(queen_mask)
                        && queen_empty
                            .iter()
                            .all(|&square| !get_bit!(all_pieces, square as u8))
                        && !self.is_square_attacked(king_square as usize, side)
                        && !self.is_square_attacked(queen_empty[0] as usize, side)
                    {
                        moves.push(encode_move!(
                            king_square as usize,
                            queen_target as usize,
                            piece,
                            moves::flags::CASTLE as usize
                        ));
                    }
                }

                while bitboard != 0 {
                    let source = get_lsb!(bitboard) as usize;
                    let mut attacks = match piece_type {
                        piece::types::KNIGHT => self.attack_table.get_knight_attacks(source),
                        piece::types::KING => self.attack_table.get_king_attacks(source),
                        piece::types::BISHOP => {
                            self.attack_table.get_bishop_attacks(source, all_pieces)
                        }
                        piece::types::ROOK => {
                            self.attack_table.get_rook_attacks(source, all_pieces)
                        }
                        piece::types::QUEEN => {
                            self.attack_table.get_queen_attacks(source, all_pieces)
                        }
                        _ => unreachable!(),
                    } & !friendly_pieces;
                    while attacks != 0 {
                        let target = get_lsb!(attacks) as usize;
                        let target_bitboard = bitboard!(target);

                        // Captures
                        if target_bitboard & enemy_pieces != 0 {
                            moves.push(encode_move!(
                                source,
                                target,
                                piece,
                                moves::flags::CAPTURE as usize
                            ));
                        } else {
                            moves.push(encode_move!(source, target, piece));
                        }
                        clear_lsb!(attacks);
                    }
                    clear_lsb!(bitboard);
                }
            });

        moves
    }

    fn can_castle(&self, mask: u8) -> bool {
        let EngineState { castling, .. } = self.state;
        match castling {
            0 => false,
            _ => castling & mask != 0,
        }
    }

    fn get_piece(&self, side: u8, target: u8) -> Option<u8> {
        let board = self.state.bitboards[side::range(side)]
            .iter()
            .enumerate()
            .find(|(_, &bitboard)| get_bit!(bitboard, target));
        if let Some((index, _)) = board {
            let captured = index + (side as usize * 6);
            Some(captured as u8)
        } else {
            None
        }
    }

    pub fn make_move(&mut self, move_: u32) -> bool {
        let mut history_item = HistoryItem {
            move_,
            captured: 0,
            side: self.state.side,
            castling: self.state.castling,
            en_passant: self.state.en_passant,
        };
        let (source, target, piece, promotion, flags) = decode_move!(move_);
        clear_bit!(self.state.bitboards[piece as usize], source);
        set_bit!(self.state.bitboards[piece as usize], target);
        let (capture, double, en_passant, castle) = flags;
        if capture {
            if let Some(captured) = self.get_piece(self.state.side ^ 1, target) {
                history_item.captured = captured;
                clear_bit!(self.state.bitboards[captured as usize], target);
            };
        };

        self.history.push(history_item);

        if promotion != 0 {
            clear_bit!(self.state.bitboards[piece as usize], target);
            set_bit!(self.state.bitboards[promotion as usize], target);
        }
        let (enemy_pawn, pawn_offset) = if self.state.side == side::WHITE {
            (BLACK_PAWN, 8)
        } else {
            (WHITE_PAWN, -8)
        };

        if en_passant {
            clear_bit!(
                self.state.bitboards[enemy_pawn as usize],
                target as i8 + pawn_offset
            );
        }
        self.state.en_passant = if double {
            Some((target as i8 + pawn_offset) as u8)
        } else {
            None
        };

        if castle {
            let (rook, king_target, queen_target, (king_start, king_end), (queen_start, queen_end)) =
                if self.state.side == side::WHITE {
                    (
                        WHITE_ROOK as usize,
                        Square::G1,
                        Square::C1,
                        (Square::H1, Square::F1),
                        (Square::A1, Square::D1),
                    )
                } else {
                    (
                        BLACK_ROOK as usize,
                        Square::G8,
                        Square::C8,
                        (Square::H8, Square::F8),
                        (Square::A8, Square::D8),
                    )
                };
            if target == king_target as u8 {
                clear_bit!(self.state.bitboards[rook], king_start as u8);
                set_bit!(self.state.bitboards[rook], king_end as u8);
            }
            if target == queen_target as u8 {
                clear_bit!(self.state.bitboards[rook], queen_start as u8);
                set_bit!(self.state.bitboards[rook], queen_end as u8);
            }
        }

        self.state.castling &= castling::CASLTING_RIGHTS[source as usize];
        self.state.castling &= castling::CASLTING_RIGHTS[target as usize];
        let king_square = if self.state.side == side::WHITE {
            get_lsb!(self.state.bitboards[WHITE_KING as usize])
        } else {
            get_lsb!(self.state.bitboards[BLACK_KING as usize])
        };
        self.state.side ^= 1;
        self.state.half_moves += 1;
        self.state.full_moves = self.state.half_moves / 2 + 1;
        if self.is_square_attacked(king_square as usize, self.state.side ^ 1) {
            self.take_back();
            return false;
        }
        true
    }

    pub fn take_back(&mut self) {
        let HistoryItem {
            move_,
            captured,
            side,
            castling,
            en_passant,
        } = self
            .history
            .pop()
            .expect("Engine history is empty. This should never happen.");
        let (source, target, piece, promotion, flags) = decode_move!(move_);
        clear_bit!(self.state.bitboards[piece as usize], target);
        set_bit!(self.state.bitboards[piece as usize], source);

        if promotion != 0 {
            clear_bit!(self.state.bitboards[promotion as usize], target);
        }

        let (capture_flag, _, en_passant_flag, castle_flag) = flags;

        if en_passant_flag {
            let (pawn, restore_square) = if self.state.side == side::WHITE {
                (WHITE_PAWN, target - 8)
            } else {
                (BLACK_PAWN, target + 8)
            };
            set_bit!(self.state.bitboards[pawn as usize], restore_square);
        } else if capture_flag {
            set_bit!(self.state.bitboards[captured as usize], target);
        };

        if castle_flag {
            let (rook, king_target, queen_target, (king_start, king_end), (queen_start, queen_end)) =
                if side == side::WHITE {
                    (
                        WHITE_ROOK as usize,
                        Square::G1,
                        Square::C1,
                        (Square::H1, Square::F1),
                        (Square::A1, Square::D1),
                    )
                } else {
                    (
                        BLACK_ROOK as usize,
                        Square::G8,
                        Square::C8,
                        (Square::H8, Square::F8),
                        (Square::A8, Square::D8),
                    )
                };
            if target == king_target as u8 {
                clear_bit!(self.state.bitboards[rook], king_end as u8);
                set_bit!(self.state.bitboards[rook], king_start as u8);
            }

            if target == queen_target as u8 {
                clear_bit!(self.state.bitboards[rook], queen_end as u8);
                set_bit!(self.state.bitboards[rook], queen_start as u8);
            }
        }

        self.state.side = side;
        self.state.castling = castling;
        self.state.en_passant = en_passant;
        self.state.half_moves -= 1;
        self.state.full_moves = self.state.half_moves / 2 + 1
    }

    pub fn parse_move(&mut self, move_: &str) -> Option<u32> {
        let mut chars = move_.chars();
        let source = algebraic_to_index(chars.by_ref().take(2).collect::<String>().as_str());
        let target = algebraic_to_index(chars.by_ref().take(2).collect::<String>().as_str());
        let piece = if let Some(piece) = chars.next() {
            fen::parse_piece(piece)
        } else {
            None
        };
        let moves = self.generate_moves();
        for &move_ in moves.iter() {
            let (source_, target_, piece_, _, _) = decode_move!(move_);
            if source == source_ && target == target_ {
                if let Some(piece) = piece {
                    if piece == piece_ {
                        return Some(move_);
                    } else {
                        continue;
                    }
                }

                return Some(move_);
            }
        }
        None
    }

    pub fn load_moves(&mut self, moves: Vec<&str>) {
        self.history.clear();
        for move_ in moves {
            if let Some(move_) = self.parse_move(move_) {
                self.make_move(move_);
                self.print();
            } else {
                println!("Invalid move: {}", move_);
                return;
            }
            println!();
        }
    }

    fn get_positional_score(&self, piece: u8, square: u8) -> i8 {
        let piece_side = piece / 6;
        let piece_type = piece % 6;
        let index = if piece_side == side::WHITE {
            square
        } else {
            square ^ 0x38
        } as usize;
        let score = match piece_type {
            piece::types::PAWN => evaluate::PAWN_SCORE[index],
            piece::types::KNIGHT => evaluate::KNIGHT_SCORE[index],
            piece::types::BISHOP => evaluate::BISHOP_SCORE[index],
            piece::types::ROOK => evaluate::ROOK_SCORE[index],
            piece::types::KING => evaluate::KING_SCORE[index],
            _ => 0,
        };
        if piece_side == side::WHITE {
            score
        } else {
            -score
        }
    }

    pub fn get_mvv_lva(&self, attacker: u8, victim: u8) -> i32 {
        let attacker_value = 5 - (attacker as i32 % 6);
        let victim_value = 1 + (victim as i32 % 6);
        victim_value * 100 + attacker_value
    }

    pub fn score_move(&self, move_: u32) -> i32 {
        let (_, target, source_piece, _, (capture, _, _, _)) = decode_move!(move_);
        if capture {
            let target_piece = self.get_piece(self.state.side ^ 1, target).unwrap_or(0);
            return self.get_mvv_lva(source_piece, target_piece) + 10_000;
        }
        let ply_index = self.search_ply as usize;
        if self.killer_moves[0][ply_index] == move_ {
            return 9_000;
        }
        if self.killer_moves[1][ply_index] == move_ {
            return 8_000;
        }
        let history_move = self.history_moves[source_piece as usize][target as usize];
        history_move as i32
    }

    pub fn sort_moves(&self, moves: &[u32]) -> Vec<u32> {
        let mut moves = moves.to_vec(); // Convert slice to Vec for sorting
        moves.sort_by(|&a, &b| self.score_move(b).cmp(&self.score_move(a)));
        moves
    }

    fn generate_captures(&self) -> Vec<u32> {
        self.generate_moves()
            .into_iter()
            .filter(|&move_| {
                let (_, _, _, _, (capture, _, _, _)) = decode_move!(move_);
                capture
            })
            .collect()
    }

    pub fn evaluate(&mut self) -> i32 {
        let mut score = 0;
        self.state
            .bitboards
            .iter()
            .enumerate()
            .for_each(|(piece, &bitboard)| {
                let piece = piece as u8;
                let mut copy = bitboard;
                while copy != 0 {
                    let square = get_lsb!(copy);
                    score += evaluate::MATERIAL_SCORES[piece as usize];
                    score += self.get_positional_score(piece, square as u8) as i32;

                    clear_lsb!(copy);
                }
            });

        if self.state.side == side::WHITE {
            score
        } else {
            -score
        }
    }

    pub fn quiescence(&mut self, alpha: i32, beta: i32) -> i32 {
        self.search_nodes += 1;
        let mut alpha = alpha;
        let score = self.evaluate();
        if score >= beta {
            return beta; // Beta cutoff
        }

        if score > alpha {
            alpha = score;
        }

        for &move_ in self.sort_moves(&self.generate_captures()).iter() {
            if !self.make_move(move_) {
                continue;
            }

            self.search_ply += 1;

            let score = -self.quiescence(-beta, -alpha);
            self.take_back();
            self.search_ply -= 1;

            if score >= beta {
                return beta; // Beta cutoff
            }

            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    pub fn negamax(&mut self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        let mut depth = depth;
        let ply_index = self.search_ply as usize;
        self.pv_length[ply_index] = ply_index as u32;
        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        let king = if self.state.side == side::WHITE {
            WHITE_KING
        } else {
            BLACK_KING
        };
        let in_check = self.is_square_attacked(
            get_lsb!(self.state.bitboards[king as usize]) as usize,
            self.state.side,
        );
        if in_check {
            depth += 1;
        }

        self.search_nodes += 1;
        let mut legal_moves = 0;

        for &move_ in self.sort_moves(&self.generate_moves()).iter() {
            if !self.make_move(move_) {
                continue;
            }

            self.search_ply += 1;
            legal_moves += 1;

            let score = -self.negamax(depth - 1, -beta, -alpha);
            self.take_back();
            self.search_ply -= 1;
            let (_, target, source_piece, _, (capture, _, _, _)) = decode_move!(move_);

            if score >= beta {
                if !capture {
                    self.killer_moves[1][ply_index] = self.killer_moves[0][ply_index];
                    self.killer_moves[0][ply_index] = move_;
                }
                return beta; // Beta cutoff
            }

            if score > alpha {
                alpha = score;
                if !capture {
                    self.history_moves[source_piece as usize][target as usize] += depth as u32;
                }
                self.pv_table[ply_index][ply_index] = move_;
                for next_ply in (ply_index + 1)..self.pv_length[ply_index + 1] as usize {
                    self.pv_table[ply_index][next_ply] = self.pv_table[ply_index + 1][next_ply];
                }
                self.pv_length[ply_index] = self.pv_length[ply_index + 1];
            }
        }

        // Handle checkmate and stalemate
        if legal_moves == 0 {
            if in_check {
                return -evaluate::MATE_SCORE + self.search_ply as i32; // Checkmate
            } else {
                return 0; // Stalemate
            }
        }

        alpha
    }

    pub fn search_position(&mut self, depth: u8) {
        self.search_ply = 0;
        self.search_nodes = 0;
        let start = Instant::now();
        let score = self.negamax(depth, -evaluate::MAX_SCORE, evaluate::MAX_SCORE);
        let elapsed = start.elapsed();
        let pv_line = self.pv_table[0]
            .into_iter()
            .take(self.pv_length[0] as usize)
            .collect::<Vec<u32>>();
        println!(
            "info score cp {} depth {} time {:.0} nodes {} nps {:.3} pv {} ",
            score,
            depth,
            elapsed.as_millis(),
            self.search_nodes,
            self.search_nodes / elapsed.as_secs(),
            pv_line
                .iter()
                .map(|&move_| moves::format(move_))
                .collect::<Vec<String>>()
                .join(" "),
        );
        println!("bestmove {}", moves::format(pv_line[0]));
    }

    pub fn perft_driver(&mut self, depth: u8) -> u64 {
        let mut nodes = 0;
        if depth == 0 {
            return 1;
        }
        for &move_ in self.generate_moves().iter() {
            if self.make_move(move_) {
                nodes += self.perft_driver(depth - 1);
                self.take_back();
            }
        }
        nodes
    }

    pub fn perft(&mut self, depth: u8) {
        let mut nodes = 0;
        let now = Instant::now();

        let print_divider = || {
            println!("{}", "─".repeat(56));
        };

        let print_headers = || {
            println!(
                "{:>5} │ {:<6} │ {:<10} │ {:<12} │ {:<10}",
                "No.", "Move", "Nodes", "Time", "kNPS"
            );
        };

        print_divider();
        println!("Performance test:");
        print_divider();
        print_headers();
        print_divider();

        for (index, &move_) in self.generate_moves().iter().enumerate() {
            if self.make_move(move_) {
                let start = Instant::now();
                let depth_nodes = self.perft_driver(depth - 1);
                nodes += depth_nodes;
                self.take_back();

                let elapsed = start.elapsed();
                let seconds = elapsed.as_secs_f64();
                let knps = if seconds > 0.0 {
                    (depth_nodes as f64 / seconds) / 1000.0
                } else {
                    0.0
                };

                println!(
                    "{:>5} │ {:<6} │ {:<10} │ {:<12?} │ {:<10.2}",
                    index + 1,
                    moves::format(move_),
                    depth_nodes,
                    elapsed,
                    knps
                );
            }
        }

        print_divider();

        let total_elapsed = now.elapsed();
        let total_seconds = total_elapsed.as_secs_f64();
        let total_knps = if total_seconds > 0.0 {
            (nodes as f64 / total_seconds) / 1000.0
        } else {
            0.0
        };

        println!("Depth: {}", depth);
        println!("Nodes: {}", nodes);
        println!("Time: {:?}", total_elapsed);
        println!("kNPS: {:.2}", total_knps);
        print_divider();
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

    pub fn print_move_scores(&self, sort: bool) {
        let print_divider = || {
            println!("{}", "─".repeat(25));
        };
        let print_headers = || {
            println!("{:>5} │ {:<6} │ {:<7}", "No.", "Move", "Score");
        };
        print_divider();
        println!("  Move Scores:");
        print_divider();
        print_headers();
        print_divider();
        let moves = self.generate_moves();
        let moves = if sort { self.sort_moves(&moves) } else { moves };
        for (index, &move_) in moves.iter().enumerate() {
            let score = self.score_move(move_);
            println!(
                "{:>5} │ {:<6} │ {:<7}",
                index + 1,
                moves::format(move_),
                score
            );
        }
        print_divider();
        print_headers();
        print_divider();
        println!("  Total moves: {}", moves.len());
        print_divider();
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
