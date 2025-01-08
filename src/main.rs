use std::{fmt::Error, fs, io};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::Style, text::{Span, Spans}, widgets::{Block, Borders, Paragraph}, Frame, Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
#[derive(Clone, Copy,Debug)]
enum Mode {
    Mode,
    Edit,
    Error,
    Quit,
    Open,
    Save,
    Find,
    FindFurther,
}
#[derive(Debug)]
struct App<'a>{
    mode: Mode,
    input: &'a String,
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
    let mut input = String::new();
    let mut mode = Mode::Mode;
    let mut command = String::new();
    let mut edit_cursor:usize = 0;
    let mut line_name = String::from("Mode");
    let mut file_name = String::from("New File");
    let mut findings: Vec<usize> = Vec::new();
    let mut find_cursor: isize = -1;
   loop {
        let _ = terminal.draw(|f|render(f, App {mode: mode,input: &input,command: &command,line_name: &line_name,file_name: &file_name},&mut edit_cursor));
        if let Event::Key(key) = event::read().unwrap() {
            match mode {
                Mode::Mode => {
                    line_name = String::from("Mode");
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {mode = get_mode(&command,&mut line_name);command = String::new()},
                        KeyCode::Esc => command = String::new(),
                        _ => ()
                    }
                },
                Mode::Open => {
                    line_name = String::from("Open");
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {if let Ok(file) = open(&command){input = file;mode = Mode::Mode;line_name = String::from("Mode");file_name = String::from(command)}else {line_name = String::from("File not found")}command = String::new()},
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode")},
                        _ => ()
                    }
                },
                Mode::Save => {
                    line_name = String::from("Open");
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {save(&command,&mut file_name,&input);mode = Mode::Mode;line_name = String::from("Mode");file_name = String::from(command);command = String::new()},
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode")},
                        _ => ()
                    }
                },
                Mode::Find => {
                    line_name = String::from("Find");
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {findings = find(&command, &input);mode = Mode::FindFurther}
                        KeyCode::Esc => {command = String::new();mode = Mode::Mode;line_name = String::from("Mode")},
                        _ => ()
                    }
                },
                Mode::FindFurther => {
                    if find_cursor >= -1 {
                        match key.code {
                        KeyCode::Enter => {if find_cursor +1 < (findings.len()as isize) {find_cursor += 1;edit_cursor = findings[find_cursor as usize]} else {find_cursor = -1}},
                        KeyCode::Esc => {find_cursor = 0; findings = Vec::new();mode = Mode::Find; line_name = String::from("Find")}
                        _ => ()
                    }
                }
                }
                Mode::Edit => match key.code {
                    KeyCode::Char(x) => {if x.is_ascii() {input.insert(edit_cursor, x)/*input += &x.to_string()*/;edit_cursor += 1;}},
                    KeyCode::Backspace => {if edit_cursor != 0 {if edit_cursor == input.len() {let _ = input.pop();} else {let _ = input.remove(edit_cursor -1);edit_cursor -= 1;}}},
                    KeyCode::Delete => {if edit_cursor != input.len() {if edit_cursor + 1 == input.len() {let _ = input.pop();} else {let _ = input.remove(edit_cursor + 1);/*edit_cursor -= 1;*/}}},
                    KeyCode::Enter => {input.insert(edit_cursor, '\n')/*input += &"\n".to_string()*/;edit_cursor += 1;},
                    KeyCode::Esc => {mode = Mode::Mode;line_name = String::from("Mode")},
                    KeyCode::Left => {if edit_cursor != 0 {edit_cursor -= 1}},
                    KeyCode::Right => edit_cursor +=1,
                    KeyCode::Up => edit_cursor = line_up(&mut input,&mut edit_cursor),
                    KeyCode::Down => edit_cursor = line_down(&mut input,&mut edit_cursor),
                    //KeyCode::Up => edit_cursor,
                    _ => ()
                },
                Mode::Error => {mode = Mode::Mode},
                Mode::Quit => return Ok(())
            }
        }
    }
}
fn render(f:&mut  Frame<'_,CrosstermBackend<io::Stdout>>, app: App,edit_cursor:&mut usize) {
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
    let mut x:u16 = 0;
    let mut y:u16 = 0;
    let mut offset:u16 = 0;
    for i in 0..*edit_cursor {
        if i< app.input.len(){ 
        x += 1;
        if &app.input.chars().collect::<Vec<char>>()[i] == &'\n' {
            x = 0;
            if y +3 < chunks[0].height{
                y += 1;
            } else {
                offset += 1;
            }
        }  
    } else {
        *edit_cursor -= 1;
    }
    }
    let input = Paragraph::new(app.input.as_ref()).scroll((offset,0)).block(Block::default().borders(Borders::ALL).title(app.file_name.as_str()));
    let command = Paragraph::new(vec![Spans::from(Span::raw(app.command))]).style(Style::default()).block(Block::default().borders(Borders::ALL).title(app.line_name.as_str()));
    f.render_widget(input,chunks[0]);
    f.render_widget(command, chunks[1]);
    f.set_cursor(chunks[0].x + x + 1, y + chunks[0].y + 1);
}
fn get_mode(command: &String,line_name: &mut String) -> Mode {
   //println!("{:?}",command);
    return match command.as_str() {
        "e" | "edit" => Mode::Edit,
        "q" | "quit" => Mode::Quit,
        "o" | "open" => {*line_name = String::from("Open");Mode::Open},
        "s" | "save" => {*line_name = String::from("Save");Mode::Save},
        "f" | "find" => {*line_name = String::from("Find");Mode::Find},
        _ => {*line_name = String::from("Error - Mode not found");Mode::Error}
    };
}
fn open(command: &String) -> Result<String,std::io::Error>{
    fs::read_to_string(command)
 }
