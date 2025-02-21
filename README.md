# Btel Text Editor 
> **B**ad **T**ext **E**ditor **L**ol

A simple text editor hobby project written in rust

## Features
- Opening and Saving files
- Scrolling
- Searching for text
- Executing shell commands
- Plugin support

## Install
To install run:
```shell
cargo install --git https://github.com/xelezard/btel.git
```
## Usage
When opening Btel you see two fields the "New File" and the "Command" block.

Currently, you are in "Command" mode.

Type a command into "Command" field to execute it. (Command explanation below)

Use the "edit" command to go into edit mode.

### Commands
Note: commands used only to switch modes aren't listed here

Note: each Command may have a shorter version

#### Open
"open" or "o"

Used to open a file replacing the current one

> open file.txt

#### Save 
"save" or "s"

Used to save the file

If no file was supplied and the file name is known, the file will be saved to the same location

> save file.txt

or if the file has previously been saved

> save

#### Command
"command" or "c"

Used to execute a shell command **temporarily** replacing the text field

> command echo Hello World

#### Help
"help" or "h"

Display the help message again

> help

### Modes
Each mode has a command to switch to the corresponding mode

Press esc to go back to command mode

> Mode Name -- Command
> 
> Mode usage

#### Edit mode -- 'e' or 'edit' 
In this mode you can write to the file.

#### Quit mode -- 'q' or 'quit'
Press any key to exit Btel if all changes are saved.

#### Force Quit mode -- 'fq' or 'force quit'
Press any key to exit Btel and not save anything.

#### Find mode -- 'f' or 'find'
Enter your pattern and hit enter to move the cursor to the next finding.

#### Command mode
Enter a command to execute it

## Plugins
Plugins are external commands configured in the 'command.txt' file located at
~/.btel/command.txt

A test plugin is located at [test_plugin/](test_plugin/)

Note: currently there is no way to automatically generate the 'command.txt'
so it must be configured manually
### command.txt
Each line corresponds to one command and is constructed like this

> command1 command2 path/of_executable

Note: a command can have infinite command versions

> cmd1 cmd2 cmd3 cmd4 cmd5 ... path/to/executable

### Writing plugins
To write a plugin first of all import btel

```bash
cargo add --git https://github.com/Xelezard/btel
```

A plugin must allways be built up like this:
```rust
use btel::{get_btel_vars,set_btel_vars};
fn main() {
    let args = std::env::args().collect();
    let mut vars = get_btel_vars(args);
    do_smth();
    set_btel_vars(vars);
}
```
otherwise it won't work