use std::collections::HashMap;
use std::collections::HashSet;
use crate::parser::AST;
use crate::parser::Node;
use crate::parser::CharClass;
use crate::multimap::*;

/// The max number of states is 2^16.
pub type StateId = u16;

/// A DFA's transitions are 1 to 1. An NFA's are 1 to many.
type DFATransitionMap = HashMap<char, StateId>;
type NFATransitionMap = MultiMap<char, StateId>;

/// Used internally in a few places.
/// Should be a character that is NOT in the language.
const DUMMY_TRANSITION: char = '_';

/// A deterministic finite automaton
/// (Not technically a DFA--can have undefined transitions)
#[derive(Debug)]
pub struct DFA {
    table: Vec<DFATransitionMap>,
    accept_states: HashSet<StateId>
}

/// A nondeterministic finite automaton
/// The epsilon table holds all epsilon transitions
#[derive(Debug)]
pub struct NFA {
    table: Vec<NFATransitionMap>,
    accept_state: StateId, // could be a HashSet, but our implementation only needs 1
    epsilon_table: MultiMap<StateId, StateId>
}

/// Convert an AST into an NFA via a post-order traversal
/// See `parse_nfa_node()` for the main algorithm
pub fn ast_to_nfa(tree: &AST) -> Box<NFA> {
    let mut nfa = NFA::new();
    nfa.accept_state = parse_nfa_node(tree.root(), nfa.add_state(), None, &mut nfa, tree);
    Box::new(nfa)
}

/// Convert an NFA into a DFA
/// The textbook's algorithm is fairly high level and requires computing the
/// power set of the states, which is expensive. This algorithm follows the same
/// general idea but without reserving space for unneeded states.
///
/// Possible Improvements:
/// I think this could be simplified using a queue, but I struggled to make it work
/// with the borrow checker. Also, the `composite_states` variable is conceptually
/// a bidirectional map, but I didn't want to import an external crate just for that.
pub fn nfa_to_dfa(nfa: &NFA) -> Box<DFA> {
    let mut current_state: StateId = 0;
    let mut highest_state: StateId = (nfa.table.len() - 1) as StateId;
    let mut composite_states: HashMap<StateId, HashSet<StateId>> = HashMap::new(); // really should be a bidi map
    let mut dfa_states = vec![];

    // Go through all the states, potentially adding new states to the end if
    // we encounter non-deterministic features along the way
    while current_state <= highest_state {

        // Get the current state's transition map, potentially creating it on
        // the spot if this is a new state
        let nfa_transitions = match composite_states.get(&current_state) {
            None => nfa.get(current_state).clone(),
            Some(set) => {
                let mut nfa_transitions = vec![];
                for &state in set { nfa_transitions.push(nfa.get(state)) }
                union_multi(&nfa_transitions)
            }
        };

        // Add this state to the DFA with a deterministic transition map
        dfa_states.push(nfa_transitions.into_iter()
            .map(|(label, next_states)| (label, match next_states.len() {
                // If this transition is 1:1, transfer it directly to the DFA
                1 => *next_states.iter().next().unwrap(),

                // If this transition is 1:many, make a new 'composite' state
                // and transfer that to the DFA
                _ => match find_key_by_value(&composite_states, &next_states) {
                    Some(state) => state,
                    None => {
                        highest_state += 1;
                        composite_states.insert(highest_state, next_states);
                        highest_state
                    }
                }}))
            .collect::<HashMap<char, StateId>>());
        current_state += 1;
    }

    // Any composite state that contains the original accept state
    // is now also an accept state
    let mut dfa_accept_states = nfa.accept_states();
    for (state, sub_states) in composite_states {
        if !sub_states.is_disjoint(&dfa_accept_states) {
            dfa_accept_states.insert(state);
        }
    }
    Box::new(DFA {table: dfa_states, accept_states: dfa_accept_states})
}

/// Recursively traverse through the AST, adding new states to the NFA
fn parse_nfa_node(node: &Node, input: StateId, output: Option<StateId>, nfa: &mut NFA, tree: &AST) -> StateId {
    use Node::*;
    match *node {
        Leaf(letter) => {
            // Regular letter, just add 1 state with 1 transition to it
            let output = nfa.get_or_add_state(output);
            nfa.add_transition(input, output, letter);
            output
        },
        LeafCharClass(class) => {
            // Character class, add 1 state with a transition for each class element
            let output = nfa.get_or_add_state(output);
            let range = match class {
                CharClass::AllLetter => 'a'..='z',
                CharClass::AllDigit => '0'..='9'
            };
            for chr in range {
                nfa.add_transition(input, output, chr);
            }
            output
        },
        And(first, second) => {
            // Concatenation, connect the two subtrees sequentially
            let intermediate = parse_nfa_node(tree.get(first), input, None, nfa, tree);
            parse_nfa_node(tree.get(second), intermediate, output, nfa, tree)
        },
        Or(choice1, choice2) => {
            // Alternation, connect the two subtrees with a branch (and rejoin them at the end)

            // slightly hacky trick--add a fake self-transition so
            // choice1 and choice2 don't treat this as a leaf state,
            // which are sometimes optimized out
            nfa.get_mut(input).add_multi(DUMMY_TRANSITION, input);

            let new_output = parse_nfa_node(tree.get(choice1), input, output, nfa, tree);
            parse_nfa_node(tree.get(choice2), input, Some(new_output), nfa, tree);
            nfa.get_mut(input).remove(&DUMMY_TRANSITION); // remove the fake self-transition
            new_output
        },
        RepeatStar(body) => {
            // Kleene star, make a cycle
            let loop_anchor = nfa.reuse_or_add_state(input);
            parse_nfa_node(tree.get(body), loop_anchor, Some(loop_anchor), nfa, tree);
            if let Some(output) = output {
                nfa.add_epsilon(loop_anchor, output);
                output
            } else {
                loop_anchor
            }
        },
        RepeatPlus(body) => {
            // Kleene plus, make a slightly different cycle
            let loop_anchor = nfa.reuse_or_add_state(input);
            let loop_output = parse_nfa_node(tree.get(body), loop_anchor, None, nfa, tree);
            nfa.add_epsilon(loop_output, loop_anchor);
            if let Some(output) = output {
                nfa.add_epsilon(loop_output, output);
                output
            } else {
                loop_output
            }
        }
    }
}

