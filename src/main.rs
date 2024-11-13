// Possible TODO: also allow pasting chess.com puzzle exported pgns into cli as input for convenience

mod api_requests;
mod notation_utils;
mod utils;
mod temp_tui;

#[tokio::main]
async fn main() {
    // let mut app = temp_tui::App::new();
    // app.run().await;

    let pgn = "d4 d5 c3 c6 c4 e5 dxe5 f5 exf6 Ne7 fxg7 dxc4 gxh8=Q c3 e4 Nf5 exf5 cxb2 f6 bxc1=R Nf3 Bg4 Qg7 Bxg7 fxg7 Qd7 g8=N Bf5 Ne7 Rc2 Nc3 a5 Nxf5 h5 Nb5 h4 Nf5d4 h3 Nf5 hxg2 N3d4 Rc1 Bxg2 Na6 O-O O-O-O Nxc6 b6 Qxd7+ Rxd7 Rd1 Rc2 Rxd7 Kxd7 Rd1+ Kc8 Rd8+ Kb7 Nfd6#";
    println!("{}", notation_utils::pgn_to_fen::pgn_to_fen(pgn));
}
