//! CSIS-616 - References and yaml example
//! 
//! Ralph W. Crosby PhD.
//! 
//! # Usage
//! 
//!    ```
//!     ./yaml_dfa filename
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

/// # Graph Structure
/// 
/// Contains a vector of nodes and the index of the start node.
#[derive(Debug)]
struct Graph {
    nodes: Vec<Node>,
    start_node: u32
}

/// # Node Structure
/// 
/// Contains a vector of all connected node indices, the transition
/// labels for those connections (can be left undefined), and a
/// boolean indicating whether this is a final node.
#[derive(Debug)]
struct Node {
    connections: Vec<u32>,
    labels: Vec<Option<char>>,
    accept_state: bool
}

fn main() {
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

    // Convert to Graph and display in stdout
    let graph = dfa.to_graph();
    println!("\nGraphViz definition:");
    graph.print_graphviz();
    println!("\nDebug printed graph structure:");
    graph.print();
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

    /// Generate a Graph structure from this DFA.
    fn to_graph(&self) -> Box<Graph> {
        // Create a vector of "blank" nodes
        let mut nodes = (0..self.n_states)
                .map(|_| Node::new(&self.alphabet))
                .collect::<Vec<Node>>();

        // Insert the start node
        let mut start = Node::new_empty();
        start.connections = vec![self.start];
        start.labels = vec![None];
        nodes.insert(0, start);

        // Flag the final states
        for state_num in &self.accept {
        nodes[*state_num as usize].accept_state = true;
        }

        // Connect the nodes
        for (node, state_num) in nodes.iter_mut().skip(1).zip(0..) {
        node.connections = self.transitions[state_num].clone();
        }

        Box::new(Graph {nodes: nodes, start_node: 0})
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
        // Collect all the transitions into a vector
        let mut transitions = vec![];
        for (i, node) in self.nodes.iter().enumerate() {
            for (target, label) in node.connections.iter().zip(&node.labels) {
                transitions.push(match *label {
                    Some(lbl) => format!("q{} -> q{} [label=\"{}\"]", i, target, lbl),
                    None =>      format!("q{} -> q{}", i, target)
                });
            }
        }

        // Here's a fun way to do the above using functional programming.
        // I ended up rewriting with a loop because this isn't as readable,
        // but it does leave the resulting variable immutable, which is nice.
        // let transitions = self.nodes.iter()
        //         .enumerate()
        //         .flat_map(|(i, node)| node.connections.iter()
        //             .zip(&node.labels)
        //             .map(move |(target, label)| match *label {
        //                 Some(lbl) => format!("q{} -> q{} [label=\"{}\"]", i, target, lbl),
        //                 None      => format!("q{} -> q{}", i, target)
        //             }))
        //         .collect::<Vec<String>>();

        // Collect the final states into a vector
        let end_nodes = self.nodes.iter()
                .enumerate()
                .filter(|node| node.1.accept_state)
                .map(|node| format!("q{}", node.0))
                .collect::<Vec<String>>();

        // Print the GraphViz definition
        format!(
"digraph {{
    rankdir=LR;
    node [shape=point]; q{};
    node [shape=doublecircle]; {};
    node [shape=circle];
    {};
}}",
            self.start_node,
            end_nodes.join("; "),
            transitions.join(";\n    ")
        )
    }
}

impl Node {
    /// Create a new node with the specified transition labels.
    fn new(labels: &Vec<char>) -> Node {
        let label_refs = labels.iter()
                .map(|&lbl| Some(lbl))
                .collect::<Vec<Option<char>>>();
        Node {
            connections: vec![],
            labels: label_refs,
            accept_state: false
        }
    }

    /// Create a new empty node.
    fn new_empty() -> Node {
        Node {
            connections: vec![],
            labels: vec![],
            accept_state: false
        }
    }
}

#[test]
fn test_to_graphviz() {
    let nodes = vec![
        Node {
            connections: vec![1],
            labels: vec![None],
            accept_state: false
        },
        Node {
            connections: vec![1, 2],
            labels: vec![Some('a'), Some('b')],
            accept_state: false
        },
        Node {
            connections: vec![2],
            labels: vec![Some('a')],
            accept_state: true
        }
    ];
    let graph = Graph {
        nodes: nodes,
        start_node: 0
    };
    
    assert_eq!(graph.to_graphviz(),
"digraph {
    rankdir=LR;
    node [shape=point]; q0;
    node [shape=doublecircle]; q2;
    node [shape=circle];
    q0 -> q1;
    q1 -> q1 [label=\"a\"];
    q1 -> q2 [label=\"b\"];
    q2 -> q2 [label=\"a\"];
}"
    );
}
