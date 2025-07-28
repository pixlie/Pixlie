use ratatui::prelude::*;

pub struct Layout;

impl Layout {
    pub fn main_layout(area: Rect) -> Vec<Rect> {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);
        chunks.to_vec()
    }

    pub fn content_layout(area: Rect) -> Vec<Rect> {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Sidebar
                Constraint::Percentage(75), // Main content
            ])
            .split(area);
        chunks.to_vec()
    }

    pub fn settings_layout(area: Rect) -> Vec<Rect> {
        // Center the settings modal
        let popup_area = Layout::centered_rect(80, 70, area);

        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tab headers
                Constraint::Min(0),    // Settings content
                Constraint::Length(3), // Action buttons
            ])
            .split(popup_area);
        chunks.to_vec()
    }

    pub fn settings_tabs_layout(area: Rect) -> Vec<Rect> {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // UI
                Constraint::Percentage(20), // Session
                Constraint::Percentage(20), // LLM
                Constraint::Percentage(20), // Database
                Constraint::Percentage(20), // Shortcuts
            ])
            .split(area);
        chunks.to_vec()
    }

    pub fn settings_content_layout(area: Rect) -> Vec<Rect> {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0), // Settings form
            ])
            .split(area);
        chunks.to_vec()
    }

    pub fn settings_actions_layout(area: Rect) -> Vec<Rect> {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Apply
                Constraint::Percentage(20), // Reset
                Constraint::Percentage(20), // Export
                Constraint::Percentage(20), // Import
                Constraint::Percentage(20), // Close
            ])
            .split(area);
        chunks.to_vec()
    }

    /// Helper function to center a rectangle within another rectangle
    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        ratatui::layout::Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
