use std::{fmt::Error, fs, io, process::Command};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::Style, text::{Span, Spans}, widgets::{Block, Borders, Paragraph}, Frame, Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
#[derive(Clone, Copy,Debug,PartialEq)]
enum Mode {
    Mode,
    Edit,
    Error,
    Quit,
    ForceQuit,
    Open,
    Save,
    Find(usize,usize),
    Command
}
#[derive(Debug)]
struct App<'a>{
    mode: Mode,
    input: &'a Vec<String>,
    output: &'a String,
    command: &'a String,
    line_name: &'a String,
    file_name: &'a String,
}
fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let _ = run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
fn run(terminal:&mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(),Error>{
    let mut input: Vec<String> = vec![String::new()];
    let mut output: String = String::new();
    let mut vert_cursor:usize = 0;
    let mut mode = Mode::Mode;
    let mut command = String::new();
    let mut edit_cursor:usize = 0;
    let mut line_name = String::from("Mode");
    let mut file_name = String::from("New File");
    let mut saved: bool = true;
    let mut scroll_y:usize = 0;
    let mut scroll_x:usize = 0;
    loop {
        let _ = terminal.draw(|f|render(f, App {mode: mode,input: &input,command: &command,line_name: &line_name,file_name: &file_name,output: &output},&mut edit_cursor,&mut vert_cursor,&mut scroll_y,&mut scroll_x));
        if let Event::Key(key) = event::read().unwrap() {
            match mode {
                Mode::Mode => {
                    line_name = String::from("Mode");
                    match key.code {
                        KeyCode::Char(x) => {command += &x.to_string();},
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {mode = get_mode(&command,&mut line_name);command = String::new()},
                        KeyCode::Esc => command = String::new(),
                        _ => ()
                    }
                },
                Mode::Command => {
                    line_name = String::from("Command");
                    match key.code {
                        KeyCode::Char(x) => {command += &x.to_string();},
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {exc_command(&command,&mut output);command = String::new();},
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode");},
                        _ => ()
                    }
                },
                Mode::Open => {
                    line_name = String::from("Open");
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {if let Some(file) = open(&command){input = file;mode = Mode::Mode;line_name = String::from("Mode");file_name = String::from(command);saved = true}else {line_name = String::from("File not found")}command = String::new()},
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode")},
                        _ => ()
                    }
                },
                Mode::Save => {
                    line_name = String::from("Save");
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {save(&command,&mut file_name,&input,&mut saved);mode = Mode::Mode;line_name = String::from("Mode");command = String::new()},
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode")},
                        _ => ()
                    }
                },
                Mode::Find(x,y) => {
                    line_name = String::from("Find");
                    match key.code {
                        KeyCode::Char(x) => {command += &x.to_string();mode = Mode::Find(0, 0)},
                        KeyCode::Backspace => {let _ = command.pop();mode = Mode::Find(0, 0)},
                        KeyCode::Enter => {if let Some((x_cursor,y_cursor)) = find(&command, &input,x,y){vert_cursor = y_cursor;edit_cursor = x_cursor;mode = Mode::Find(x_cursor +1, y_cursor)} else if x == 0 && y == 0{line_name = String::from("Pattern not found")}else {mode = Mode::Find(0, 0)}}
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode")},
                        _ => ()
                    }
                },
                Mode::Edit => match key.code {
                    KeyCode::Char(x) => {if x.is_ascii() {input[vert_cursor].insert(edit_cursor, x)/*input += &x.to_string()*/;edit_cursor += 1;saved = false}},
                    KeyCode::Backspace => {if edit_cursor != 0 {if edit_cursor +1== input[vert_cursor].len() {let _ = input[vert_cursor].pop();} else {let _ = input[vert_cursor].remove(edit_cursor -1);edit_cursor -= 1;saved = false}}else if vert_cursor != 0{let rest = input.remove(vert_cursor);input[vert_cursor-1] += &rest;vert_cursor -= 1;edit_cursor = input[vert_cursor].len() - rest.len()} },
                    KeyCode::Delete => {if edit_cursor != input[vert_cursor].len() {if edit_cursor + 1 == input[vert_cursor].len() {let _ = input[vert_cursor].pop();}else {let _ = input[vert_cursor].remove(edit_cursor + 1);/*edit_cursor -= 1;*/}}else if vert_cursor +1 != input.len(){let rest = input.remove(vert_cursor +1); input[vert_cursor] += &rest}saved = false},
                    KeyCode::Enter => {let line = input[vert_cursor].split_off(edit_cursor);input.insert(vert_cursor+1,line)/*input += &"\n".to_string()*/;vert_cursor += 1; if input[vert_cursor].len() < edit_cursor +2 {edit_cursor = input[vert_cursor].len()}},
                    KeyCode::Esc => {mode = Mode::Mode;line_name = String::from("Mode")},
                    KeyCode::Left => {if edit_cursor != 0 {edit_cursor -= 1} else if vert_cursor != 0 {vert_cursor -=1;edit_cursor = input[vert_cursor].len()}},
                    KeyCode::Right => if edit_cursor +1 <= input[vert_cursor].len() {edit_cursor  +=1}else if vert_cursor +1 != input.len(){vert_cursor +=1;edit_cursor = 0},
                    KeyCode::Up => if vert_cursor != 0 {vert_cursor -= 1; if input[vert_cursor].len() < edit_cursor +2 {edit_cursor = input[vert_cursor].len()}},
                    KeyCode::Down => if vert_cursor +2 <= input.len() {vert_cursor += 1; if input[vert_cursor].len() < edit_cursor +2 {edit_cursor = input[vert_cursor].len()}},
                    _ => ()
                },
                Mode::Error => {mode = Mode::Mode;line_name = String::from("Mode")},
                Mode::Quit => if saved {return Ok(())} else {mode = Mode::Error;line_name = String::from("Unsaved changes use fq to quit anyway")},
                Mode::ForceQuit => return Ok(()),
            }
        }
    }
}
fn render(f:&mut  Frame<'_,CrosstermBackend<io::Stdout>>, app: App,edit_cursor:&mut usize,vert_cursor:&mut usize,scroll: &mut usize,scroll_x: &mut usize) {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints(
    match app.mode {
        Mode::Edit =>[Constraint::Percentage(100),Constraint::Percentage(0),].as_ref(),
        _ => [Constraint::Percentage(70),Constraint::Percentage(30),].as_ref()
    }
    )
    .split(f.size());
    if *vert_cursor  > (*scroll + (chunks[0].height as usize) -3) {
        *scroll += 1
    }else if *vert_cursor < *scroll  {
        *scroll -= 1
    }
    while *edit_cursor  > (*scroll_x + (chunks[0].width as usize) -3) {
        *scroll_x += 1
    }
    while *edit_cursor < *scroll_x  {
        *scroll_x -= 1
    }
    let mut text = String::new();
    if app.input.len() <= (*scroll + (chunks[0].height as usize)+2) {
        for line in &app.input[*scroll..] {
            text += &line[*scroll_x..];
            text += "\n"
        }
    } else {
        for line in &app.input[*scroll..(*scroll + (chunks[0].height as usize) -1)] {
            text += &line[*scroll_x..];
            text += "\n"
        }
    }
    let output = Paragraph::new(app.output.to_string()).block(Block::default().borders(Borders::ALL).title(app.file_name.as_str()));
    let input = Paragraph::new(text.as_ref()).block(Block::default().borders(Borders::ALL).title(app.file_name.as_str()));
    let command = Paragraph::new(vec![Spans::from(Span::raw(app.command))]).style(Style::default()).block(Block::default().borders(Borders::ALL).title(app.line_name.as_str()));
    if app.mode == Mode::Command {
        f.render_widget(output,chunks[0]);    
        f.set_cursor(chunks[0].x + 1, chunks[0].y + 1);
    } else {
        f.render_widget(input,chunks[0]);   
        f.set_cursor(chunks[0].x + (*edit_cursor as u16) + 1 - (*scroll_x as u16), (*vert_cursor as u16) + chunks[0].y + 1 - (*scroll as u16));
    }
    f.render_widget(command, chunks[1]);
}
fn get_mode(command: &String,line_name: &mut String) -> Mode {
   //println!("{:?}",command);
    return match command.as_str() {
        "e" | "edit" => Mode::Edit,
        "q" | "quit" => Mode::Quit,
        "o" | "open" => {*line_name = String::from("Open");Mode::Open},
        "s" | "save" => {*line_name = String::from("Save");Mode::Save},
        "c" | "command" => {*line_name = String::from("Command");Mode::Command},
        "f" | "find" => {*line_name = String::from("Find");Mode::Find(0,0)},
        "fq" | "force quit" => {*line_name = String::from("Force Quit");Mode::ForceQuit},
        _ => {*line_name = String::from("Error - Mode not found");Mode::Error}
    };
}
fn open(command: &String) -> Option<Vec<String>>{
    let file_option =fs::read_to_string(command);
    if let Ok(file) = file_option {
        let split:Vec<&str> = file.split("\n").collect();
        let s: Vec<String> = split.iter().map(|f|f.to_string()).collect();
        return Some(s);
    }
    None
 }
