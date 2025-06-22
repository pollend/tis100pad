tis100-rs
---------

A simulator for the fictional computer depicted in the video game "TIS-100",
written in rust, for fun.

# Current Status

I have implemented support for puzzles 00150 through 51781.

# Usage

Find your save files. On Windows, they should be located at
`C:\Users\<you>\Documents\My Games\TIS-100\<random>\save\` and each file is named after
the puzzle it is for.

Run the program using `cargo run <savefile>`. It'll detect the puzzle name based on the filename.
If it gets it wrong (you're using some other files), use `-p <number>` to override it.

You can use `-v[vvv]` to turn on logging. Additional `v`s increase verbosity, up to 4. Also you
can pass `-d` as a synonym for `-vvvv`.
