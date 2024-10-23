// main.rs

// TODO: make directly imported puzzles correctly say they were imported from id, instead of from puzzle history
// TODO: start working on the cli application

// Possible TODO: also allow pasting chess.com puzzle exported pgns into cli as input for convenience
// Possible TODO: make auto generation skip puzzles manually imported into the same set by user to prevent duplicates

use std::error::Error;

mod api_requests;
mod notation_utils;

use api_requests::{get_from_ids::get_from_ids, get_last_n_incorrect::get_last_n_incorrect, post_overwrite::post_overwrite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut puzzles = get_from_ids(vec!["sKGZw", "c5A8O", "f1kLA", "8K3p9"]).await?;
    for puzzle in get_last_n_incorrect(4).await? {
        puzzles.push(puzzle);
    }

    post_overwrite("n38KtP3G", puzzles).await?;

    Ok(())
}