fn save(command: &String,file_name: &mut String,input:&Vec<String>,saved:&mut bool) {
    let mut text =String::new();
    for line in input {
        text += line;
        text += "\n"
    }
    if command.len() == 0 && *file_name != String::from("New File"){
        if let Ok(_) = fs::write(file_name, text) {
            *saved = true;
        }
    } else {
        if let Ok(_) = fs::write(command, text) {
            *saved = true;
        }
        *file_name = String::from(command);//command.clone();
    }
}
fn find(command: &String,input:&Vec<String>,x:usize,y:usize) ->Option<(usize,usize)> {
    let mut x_used = false;
    for (i,line) in (&input[y..]).iter().enumerate() {
        if !x_used {
        if let Some(finding) = &line[x..].find(command) {
            return Some((*finding+x,i +y));
        }
        }else {
        if let Some(finding) = line.find(command) {
            return Some((finding,i+y));
        }
        }   
        x_used = true;
    }
    None
}
fn exc_command(command: &String,output:&mut String) {
    let mut exc_command = Command::new("bash");
    exc_command.arg("-c").arg(command);
    let mut result = String::new();
    if let Ok(output) = exc_command.output(){
        if let Ok(stdout) = String::from_utf8(output.stdout){
            result += &stdout;
        }
        if let Ok(stdout) = String::from_utf8(output.stderr){
            result += &stdout;
        }
    }
    *output = result
}