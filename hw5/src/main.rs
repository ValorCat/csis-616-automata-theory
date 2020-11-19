//! CSIS-616 - Chapter 5 Homework
//! 
//! Ralph W. Crosby PhD.
//! 
//! # Usage
//! 
//!    ```
//!     ./hw5 filename
//!     ```
//! 
//!    where: `filename` is a yaml file containing the PDA definition
//! 
//! # Output
//! 
//! Debugging output is produced to `stdout`. Build and run using:
//! 
//! ```
//! cargo run sample.yaml
//! ```

use std::io::Write;
use serde::Deserialize;

/// # Push Down Automaton
/// 
/// Create a structure that the YAML files will be deserialized into.
#[derive(Debug, Deserialize)]
struct PDA {
    alphabet: Vec<String>,
    stack_alphabet: Vec<String>,
    start: usize,
    accept: Vec<usize>,
    transitions: Vec<Vec<Transition>>,
}

/// # Transition Structure
///
/// Represent a 4-tuple: (input_char, top_of_stack, push_to_stack, new_state)
#[derive(Clone, Debug, Deserialize)]
struct Transition(String, String, String, usize);

/// # Graph Structure
/// 
/// Contains a vector of nodes and the index of the start node.
#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    start_node: usize
}

/// # Node Structure
/// 
/// Contains a vector of all connected node indices, the transition
/// labels for those connections (can be left undefined), and a
/// boolean indicating whether this is a final node.
#[derive(Debug)]
struct Node {
    connections: Vec<Transition>,
    accept_state: bool
}

fn main() {
    let filename = get_filename(std::env::args());

    // Load the yaml file getting a Box pointing to a PDA
    // instance on the heap
    let pda = PDA::new_from_file(&filename);

    // Check PDA for errors
    if let Err(msg) = pda.validate() {
        eprintln!("Failed to parse `{}`: {}", filename, msg);
        std::process::exit(1);
    }

    // Convert to Graph and display in stdout
    let graph = pda.to_graph();
    println!("\nGraphViz definition:\n");
    graph.print_graphviz();
    println!("\nDebug printed graph structure:\n");
    graph.print();
}

/// Get the filename passed as the first parameter
fn get_filename(args: std::env::Args) -> String {
    // Get the arguments as a vector
    let args: Vec<String> = args.collect();

    // Make sure only one argument was passed
    if args.len() != 2 {
        writeln!(std::io::stderr(), "Usage: ./hw5 <filename.yaml>")
                    .unwrap();
        std::process::exit(1);
    }
    
    args[1].to_string()
}  

/// Check if a letter is in the given alphabet
fn in_alphabet(ltr: &String, alphabet: &Vec<String>) -> bool {
    ltr.is_empty() || alphabet.contains(ltr)
}

impl PDA {
    /// Load the .yaml file specified into a PDA structure
    /// on the heap and return a point to it via a Box.
    fn new_from_file(filename: &str) -> Box<PDA> {
        let file = std::fs::File::open(filename)
                .expect("Unable to open input");

        // Deserialize using serde
        let pda: PDA = serde_yaml::from_reader(file)
                .expect("Unable to parse yaml");

        Box::new(pda)
    }

    /// Check whether this PDA is well-formed.
    fn validate(&self) -> Result<(), String> {
        let num_states = self.transitions.len();
        let out_of_range = |s| !(1..=num_states).contains(s);

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
        for (state, transitions) in self.transitions.iter().enumerate() {
            for trans in transitions {
                if !in_alphabet(&trans.0, &self.alphabet) {
                    return Err(format!("State {} cannot transition on unknown input character `{}`", state, trans.0));
                }
                if !in_alphabet(&trans.1, &self.stack_alphabet) {
                    return Err(format!("State {} cannot pop unknown stack character `{}`", state, trans.1));
                }
                if !in_alphabet(&trans.2, &self.stack_alphabet) {
                    return Err(format!("State {} cannot push unknown stack character `{}`", state, trans.2));
                }
                if out_of_range(&trans.3) {
                    return Err(format!("State {} cannot transition to unknown state `{}`", state, trans.3));
                }
            }
        }

        return Ok(());
    }

    /// Generate a Graph structure from this PDA.
    fn to_graph(&self) -> Box<Graph> {
        // Create vec of nodes
        let mut nodes = vec![];
        for (state, transitions) in self.transitions.iter().enumerate() {
            nodes.push(Node {
                connections: transitions.to_vec(),
                accept_state: self.accept.contains(&(state + 1))
            })
        }

        // Insert start node
        let start_trans = Transition("".to_string(), "".to_string(), "".to_string(), self.start);
        nodes.insert(0, Node {connections: vec![start_trans], accept_state: false});

        Box::new(Graph {nodes, start_node: 0})
    }
}

impl Graph {
    /// Print this graph in debug format to stdout.
    fn print(&self) {
        println!("{:?}", self); // could use {:#?} for pretty printing
    }

    /// Print this graph in GraphViz format to stdout.
    fn print_graphviz(&self) {
        println!("{}", self.to_graphviz());
    }

    /// Return this graph as a string in GraphViz format.
    fn to_graphviz(&self) -> String {
        let mut transitions = vec![];

        let check_epsilon = |ltr: String| if ltr.is_empty() { "&epsilon;".to_string() } else { ltr } ;

        // Build list of transitions
        for (num, node) in self.nodes.iter().enumerate() {
            for trans in &node.connections {
                let input = check_epsilon(trans.0.clone());
                let pop_stack = check_epsilon(trans.1.clone());
                let push_stack = check_epsilon(trans.2.clone());
                transitions.push(format!("q{} -> q{} [label=\"{}, {} &rarr; {}\"]", num, trans.3, input, pop_stack, push_stack));
            }
        }

        // Build string of accept nodes
        let accept_nodes = self.nodes.iter()
                .enumerate()
                .filter(|node| node.1.accept_state)
                .map(|node| format!("q{}", node.0))
                .collect::<Vec<String>>()
                .join("; ");
        
        format!(
            "digraph {{\n\
                rankdir=LR;\n\
                node [shape=point]; q{};\n\
                node [shape=doublecircle]; {};\n\
                node [shape=circle];\n\
                {};\n\
            }}",
            self.start_node,
            accept_nodes,
            transitions.join(";\n")
        )
    }
}

// impl Node {
//     /// Create a new node with the specified transition labels.
//     fn new(labels: &Vec<char>) -> Node {
//         let label_refs = labels.iter()
//                 .map(|&lbl| Some(lbl))
//                 .collect::<Vec<Option<char>>>();
//         Node::new_empty()
//     }

//     /// Create a new empty node.
//     fn new_empty() -> Node {
//         Node {
//             connections: vec![],
//             accept_state: false
//         }
//     }
// }

#[test]
fn test_to_graphviz() {
    let pda = PDA::new_from_file("sample.yaml");
    let graph = pda.to_graph();
    assert_eq!(graph.to_graphviz(),
"digraph {
rankdir=LR;
node [shape=point]; q0;
node [shape=doublecircle]; q2;
node [shape=circle];
q0 -> q1 [label=\"&epsilon;, &epsilon; &rarr; &epsilon;\"];
q1 -> q1 [label=\"0, &epsilon; &rarr; 0\"];
q1 -> q1 [label=\"1, &epsilon; &rarr; 1\"];
q1 -> q2 [label=\"&epsilon;, &epsilon; &rarr; &epsilon;\"];
q2 -> q2 [label=\"0, 0 &rarr; &epsilon;\"];
q2 -> q2 [label=\"1, 1 &rarr; &epsilon;\"];
}"
    );
}
