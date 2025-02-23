use std::{fmt::Error, fs::{self, File}, io::{self, Write}, process::Command};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, text::{Span, Spans, Text}, widgets::{Block, Borders, Paragraph}, Frame, Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use btel::*;
mod highlight;
const HELP_MESSAGE: &str = "Welcome to Btel!\n\nMost needed commands:\n\"e\" - switch to edit mode,\n\"q\" - quit if everything is saved\n\nfor more information please read the part in the README.md\nhttps://github.com/Xelezard/btel";
#[cfg(target_os = "windows")]
compile_error!("This crate does not support Windows.");
fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    if !std::path::Path::new(&format!("{}/.btel",env!("HOME"))).exists() {
        let _ = std::fs::create_dir(format!("{}/.btel",env!("HOME")));
        let mut extern_modes = File::create_new(format!("{}/.btel/command.txt",env!("HOME")))?;
        extern_modes.write_all(b"")?;
    }
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
    let args: Vec<String> = std::env::args().collect();
    let mut input: Vec<String> = vec![String::new()];
    let mut output: String = String::new();
    let mut vert_cursor:usize = 0;
    let mut mode = Mode::Command;
    let mut command = String::new();
    let mut edit_cursor:usize = 0;
    let mut line_name = String::from("Command");
    let mut file_name = String::from("New File");
    let mut saved: bool = true;
    let mut scroll_y:usize = 0;
    let mut scroll_x:usize = 0;
    let mut display = Display::Help;
    let commands = fs::read_to_string(format!("{}/.btel/command.txt",env!("HOME"))).expect("command.txt not found at ~/.btel/command.txt");
    let commands: Vec<Extern> = load_commands(commands);
    if args.len() == 2 {display = Display::Input;exc_command(&mut format!("o {}", args[1]),&mut output,&mut mode,&mut display,&mut input,&mut saved,&mut file_name,&mut line_name,&commands,&mut edit_cursor,&mut vert_cursor,&mut scroll_x,&mut scroll_y)}
    loop {
        let _ = terminal.draw(|f|render(f, App {mode: mode,input: &input,command: &command,line_name: &line_name,file_name: &file_name,output: &output,display: &display},&mut edit_cursor,&mut vert_cursor,&mut scroll_y,&mut scroll_x));
        if let Event::Key(key) = event::read().unwrap() {
            match mode {
                Mode::Command => {
                    line_name = String::from("Command");
                    display = Display::Input;
                    match key.code {
                        KeyCode::Char(x) => {command += &x.to_string();},
                        KeyCode::Backspace => {let _ = command.pop();},
                        KeyCode::Enter => {exc_command(&mut command,&mut output,&mut mode,&mut display,&mut input,&mut saved,&mut file_name,&mut line_name,&commands,&mut edit_cursor,&mut vert_cursor,&mut scroll_x,&mut scroll_y);command = String::new();},
                        KeyCode::Esc => {command = String::new();},
                        _ => ()
                    }
                },
                Mode::Find(x,y) => {
                    line_name = String::from("Find");
                    match key.code {
                        KeyCode::Char(x) => {command += &x.to_string();mode = Mode::Find(0, 0)},
                        KeyCode::Backspace => {let _ = command.pop();mode = Mode::Find(0, 0)},
                        KeyCode::Enter => {if let Some((x_cursor,y_cursor)) = find(&command, &input,x,y){vert_cursor = y_cursor;edit_cursor = x_cursor;mode = Mode::Find(x_cursor +1, y_cursor)} else if x == 0 && y == 0{line_name = String::from("Pattern not found")}else {mode = Mode::Find(0, 0)}}
                        KeyCode::Esc => {command = String::new();mode = Mode::Command;line_name = String::from("Command")},
                        _ => ()
                    }
                },
                Mode::Edit => match key.code {
                    KeyCode::Char(x) => {if x.is_ascii() {input[vert_cursor].insert(edit_cursor, x)/*input += &x.to_string()*/;edit_cursor += 1;saved = false}},
                    KeyCode::Backspace => {if edit_cursor != 0 {if edit_cursor == input[vert_cursor].len() {let _ = input[vert_cursor].pop();edit_cursor -= 1} else {let _ = input[vert_cursor].remove(edit_cursor -1);edit_cursor -= 1;saved = false}}else if vert_cursor != 0{let rest = input.remove(vert_cursor);input[vert_cursor-1] += &rest;vert_cursor -= 1;edit_cursor = input[vert_cursor].len() - rest.len()} },
                    KeyCode::Delete => {if edit_cursor != input[vert_cursor].len() {if edit_cursor + 1 == input[vert_cursor].len() {let _ = input[vert_cursor].pop();}else {let _ = input[vert_cursor].remove(edit_cursor + 1);/*edit_cursor -= 1;*/}}else if vert_cursor +1 != input.len(){let rest = input.remove(vert_cursor +1); input[vert_cursor] += &rest}saved = false},
                    KeyCode::Enter => {let line = input[vert_cursor].split_off(edit_cursor);input.insert(vert_cursor+1,line)/*input += &"\n".to_string()*/;vert_cursor += 1;edit_cursor = 0;},
                    KeyCode::Esc => {mode = Mode::Command;line_name = String::from("Command")},
                    KeyCode::Left => {if edit_cursor != 0 {edit_cursor -= 1} else if vert_cursor != 0 {vert_cursor -=1;edit_cursor = input[vert_cursor].len()}},
                    KeyCode::Right => if edit_cursor +1 <= input[vert_cursor].len() {edit_cursor  +=1}else if vert_cursor +1 != input.len(){vert_cursor +=1;edit_cursor = 0},
                    KeyCode::Up => if vert_cursor != 0 {vert_cursor -= 1; if input[vert_cursor].len() < edit_cursor +2 {edit_cursor = input[vert_cursor].len()}},
                    KeyCode::Down => if vert_cursor +2 <= input.len() {vert_cursor += 1; if input[vert_cursor].len() < edit_cursor +2 {edit_cursor = input[vert_cursor].len()}},
                    KeyCode::Tab => {for _ in 0..4 {input[vert_cursor].insert(edit_cursor, ' ');}edit_cursor += 4;saved = false}
                    _ => ()
                },
                Mode::Quit => if saved {return Ok(())} else {mode = Mode::Command;line_name = String::from("Unsaved changes use fq to quit anyway")},
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
    while *vert_cursor  > (*scroll + (chunks[0].height as usize) -3) {
        *scroll += 1
    }
    while *vert_cursor < *scroll  {
        *scroll -= 1
    }
    while *edit_cursor  > (*scroll_x + (chunks[0].width as usize) -3) {
        *scroll_x += 1
    }
    while *edit_cursor < *scroll_x  {
        *scroll_x -= 1
    }
    let text = app.input.join("\n");
    let mut  text_spans = match app.file_name {
        x if x.ends_with(".rs")=> highlight::rust_highlight(&text),
        x if x.ends_with(".json") => highlight::json_highlight(&text),
        _ => Text::from(text)
    };
    if app.input.len() <= (*scroll + (chunks[0].height as usize)+2) {
        let on_screnn = Text::from(text_spans.lines.drain(*scroll..).as_slice().iter().map(|l|l.clone()).collect::<Vec<Spans>>());
        text_spans = on_screnn
    } else {
        let on_screen = Text::from(text_spans.lines.drain(*scroll..(*scroll + (chunks[0].height as usize) -1)).as_slice().iter().map(|l|l.clone()).collect::<Vec<Spans>>());
        text_spans = on_screen
    }
    let command = Paragraph::new(Spans::from(vec![Span::raw(app.command)])).block(Block::default().borders(Borders::ALL).title(app.line_name.as_str()));
    if *app.display == Display::Output {
        let output = Paragraph::new(app.output.to_string()).block(Block::default().borders(Borders::ALL).title("Output"));
        f.render_widget(output,chunks[0]);    
        f.set_cursor(chunks[0].x + 1, chunks[0].y + 1);
    } else if *app.display == Display::Input {
        let input = Paragraph::new(text_spans).scroll((0,(*scroll_x as u16))).block(Block::default().borders(Borders::ALL).title(app.file_name.as_str()));
        f.render_widget(input,chunks[0]);   
        f.set_cursor(chunks[0].x + (*edit_cursor as u16) + 1 - (*scroll_x as u16), (*vert_cursor as u16) + chunks[0].y + 1 - (*scroll as u16));
    } else if *app.display == Display::Help {
        let help = Paragraph::new(HELP_MESSAGE).block(Block::default().borders(Borders::ALL).title(app.file_name.as_str()));
        f.render_widget(help,chunks[0]);    
        f.set_cursor(chunks[0].x + 1, chunks[0].y + 1);
    }
    f.render_widget(command, chunks[1]);
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
    let text =input.join("\n");
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
fn exc_command(command: &mut String,output:&mut String,mode: &mut Mode,display: &mut Display,input:&mut Vec<String>,saved: &mut bool,file_name: &mut String,line_name:&mut String,commands: &Vec<Extern>,edit_cursor:&mut usize,vert_cursor:&mut usize,scroll_x: &mut usize,scroll_y: &mut usize) {
    let mut pieces: Vec<&str> = command.split_ascii_whitespace().collect();
    if pieces.len() == 0 {
        return ();
    }
    let btel_command = match pieces[0] {
        "e" | "edit" => BtelCommand::Edit,
        "q" | "quit" => BtelCommand::Quit,
        "o" | "open" => {BtelCommand::Open},
        "s" | "save" => {BtelCommand::Save},
        "c" | "command" => {BtelCommand::Command},
        "f" | "find" => {BtelCommand::Find},
        "fq" | "force quit" => {BtelCommand::ForceQuit},
        "h" | "help" => {BtelCommand::Help},
        x if (commands.iter().map(|c|c.names.clone()).collect::<Vec<Vec<String>>>().concat()).contains(&x.to_string()) => {BtelCommand::Extern(String::from(x))}
        _ => {BtelCommand::Error}
    };
    match btel_command {
        BtelCommand::Command if pieces.len() > 1 => {
            *display = Display::Output;
            let mut exc_command = Command::new("bash");
            let mut shell_command = String::new();
            for piece in &pieces[1..] {
                shell_command += piece;
                shell_command += " "
            }
            exc_command.arg("-c").arg(shell_command);
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
        },
        BtelCommand::Open if pieces.len() == 2 => {
            if let Some(file) = open(&pieces[1].to_string()) {
                *input = file;
                *file_name = String::from(pieces[1]);
                *saved = true;
                *vert_cursor = 0;
                *edit_cursor = 0;
            } else {
                *line_name = String::from("File not found")
            }
        },
        BtelCommand::Save => {pieces.push("");save(&pieces[1].to_string(),file_name,&input,saved);}
        BtelCommand::Edit => {*mode = Mode::Edit},
        BtelCommand::ForceQuit => {*mode = Mode::ForceQuit},
        BtelCommand::Quit => {*mode = Mode::Quit},
        BtelCommand::Find => {*mode = Mode::Find(0, 0)}      
        BtelCommand::Help => {*display = Display::Help}
        BtelCommand::Extern(command) => {let mut path: &String = &String::new();for c in commands {if c.names.contains(&command) {path = &c.path};let mut plugin = Command::new(path);plugin.args(vec![&input.join("\n"),&output,&edit_cursor.to_string(),&vert_cursor.to_string(),&format!("{mode:?}"),line_name,file_name,&saved.to_string(),&scroll_x.to_string(),&scroll_y.to_string(),&format!("{display:?}"),&pieces[1..].join(" ")]);let out = String::from_utf8(plugin.output().expect("Plugin didn't work").stdout).expect("plugin didn't work");let mut new_args: Vec<String> = vec![String::new()];new_args.append(&mut out.split("\n\t\n").map(|l|l.to_string()).collect::<Vec<String>>());let new_vars: BtelVars = get_btel_vars(new_args);set_from_btel_vars(new_vars, input, output, edit_cursor, vert_cursor, mode, line_name, file_name, saved, scroll_x, scroll_y, display);};} 
        _ => *line_name = String::from("Command not found."),
    }
}
fn load_commands(modefile: String) -> Vec<Extern>{
    let mut result:Vec<Extern> = Vec::new();
    for modeline in modefile.split("\n") {
        let mut pieces: Vec<&str> = modeline.split_ascii_whitespace().collect();
        result.push(Extern {path: pieces.pop().expect("Commandfile Error").to_string(),names: pieces.iter().map(|p|p.to_string()).collect()});
    }
    result
}
fn set_from_btel_vars(vars: BtelVars,input:&mut Vec<String>,output:&mut String,edit_cursor:&mut usize,vert_cursor:&mut usize,mode: &mut Mode,line_name:&mut String,file_name: &mut String,saved: &mut bool,scroll_x: &mut usize,scroll_y: &mut usize,display: &mut Display) {
    *input = vars.input;
    *output = vars.output;
    *edit_cursor = vars.edit_cursor;
    *vert_cursor = vars.vert_cursor;
    *mode = vars.mode;
    *line_name = vars.line_name;
    *file_name = vars.file_name;
    *saved = vars.saved;
    *scroll_x = vars.scroll_x;
    *scroll_y = vars.scroll_y;
    *display = vars.display;
}