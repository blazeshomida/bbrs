use bbrs::engine::Engine;

#[allow(unused_variables)]
fn main() {
    let greek_gift = "rnbq1rk1/ppp1nppp/4p3/b2pP3/3P4/2PB1N2/PP3PPP/RNBQK2R w KQ - 5 7";
    let tricky_position = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -  0 1";
    let killer_position = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";

    let mut engine = Engine::new(tricky_position).unwrap();

    engine.print();
    engine.search_position(8);
}
