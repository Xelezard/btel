use regex::Regex;
use tui::{style::{Modifier, Style}, text::{Span, Spans,Text}};
use tui::style::Color;
pub fn rust_highlight(text: &String) -> Text<'static> {
    let mut spans: Vec<Span> = text.chars().map(|c|Span::styled(c.to_string(), Style::default())).collect();
    color(&mut spans, vec![(Regex::new(r"([\w\-_!]+\([^\)]*\)|\(|\))").unwrap(),Style::default().fg(Color::Rgb(167, 125, 0))),(Regex::new(r"(loop |for |in |while |return |match )").unwrap(),Style::default().fg(Color::Rgb(73, 22, 70))),(Regex::new(r"(?m)^[\t ]*(fn|pub |let (mut )*[^=]+|use|impl|mod|static [^=]+|const [^=]+|if (let )*)").unwrap(),Style::default().fg(Color::Rgb(0, 0, 200))),(Regex::new(r"(//[^\n]+|/\*[^(*/)]+\*/)").unwrap(),Style::default().fg(Color::DarkGray)),(Regex::new(r"(#\[[^\]]*\]|!|\?|;|(-|\+|\*|/)*=(>|<|=)*| >|< |\+ |\- |\*| / |->)").unwrap(),Style::default().fg(Color::Red)),(Regex::new(r"[A-Z][a-z]+*").unwrap(),Style::default().add_modifier(Modifier::UNDERLINED).fg(Color::Yellow)),(Regex::new("\"[^\"]+\"").unwrap(),Style::default().fg(Color::Green))], text);
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