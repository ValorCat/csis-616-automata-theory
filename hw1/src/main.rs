fn main() {
    // collect commandline args
    let args: Vec<String> = std::env::args().collect();
    
    // exit if wrong amount provided
    if args.len() != 2 {
        println!("Usage: {} q1,q2,q3,...", args[0]);
        return;
    }
    
    // split nodes into vector
    let nodes: Vec<&str> = args[1].split(',').collect();

    println!("{}", build_graph_def(nodes));
}

fn build_graph_def(nodes: Vec<&str>) -> String {
    // combine each pair of nodes into Graphviz's "a -> b" format
    let transitions: String = nodes.windows(2)
        .map(|pair| format!("{} -> {};\n    ", pair[0], pair[1]))
        .collect();

    // return the graph definition
    format!(
"digraph {{
    rankdir=LR;
    node [shape=point]; start;
    node [shape=doublecircle]; {end_node};
    node [shape=circle];
    start -> {start_node};
    {transitions}
}}",
        start_node=nodes[0],
        end_node=nodes[nodes.len() - 1],
        transitions=transitions.trim_end())
}

#[test]
fn test_graph() {
    let expected =
"digraph {
    rankdir=LR;
    node [shape=point]; start;
    node [shape=doublecircle]; d;
    node [shape=circle];
    start -> a;
    a -> b;
    b -> c;
    c -> d;
}";
    assert_eq!(build_graph_def(vec!["a", "b", "c", "d"]), expected);
}