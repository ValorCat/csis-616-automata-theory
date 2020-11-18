//! CSIS-616 - Regex Parser and Evaluator
//! 
//! Anthony Morrell
//! 
//! # Usage
//! 
//!    ```
//!     ./regex string
//!     ```
//! 
//!    where: `string` is a regular expression
//! 
//! # Output
//! 
//! Output is sent to `stdout` and `stderr`. Build and run using:
//! 
//! ```
//! cargo run "ab*"
//! ```

mod lexer;
mod parser;
mod automata;
mod multimap;

/// If the regex contains contiguous spaces, you must wrap it in quotes, e.g. "a  b"
fn main() {
    use std::io::BufRead;

    // get command line args as a string
    let args = std::env::args()
            .skip(1)
            .collect::<Vec<String>>()
            .join(" ");

    // print the usage if there's no args
    if args.is_empty() {
        println!("Usage: ./regex <regex>");
        std::process::exit(0);
    }

    // convert the regex to a dfa
    let dfa = regex_to_dfa(&args);

    // print the graphviz definition
    println!("---[ DFA Graph ]----------------");
    println!("{}", dfa.to_graph());
    println!("--------------------------------");

    // run user given strings through dfa
    println!("Enter strings to test them:");
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) if dfa.accepts(&line) => eprintln!("Accept {}", line),
            Ok(line) => eprintln!("Reject {}", line),
            Err(error) => {
                eprintln!("Error reading from stdin: {}", error);
                std::process::exit(1);
            }
        }
    }
}

fn regex_to_dfa(regex: &str) -> Box<automata::DFA> {
    // lex and parse
    let tokens = lexer::tokenize(regex);
    let mut tree = parser::tree();
    parser::parse(&tokens, &mut tree);

    // make nfa, then dfa
    let nfa = automata::ast_to_nfa(&tree);
    automata::nfa_to_dfa(&nfa)
}

mod graphviz {
    use crate::automata::StateId;
    use std::collections::HashSet;

    pub type Edge = (StateId, StateId, char);

    pub fn generate(start: StateId, end: &HashSet<StateId>, edges: &Vec<Edge>) -> String {
        format!(
            "digraph {{\n\
                rankdir=LR;\n\
                node [shape=point]; start;\n\
                node [shape=doublecircle]; {end_nodes}\n\
                node [shape=circle];\n\
                start -> {start_node};\n\
                {edges}\
            }}",
            start_node=start,
            end_nodes=end.iter()
                .map(|&s| s.to_string() + "; ")
                .collect::<String>(),
            edges=edges.iter()
                .map(|(from, to, label)| format!("{} -> {} [label=\"{}\"];\n", from, to, label))
                .collect::<String>()
        )
    }
}

#[test]
fn test() {
    let dfa = regex_to_dfa("abab*");
    assert!(dfa.accepts("aba"));
    assert!(dfa.accepts("abab"));
    assert!(dfa.accepts("ababb"));

    assert!(!dfa.accepts("ab"));
    assert!(!dfa.accepts(""));
    assert!(!dfa.accepts("abaa"));
}
