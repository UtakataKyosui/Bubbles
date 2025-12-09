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
    border: Color::Rgb(0, 100, 255), // Blue
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
    use ratatui::widgets::Widget;
    use ratatui::buffer::Buffer;
    
    struct SnakeBorderWidget {
        elapsed: f32,
        speed: f32,
        length: f32,
        color: Color,
    }

    impl Widget for SnakeBorderWidget {
        fn render(self, area: Rect, buf: &mut Buffer) {
            let w = area.width as f32;
            let h = area.height as f32;
            if w < 2.0 || h < 2.0 { return; }
            
            // Perimeter path: Top(w) -> Right(h-1) -> Bottom(w-1) -> Left(h-2)
            // Total = w + h-1 + w-1 + h-2 = 2w + 2h - 4
            let perimeter = 2.0 * (w + h) - 4.0;
            let current_pos = (self.elapsed * self.speed) % perimeter;
            
            // Helper to determine if a point on perimeter is "active"
            // We want a gradient tail.
            let apply_snake = |x: u16, y: u16, dist_on_p: f32, buf: &mut Buffer| {
                 if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                    // Calculate distance from head `current_pos`
                    // d = (head - point + P) % P
                    let d = (current_pos - dist_on_p + perimeter) % perimeter;
                    
                    if d < self.length {
                        // It is within the snake (head or tail)
                        let intensity = 1.0 - (d / self.length);
                        // KWin-like: Head is White/Bright, Tail fades to Color
                        if intensity > 0.0 {
                             // Simple linear fade logic
                             // Head (intensity near 1.0) -> White
                             // Tail (intensity near 0.0) -> self.color
                             if intensity > 0.8 {
                                 cell.set_fg(Color::White); // Head highlight
                             } else if intensity > 0.1 {
                                 cell.set_fg(self.color);
                             }
                             // We don't clear the fg if not in snake, to keep original border visible?
                             // User says "Only this animation". Maybe we should Dim the rest?
                             // But the border characters (│, ─) must exist. They are drawn by Block.
                             // We are just changing their color.
                        }
                    }
                    // Removed else block to maintain original border visibility
                 }
            };

            // Top Edge (0..w-1) -> P=0..w-1
            for x in 0..area.width {
                apply_snake(x, 0, x as f32, buf);
            }
            // Right Edge (1..h-1) -> P=w..w+h-2
            for y in 1..area.height {
                apply_snake(area.width - 1, y, w - 1.0 + y as f32, buf);
            }
            // Bottom Edge (w-2..0) -> P=w+h-2 .. 
            // Correct path for bottom is Right to Left.
            // Start at x=w-1 (Corner) was right. Next is x=w-2.
            // Distance accumulates.
            // Top (w) + Right (h-1) is end of Right edge (bottom-right corner)
            // Bottom edge starts at bottom-right corner? No, corner is shared.
            // Let's model cells uniquely.
            // Top: (0,0) to (w-2, 0). (w-1,0) is TopRight corner.
            // Right: (w-1, 0) to (w-1, h-2). (w-1, h-1) is BottomRight.
            // But lets use the visual path.
            
            // Let's refine P mapping to be strictly linear along the visual line.
            
            // Top: x from 0 to w-1. P = x.
            // Right: y from 1 to h-1. P = w-1 + y. (Start at P=w)
            // Bottom: x from w-2 down to 0. P = (w-1 + h-1) + (w-1 - 1 - x).
            //   Start (x=w-2): P = w+h-2 + 0.
            //   End (x=0): P = w+h-2 + w-2.
            // Left: y from h-2 down to 1. P = (w+h-2 + w-1) + (h-1 - 1 - y).
            //   Start (y=h-2): P = 2w+h-3.
            //   End (y=1): P = 2w+h-3 + h-3 = 2w+2h-6... wait maths.
            
            // Let's just run loop and increment counter `p`.
            let mut p = 0.0;
            // Top (0 to w-1)
            for x in 0..area.width {
                apply_snake(x, 0, p, buf);
                p += 1.0;
            }
            // Right (1 to h-1)
            for y in 1..area.height {
                 apply_snake(area.width - 1, y, p, buf);
                 p += 1.0;
            }
            // Bottom (w-2 down to 0)
            if area.width > 1 {
                for x in (0..area.width - 1).rev() {
                    apply_snake(x, area.height - 1, p, buf);
                    p += 1.0;
                }
            }
            // Left (h-2 down to 1)
            if area.height > 1 {
                for y in (1..area.height - 1).rev() {
                    apply_snake(0, y, p, buf);
                    p += 1.0;
                }
            }
        }
    }

    let elapsed_secs = app.start_time.elapsed().as_secs_f32();
    let widget = SnakeBorderWidget {
        elapsed: elapsed_secs,
        speed: 60.0,    // 60 cells/sec is smooth
        length: 40.0,   // Long tail for KWin effect
        color: Color::Cyan, // Cyan is classic KWin
    };
    
    // Render custom widget on top of area
    f.render_widget(widget, area);
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