fn save(command: &String,file_name: &mut String,input:&String) {
    if command.len() == 0 && *file_name != String::from("New File"){
        let _ = fs::write(file_name, input);
    } else {
        let _ = fs::write(command, input);
        *file_name = command.clone();
    }
}
fn lines<'a>(input:&'a String) -> Vec<&'a str> {
    let mut lines:Vec<&str> = input.split("\n").collect();
    lines.push("");
    lines.push("");
    lines
}
fn line_up(input:&String,cursor: &mut usize) -> usize {
    if *(&(&input.chars().collect::<Vec<char>>())[..*cursor].contains(&'\n')) {
        let target_string = &(String::from_utf8(input.as_bytes()[..*cursor].to_vec())).unwrap_or(format!("{}",input));
        let mut lines = lines(target_string);
        lines.pop();
        lines.pop();
        if let Some(last) = lines.last() {
            if let Some(second_last) = lines.get(lines.len().wrapping_sub(2)) {
                let mut  new_cursor:usize = 0;
                for i in &lines[0..lines.len()-2] {
                    for _ in 0..i.len() {
                        new_cursor += 1;
                    }
                    new_cursor += 1;
                }
                if last.len() <= second_last.len() {
                    new_cursor += last.len();
                } else {
                    new_cursor += second_last.len();
                }
                return new_cursor;
            }
        }

    }
    *cursor
}
fn line_down(input:&String,cursor: &usize) -> usize {
    if *(&(&input.chars().collect::<Vec<char>>())[*cursor..].contains(&'\n')) {
        let mut current_line:usize = 0;
        for i in (0..*cursor).rev() {
            if input.chars().collect::<Vec<char>>()[i] != '\n'  {
                current_line += 1
            }else {
                break;
            }
        }
        let target_string = &(String::from_utf8(input.as_bytes()[*cursor..].to_vec())).unwrap_or(format!("{}",input));
        let mut lines = lines(target_string);
        lines.pop();
        lines.pop();
            if let Some(second) = lines.get(1) {
                let mut  new_cursor:usize = 0;
                for i in &lines[2..] {
                    for _ in 0..i.len() {
                        new_cursor += 1;
                    }
                    new_cursor += 1;
                }
                    new_cursor += second.len();
                return (input.len() - new_cursor) + current_line;
            }

    }
    *cursor
}
fn find(command: &String,input:&String) ->Vec<usize> {
    let mut pos = 0;
    let mut findings: Vec<usize> = Vec::new();
    loop {
        if let Some(finding) = input[pos..].find(command).map(|i| i+pos) {
            findings.push(finding);
            pos = finding + command.len()
        } else {
            break;
        }
    }
    findings
}