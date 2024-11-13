// Possible TODO: also allow pasting chess.com puzzle exported pgns into cli as input for convenience

mod api_requests;
mod notation_utils;
mod utils;
mod temp_tui;

#[tokio::main]
async fn main() {
    let mut app = temp_tui::App::new();
    app.run().await;
}
