use serde::Deserialize;
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

//lip_NfKGhBFyeXxX0yerqoCi
const PAT: &str = "lip_NfKGhBFyeXxX0yerqoCi";

fn main() -> Result<(), Box<dyn Error>> {
    //get_study_text_blocking("pr3w5plu").unwrap();

    get_puzzle_history_blocking(3)?;

    Ok(())
}

fn get_study_text_blocking(id: &str) -> Result<(), Box<dyn Error>> {
    let body = reqwest::blocking::get(format!("https://lichess.org/api/study/{id}.pgn"))?
        .text()?;
    println!("body = {:?}", body);

    Ok(())
}


fn get_puzzle_history_blocking(max: i32) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let response = client
        .get("https://lichess.org/api/puzzle/activity")
        .headers(headers)
        //.query(&[("max", max)])
        .send()?;

    if response.status().is_success() {
        let body = response.text()?;
        println!("Response: {}", body);
    } else {
        eprintln!("Error: {}", response.status());
    }

    Ok(())
}


// fn post_to_study_blocking() -> Result<(), Box<dyn Error>> {
    
// }
