use btel::{App, Display, Highlight, Mode, Theme};
use tree::Root;
use tui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Span, Spans, Text}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph}, Frame};

use crate::{gen_stat, highlight::highlight, HELP_MESSAGE};

pub fn render(f:&mut  Frame<'_,CrosstermBackend<std::io::Stdout>>, app: App,scroll: &mut usize,scroll_x: &mut usize,opened_folder: &mut Option<String>,files_in_folder: &Vec<String>,targets_folder: &bool,folder_cursor: &usize,folder_error: &Option<String>,highlight_config: &mut Vec<(String,Highlight)>,theme: &Theme,config_tree: &mut Root<String>,forced_save: &mut bool) {
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
    while app.textbox.vert_cursor  > (*scroll + (chunks[0].height as usize) -3) {
        *scroll += 1
    }
    while app.textbox.vert_cursor < *scroll  {
        *scroll -= 1
    }
    while app.textbox.edit_cursor  > (*scroll_x + (chunks[0].width as usize) -3) {
        *scroll_x += 1
    }
    while app.textbox.edit_cursor < *scroll_x  {
        *scroll_x -= 1
    }
    let text = app.textbox.input.iter().map(|v|v.iter().map(|c|c.to_string()).collect::<String>()).collect::<Vec<String>>();
    let text = text.join("\n");
    let mut standard_stat = Root::new("stat-bar", String::from("standard"));
    let status_bar = gen_stat(&app,theme,&app.textbox.vert_cursor,&app.textbox.edit_cursor,config_tree.get_child("stat-bar").unwrap_or(&mut standard_stat));
    let mut  text_spans = highlight(&text, highlight_config, app.file_name);
    let input_block = Block::default().borders(Borders::ALL).border_type(theme.border_type).title(Spans::from(vec![if  *forced_save{Span::styled("*", Style::default().fg(Color::Red).add_modifier(Modifier::CROSSED_OUT | Modifier::DIM))} else if app.textbox.saved {Span::raw("")} else {Span::styled("*",Style::default().fg(Color::Red))},Span::raw(app.file_name)])).border_style(match app.mode {Mode::Edit => Style::default().fg(theme.target), _ => Style::default().fg(theme.no_target).add_modifier(Modifier::DIM)});
    let command_block = Block::default().borders(Borders::ALL).border_type(theme.border_type).title(app.line_name.to_string()).border_style(match targets_folder {false => Style::default().fg(theme.target), true => Style::default().fg(theme.no_target).add_modifier(Modifier::DIM)});
    if app.textbox.input.len() <= (*scroll + (chunks[0].height as usize)+2) {
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
    if matches!(app.mode,Mode::Edit|Mode::Find(_,_)) {
        f.set_cursor(chunks[0].x + (app.textbox.edit_cursor as u16) + 1 - (*scroll_x as u16), (app.textbox.vert_cursor as u16) + chunks[0].y + 1 - (*scroll as u16));
    }   else {
        f.set_cursor(chunks[2].x +1 + (app.command.len() as u16), chunks[2].y + 1);
    }
    f.render_widget(command, chunks[2]);
    f.render_widget(status_bar, chunks[1]);
}