/// Find the first key that maps to the given value
/// This is where a bidirectional map would be handy.
fn find_key_by_value<K, V>(map: &HashMap<K, V>, value: &V) -> Option<K> where
        K: Copy, V: PartialEq {
    map.iter()
        .find(|(_k, v)| **v == *value)
        .map(|(k, _v)| *k)
}

impl NFA {
    /// Make a new, empty NFA
    pub fn new() -> NFA {
        NFA {
            table: vec![],
            accept_state: 0,
            epsilon_table: HashMap::new()
        }
    }

    /// Add a new state to the NFA and return its index
    pub fn add_state(&mut self) -> StateId {
        self.table.push(HashMap::new());
        (self.table.len() - 1) as StateId
    }

    /// If the given state exists, return it; otherwise make a new one
    pub fn get_or_add_state(&mut self, state: Option<StateId>) -> StateId {
        state.unwrap_or_else(|| self.add_state())
    }

    /// If the given state is a leaf (no outgoing transitions), return it;
    /// otherwise make a new one and add an epsilon transition to it
    pub fn reuse_or_add_state(&mut self, state: StateId) -> StateId {
        if self.is_leaf_state(state) {
            state
        } else {
            let new_state = self.add_state();
            self.add_epsilon(state, new_state);
            new_state
        }
    }
    
    /// Add a labeled transition between two states
    pub fn add_transition(&mut self, from: StateId, to: StateId, label: char) {
        self.get_mut(from).add_multi(label, to);
        for state in self.epsilon_table.get_multi(from) {
            self.get_mut(state).add_multi(label, to);
        }
    }

    /// Add an epsilon transition between two states
    pub fn add_epsilon(&mut self, from: StateId, to: StateId) {
        self.epsilon_table.add_multi(to, from);
        for state in self.epsilon_table.get_multi(from) {
            self.epsilon_table.add_multi(to, state);
        }
        for (label, states) in self.get(to).clone() {
            for epsilon_state in self.epsilon_table.get_multi(to) {
                self.get_mut(epsilon_state).add_all_multi(label, &states);
            }
        }
    }

    /// Get a state's transition map immutably
    pub fn get(&self, state: StateId) -> &NFATransitionMap {
        &self.table[state as usize]
    }

    /// Get a state's transition map mutably
    pub fn get_mut(&mut self, state: StateId) -> &mut NFATransitionMap {
        &mut self.table[state as usize]
    }

    /// Get this NFA's accept states as a set
    pub fn accept_states(&self) -> HashSet<StateId> {
        let mut states = self.epsilon_table.get_multi(self.accept_state);
        states.insert(self.accept_state);
        states
    }

    /// Check whether a state has any outgoing transitions
    pub fn is_leaf_state(&self, state: StateId) -> bool {
        self.get(state).is_empty()
    }

    /// Get the GraphViz representation of this NFA
    #[allow(dead_code)]
    pub fn to_graph(&self) -> String {
        crate::graphviz::generate(0, &self.accept_states(), &self.edges())
    }

    fn edges(&self) -> Vec<crate::graphviz::Edge> {
        self.table.iter()
                .enumerate()
                .flat_map(|(s, trans)| trans.iter()
                    .flat_map(move |(label, set)| set.iter()
                        .map(move |dest| (s as StateId, *dest, *label))))
                .collect()
    }
}

impl DFA {
    /// Get a state's transition map immutably
    pub fn get(&self, state: StateId) -> &DFATransitionMap {
        return self.table.get(state as usize).unwrap();
    }

    /// Check whether a string is accepted by this DFA
    pub fn accepts(&self, input: &str) -> bool {
        let mut state = 0;
        for letter in input.chars() {
            match self.get(state).get(&letter) {
                None => return false,                    // reject if no transition defined
                Some(&next_state) => state = next_state  // otherwise move to next state
            }
        }
        self.accept_states.contains(&state)
    }

    /// Get the GraphViz representation of this DFA
    pub fn to_graph(&self) -> String {
        crate::graphviz::generate(0, &self.accept_states, &self.edges())
    }

    fn edges(&self) -> Vec<crate::graphviz::Edge> {
        let reachable = self.get_reachable();
        self.table.iter()
                .enumerate()
                .flat_map(|(s, trans)| trans.iter()
                    .map(move |(label, dest)| (s as StateId, *dest, *label)))
                .filter(|(from, to, _label)| reachable.contains(from) && reachable.contains(to))
                .collect()
    }

    fn get_reachable(&self) -> HashSet<StateId> {
        let mut reachable = HashSet::new();
        self.visit_reachable(0, &mut reachable);
        reachable
    }

    fn visit_reachable(&self, state: StateId, reachable: &mut HashSet<StateId>) {
        if reachable.insert(state) {
            for (_label, neighbor) in self.get(state) {
                self.visit_reachable(*neighbor, reachable);
            }
        }
    }
}