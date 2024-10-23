// main.rs

// TODO: make directly imported puzzles correctly say they were imported from id, instead of from puzzle history
// TODO: start working on the cli application

// Possible TODO: also allow pasting chess.com puzzle exported pgns into cli as input for convenience
// Possible TODO: make auto generation skip puzzles manually imported into the same set by user to prevent duplicates

use std::error::Error;
mod api_requests;
mod notation_utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // clear_and_upload("l2Yn1iSK", puzzles).await?;

    Ok(())
}
