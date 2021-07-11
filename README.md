# chip-aight

## Description

A Chip-8 Emulator written in Rust!

## Running

Clone by doing

`git clone https://github.com/Orenchon/chip-aight`

Run the program by doing

`cargo run`

### CLI Options

Running normally:

`cargo run rom_path`

Change the cpu frequency:

`cargo run rom_path --hertz <cycles_per_second>`

Chip-8 Load Write Quirks:

`cargo run rom-path --store-load-quirks`

Shift into Y register instead of X:

`cargo run rom-path --shift-y`

Experiment with these options if one of the roms doesn't work properly.

## Tests

Execute the test suite by doing:

`cargo test`