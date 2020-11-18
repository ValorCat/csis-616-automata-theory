use crate::lexer::Token;

pub type NodeId = usize;

/// Represents an abstract syntax tree
/// The last element is the root
#[derive(Debug)]
pub struct AST {
    nodes: Vec<Node>
}

/// An abstract syntax tree node
#[derive(Debug)]
pub enum Node {
    Leaf(char),
    LeafCharClass(CharClass),
    And(NodeId, NodeId),
    Or(NodeId, NodeId),
    RepeatStar(NodeId),
    RepeatPlus(NodeId)
}

/// A character class, either "all letters" or "all digits"
#[derive(Clone, Copy, Debug)]
pub enum CharClass {
    AllLetter, AllDigit
}

/// Creates an empty abstract syntax tree
pub fn tree() -> AST {
    AST {nodes: vec![]}
}

/// Parses a sequence of tokens into a tree from the bottom up
pub fn parse(tokens: &[Token], tree: &mut AST) -> NodeId {
    use crate::lexer;
    use Node::*;
    
    let new_node =
        // find alternations (|)
        if let Some(index) = lexer::find(tokens, Token::Union) {
            let left = parse(&tokens[..index], tree);
            let right = parse(&tokens[index+1..], tree);
            Or(left, right)

        // find concatenations
        } else if let Some((_, index)) = lexer::find_adjacent_values(tokens) {
            let left = parse(&tokens[..index], tree);
            let right = parse(&tokens[index..], tree);
            And(left, right)

        // find repetitions (*)
        } else if let Some(index) = lexer::find(tokens, Token::Star) {
            let child = parse(&tokens[..index], tree);
            RepeatStar(child)

        // find plus repetitions (+)
        } else if let Some(index) = lexer::find(tokens, Token::Plus) {
            let child = parse(&tokens[..index], tree);
            RepeatPlus(child)

        // find letters and parenthesized expressions
        } else if tokens.len() == 1 {
            match tokens.first().unwrap() {
                Token::Letter(chr) => Leaf(*chr),
                Token::Group(tokens) => return parse(tokens, tree),
                Token::AnyLetter => LeafCharClass(CharClass::AllLetter),
                Token::AnyDigit => LeafCharClass(CharClass::AllDigit),
                _ => panic!("Badly malformed regex") // shouldn't ever happen
            }
        
        // anything else must be a syntax error
        } else {
            eprintln!("Malformed regex: {:?}", tokens);
            std::process::exit(1);
        };

    tree.add(new_node)
}

impl AST {
    /// Add a node to the *top* of this tree
    pub fn add(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Get the root node (the last element of the node list)
    pub fn root(&self) -> &Node {
        &self.nodes.last().unwrap()
    }

    /// Get a particular node given its index
    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }
}
