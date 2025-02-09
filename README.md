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
Type a mode abreviation into the mode field to enter that mode.
To go back to "Mode" mode use the esc key except the mode that your currently in states otherwise

### Modes
> Mode Name -- Mode abreviation
> 
> Mode usage

#### Mode mode
Enter a mode abreviation and press enter to switch to that mode.

Press escape to clear the input field.

#### Edit mode -- 'e' or 'edit' 
In this mode you can write to the file.

#### Quit mode -- 'q' or 'quit'
Press any key to exit Btel if all changes are saved.

#### Force Quit mode -- 'fq' or 'force quit'
Press any key to exit Btel and not save anything.

#### Open mode -- 'o' or 'open'
Enter your file path inside of the "Open" field to open that file.

#### Save mode -- 's' or 'save'
To save type your desired file path into the "Save" field.

If your upper field doesn't read "New File" anymore and you enter no file path it will save to the previously entered file path.

#### Find mode -- 'f' or 'find'
Enter your pattern and hit enter to move the cursor to the next finding.

#### Command mode -- 'c' or 'command'
Enter a command to run it with the bash shell temporarily replacing the text field

#### Error mode
This mode will load when an error occurs and it will display an error message on the lower text field.

Press any key to go back to Mode mode.
