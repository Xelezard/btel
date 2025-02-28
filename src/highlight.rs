use regex::Regex;
use tui::{style::{Modifier, Style}, text::{Span, Spans,Text}};
use tui::style::Color;
const  BRACKET_COLOR: [Color;3] = [Color::Yellow,Color::Rgb(255, 123, 207),Color::Rgb(73, 22, 255)];
pub fn rust_highlight(text: &String) -> Text<'static> {
    let mut spans: Vec<Span> = text.chars().map(|c|Span::styled(c.to_string(), Style::default())).collect();
    color_brackets(&mut spans,"{","}");
    color(&mut spans, vec![(Regex::new(r"(fn |pub |let [^=]+|use |impl |mod |static [^=]+|const [^=]+)").unwrap(),Style::default().fg(Color::Rgb(0, 0, 200))),(Regex::new(r"(loop |for |in |while |return |match |mut |if )").unwrap(),Style::default().fg(Color::Rgb(173, 12, 170))),(Regex::new(r"([\w\-_!]+\(|\(|\))").unwrap(),Style::default().fg(Color::Rgb(167, 125, 0))),(Regex::new(r"([^ ]+::|[A-Z][a-z]+)").unwrap(),Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Yellow))], text);
    color_brackets(&mut spans, "(", ")");
    color_brackets(&mut spans, "[", "]");
    color_brackets(&mut spans, "<", ">");
    color(&mut spans, vec![(Regex::new(r"(#(!)*\[[^\]]+\]|!|\?|;|&|(-|\+|\*|/)*=(>|<|=)*| >|< |\+ |\- |\*| / |->)").unwrap(),Style::default().fg(Color::Rgb(156, 1, 10))),(Regex::new(r"\d").unwrap(),Style::default().fg(Color::Rgb(223, 123, 123))),(Regex::new("[a-z]*\"[^(\\*\")]*\"").unwrap(),Style::default().fg(Color::Rgb(2, 80, 0))),(Regex::new(r"(//[^\n]+|/\*[^(*/)]+\*/)").unwrap(),Style::default().add_modifier(Modifier::DIM).fg(Color::DarkGray))], text);
    text_from_spans(spans)
}
pub fn json_highlight(text: &String) -> Text<'_>{
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