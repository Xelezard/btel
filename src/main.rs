use std::{env, fmt::Error, fs::{self, File}, io::{self, Write}, path::PathBuf, process::Command, vec};
use tui::{
    backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Span, Spans, Text}, widgets::{Block, Borders ,List, ListItem, ListState, Paragraph, Tabs}, Frame, Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use btel::*;
use tui::widgets::BorderType;
mod highlight;
use tree::Root;
const HELP_MESSAGE: &str = include_str!("../HELP.msg");
const DEFAULT_CONFIG_FILE: &str = include_str!("../config.tr");
fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut themes = vec![
        /*classic*/ Theme {border_type: BorderType::Plain,target: Color::White,no_target: Color::DarkGray},
        /*modern*/ Theme { border_type: BorderType::Rounded,target: Color::White,no_target: Color::Gray},
        /*clear*/ Theme { border_type: BorderType::Plain,target: Color::DarkGray,no_target: Color::Black},
        /*red-and-blue*/ Theme { border_type: BorderType::Rounded,target: Color::Red,no_target: Color::LightBlue},
        /*green*/ Theme { border_type: BorderType::Rounded,target: Color::Green,no_target: Color::LightGreen},
        ];
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut config_tree = btel_init()?;
    let mut highlight_config: Vec<(String,Highlight)> = highlight::generate_hightlight(&mut config_tree);
    let mut history = btel_history();
    let theme = theme(&mut config_tree, &mut themes);
    let _ = run(&mut terminal,&mut config_tree,&mut history,&mut highlight_config,&theme);
    let _ = fs::write(&format!("{}/history",btel_path()), history.iter().rev().take(1000).rev().map(|l|l.to_string()).collect::<Vec<String>>().join("\n"));
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
fn theme(config_tree: &mut Root<String>,themes: &mut Vec<Theme>) -> Theme{
    if let Ok(root) = config_tree.get_child("theme") {
        return match root.get_value().unwrap().as_str() {
            "modern" => themes.remove(1),
            "clear" => themes.remove(2),
            "red-and-blue" => themes.remove(3),
            "green" => themes.remove(4),
            "custom" => custom_theme(config_tree.get_child("theme").unwrap()),
            _ => themes.remove(0)
        };
    }
    themes.remove(0)
}
fn custom_theme(conf: &mut Root<String>) -> Theme{
    Theme {border_type: highlight::border_type_from_string(conf.get_child("border_type")),target: {if let Ok(child) = conf.get_child("target"){highlight::color_from_string(child.get_value().unwrap())} else {Color::White}}, no_target: {if let Ok(child) = conf.get_child("no_target"){highlight::color_from_string(child.get_value().unwrap())} else {Color::White}}}
}
fn btel_history() -> Vec<String> {
    let file = fs::read_to_string(&format!("{}/history",btel_path())).expect("Error loading history");
    file.split("\n").map(|l|l.to_string()).collect()
}
fn btel_init() -> Result<Root<String>,std::io::Error>{
    if !std::path::Path::new(&btel_path()).exists() {
        let _ = std::fs::create_dir(&btel_path());
        let _ = fs::write(&format!("{}/history",btel_path()), "");
        let mut extern_modes = File::create_new(format!("{}/config.tr",&btel_path()))?;
        extern_modes.write_all(DEFAULT_CONFIG_FILE.as_bytes())?;
    }
    Root::from_tree_file(&format!("{}/config.tr",btel_path()))
}
fn run(terminal:&mut Terminal<CrosstermBackend<io::Stdout>>,config_tree: &mut Root<String>,history: &mut Vec<String>,highlight_config: &mut Vec<(String,Highlight)>,them: &Theme) -> Result<(),Error>{
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
    let mut hist_cursor: usize = 0;
    let commands: Vec<Extern> = load_commands(config_tree)?;
    let mut files_in_folder: Vec<String> = Vec::new();
    let mut opened_folder: Option<String> = None;
    let mut targets_folder = false;
    let mut folder_cursor: usize = 0;
    let mut folder_error: Option<String> = None;
    if args.len() == 2 {display = Display::Input;exc_command(&mut format!("o {}", args[1]),&mut output,&mut mode,&mut display,&mut input,&mut saved,&mut file_name,&mut line_name,&commands,&mut edit_cursor,&mut vert_cursor,&mut scroll_x,&mut scroll_y,&mut opened_folder,&mut files_in_folder,&mut folder_error)}
    loop {
        let _ = terminal.draw(|f|render(f, App {mode: mode,input: &input,command: &command,line_name: &line_name,file_name: &file_name,output: &output,display: &display},&mut edit_cursor,&mut vert_cursor,&mut scroll_y,&mut scroll_x,&mut opened_folder,&files_in_folder,&targets_folder,&folder_cursor,&folder_error,highlight_config,them,config_tree));
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                folder_error = None;
                match mode {
                    Mode::Command => {
                        line_name = String::from("Command");
                        display = Display::Input;
                        match key.code {
                            KeyCode::Char(x) => {command += &x.to_string();},
                            KeyCode::Backspace => {let _ = command.pop();},
                            KeyCode::Enter if !targets_folder => {exc_command(&mut command,&mut output,&mut mode,&mut display,&mut input,&mut saved,&mut file_name,&mut line_name,&commands,&mut edit_cursor,&mut vert_cursor,&mut scroll_x,&mut scroll_y,&mut opened_folder,&mut files_in_folder,&mut folder_error);hist_cursor = 0;if command != String::new() {history.push(command)}command = String::new();},
                            KeyCode::Esc => {command = String::new();},
                            KeyCode::Up if !targets_folder => {if hist_cursor + 1 < history.len() {hist_cursor += 1;command = get_from_history(history, &hist_cursor)} else {command = String::new();hist_cursor = 0}},
                            KeyCode::Down if !targets_folder => {if hist_cursor > 0 {hist_cursor -= 1;command = get_from_history(history, &hist_cursor)}else {command = String::new()}},
                            KeyCode::Down => {if folder_cursor + 1 < files_in_folder.len(){folder_cursor += 1} else {folder_cursor = 0}},
                            KeyCode::Up => {if folder_cursor > 0 {folder_cursor -= 1} else {folder_cursor = files_in_folder.len() - 1}},
                            KeyCode::Right => {if let Some(_) = opened_folder {targets_folder = false}},
                            KeyCode::Left => {if let Some(_) = opened_folder {targets_folder = true}},
                            KeyCode::Enter => {exc_command(&mut format!("o {}/{}", opened_folder.as_ref().unwrap(),files_in_folder[folder_cursor]),&mut output,&mut mode,&mut display,&mut input,&mut saved,&mut file_name,&mut line_name,&commands,&mut edit_cursor,&mut vert_cursor,&mut scroll_x,&mut scroll_y,&mut opened_folder,&mut files_in_folder,&mut folder_error)},
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
                    Mode::Quit => if saved {return Ok(())} else {mode = Mode::Command;line_name = String::from("Unsaved changes use fs to force the saved state")},
                }
            }
        }
    }
}
fn render(f:&mut  Frame<'_,CrosstermBackend<io::Stdout>>, app: App,edit_cursor:&mut usize,vert_cursor:&mut usize,scroll: &mut usize,scroll_x: &mut usize,opened_folder: &mut Option<String>,files_in_folder: &Vec<String>,targets_folder: &bool,folder_cursor: &usize,folder_error: &Option<String>,highlight_config: &mut Vec<(String,Highlight)>,theme: &Theme,config_tree: &mut Root<String>) {
    let big_chunks = Layout::default()
    .direction(Direction::Horizontal)
    .margin(0)
    .constraints(
        {
        if let Some(_) = opened_folder {
            if app.mode != Mode::Command {
                [Constraint::Percentage(0),Constraint::Percentage(100)]
            }
            else if *targets_folder {
                [Constraint::Max(30),Constraint::Percentage(70)]
            } else {
                [Constraint::Max(15),Constraint::Percentage(70)]
            }
        } else {
            [Constraint::Percentage(0),Constraint::Percentage(100)]
        }
        }
    )
    .split(f.size());
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(0)
    .constraints(
    match app.mode {
        Mode::Edit => {[Constraint::Min(4),Constraint::Length(3),Constraint::Percentage(0)].as_ref()},
        _ => {[Constraint::Min(4),Constraint::Length(3),Constraint::Length(3),].as_ref()},
    }
    )
    .split(Rect {x: big_chunks[1].x,y: big_chunks[1].y,width: big_chunks[1].width,height: big_chunks[1].height});
    if let Some(folder) = opened_folder {
        let folder = List::new(files_in_folder.iter().map(|f|ListItem::new(f.to_string())).collect::<Vec<ListItem>>()).highlight_symbol("> ").highlight_style(Style::default().bg(tui::style::Color::Green)).block(Block::default().borders(Borders::ALL).border_type(theme.border_type).border_style(if *targets_folder {Style::default().fg(theme.target)} else {Style::default().fg(theme.no_target).add_modifier(Modifier::DIM)}).title(if let Some(error) = folder_error{error.to_string()}else{folder.to_string()}));
        let mut state = ListState::default();
        state.select(Some(*folder_cursor));
        f.render_stateful_widget(folder, big_chunks[0],&mut state);
    }
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
    let mut standard_stat = Root::new("stat-bar", String::from("standard"));
    let status_bar = gen_stat(&app,theme,vert_cursor,edit_cursor,config_tree.get_child("stat-bar").unwrap_or(&mut standard_stat));
    let mut  text_spans = highlight::highlight(&text, highlight_config, app.file_name);
    let input_block = Block::default().borders(Borders::ALL).border_type(theme.border_type).title(app.file_name.to_string()).border_style(match app.mode {Mode::Edit => Style::default().fg(theme.target), _ => Style::default().fg(theme.no_target).add_modifier(Modifier::DIM)});
    let command_block = Block::default().borders(Borders::ALL).border_type(theme.border_type).title(app.line_name.to_string()).border_style(match targets_folder {false => Style::default().fg(theme.target), true => Style::default().fg(theme.no_target).add_modifier(Modifier::DIM)});
    if app.input.len() <= (*scroll + (chunks[0].height as usize)+2) {
        let on_screnn = Text::from(text_spans.lines.drain(*scroll..).as_slice().iter().map(|l|l.clone()).collect::<Vec<Spans>>());
        text_spans = on_screnn
    } else {
        let on_screen = Text::from(text_spans.lines.drain(*scroll..(*scroll + (chunks[0].height as usize) -1)).as_slice().iter().map(|l|l.clone()).collect::<Vec<Spans>>());
        text_spans = on_screen
    }
    let command = Paragraph::new(Spans::from(vec![Span::raw(app.command)])).block(command_block);
    if *app.display == Display::Output {
        let output = Paragraph::new(app.output.to_string()).block(Block::default().borders(Borders::ALL).border_type(theme.border_type).title("Output").border_style(Style::default().fg(theme.target)));
        f.render_widget(output,chunks[0]);    
    } else if *app.display == Display::Input {
        let input = Paragraph::new(text_spans).scroll((0,(*scroll_x as u16))).block(input_block);
        f.render_widget(input,chunks[0]);   
    } else if *app.display == Display::Help {
        let help = Paragraph::new(HELP_MESSAGE).block(input_block);
        f.render_widget(help,chunks[0]);    
    }
    if app.mode == Mode::Edit {
        f.set_cursor(chunks[0].x + (*edit_cursor as u16) + 1 - (*scroll_x as u16), (*vert_cursor as u16) + chunks[0].y + 1 - (*scroll as u16));
    }   else {
        f.set_cursor(chunks[2].x +1 + (app.command.len() as u16), chunks[2].y + 1);
    }
    f.render_widget(command, chunks[2]);
    f.render_widget(status_bar, chunks[1]);
}
fn open(command: &String) -> Option<Vec<String>>{
    let command = trim_home(command);
    let file_option =fs::read_to_string(command);
    if let Ok(file) = file_option {
        if !file.is_ascii() {
            return None;
        }
        let file = file.replace("\t", "    ");
        let split:Vec<&str> = file.split("\n").collect();
        let s: Vec<String> = split.iter().map(|f|f.to_string()).collect();
        return Some(s);
    }
    None
 }
