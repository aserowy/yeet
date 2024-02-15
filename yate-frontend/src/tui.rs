use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use notify::{RecursiveMode, Watcher};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::{
    io::{stderr, BufWriter},
    path::PathBuf,
};
use yate_keymap::MessageResolver;

use crate::{
    error::AppError,
    event::{self, PostRenderAction},
    layout::AppLayout,
    model::{
        history::{self},
        Model,
    },
    task::Task,
    update::{self},
    view::{self},
};

pub async fn run(_address: String) -> Result<(), AppError> {
    stderr().execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut model = Model::default();
    if history::cache::load(&mut model.history).is_err() {
        // TODO: add notifications in tui and show history load failed
    }

    let (mut watcher, mut tasks, mut receiver) = event::listen();

    let mut resolver = MessageResolver::default();
    let mut result = Vec::new();

    'app_loop: while let Some(event) = receiver.recv().await {
        let messages = event::convert_to_messages(event, &mut resolver);

        let size = terminal.size().expect("Failed to get terminal size");
        let layout = AppLayout::default(size);

        let post_render_actions: Vec<_> = messages
            .iter()
            .flat_map(|message| update::update(&mut model, &layout, message))
            .flatten()
            .collect();

        // TODO: introduce skip rendering on certain message sents to prevent flickering (e.g. enumeration, preview)
        terminal.draw(|frame| view::view(&mut model, frame, &layout))?;

        for post_render_action in post_render_actions {
            match post_render_action {
                PostRenderAction::ModeChanged(mode) => resolver.mode = mode,
                PostRenderAction::OptimizeHistory => {
                    // TODO: add task to task manager
                    if let Err(_error) = history::cache::save(&model.history) {
                        // TODO: log error
                    }
                }
                PostRenderAction::Quit => {
                    break 'app_loop;
                }
                PostRenderAction::Task(task) => tasks.run(task),
                PostRenderAction::UnwatchPath(p) => {
                    if p == PathBuf::default() {
                        continue;
                    }

                    tasks.abort(&Task::EnumerateDirectory(p.clone()));

                    if let Err(_error) = watcher.unwatch(p.as_path()) {
                        // TODO: log error
                    }
                }
                PostRenderAction::WatchPath(p) => {
                    if p == PathBuf::default() {
                        continue;
                    }

                    if p.is_dir() {
                        tasks.run(Task::EnumerateDirectory(p.clone()));
                    } else {
                        tasks.run(Task::LoadPreview(p.clone()));
                    }

                    if let Err(_error) = watcher.watch(p.as_path(), RecursiveMode::NonRecursive) {
                        // TODO: log error
                    }
                }
            }
        }
    }

    if let Err(error) = tasks.finishing().await {
        result.push(error);
    }

    stderr().execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if result.is_empty() {
        Ok(())
    } else {
        Err(AppError::Aggregate(result))
    }
}
