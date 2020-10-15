//! CSIS-616 - References and yaml example
//! 
//! Ralph W. Crosby PhD.
//! 
//! # Usage
//! 
//!    ```
//!     ./hw3 filename
//!     ```
//! 
//!    where: `filename` is a yaml file containing the DFA definition
//! 
//! # Output
//! 
//! Debugging output is produced to `stdout`. Build and run using:
//! 
//! ```
//! cargo run sample.yaml
//! ```

use std::io::Write;
use serde::{Deserialize};

/// # Deterministic Finite Automaton Structure
/// 
/// Create a structure that the YAML files will be deserialized into.
#[derive(Debug, Deserialize)]
struct DFA {
    alphabet: Vec<char>,
    start: u32,
    accept: Vec<u32>,
    transitions: Vec<Vec<u32>>,
    
    // This field isn't loaded from the YAML file so we need
    // to provide a default value for it
    #[serde(default)]
    n_states: u32
}

fn main() {
    use std::io::BufRead;

    let filename = get_filename(std::env::args());

    // Load the yaml file getting a Box pointing to a DFA
    // instance on the heap
    let dfa = DFA::new_from_file(&filename);

    // Check DFA for errors
    if let Err(msg) = dfa.validate() {
        writeln!(std::io::stderr(), "Failed to parse `{}`: {}", filename, msg)
                    .unwrap();
        std::process::exit(1);
    }

    // Get input from stdin
    println!("Enter strings to check if they are accepted or rejected:");
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        match line {
            // Panic on an error
            Err(error) => panic!("Error reading from stdin: {}", error),

            // Otherwise pass the string to the DFA
            Ok(line) => println!("{}", match dfa.accepts(&line) {
                true => "ACCEPT",
                false => "REJECT"
            })
        }
        println!()
    }
}

/// Get the filename passed as the first parameter
fn get_filename(args: std::env::Args) -> String {
    // Get the arguments as a vector
    let args: Vec<String> = args.collect();

    // Make sure only one argument was passed
    if args.len() != 2 {
        writeln!(std::io::stderr(), "Usage: ./hw2 <filename.yaml>")
                    .unwrap();
        std::process::exit(1);
    }
    
    args[1].to_string()
}  

impl DFA {
    /// Load the .yaml file specified into a DFA structure
    /// on the heap and return a point to it via a Box.
    fn new_from_file(filename: &str) -> Box<DFA> {
        let file = std::fs::File::open(filename)
                    .expect("Unable to open input");

        // Deserialize using serde
        let mut dfa: DFA = serde_yaml::from_reader(file)
                    .expect("Unable to parse yaml");
        
        // Compute number of states
        dfa.n_states = dfa.transitions.len() as u32;

        Box::new(dfa)
    }

    /// Check whether this DFA is well-formed.
    fn validate(&self) -> Result<(), String> {
        let alphabet_len = self.alphabet.len();
        let out_of_range = |s| !(1..=self.n_states).contains(s);

        // Check start state
        if out_of_range(&self.start) {
            return Err(format!("Unknown start state `{}`", self.start));
        }

        // Check final states
        for final_state in &self.accept {
            if out_of_range(final_state) {
                return Err(format!("Unknown final state `{}`", final_state));
            }
        }

        // Check transitions
        for (state, dest_states) in &mut self.transitions.iter().enumerate() {
            // Check number of transitions
            if dest_states.len() != alphabet_len {
                return Err(format!("State `{}` defines {} transitions (should define {})",
                        state + 1, dest_states.len(), alphabet_len));
            }

            // Check transition destinations
            for dest_state in dest_states {
                if out_of_range(&dest_state) {
                    return Err(format!("State `{}` cannot transition to unknown state `{}`",
                            state + 1, &dest_state));
                }
            }
        }

        return Ok(());
    }

    /// Check whether this DFA accepts the given string.
    fn accepts(&self, input: &str) -> bool {
        let mut state = self.start;
        for letter in input.chars() {
            // Get the transition index for this letter
            let transition = self.alphabet.iter().position(|&ltr| ltr == letter);

            // Follow the transition to the next state
            let new_state = match transition {
                None => panic!("Cannot parse string with non-alphabet letters"),
                Some(index) => self.transitions[state as usize - 1][index]
            };

            // Print the transition and actually update the state
            println!("  \u{3B4}(q{}, {}) \u{2192} q{}", state, letter, new_state);
            state = new_state;
        }
        self.accept.contains(&state)
    }
}

#[test]
fn test_accept() {
    // Accepts strings of {a, b} that end with a b
    let dfa = DFA {
        alphabet: vec!['a', 'b'],
        start: 1,
        accept: vec![2],
        transitions: vec![vec![2, 1], vec![2, 1]],
        n_states: 2
    };
    // positive inputs
    assert!(dfa.accepts("a"));
    assert!(dfa.accepts("baa"));
    assert!(dfa.accepts("baba"));

    // negative inputs
    assert!(!dfa.accepts(""));
    assert!(!dfa.accepts("b"));
    assert!(!dfa.accepts("ab"));
    assert!(!dfa.accepts("abab"));
}