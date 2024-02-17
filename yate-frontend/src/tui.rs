use std::{
    env,
    io::{stderr, BufWriter},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use notify::{RecursiveMode, Watcher};
use ratatui::prelude::{CrosstermBackend, Terminal};
use tokio::time;

use crate::{
    error::AppError,
    event::{self, PostRenderAction, PreRenderAction, RenderAction},
    layout::AppLayout,
    model::{
        history::{self},
        Model,
    },
    settings::Settings,
    task::Task,
    update::{self},
    view::{self},
};

pub async fn run(settings: Settings) -> Result<(), AppError> {
    stderr().execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut model = Model::default();
    if history::cache::load(&mut model.history).is_err() {
        // TODO: add notifications in tui and show history load failed
    }

    let initial_path = get_initial_path(settings.startup_path);
    let (resolver_mutex, mut watcher, mut tasks, mut receiver) = event::listen(initial_path);
    let mut result = Vec::new();

    'app_loop: while let Some(messages) = receiver.recv().await {
        let size = terminal.size().expect("Failed to get terminal size");
        let layout = AppLayout::default(size);

        let render_actions: Vec<_> = messages
            .iter()
            .flat_map(|message| update::update(&mut model, &layout, message))
            .flatten()
            .collect();

        // TODO: refactor pre render actions
        let pre_render_actions = render_actions.iter().filter_map(|actn| match actn {
            RenderAction::Pre(pre) => Some(pre),
            _ => None,
        });

        for pre_render_action in pre_render_actions {
            match pre_render_action {
                PreRenderAction::SleepBeforeRender => {
                    time::sleep(Duration::from_millis(25)).await;
                }
            }
        }

        terminal.draw(|frame| view::view(&mut model, frame, &layout))?;

        // TODO: refactor post render actions
        let post_render_actions = render_actions.iter().filter_map(|actn| match actn {
            RenderAction::Post(post) => Some(post),
            _ => None,
        });

        for post_render_action in post_render_actions {
            match post_render_action {
                PostRenderAction::ModeChanged(mode) => {
                    let mut resolver = resolver_mutex.lock().await;
                    resolver.mode = mode.clone();
                }

                PostRenderAction::Quit => {
                    break 'app_loop;
                }
                PostRenderAction::Task(task) => tasks.run(task.clone()),
                PostRenderAction::UnwatchPath(p) => {
                    if p == &PathBuf::default() {
                        continue;
                    }

                    tasks.abort(&Task::EnumerateDirectory(p.clone()));

                    if let Err(_error) = watcher.unwatch(p.as_path()) {
                        // TODO: log error
                    }
                }
                PostRenderAction::WatchPath(p) => {
                    if p == &PathBuf::default() {
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

fn get_initial_path(initial_selection: Option<PathBuf>) -> PathBuf {
    if let Some(path) = initial_selection {
        if path.exists() {
            return path;
        }
    }

    env::current_dir().expect("Failed to get current directory")
}
