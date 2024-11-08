// temp_tui.rs

use std::error::Error;
use std::io::{self, Write};

use regex::Regex;

use crate::api_requests;

use crate::api_requests::json_objects::Puzzle;
use api_requests::{get_from_ids::get_from_ids, get_last_n_incorrect::get_last_n_incorrect, post_overwrite::post_overwrite};

pub struct App {
    pat: String,
    study_id: String,
    puzzles: Vec<Puzzle>,
    is_data_stale: bool
}

impl App {
    pub fn new() -> Self {
        Self {
            pat: "".to_string(),
            study_id: "".to_string(),
            puzzles: Vec::new(),
            is_data_stale: false
        }
    }

    fn state_message(&self) {
        println!("\nPAT: {}", self.pat);
        if self.study_id.is_empty() {
            println!("Target study not set");
        } else {
            println!("Study ID: {}", self.study_id);
        }
        println!("{}/64 puzzles staged", self.puzzles.len());
    }

    fn prompt(&self) -> String {
        print!("> ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
    
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
    
        let input = input.trim();
    
        input.to_string()
    }

    fn help_message(&self) {
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

    fn options_message(&self) {
        println!("\nq - quit");
        println!("h - show this menu");
        println!("p - change PAT");
        println!("s - set/change study ID");
        println!("f - *autofill puzzle set with your account's recent incorrect puzzles");
        println!("[puzzle ID]... - *add one or more puzzles by their IDs (whitespace or comma delimited)");
        println!("u - *upload all staged puzzles to the current study ID");
        println!("*uses api requests, will involve some delay");
    }

    fn get_initial_user_pat(&mut self) {
        println!("Congrats, you got this working. To get started, enter a lichess PAT, or press [ENTER] if you're confused.");
        loop {
            let input = self.prompt();
    
            let re = Regex::new(r"^lip_[a-zA-Z0-9]{20}$").unwrap();
    
            if re.is_match(&input) {
                println!("Using PAT {}. Note that this won't be validated until an authenticated request is sent.", input);
                self.pat = input.to_string();
                return;
            } else if input.is_empty() {
                self.help_message();
            } else {
                println!("Failed to parse input (did you copy your PAT correctly?)");
            }
        }
    }

    fn get_user_pat(&mut self) {
        self.is_data_stale = false;
        loop {
            println!("Paste your PAT below.");
            let input = self.prompt();
    
            let re = Regex::new(r"^lip_[a-zA-Z0-9]{20}$").unwrap();
    
            if re.is_match(&input) {
                println!("Using PAT {}.", input);
                self.pat = input.to_string();
                return;
            } else {
                println!("Failed to parse input (did you copy your PAT correctly?)");
            }
        }
    }

    fn get_study_id(&mut self) {
        self.is_data_stale = false;
        loop {
            println!("Paste the study ID below.");
            let input = self.prompt();
    
            let re = Regex::new(r"^[a-zA-Z0-9]{8}$").unwrap();
    
            if re.is_match(&input) {
                println!("Using target study ID to {}.", input);
                self.study_id = input.to_string();
                return;
            } else {
                println!("Failed to parse input (did you copy the study ID correctly?");
            }
        }
    }

    async fn autofill(&mut self) -> Result<(), Box<dyn Error>> {
        if self.puzzles.len() >= 64 {
            println!("Error: stage is full"); // return Error here eventually
        }
        let n = 64 - self.puzzles.len();
        println!("Autofilling {} puzzles", n);
        let puzzles: Vec<Puzzle> = get_last_n_incorrect(self.pat.clone(), n).await?;
        match puzzles.len() {
            1 => println!("\nStaged 1 puzzle"),
            _ => println!("\nStaged {} puzzles", puzzles.len())
        }
        self.puzzles.extend(puzzles);
        Ok(())
    }

    async fn upload(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_data_stale {
            println!("Error: this set has already been uploaded");
        }
        self.is_data_stale = true;
        println!("Clearing study {} and uploading {} staged puzzles", self.study_id, self.puzzles.len());
        println!("This may take a while...\n");
        post_overwrite(self.pat.clone(), &self.study_id, self.puzzles.clone()).await?;
        Ok(())
    }
    
    pub async fn run(&mut self) {
        self.get_initial_user_pat();

        loop {
            self.state_message();
            println!("\nPlease enter an action (enter 'h' for a list of valid commands)");
            let input: String = self.prompt();

            match input.as_str() {
                "q" | "Q" => break,
                "h" | "H" => self.options_message(),
                "p" | "P" => self.get_user_pat(),
                "s" | "S" => self.get_study_id(),
                "f" | "F" => {
                    if let Err(e) = self.autofill().await {
                        eprintln!("{}", e);
                    }
                }
                "u" | "U" => {
                    if let Err(e) = self.upload().await {
                        eprintln!("{}", e);
                    }
                },
                _ => { 
                    let re = Regex::new(r"(?i)\b[a-z0-9]{5}\b(?:[, ]\s*)?").unwrap();
                    if re.is_match(&input) {
                        if self.puzzles.len() >= 64 {
                            println!("Error: stage is full");
                        } else {
                            self.is_data_stale = false;
                            let re = Regex::new(r"(?i)\b[a-z0-9]{5}\b").unwrap();
                            let puzzle_ids: Vec<String> = re.find_iter(&input)
                                .map(|mat| mat.as_str().to_string())
                                .collect();

                            let mut puzzles: Vec<Puzzle> = Vec::new();

                            match get_from_ids(puzzle_ids).await {
                                Ok(result) => {
                                    puzzles = result;
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                }
                            }

                            let new_size =  self.puzzles.len() + puzzles.len();
                            let truncated_set: Vec<Puzzle>;
                            if new_size > 64 {
                                let plural_char = if new_size - 64 == 1 { "" } else { "s" };
                                println!("Truncated {} puzzle{} that would exceed stage capacity", new_size-64, plural_char);
                                truncated_set = puzzles[0..64 - self.puzzles.len()].iter().cloned().collect();
                                
                            } else {
                                truncated_set = puzzles.clone();
                            }
                            self.puzzles.extend(truncated_set);
                            match puzzles.len() {
                                1 => println!("\nStaged 1 puzzle"),
                                _ => println!("\nStaged {} puzzles", puzzles.len())
                            }
                        }
                    } else {
                        println!("Failed to parse input (enter 'h' for a list of valid commands)\n"); 
                    }
                }
            }
        }
    }
}
