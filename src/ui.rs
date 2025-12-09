use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap, BorderType},
    Frame,
};
use std::time::Duration;
use tachyonfx::{fx, EffectRenderer};
use crate::app::App;
use chrono::{DateTime, Utc};

// --- Theme Definition ---
struct Theme {
    bg: Color,
    text: Color,
    border: Color,
    trust_high: Color,
    trust_med: Color,
    trust_low: Color,
    warning: Color,
    highlight_bg: Color,
}

const NEON_THEME: Theme = Theme {
    bg: Color::Black,
    text: Color::Rgb(220, 220, 220), // Off-white
    border: Color::Rgb(0, 255, 255), // Cyan
    trust_high: Color::Rgb(57, 255, 20), // Neon Green
    trust_med: Color::Rgb(255, 240, 31), // Neon Yellow
    trust_low: Color::Rgb(100, 100, 100), // Grey
    warning: Color::Rgb(255, 0, 255), // Magenta
    highlight_bg: Color::Rgb(20, 20, 40), // Dark Blue-ish for selection
};

pub fn ui(f: &mut Frame, app: &mut App) {
    // Root layout: Content + Footer
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1), 
        ])
        .split(f.area());

    // Main layout: Timeline (Left 80%) + Sidebar (Right 20%)
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(root_layout[0]);

    render_timeline(f, app, main_layout[0]);
    render_sidebar(f, app, main_layout[1]); // New Sidebar Component
    render_status_bar(f, app, root_layout[1]);
    render_popup(f, app);
    
    // Apply Global Visual Effects (HSL Shift on Border/Glow)
    render_visual_effects(f, app, main_layout[0]);
}

fn render_timeline(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.timeline
        .iter()
        .map(|event| {
            // -- Trust Score Logic --
            let trust_score = app.trust_scores.get(&event.pubkey).copied().unwrap_or(0.0);
            let trust_color = if trust_score >= 1.0 {
                NEON_THEME.trust_high
            } else if trust_score >= 0.5 {
                NEON_THEME.trust_med
            } else {
                NEON_THEME.trust_low
            };

            // -- Time Logic --
            let created_at = DateTime::<Utc>::from_timestamp(event.created_at.as_u64() as i64, 0)
                .unwrap_or(Utc::now());
            let time_str = created_at.format("%H:%M").to_string();

            // -- Metadata Line --
            let metadata_line = Line::from(vec![
                Span::styled(format!("TRUST {:.2}", trust_score), Style::default().fg(trust_color).add_modifier(Modifier::BOLD)),
                Span::raw(" │ "),
                Span::styled(format!("@{}", &event.pubkey.to_string()[0..8]), Style::default().fg(Color::Cyan)),
                Span::raw(".. │ "),
                Span::styled(time_str, Style::default().fg(Color::DarkGray)),
            ]);

            // -- Content Handling --
            let content_lines: Vec<Line> = event.content.lines().map(|l| {
                Line::from(Span::styled(l, Style::default().fg(NEON_THEME.text)))
            }).collect();

            // -- Fact Check Warning --
            let mut lines = vec![
                Line::from(""), // Spacer top
                metadata_line,
                Line::from(""), // Spacer
            ];
            lines.extend(content_lines);
            
            if let Some(verdict) = app.verifications.get(&event.id) {
                 lines.push(Line::from(""));
                 lines.push(Line::from(Span::styled(format!("⚠ VERIFIED: {}", verdict), Style::default().fg(NEON_THEME.warning).add_modifier(Modifier::BOLD))));
            }
            
            lines.push(Line::from("")); // Spacer bottom
            lines.push(Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(Color::DarkGray)))); // Separator

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(NEON_THEME.border))
            .title(" 🫧 BUBBLE (Press 'i' to Post) ")
            .title_alignment(Alignment::Center))
        .highlight_style(Style::default().bg(NEON_THEME.highlight_bg))
        .highlight_symbol(""); // Removed symbol to make it cleaner, background change is enough

    f.render_stateful_widget(list, area, &mut app.scroll_state);

    // Scrollbar
    use ratatui::widgets::Scrollbar;
    use ratatui::widgets::ScrollbarOrientation;
    use ratatui::widgets::ScrollbarState;

    let total_height = app.timeline.len();
    let viewport_height = area.height as usize;
    if total_height > viewport_height {
        let mut scrollbar_state = ScrollbarState::default()
             .content_length(total_height)
             .position(app.scroll_state.selected().unwrap_or(0));
        
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .style(Style::default().fg(NEON_THEME.border)),
            area,
            &mut scrollbar_state
        );
    }
}

