use std::{fmt::Error, io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::Style, text::{Span, Spans}, widgets::{Block, Borders, Paragraph, Widget}, Frame, Terminal
};
use crossterm::{
    cursor, event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
#[derive(Clone, Copy,Debug)]
enum Mode {
    Mode,
    Edit,
    Error,
    Quit
}
#[derive(Debug)]
struct App<'a>{
    mode: Mode,
    input: &'a String,
    command: &'a String
}
fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run(&mut terminal);

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
    loop {
        terminal.draw(|f|render(f, App {mode: mode,input: &input,command: &command},&mut edit_cursor));
        if let Event::Key(key) = event::read().unwrap() {
            match mode {
                Mode::Mode => {
                    match key.code {
                        KeyCode::Char(x) => command += &x.to_string(),
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {mode = get_mode(&command);command = String::new()},
                        _ => ()
                    }
                },
                Mode::Edit => match key.code {
                    KeyCode::Char(x) => {input.insert(edit_cursor, x)/*input += &x.to_string()*/;edit_cursor += 1;},
                    KeyCode::Backspace => {let x = input.remove(edit_cursor);if x == '\n' {edit_cursor;};edit_cursor -= 1;},
                    KeyCode::Enter => {input.insert(edit_cursor, '\n')/*input += &"\n".to_string()*/;edit_cursor;edit_cursor += 1;},
                    KeyCode::Esc => {mode = Mode::Mode},
                    KeyCode::Left => edit_cursor -= 1,
                    KeyCode::Right => edit_cursor +=1,
                    //KeyCode::Up => edit_cursor,
                    _ => ()
                },
                Mode::Error => {command = String::from("No mode found with this name");mode = Mode::Mode},
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
    let offset: u16;
    let lines = lines(app.input);
    if lines.len() > chunks[0].height as usize {
        offset = lines.len() as u16 - chunks[0].height;
    } else {
        offset = 0
    }
    let input = Paragraph::new(app.input.as_ref()).scroll((offset,0)).block(Block::default().borders(Borders::ALL));
    let command = Paragraph::new(vec![Spans::from(Span::raw(app.command))]).style(Style::default()).block(Block::default().borders(Borders::ALL).title("Mode"));
    f.render_widget(input,chunks[0]);
    f.render_widget(command, chunks[1]);
    let mut x:u16 = 0;
    let mut y:u16 = 0;
    for i in 0..*edit_cursor {
        if i< app.input.len(){ 
        x += 1;
        if &app.input.chars().collect::<Vec<char>>()[i] == &'\n' {
            x = 0;
            y += 1;
        }  
    } else {
        *edit_cursor -= 1;
    }
    }
    f.set_cursor(chunks[0].x + x + 1, y + chunks[0].y + 1);
}
fn get_mode(command: &String) -> Mode {
   //println!("{:?}",command);
    return match command.as_str() {
        "e" | "edit" => Mode::Edit,
        "q" | "quit" => Mode::Quit,
        _ => Mode::Error
    };
}
fn lines<'a>(input:&'a String) -> Vec<&'a str> {
    let mut lines:Vec<&str> = input.split("\n").collect();
    lines.push("");
    lines.push("");
    lines
}