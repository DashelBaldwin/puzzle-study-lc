// temp_tui.rs

use std::error::Error;
use std::io::{self, Write};

use regex::Regex;

use crate::api_requests;

use crate::api_requests::json_objects::Puzzle;
use api_requests::{get_from_ids::get_from_ids, get_last_n_incorrect::get_last_n_incorrect, post_overwrite::post_overwrite};

use crate::utils::termcolors::{Color, color};

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

    fn get_staged_ids(&self) -> Vec<String> {
        self.puzzles.iter().map(|puzzle| puzzle.id.clone()).collect()
    }

    fn state_message(&self) {
        println!("\n{}{}", color("PAT: ", Color::Cyan), self.pat);
        if self.study_id.is_empty() {
            println!("{}", color("Target study not set", Color::Yellow));
        } else {
            println!("{}{}", color("Study ID: ", Color::Cyan), self.study_id);
        }
        let staged_puzzles_color = if self.puzzles.len() > 0 { Color::Cyan } else { Color::Yellow };
        println!("{}/64 puzzles staged", color(&format!("{}", self.puzzles.len()), staged_puzzles_color));
    }

    fn prompt(&self) -> String {
        print!("> ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
    
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
    
        let input = input.trim();

        println!();
    
        input.to_string()
    }

    fn help_message(&self) {
        println!("Welcome to this strikingly beautiful temporary \"UI\"");
        
        println!("\nThis is planned to be replaced with a cross-platform TUI, which is far outside the scope of this \
                project, but it will look really cool");
        println!("Until that's done, this works well enough, which is good, because adding hundreds of pgns to a study by hand is \
                even less fun than making async UIs in Rust");
        println!("This is an example of the studies this script generates: https://lichess.org/study/UXdmGPS4/");

        println!("\nAnyway, you'll first need to setup a Personal Access Token on your lichess.org account");
        println!("You can do this here: https://lichess.org/account/oauth/token/create?scopes[]=puzzle:read&scopes[]=study:read&scopes[]=study:write&description=Puzzle+Studies");
    
        println!("\nIf you don't want to make an account just to see how this works, I've created an example one for you");
        println!("Note that anyone can upload stuff to the studies on this account, since the PAT is public");
        println!("The account's profile is: https://lichess.org/@/lazy_woodpecker");
        println!("You can use this PAT for full access to the endpoints this script invokes: lip_ewwjVusZzl6ovLXainV5");
        println!("The other studies it has access to can be found by searching 'owner:lazy_woodpecker' at https://lichess.org/study/");
    
        println!("\nOnce you have a PAT, paste it below. Note that tokens are not validated until they are used in an actual request");
    }

    fn options_message(&self) {
        println!("q - quit");
        println!("h - show this menu");
        println!("p - change PAT");
        println!("s - set/change study ID");
        println!("f - autofill puzzle set with your account's recent incorrect puzzles*");
        println!("[puzzle ID]... - add one or more puzzles by their IDs (whitespace or comma delimited)*");
        println!("u - upload all staged puzzles to the current study ID*");

        println!("\n*uses api requests, will involve some delay");
    }

    fn get_initial_user_pat(&mut self) {
        println!("Welcome. To get started, enter a lichess PAT, or press [ENTER] if you're confused.");
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
                eprintln!("{}", color("Failed to parse input (did you copy your PAT correctly?)\n", Color::Yellow)); 
            }
        }
    }

    fn get_user_pat(&mut self) {
        loop {
            println!("Paste your PAT below.");
            let input = self.prompt();
    
            let re = Regex::new(r"^lip_[a-zA-Z0-9]{20}$").unwrap();
    
            if re.is_match(&input) {
                println!("Using PAT {}.", input);
                self.pat = input.to_string();
                return;
            } else {
                eprintln!("{}", color("Failed to parse input (did you copy your PAT correctly?)\n", Color::Yellow)); 
            }
        }
    }

    fn get_study_id(&mut self) {
        self.is_data_stale = false;
        loop {
            println!("Paste the study ID below");
            let input = self.prompt();
    
            let re = Regex::new(r"^[a-zA-Z0-9]{8}$").unwrap();
    
            if re.is_match(&input) {
                println!("Set target study ID to {}.", input);
                self.study_id = input.to_string();
                return;
            } else {
                eprintln!("{}", color("Failed to parse input (did you copy the study ID correctly?)\n", Color::Yellow)); 
            }
        }
    }

    fn clear_puzzles(&mut self) {
        let plural_char = if self.puzzles.len() == 1 { "" } else { "s" };
        println!("Cleared {} puzzle{}", self.puzzles.len(), plural_char);
        self.puzzles.clear();
        self.is_data_stale = false;
    }

    async fn autofill(&mut self) -> Result<(), Box<dyn Error>> {
        if self.puzzles.len() >= 64 {
            return Err(Box::from("Stage is already full; use 'c' to clear it first"));
        }
        self.is_data_stale = false;
        let n = 64 - self.puzzles.len();
        println!("Autofilling {} puzzles (this may take a while)", n);
        let puzzles: Vec<Puzzle> = get_last_n_incorrect(self.pat.clone(), n, self.get_staged_ids()).await?;
        match puzzles.len() {
            1 => println!("Staged 1 puzzle"),
            _ => println!("Staged {} puzzles", puzzles.len())
        }
        self.puzzles.extend(puzzles);
        Ok(())
    }

    async fn upload(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_data_stale {
            return Err(Box::from("The currently staged puzzles have already been uploaded to this study"));
        } else if self.study_id.is_empty() {
            return Err(Box::from("Must enter a target study id before attempting to upload"));
        } else if self.puzzles.is_empty() {
            return Err(Box::from("Must stage at least one puzzle before attempting to upload"));
        }
        println!("Clearing study {} and uploading {} staged puzzles (this may take a while)", self.study_id, self.puzzles.len());
        post_overwrite(self.pat.clone(), &self.study_id, self.puzzles.clone()).await?;
        self.is_data_stale = true;
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
                "c" | "C" => self.clear_puzzles(),
                "f" | "F" => {
                    if let Err(e) = self.autofill().await {
                        eprintln!("{}", color(&e.to_string(), Color::Red));
                    }
                }
                "u" | "U" => {
                    if let Err(e) = self.upload().await {
                        eprintln!("{}", color(&e.to_string(), Color::Red));
                    }
                },
                _ => { 
                    let re = Regex::new(r"(?i)\b[a-z0-9]{5}\b(?:[, ]\s*)?").unwrap();
                    if re.is_match(&input) {
                        if self.puzzles.len() >= 64 {
                            eprintln!("{}", color("Stage is already full; use 'c' to clear it first", Color::Red));
                        } else {
                            self.is_data_stale = false;
                            let re = Regex::new(r"(?i)\b[a-z0-9]{5}\b").unwrap();
                            let puzzle_ids: Vec<String> = re.find_iter(&input)
                                .map(|mat| mat.as_str().to_string())
                                .collect();

                            let mut puzzles: Vec<Puzzle> = Vec::new();

                            match get_from_ids(puzzle_ids, self.get_staged_ids()).await {
                                Ok(result) => {
                                    puzzles = result;
                                }
                                Err(e) => {
                                    eprintln!("{}", color(&e.to_string(), Color::Red));
                                }
                            }

                            let new_size =  self.puzzles.len() + puzzles.len();
                            let truncated_set: Vec<Puzzle>;
                            if new_size > 64 {
                                let plural_char = if new_size - 64 == 1 { "" } else { "s" };
                                let warning_msg = &format!("Truncated {} puzzle{} that would exceed stage capacity", new_size-64, plural_char);
                                println!("{}", color(warning_msg, Color::Yellow));
                                truncated_set = puzzles[0..64 - self.puzzles.len()].iter().cloned().collect();
                                
                            } else {
                                truncated_set = puzzles.clone();
                            }
                            self.puzzles.extend(truncated_set);
                            match puzzles.len() {
                                1 => println!("Staged 1 puzzle"),
                                _ => println!("Staged {} puzzles", puzzles.len())
                            }
                        }
                    } else {
                        println!("{}", color("Failed to parse input (enter 'h' for a list of valid commands)", Color::Yellow)); 
                    }
                }
            }
        }
    }
}
