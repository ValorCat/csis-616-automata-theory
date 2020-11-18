## Build Instructions
Source files are in `/src`. Using Cargo, you can build and run with:

`$ cargo run --release <regex>` 

You must wrap the regex in quotes if it contains shell operators or contiguous spaces, e.g. `"a  b"` (two spaces back-to-back).

## Features
The following regular expression features are supported:
- Character Set: a-z, 0-9, space
- Concatenation and parentheses
- Operators: `|` (union), `*` (Kleene star), `+` (Kleene plus)
- Character Classes: `\w` (a-z), `\d` (0-9)

## Outstanding Issues
None known.

## Testing Instructions
You can test with:

`$ cargo test`