fn render_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" PROFILE ")
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Profile Info
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Avatar/Name area
            Constraint::Min(0),    // Activity/Stats
        ])
        .split(inner_area);

    let pubkey_short = &app.own_pubkey.to_string()[0..8];
    
    let profile_text = vec![
        Line::from(Span::styled("ME", Style::default().fg(NEON_THEME.trust_high).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("@{}", pubkey_short), Style::default().fg(Color::Cyan))),
    ];
    
    let profile_widget = Paragraph::new(profile_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
        
    f.render_widget(profile_widget, chunks[0]);
    
    
    // Network Stats
    let stats_text = vec![
        Line::from(""),
        Line::from(Span::styled("Statistics", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from(format!("Posts: {}", app.timeline.iter().filter(|e| e.pubkey == app.own_pubkey).count())),
        Line::from("Trust: 100%"),
        Line::from(""),
        Line::from(Span::styled("Network", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from("Relays: 1"),
        Line::from(format!("Peers: {}", app.trust_scores.len())),
        Line::from(""),
        Line::from(Span::styled("Controls", Style::default().add_modifier(Modifier::UNDERLINED))),
        Line::from(vec![Span::styled("i", Style::default().fg(NEON_THEME.trust_med)), Span::raw(" : Post")]),
        Line::from(vec![Span::styled("Enter", Style::default().fg(NEON_THEME.trust_med)), Span::raw(" : Submit")]),
        Line::from(vec![Span::styled("S+Enter", Style::default().fg(NEON_THEME.trust_med)), Span::raw(": NewLine")]),
        Line::from(vec![Span::styled("r", Style::default().fg(NEON_THEME.trust_med)), Span::raw(" : Refresh")]),
        Line::from(vec![Span::styled("j/k", Style::default().fg(NEON_THEME.trust_med)), Span::raw(" : Scroll")]),
        Line::from(vec![Span::styled("Esc", Style::default().fg(NEON_THEME.trust_med)), Span::raw(" : Quit")]),
    ];
    
    let stats_widget = Paragraph::new(stats_text)
        .style(Style::default().fg(NEON_THEME.text))
        .wrap(Wrap { trim: true });
        
    f.render_widget(stats_widget, chunks[1]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    // Simplified Status Bar
    let mode_str = if app.input_mode { "EDIT MODE" } else { "NORMAL" };
    let mode_color = if app.input_mode { NEON_THEME.trust_med } else { NEON_THEME.trust_high };

    let status_text = vec![
        Span::styled(format!(" {} ", mode_str), Style::default().fg(Color::Black).bg(mode_color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(format!(" {} ", app.status), Style::default().fg(NEON_THEME.text)),
    ];
    let status_p = Paragraph::new(Line::from(status_text))
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(status_p, area);
}

fn render_popup(f: &mut Frame, app: &mut App) {
    if app.input_mode {
        let area = centered_rect(60, 30, f.area());
        f.render_widget(Clear, area);
        
        app.input.set_block(
             Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double) 
                .border_style(Style::default().fg(NEON_THEME.trust_med))
                .title(" NEW POST ")
                .title_alignment(Alignment::Center)
                .title_bottom(Line::from(" Enter: Submit │ Shift+Enter: Newline │ Esc: Cancel ").alignment(Alignment::Right))
        );
        app.input.set_style(Style::default().fg(NEON_THEME.text));
        app.input.set_cursor_style(Style::default().bg(NEON_THEME.trust_med).fg(Color::Black));
        
        f.render_widget(&app.input, area);
    }
}

fn render_visual_effects(f: &mut Frame, app: &App, area: Rect) {
     use tachyonfx::{EffectTimer, Interpolation, CellFilter, Motion}; // Import Motion directly
     use ratatui::layout::Margin;
     
     let elapsed_secs = app.start_time.elapsed().as_secs_f32();
     
    // Snake effect removed as per user request (was interpreted as update indicator)
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
