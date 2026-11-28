use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = TestBackend::new(40, 8);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| {
        let area: Rect = frame.area();
        let widget = Paragraph::new("Ratatui 0.28 fixture")
            .block(Block::default().borders(Borders::ALL).title("Fixture"));
        frame.render_widget(widget, area);
    })?;
    Ok(())
}
