
use btel::{CustomHighlight, Highlight, HighlightInstr, InclHighlight};
use regex::Regex;
use tui::{style::{Modifier, Style}, text::{Span, Spans,Text}, widgets::BorderType};
use tui::style::Color;
use tree::Root;
const  BRACKET_COLOR: [Color;3] = [Color::Yellow,Color::Rgb(255, 123, 207),Color::Rgb(73, 22, 255)];
fn rust_highlight<'a>(text: &String) -> Text<'a> {
    let mut spans: Vec<Span> = text.chars().map(|c|Span::styled(c.to_string(), Style::default())).collect();
    color_brackets(&mut spans, "{", "}");
    color_brackets(&mut spans, "(", ")");
    color_brackets(&mut spans, "[", "]");
    color(&mut spans, vec![    
        (Regex::new(r"u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64|bool|char|str").unwrap(),Style::default().fg(Color::Rgb(229, 192, 123))),
        (Regex::new(r"fn|let|mut|if|else|match|while|for|loop|in|return|break|continue|struct|enum|impl|trait|const|static|use|pub|crate|mod|super|self|Self|as|ref|type|where|unsafe|dyn|async|await|move|extern|box|union|default").unwrap(),Style::default().fg(Color::Rgb(198, 120, 221))),
        (Regex::new(r"'[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),Style::default().fg(Color::Rgb(255, 85, 85))),
        (Regex::new(r"[0-9][0-9_]*(\.[0-9_]+)?([eE][+-]?[0-9_]+)?").unwrap(),Style::default().fg(Color::Rgb(209, 154, 102))),
        (Regex::new(r"//.*").unwrap(),Style::default().fg(Color::Rgb(106, 153, 85)).add_modifier(Modifier::DIM)),
        (Regex::new(r"/\*.*?\*/").unwrap(),Style::default().fg(Color::Rgb(106, 153, 85)).add_modifier(Modifier::DIM)),
        (Regex::new(r"fn [a-zA-Z_][a-zA-Z0-9_]*").unwrap(),Style::default().fg(Color::Rgb(97, 175, 239))),
        (Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]*!").unwrap(),Style::default().fg(Color::Rgb(255, 203, 107))),
        (Regex::new(r"#\[[^\]]*\]").unwrap(),Style::default().fg(Color::Rgb(130, 170, 255))),
        (Regex::new(r"[A-Z][a-zA-Z0-9_]+").unwrap(),Style::default().fg(Color::Rgb(86, 156, 214))),
        (Regex::new("[a-z]*\"[^(\\*\")]*\"").unwrap(),Style::default().fg(Color::Rgb(152, 195, 121))),
    ], text);
    text_from_spans(spans)
}
fn json_highlight<'a>(text: &String) -> Text<'a>{
    let mut spans: Vec<Span> = text.chars().map(|c|Span::styled(c.to_string(), Style::default())).collect();
    color_brackets(&mut spans, "{", "}");
    color_brackets(&mut spans, "[", "]");
    color(&mut spans, vec![(Regex::new(r":").unwrap(),Style::default().fg(Color::Rgb(0, 0, 200))),(Regex::new(r"\d").unwrap(),Style::default().fg(Color::Rgb(223, 123, 123))),(Regex::new(r"(true|false)").unwrap(),Style::default().fg(Color::LightMagenta)),(Regex::new("[a-z]*\"[^(\\*\")]*\"").unwrap(),Style::default().fg(Color::Rgb(2, 80, 0)))], text);
    text_from_spans(spans)
}
fn text_from_spans(spans: Vec<Span>) -> Text {
    let mut text_lines: Vec<Spans> = vec![Spans::default()];
    for span in spans {
        if span.content == "\n" {
            text_lines.push(Spans::default());
        } else {
            text_lines.last_mut().unwrap().0.push(span);
        }
    }
    Text::from(text_lines)
}
pub fn highlight<'a>(text: &String,highlight_conf:&mut Vec<(String,Highlight)>,name: &String) -> Text<'a>{
    for highlight in highlight_conf {
        if name.ends_with(&highlight.0) {
            if let Highlight::Incl(incl_highlight) = &highlight.1 {
                match incl_highlight {
                    InclHighlight::Json => {return json_highlight(text);},
                    InclHighlight::Rust => {return rust_highlight(text);}
                }
            } else if let Highlight::Cstm(custom) = &highlight.1  {
                return custom_highlight(text, &custom.0);
            }
        }
    }
    Text::from(text.to_string())
}
fn custom_highlight<'a>(text: &String,instrs: &Vec<HighlightInstr>) -> Text<'a>{
    let mut spans: Vec<Span> = text.chars().map(|c|Span::styled(c.to_string(), Style::default().fg(Color::White))).collect();
    for instr in instrs {
        match instr {
            HighlightInstr::Brackets(start,end  ) => color_brackets(&mut spans, start, end),
            HighlightInstr::Regex(reg, col) => color(&mut spans, vec![(reg.to_owned(),Style::default().fg(*col))], text),
            HighlightInstr::None => ()
        }
    }
    text_from_spans(spans)
}
fn color(spans: &mut Vec<Span>,data: Vec<(Regex,Style)>,text: &String) {
    for (regex,style) in data {
        let findings = regex.find_iter(&text);
        for finding in findings {
            for i in finding.range() {
                spans[i].style = style
            }
        }
    }
}
fn color_brackets(spans: &mut Vec<Span>,start: &str,end: &str) {
    let mut i: usize = 0;
    let mut j: isize = 0;
    while i < spans.len() {
        if spans[i].content == start {
                j += 1;
                spans[i].style = Style::default().fg(BRACKET_COLOR[((j as usize) - 1) % 3]);
        }
        if spans[i].content == end {
            if j <= 0 {
                spans[i].style = Style::default().fg(Color::Rgb(255, 0, 0));
                j = 0;
            } else {
                spans[i].style = Style::default().fg(BRACKET_COLOR[((j as usize) - 1) % 3]);   
                j -= 1;   
            }
        }
        i += 1
    }
}
pub fn generate_hightlight(conf: &mut Root<String>) -> Vec<(String,Highlight)>{
    let mut result = Vec::new();
    if let Ok(highlight_conf) = conf.get_child("highlighting") {
        for root in highlight_conf.roots.iter_mut() {
            let mut sub_result: (String,Highlight) = (root.name.to_string(),Highlight::None);
            if let Some(val) = root.get_value() {
                sub_result.1 = match val.as_str() {
                    "json" => Highlight::Incl(btel::InclHighlight::Json),
                    "rust" => Highlight::Incl(btel::InclHighlight::Rust),
                    "custom" => generate_custom(root),
                    _ => Highlight::None
                }
            }
            result.push(sub_result);
        }
    }
    result
}
fn generate_custom(root: &mut Root<String>) -> Highlight {
    let mut rules: Vec<HighlightInstr> = Vec::new();
    for subroot in root.roots.iter_mut() {
        let mut rule = HighlightInstr::None;
        let re = Regex::new("^_(?<start>[^_])_(?<end>[^_])_").unwrap();
        if re.is_match(&subroot.get_value().unwrap()) {
            let caps = re.captures(&subroot.get_value().unwrap()).unwrap();
            rule = HighlightInstr::Brackets(caps["start"].to_string(), caps["end"].to_string())

        } else {
            let color = color_from_string(subroot.get_value().unwrap());
            let cst_re = Regex::new(&subroot.name);
            if let Ok(re) = cst_re {
                rule = HighlightInstr::Regex(re, color)
            }
        } 
        if let HighlightInstr::None = rule {} else {
            rules.push(rule);
        } 
    }
    Highlight::Cstm(CustomHighlight(rules))
}
pub fn color_from_string(color: &String) -> Color {
    match color.as_str() {
        "Black" => Color::Black,
        "DarkGray" => Color::DarkGray,
        "Blue" => Color::Blue,
        "Cyan" => Color::Cyan,
        "Green" => Color::Green,
        "LightBlue" => Color::LightBlue,
        "LightCyan" => Color::LightCyan,
        "LightGreen" => Color::LightGreen,
        "LightMagenta" => Color::LightMagenta,
        "LightRed" => Color::LightRed,
        "LightYellow" => Color::LightYellow,
        "Magenta" => Color::Magenta,
        "Red" => Color::Red,
        "Yellow" => Color::Yellow,
        "White" => Color::White,
        x if matches!(rgb_from_string(x),Some(_)) => rgb_from_string(x).unwrap(),
        _ => Color::White
    }
}
pub fn border_type_from_string(border_type: Result<&mut Root<String>,std::fmt::Error>) -> BorderType {
    if let Ok(b_type) = border_type  {
        return match b_type.get_value().unwrap().as_str() {
            "Double" => BorderType::Double,
            "Rounded" => BorderType::Rounded,
            "Thick" => BorderType::Thick,
            _ => BorderType::Plain
        }   
    };
    BorderType::Plain

}
fn rgb_from_string(string:&str) -> Option<Color>{
    let re = Regex::new(r"(\d+), (\d+), (\d+)").unwrap();
    if re.is_match(string) {
        let caps = re.captures(string)?;
        return Some(Color::Rgb(caps[1].parse().unwrap_or(3), caps[2].parse().unwrap_or(0), caps[3].parse().unwrap_or(0)));
    }
    None
}