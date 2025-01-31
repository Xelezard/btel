# Btel Text Editor 
> **B**ad **T**ext **E**ditor **L**ol

A simple text editor hobby project written in rust

## Features
- Opening and Saving files
- Scrolling
- Searching for text

## Install
To install run:
```shell
cargo install --git https://github.com/xelezard/btel.git
```
## Usage
When opening Btel you see two fields the "New File" and the "Mode" block.
Currently, you are in "Mode" mode.
This mode is used to switch modes.

### Modes

#### Mode mode
Enter a mode name and press enter to switch to that mode.

Press escape to clear the input field.

#### Edit mode
This mode can be accesed by typing "e" or "edit" into the "mode" field in Mode mode.

In this mode you can write to the file.

Press esc to go back to Mode mode.

#### Quit mode
This mode can be accesed by typing "q" or "quit" into the "mode" field in Mode mode.

Press any key to exit Btel if all changes are saved.

Press esc to go back to Mode mode.

#### Force Quit mode
This mode can be accesed by typing "fq" or "force quit" into the "mode" field in Mode mode.

Press any key to exit Btel and not save anything.

Press esc to go back to Mode mode.

#### Open mode
This mode can be accesed by typing "o" or "open" into the "mode" field in Mode mode.

Enter your file path inside of the "Open" field to open that file.

Press esc to go back to Mode mode.

#### Save mode
This mode can be accesed by typing "s" or "save" into the "mode" field in Mode mode.

To save type your desired file path into the "Save" field.

If your upper field doesn't read "New File" anymore and you enter no file path it will save to the previously entered file path.

Press esc to go back to Mode mode.

#### Find mode
This mode can be accesed by typing "f" or "find" into the "mode" field in Mode mode.

Enter your pattern and hit enter to move the cursor to the first finding.

Press esc to go back to Mode mode.

#### Command mode
This mode can be accesed by typing "f" or "find" into the "mode" field in Mode mode.

Enter a command to run it with the bash shell temporarily replacing the text field

Press esc to go back to Mode mode.

#### Error mode
This mode will load when an error occurs and it will display an error message on the lower text field.

Press any key to go back to Mode mode.