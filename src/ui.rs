use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::App;

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let items: Vec<ListItem> = app.timeline
        .iter()
        .map(|event| {
            let content = event.content.clone();
            
            let trust_score = app.trust_scores.get(&event.pubkey).copied().unwrap_or(0.0);
            let trust_color = if trust_score >= 1.0 {
                Color::Green
            } else if trust_score >= 0.5 {
                Color::Yellow
            } else {
                Color::Gray
            };

            let verification_line = if let Some(verdict) = app.verifications.get(&event.id) {
               Line::from(Span::styled(format!("⚠ FACT CHECK: {}", verdict), Style::default().fg(Color::Magenta)))
            } else {
               Line::from(Span::raw(""))
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!("[Trust: {:.2}]", trust_score), Style::default().fg(trust_color)),
                ]),
                Line::from(Span::raw(content)),
                verification_line,
                Line::from(Span::styled("---", Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Bubble Timeline (Press 'r' to refresh, 'Esc' to quit)"));

    f.render_widget(list, chunks[0]);
    
    
    if app.input_mode {
        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area); // Clear background for popup effect
        
        let input = Paragraph::new(app.input.clone())
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Post (Esc to cancel, Enter to submit)"));
        f.render_widget(input, area);
    }

    let status = Paragraph::new(app.status.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(status, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
