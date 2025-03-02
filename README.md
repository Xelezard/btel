# Btel Text Editor 
> **B**ad **T**ext **E**ditor **L**ol

A simple hobby project text editor written in rust

![screenshot](screenshot.png)
## Features
- Opening and Saving files
- Scrolling
- Searching for text
- Executing shell commands
- Plugin support
- Syntax highlighting for 
  - rust
  - json 
  - any other language if you [configure](#configuration) it
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

If you open a directory, a side drawer will open displaying all the files

Now press right to be able to choose a file and accept with enter

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

## Configuration
The config file is located at:

linux -> ~/.btel/config.tr

windows -> %AppData%/btel/config.tr

### Plugins
Configure your plugins like this
```
commands -> alt text
| cmd1 or cmd2 -> path/to/plugin
| other_cmd1 or other_cmd2 -> path/to/other/plugin
```

Note: you can technically have infinite commands for one plugin
```
| cmd1 or cmd2 or cmd3 or cmd4 ...
```

### Syntax highlighting
You can configure which file extensions get highlighted with internal highlighting like this:
```
highlighting -> declare your syntax highlighting rules here
| .rs -> rust
| .json -> json
```

To configure your own highlighting use:
```
| .ext -> custom
```
You can highlight brackets like this:
```
|| brackets -> _start_end_
```
where start is the opening bracket and end the closing one

And regex matches like this:
```
regex1 -> Green
regex2 -> 12, 23, 4
```
Note: You can use either use any color from the 'tui::style::Color' enum or an rgb value that is split like this
> r, g, b

## Plugins
Plugins are external commands configured in the 'commands' section of the config.tr

A test plugin is located at [test_plugin/](test_plugin/)

Note: currently there is no way to automatically generate the 'command.txt'
so it must be configured manually
### Writing plugins
To write a plugin first of all import btel

```bash
cargo add --git https://github.com/Xelezard/btel btel
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