use std::fmt;

use regex::Regex;
use tui::{style::Color, widgets::BorderType};

#[derive(Clone, Copy,Debug,PartialEq)]
pub enum Mode {
    Edit,
    Quit,
    Find(usize,usize),
    Command
}
impl Mode {
    pub fn from_string(string: &String) -> Mode {
        match string.as_str() {
            "Edit"  => Mode::Edit,
            "Quit" => Mode::Quit,
            "Find" => Mode::Find(0, 0),
            "Command" => Mode::Command,
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
            Mode::Quit => "Quit"
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
    Extern(String)
}
#[derive(Debug)]
pub struct App<'a>{
    pub mode: Mode,
    pub input: &'a Vec<String>,
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