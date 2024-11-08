// main.rs

// TODO: make auto generation skip puzzles manually imported into the same set by user to prevent duplicates
// Possible TODO: also allow pasting chess.com puzzle exported pgns into cli as input for convenience

use std::error::Error;

mod api_requests;
mod notation_utils;
mod utils;
mod temp_tui;

#[tokio::main]
async fn main() {
    // let mut puzzles = get_from_ids(vec!["VwGJ7", "f1kLA", "c5A8O", "eqlJZ", "AIlWN"]).await?;
    // for puzzle in get_last_n_incorrect(5).await? {
    //     puzzles.push(puzzle);
    // }
    // post_overwrite("n38KtP3G", puzzles).await?;

    let mut app = temp_tui::App::new();
    app.run().await;

}
