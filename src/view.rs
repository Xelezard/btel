
use std::fs;

use aho_corasick::AhoCorasick;
use regex::Regex;
use tui::{style::{Modifier, Style}, text::{Span, Text}};

use crate::{highlight::text_from_spans, open_folder, textblock::TextBlock};

pub enum ViewRule {
    Url(String),
    File(String)
}
pub fn view_from_input(textbox: &mut TextBlock,folder: &mut Vec<String>){
    let res = textbox.input.clone();
    let mut info = Vec::new();
    let re = Regex::new(r"(http(s)?://.)?(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)").unwrap();
    let hay = &res.iter().map(|f|f.iter().collect::<String>()).collect::<Vec<String>>().join("\n");
    for file in folder {
        let matcher = AhoCorasick::new(vec![&file]).unwrap();
        for m in matcher.find_iter(hay) {
            for i in m.range() {
                info.push((i,ViewRule::File(file.to_string())));
            }
        }
        // while let Some(finding) = found {
        //     for i in finding..(finding +file.chars().count()) {
        //         info.push((i,ViewRule::File(file.to_string())));
        //     }
        //     found = (&hay.get((..finding+file.chars().count())).unwrap_or("")).find(&file.to_string())
        // }
    }
    for url in re.find_iter(hay) {
        for i in url.range() {
            if info.iter().all(|f: &(usize,ViewRule)|f.0 != i) {
                info.push((i,ViewRule::Url(url.as_str().to_string())));
            }
        }
    }
    textbox.view_info = info;
    textbox.view = res
}
pub fn view<'a>(text: String,info: &Vec<(usize,ViewRule)>) -> Text<'a> {
    let mut spans: Vec<Span> = text.chars().map(|c|Span::styled(c.to_string(), Style::default())).collect();
    for (i,rule) in info {
        if let Some(span) = spans.get_mut(*i) {
            span.style = style_from_viewrule(rule)
        }
    }
    text_from_spans(spans)
}
fn style_from_viewrule(rule: &ViewRule) -> Style {
    match rule {
        ViewRule::Url(_) => Style::default().fg(tui::style::Color::Blue).add_modifier(Modifier::UNDERLINED),
        ViewRule::File(_) => Style::default().fg(tui::style::Color::Green).add_modifier(Modifier::UNDERLINED)
    }
}
pub fn action(textbox: &mut TextBlock,file_name: &mut String,files_in_folder: &mut Vec<String>,opened_folder: &mut Option<String>,line_name: &mut String,folder_error: &mut Option<String>) {
    let mut cursor = 0;
    for i in textbox.view.get(..textbox.vert_cursor).unwrap_or(&[]) {
        cursor += i.len() + 1;
    }
    for _ in textbox.view[textbox.vert_cursor].get(..textbox.edit_cursor).unwrap_or(&[]) {
        cursor += 1;
    }
    for (i,rule) in &textbox.view_info {
        if *i == cursor {
            let _ = match rule {
                ViewRule::Url(url) => {let _ = webbrowser::open(&get_url(url.to_string())).is_ok();},
                ViewRule::File(target_file) => {
                    if textbox.saved {
                        if let Some(file) = crate::open(target_file) {
                            textbox.input = file.iter().map(|s| s.chars().collect()).collect();
                            *file_name = target_file.to_string();
                            textbox.saved = true;
                            textbox.vert_cursor = 0;
                            textbox.edit_cursor = 0;
                        } else if let  Some(folder) = open_folder(target_file) {
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
                }
            };
            break;
        }
    }
}
fn get_url(url: String) -> String {
    if url.contains("://") {
        return url;
    }
    "https://".to_string() + &url
}