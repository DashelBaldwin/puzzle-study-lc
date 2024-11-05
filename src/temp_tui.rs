// temp_tui.rs

use std::error::Error;
use std::io::{self, Write};

use regex::Regex;

use crate::api_requests;
use crate::notation_utils;

use crate::api_requests::json_objects::Puzzle;
use api_requests::{get_from_ids::get_from_ids, get_last_n_incorrect::get_last_n_incorrect, post_overwrite::post_overwrite};

pub struct App {
    pat: String,
    study_id: String,
    puzzles: Vec<Puzzle>
}

impl App {
    fn state_message(&self) {
        println!("PAT: {}", self.pat);
        if self.study_id.is_empty() {
            println!("Target study not set");
        } else {
            println!("Study ID: {}", self.study_id);
        }
        println!("{} puzzles staged", self.puzzles.len());
    }
}

fn prompt() -> String {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let input = input.trim();

    input.to_string()
}

fn help_message() {
    println!("\nWelcome to this scintillatingly beautiful temporary \"UI\".");
    println!("This is planned to be replaced with a cross-platform TUI, which is far outside the scope of this \
            project, but it will look really cool.");
    println!("Until that's done, this works well enough, which is good, because adding hundreds of pgns to a study by hand is \
            even less fun than making async UIs in Rust.");

    println!("\nAnyway, you'll first need to setup a Personal Access Token on your lichess.org account.");
    println!("You can do this here: https://lichess.org/account/oauth/token/create?scopes[]=puzzle:read&scopes[]=study:read&scopes[]=study:write&description=Puzzle+Studies");

    println!("\nIf you don't want to make an account just to see how this works, I've created an example one for you.");
    println!("Note that anyone can upload stuff to the studies on this account, since the PAT is public.");
    println!("(Account sharing is probably against lichess TOS, so don't tell anyone.)");

    println!("\nOnce you have your PAT, paste it below. Note that tokens are not validated until they are used in an actual request.");

    // Need to make this account and add a link to its study page
}

fn options_message() {
    println!("q - quit");
    println!("h - show this menu");
    println!("p - change PAT");
    println!("s - set/change study ID");
    println!("f - *autofill puzzle set with your account's recent incorrect puzzles");
    println!("[puzzle ID] - *add a puzzle by its ID");
    println!("u - *upload all staged puzzles to the current study ID");
    println!("*uses api requests, will involve some delay");
}

pub fn get_initial_user_pat() -> String {
    println!("Congrats, you got this working. To get started, enter a lichess PAT, or press [ENTER] if you're confused.");
    loop {
        let input = prompt();

        let re = Regex::new(r"^lip_[a-zA-Z0-9]{20}$").expect("Failed to init regex");

        if re.is_match(&input) {
            println!("Using PAT {}. Note that this won't be validated until an authenticated request is sent.", input);
            return input.to_string();
        } else if input.is_empty() {
            help_message();
        } else {
            println!("Failed to parse input (did you copy your PAT correctly?)");
        }
    }
}

pub fn get_user_pat() -> String {
    loop {
        let input = prompt();

        let re = Regex::new(r"^lip_[a-zA-Z0-9]{20}$").expect("Failed to init regex");

        if re.is_match(&input) {
            println!("Using PAT {}.", input);
            return input.to_string();
        } else {
            println!("Failed to parse input (did you copy your PAT correctly?)");
        }
    }
}

pub fn get_study_id() -> String {
    loop {
        let input = prompt();

        let re = Regex::new(r"^[a-zA-Z0-9]{8}$").expect("Failed to init regex");

        if re.is_match(&input) {
            println!("Using target study ID to {}.", input);
            return input.to_string();
        } else {
            println!("Failed to parse input (did you copy the study ID correctly?");
        }
    }
}

pub fn app_prompt() {

}