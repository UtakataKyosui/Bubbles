use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
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
                    Span::styled(format!("Pubkey: {}", event.pubkey), Style::default().fg(Color::Blue)),
                    Span::raw(" "),
                    Span::styled(format!("[Trust: {:.2}]", trust_score), Style::default().fg(trust_color)),
                ]),
                Line::from(Span::raw(content)),
                verification_line,
                Line::from(Span::styled("---", Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Bubble Timeline (Press 'r' to refresh, 'q' to quit)"));

    f.render_widget(list, chunks[0]);

    let status = Paragraph::new(app.status.clone())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(status, chunks[1]);
}
