use std::{env, fmt, fs};
use regex::Regex;
use tui::{style::Color, widgets::BorderType};
pub mod textblock;
pub mod view;
pub mod highlight;
#[derive(Clone, Copy,Debug,PartialEq)]
pub enum Mode {
    Edit,
    Quit,
    Find(usize,usize),
    Command,
    View
}
impl Mode {
    pub fn from_string(string: &String) -> Mode {
        match string.as_str() {
            "Edit"  => Mode::Edit,
            "Quit" => Mode::Quit,
            "Find" => Mode::Find(0, 0),
            "Command" => Mode::Command,
            "View" => Mode::View,
            _ => panic!("")
        }
    }
}
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",match self {
            Mode::Edit => "Edit",
            Mode::Command => "Command",
            Mode::Find(_, _) => "Find",
            Mode::Quit => "Quit",
            Mode::View => "View"
        })
    }
}
#[derive(Clone, Copy,Debug,PartialEq)]
pub enum Display {
    Input,
    Output,
    Help
}
impl Display {
    pub fn from_string(string: &String) -> Display {
        match string.as_str() {
            "Input\n" | "Input" => Display::Input,
            "Output\n" | "Output" => Display::Output,
            "Help\n" | "Help" => Display::Help,
            _ => panic!("\"{string}\"")
        }
    }
}
#[derive(Clone,Debug,PartialEq)]
pub enum BtelCommand {
    Edit,
    Error,
    Quit,
    ForceSave,
    Find,
    Command,
    Open,
    Save,
    Help,
    View,
    Extern(String)
}
pub struct App<'a>{
    pub mode: Mode,
    pub textbox: &'a textblock::TextBlock,
    pub output: &'a String,
    pub command: &'a String,
    pub line_name: &'a String,
    pub file_name: &'a String,
    pub display: &'a Display,
}
#[derive(Debug)]
pub struct BtelVars {
    pub input: Vec<String>,
    pub output: String,
    pub edit_cursor:  usize,
    pub vert_cursor: usize,
    pub mode: Mode,
    pub line_name: String,
    pub file_name: String,
    pub saved: bool,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub display: Display,
    pub args: Option<Vec<String>>
}
#[derive(Debug)]
pub struct Extern {
    pub names: Vec<String>,
    pub path: String
}
pub fn get_btel_vars(args:  Vec<String> ) -> BtelVars{
    if args.len() < 11 {
        panic!("Too few args: {:?}",args)
    }
    BtelVars {
        input: args[1].split('\n').map(|l|l.to_string()).collect(),
        output: args[2].clone(),
        edit_cursor: args[3].parse().unwrap(),
        vert_cursor: args[4].parse().unwrap(),
        mode: Mode::from_string(&args[5]),
        line_name: args[6].clone(),
        file_name: args[7].clone(),
        saved: match args[8].as_str() {"true" => true,_ => false},
        scroll_x: args[9].parse().unwrap(),
        scroll_y: args[10].parse().unwrap(),
        display: Display::from_string(&args[11]) ,
        args: get_args(args)
    }
}
#[derive(Debug)]
pub enum Highlight {
    Cstm(CustomHighlight),
    Incl(InclHighlight),
    None
}
#[derive(Clone, Copy)]
pub struct Theme {
    pub border_type: BorderType,
    pub target: Color,
    pub no_target: Color
}
#[derive(Debug)]
pub enum HighlightInstr {
    Regex(Regex,Color),
    Brackets(String,String),
    None
}
#[derive(Debug)]
pub struct CustomHighlight(pub Vec<HighlightInstr>);
#[derive(Debug)]
pub enum InclHighlight {
    Rust,
    Json
}
fn get_args(full_args: Vec<String>) -> Option<Vec<String>> {
    if full_args.len() > 12 {
        return Some(full_args[12].split(" ").map(|a|a.to_string()).collect());
    }
    None
}
pub fn set_btel_vars(vars: BtelVars) {
    println!("{}\n\t\n{}\n\t\n{}\n\t\n{}\n\t\n{:?}\n\t\n{}\n\t\n{}\n\t\n{}\n\t\n{}\n\t\n{}\n\t\n{:?}",vars.input.join("\n"),vars.output,vars.edit_cursor,vars.vert_cursor,vars.mode,vars.line_name,vars.file_name,vars.saved,vars.scroll_x,vars.scroll_y,vars.display)
}
#[cfg(target_os = "linux")]
pub fn btel_path() -> String{
    format!("{}/.btel",std::env::var("HOME").unwrap())
}
#[cfg(target_os = "windows")]
pub fn btel_path() -> String{
    format!("{}/.btel",std::env::var("AppData").unwrap())
}
pub fn open(command: &String) -> Option<Vec<String>>{
    let command = trim_home(command);
    let file_option = fs::read_to_string(command);
    if let Ok(file) = file_option {
        let file = file.replace("\t", "    ");
        let split:Vec<&str> = file.split("\n").collect();
        let s: Vec<String> = split.iter().map(|f|f.to_string()).collect();
        return Some(s);
    }
    None
 }
pub fn open_folder(command: &String) -> Option<String>{
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
        env::set_current_dir(&new_command).unwrap();
        return Some(new_command.to_owned());
    }
    None
}
#[cfg(target_os = "linux")]
fn trim_home(command: &String) -> String{
    command.replace("~", &format!("{}",std::env::var("HOME").unwrap_or("~".to_string())))
}
#[cfg(target_os = "windows")]
fn trim_home(command: &String) -> String {
    command.to_string()
}