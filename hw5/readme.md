## Build Instructions
Source files are in `/src`. Using Cargo, you can build and run with:

`$ cargo run --release <filename>` 

## YAML Format

See `sample.yaml` for a working example. This format represents a nondeterministic push-down automaton.

```
alphabet: [list of strings]
stack_alphabet: [list of strings]
start: int
accept: [list of ints]
transitions:
  - - [input, pop, push, goto] # state 1, transition 1
    - [input, pop, push, goto] # state 1, transition 2
    - [input, pop, push, goto] # state 1, transition 3

  - - [input, pop, push, goto] # state 2, transition 1
    - ...
```

State numbers begin at 1. An empty string represents epsilon. `input`, `pop`, and `push` are all strings.

## Outstanding Issues
The GraphViz graph displays a transition label over the arrow pointing to the start node, however the transition is all epsilons (it doesn't read from the input or stack) so it doesn't affect the model's functionality.

## Testing Instructions
You can test with:

`$ cargo test`
