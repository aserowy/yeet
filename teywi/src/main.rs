use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::io::{stderr, stdout, BufWriter, Result, Write};

#[tokio::main]
async fn main() -> Result<()> {

    let server_handle = tokio::spawn(async {

    });

    let frontend_handle = tokio::spawn(async {
        stderr().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
        terminal.clear()?;

        loop {
            terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(
                    Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                        .white()
                        .on_blue(),
                    area,
                );
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        stderr().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        stdout().lock().write_all(b"this is a test\n")?;
        Ok(())
    });

    tokio::join!(frontend_handle, server_handle).0?
}
