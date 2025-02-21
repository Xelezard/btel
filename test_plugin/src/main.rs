use btel::{get_btel_vars,set_btel_vars,Display};
use std::process::Command;
use std::fs;
fn main() {
    let args = std::env::args().collect();
    let mut vars = get_btel_vars(args);
    fs::write("/home/xelemander/grass_saas.txt",&format!("{:?}",vars.args));
    vars.display = Display::Output;
    let mut exc_command = Command::new("bash");
    exc_command.arg("-c").arg(vars.args.clone().unwrap().join(" "));
    let mut result = String::new();
    if let Ok(output) = exc_command.output(){
        if let Ok(stdout) = String::from_utf8(output.stdout){
            result += &stdout;
        }
        if let Ok(stdout) = String::from_utf8(output.stderr){
            result += &stdout;
        }
    }
    vars.output = result;
    set_btel_vars(vars);
}