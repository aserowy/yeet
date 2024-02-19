use std::io::{stderr, BufWriter, Stderr};

use crossterm::{
    terminal::{self, EnterAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Frame, Terminal};

use crate::error::AppError;

pub struct TerminalWrapper {
    inner: Option<Terminal<CrosstermBackend<BufWriter<Stderr>>>>,
}

impl TerminalWrapper {
    pub fn start() -> Result<Self, AppError> {
        stderr().execute(EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;

        let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
        terminal.clear()?;

        let result = Self {
            inner: Some(terminal),
        };

        Ok(result)
    }

    pub fn shutdown(&mut self) -> Result<(), AppError> {
        self.inner = None;
        self.stop()
    }

    pub fn size(&self) -> Result<Rect, AppError> {
        if let Some(term) = &self.inner {
            Ok(term.size()?)
        } else {
            Err(AppError::TerminalNotInitialized)
        }
    }

    pub fn draw(&mut self, layout: impl FnMut(&mut Frame<'_>)) -> Result<(), AppError> {
        if let Some(term) = &mut self.inner {
            if let Err(err) = term.draw(layout) {
                return Err(AppError::from(err));
            }
        }

        Ok(())
    }

    pub fn suspend(&mut self) {
        self.stop().expect("Failed to stop terminal");
        self.inner = None;
    }

    pub fn resume(&mut self) -> Result<(), AppError> {
        if self.inner.is_none() {
            stderr().execute(EnterAlternateScreen)?;
            terminal::enable_raw_mode()?;

            let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
            terminal.clear()?;

            self.inner = Some(terminal);
        }

        Ok(())
    }

    pub fn resize(&mut self, x: u16, y: u16) -> Result<(), AppError> {
        if let Some(term) = &mut self.inner {
            if let Err(err) = term.resize(Rect::new(0, 0, x, y)) {
                return Err(AppError::from(err));
            }
        }

        Ok(())
    }

    fn stop(&self) -> Result<(), AppError> {
        terminal::disable_raw_mode()?;
        stderr().execute(terminal::LeaveAlternateScreen)?;

        Ok(())
    }
}
