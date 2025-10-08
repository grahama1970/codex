use crate::anim_config;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::Styled;
use ratatui::widgets::Widget;

/// Frame sets (Unicode) with ASCII fallback.
const FRAMES_ARC: &[&str] = &["◴", "◷", "◶", "◵"]; // orbit spinner
const FRAMES_PIE: &[&str] = &["◐", "◓", "◑", "◒"]; // minimal spinner
const FRAMES_ASCII: &[&str] = &["-", "\\", "|", "/"]; // fallback

fn reduced_motion() -> bool {
    std::env::var("CXPLUS_REDUCE_MOTION")
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// A compact, brand‑accented orbit spinner. Centered in the given area.
/// Pass a monotonically increasing tick (e.g., 0..∞) to advance frames.
pub struct OrbitSpinner {
    pub tick: u64,
}

impl Widget for OrbitSpinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let cx = area.x + area.width / 2;
        let cy = area.y + area.height / 2;

        // Center nucleus (secondary text dimmed)
        let nucleus = Span::from("•").dim();
        buf.set_span(cx, cy, &nucleus, 1);

        // Reduced‑motion: show static tip or pie tick instead of orbit.
        if reduced_motion() {
            if anim_config::tips_enabled() {
                let tips = anim_config::tips_list();
                let idx = (self.tick / anim_config::tips_interval_secs()) as usize % tips.len();
                let tip = &tips[idx];
                if area.height >= 3 {
                    let start_x = cx.saturating_sub((tip.len() / 2) as u16);
                    buf.set_span(
                        start_x,
                        cy.saturating_add(1),
                        &Span::from(tip.clone()).dim(),
                        tip.len() as u16,
                    );
                }
            } else {
                let s = Span::from(FRAMES_PIE[(self.tick as usize) % FRAMES_PIE.len()])
                    .set_style(crate::style::brand_accent_style());
                if cx > 0 {
                    buf.set_span(cx - 1, cy, &s, 1);
                }
            }
            return;
        }

        // Choose frame set with a graceful ASCII fallback if needed.
        let frames = if supports_color::on_cached(supports_color::Stream::Stdout)
            .map(|_| true)
            .unwrap_or(true)
        {
            FRAMES_ARC
        } else {
            FRAMES_ASCII
        };
        let frame = frames[(self.tick as usize) % frames.len()];
        let orb = Span::from(frame).set_style(crate::style::brand_accent_style());

        // Place the orbit glyph one cell to the right of the nucleus (simple illusion).
        if cx + 1 < area.x + area.width {
            buf.set_span(cx + 1, cy, &orb, 1);
        }

        // Semantic tip overlay (Claude‑style) — toggled by config/env/slash.
        if anim_config::tips_enabled() && area.height >= 3 {
            let tips = anim_config::tips_list();
            let idx = (self.tick / anim_config::tips_interval_secs()) as usize % tips.len();
            let tip = &tips[idx];
            let start_x = cx.saturating_sub((tip.len() / 2) as u16);
            buf.set_span(
                start_x,
                cy.saturating_add(1),
                &Span::from(tip.clone()).dim(),
                tip.len() as u16,
            );
        }
    }
}

/// Determinate gauge with a subtle shimmer across the filled region.
/// - progress: 0.0..=1.0
/// - tick: advancing phase for shimmer; ignored under reduced motion.
pub struct ShimmerGauge {
    pub progress: f64,
    pub tick: u64,
}

impl Widget for ShimmerGauge {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let width = area.width as usize;
        let filled = ((self.progress.clamp(0.0, 1.0)) * width as f64).round() as usize;

        // Track (border-ish)
        let track_style = Style::default().fg(Color::DarkGray);
        for x in 0..width {
            if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y)) {
                cell.set_char(' ').set_style(track_style);
            }
        }

        // Fill (brand accent)
        let fill_style = crate::style::brand_accent_style();
        for x in 0..filled.min(width) {
            if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y)) {
                cell.set_char('█').set_style(fill_style);
            }
        }

        // Shimmer (accent-secondary illusion via alternating density)
        if !reduced_motion() && filled > 0 {
            let phase = (self.tick % 8) as usize;
            for x in 0..filled.min(width) {
                if (x + phase).is_multiple_of(8) {
                    if let Some(cell) = buf.cell_mut((area.x + x as u16, area.y)) {
                        cell.set_char('▓');
                    }
                } else if (x + phase) % 8 == 4
                    && let Some(cell) = buf.cell_mut((area.x + x as u16, area.y))
                {
                    cell.set_char('▒');
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn ascii_spinner_frame(tick: u64) -> &'static str {
    FRAMES_ASCII[(tick as usize) % FRAMES_ASCII.len()]
}