fn open_folder(command: &String) -> Option<String>{
    let command = trim_home(command);
    if std::path::Path::new(&command).is_dir() {
        let mut new_command = command.to_string();
        if !command.contains(std::env::current_dir().unwrap().to_str().unwrap()) && !command.starts_with("/") {
            new_command = format!("{}/{}",std::env::current_dir().unwrap().to_str().unwrap(),command.to_string());
        }
        if new_command.ends_with("..") {
            let new = new_command.split("/").collect::<Vec<&str>>().iter().rev().collect::<Vec<&&str>>().iter().skip(2).rev().map(|c|c.to_string()).collect::<Vec<String>>().join("/");
            new_command = new;
        } 
        if new_command.ends_with(".") {
            new_command.remove(new_command.len()-1);
            new_command.remove(new_command.len()-1);
        }
        if new_command == String::new() {
            return None;
        }
        return Some(new_command.to_owned());
    }
    None
}
#[cfg(target_os = "linux")]
fn trim_home(command: &String) -> String{
    command.replace("~", &format!("{}",env!("HOME")))
}
#[cfg(target_os = "windows")]
fn trim_home(command: &String) -> String {
    command.to_string()
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
fn exc_command(command: &mut String,output:&mut String,mode: &mut Mode,display: &mut Display,input:&mut Vec<String>,saved: &mut bool,file_name: &mut String,line_name:&mut String,commands: &Vec<Extern>,edit_cursor:&mut usize,vert_cursor:&mut usize,scroll_x: &mut usize,scroll_y: &mut usize,opened_folder: &mut Option<String>,files_in_folder: &mut Vec<String>,folder_error: &mut Option<String>) {
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
        "fs" | "force save" => {BtelCommand::ForceSave},
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
            if *saved {
                if let Some(file) = open(&pieces[1].to_string()) {
                    *input = file;
                    *file_name = String::from(pieces[1]);
                    *saved = true;
                    *vert_cursor = 0;
                    *edit_cursor = 0;
                } else if let  Some(folder) = open_folder(&pieces[1].to_string()) {
                    *files_in_folder = vec![String::from("..")];
                    *opened_folder = Some(folder.to_string());
                    for result in fs::read_dir(&folder).expect(&folder) {
                        if let Ok(file) = result {
                            files_in_folder.push(file.file_name().into_string().unwrap());
                        }
                    }
                } else {
                    *line_name = String::from("File not found");
                    *folder_error = Some(String::from("File not found"));
                }
            } else {
                *line_name = String::from("Unsaved Changes");
                *folder_error = Some(String::from("Unsaved Changes"));
            }
        },
        BtelCommand::Save => {pieces.push("");save(&pieces[1].to_string(),file_name,&input,saved);}
        BtelCommand::Edit => {*mode = Mode::Edit},
        BtelCommand::ForceSave => {*saved = true},
        BtelCommand::Quit => {*mode = Mode::Quit},
        BtelCommand::Find => {*mode = Mode::Find(0, 0)}      
        BtelCommand::Help => {*display = Display::Help}
        BtelCommand::Extern(command) => {let mut path: &String = &String::new();for c in commands {if c.names.contains(&command) {path = &c.path};let mut plugin = Command::new(path);plugin.args(vec![&input.join("\n"),&output,&edit_cursor.to_string(),&vert_cursor.to_string(),&format!("{mode:?}"),line_name,file_name,&saved.to_string(),&scroll_x.to_string(),&scroll_y.to_string(),&format!("{display:?}"),&pieces[1..].join(" ")]);let out = String::from_utf8(plugin.output().expect("Plugin didn't work").stdout).expect("plugin didn't work");let mut new_args: Vec<String> = vec![String::new()];new_args.append(&mut out.split("\n\t\n").map(|l|l.to_string()).collect::<Vec<String>>());let new_vars: BtelVars = get_btel_vars(new_args);set_from_btel_vars(new_vars, input, output, edit_cursor, vert_cursor, mode, line_name, file_name, saved, scroll_x, scroll_y, display);};} 
        _ => *line_name = String::from("Command not found."),
    }
}
fn load_commands(config_tree: &mut Root<String>) -> Result<Vec<Extern>,std::fmt::Error>{
    let mut result:Vec<Extern> = Vec::new();
    let mut test = String::new();
    for root in &config_tree.get_child("commands")?.roots{
        test += ":";
        let names: Vec<&str> = root.name.split(" or ").collect();
        if let tree::Val::Val(value) = &root.value {
            result.push(Extern {path: value.to_string(),names: names.iter().map(|n|n.to_string()).collect()});
        } else {
            return Err(std::fmt::Error);
        }
    }
    Ok(result)
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
fn get_from_history(history: &Vec<String>,hist_cursor:&usize) -> String {
    let mut history: Vec<String> = history.iter().rev().map(|l|l.to_string()).collect();
    history.reverse();
    history.push(String::new());
    history.reverse();
    history.remove(*hist_cursor)
}
fn gen_stat<'a>(app: &App,theme: &Theme,vert_cursor:&usize,edit_cursor:&usize,conf: &'a mut Root<String>) -> Tabs<'a> {
    let mut bar: Vec<Spans> = Vec::new();
    match conf.get_value().unwrap().as_str() {
        "standard" => bar.append(&mut vec![Spans::from(vec![Span::styled("Line: ", Style::default().fg(theme.no_target)),Span::styled(vert_cursor.to_string(), Style::default().fg(theme.target)),Span::styled(", Col: ", Style::default().fg(theme.no_target)),Span::styled(edit_cursor.to_string(), Style::default().fg(theme.target))]),Spans::from(vec![Span::styled("Mode: ", Style::default().fg(theme.no_target)),Span::styled(format!("{}",app.mode), Style::default().fg(theme.target))]),Spans::from(vec![Span::styled("Dir: ",Style::default().fg(theme.no_target)),Span::styled(env::current_dir().unwrap_or(PathBuf::new()).display().to_string(), Style::default().fg(theme.target))])]),
        "custom" => bar.append(&mut gen_custom_stat(conf,theme,vert_cursor,edit_cursor,app)),
        _ => (),
    }
    if bar.len() == 0 {
        bar.push(
            Spans::from(Span::styled("-",Style::default().fg(theme.no_target)))
        );
    }
    tui::widgets::Tabs::new(bar).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.no_target)))
}
fn gen_custom_stat(conf: &mut Root<String>,theme: &Theme,vert_cursor:&usize,edit_cursor:&usize,app: &App) -> Vec<Spans<'static>>{
    let mut res = Vec::new();
    for root in conf.roots.iter_mut() {
        match root.get_value().unwrap().as_str() {
            "pos" => res.push(Spans::from(vec![Span::styled("Line: ", Style::default().fg(theme.no_target)),Span::styled(vert_cursor.to_string(), Style::default().fg(theme.target)),Span::styled(", Col: ", Style::default().fg(theme.no_target)),Span::styled(edit_cursor.to_string(), Style::default().fg(theme.target))])),
            "mode" => res.push(Spans::from(vec![Span::styled("Mode: ", Style::default().fg(theme.no_target)),Span::styled(format!("{}",app.mode), Style::default().fg(theme.target))])),
            "dir" => res.push(Spans::from(vec![Span::styled("Dir: ",Style::default().fg(theme.no_target)),Span::styled(env::current_dir().unwrap_or(PathBuf::new()).display().to_string(), Style::default().fg(theme.target))])),
            _ => ()
        }
    }
    res
}