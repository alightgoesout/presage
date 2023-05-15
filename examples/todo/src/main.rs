mod app;
mod configuration;
mod error;
mod persistence;
mod todo;

use dialoguer::console::Term;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use presage::Id;
use std::io::Write;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::todo::commands::{ArchiveTodo, CheckTodo, DeleteArchivedTodos};
use crate::todo::{Todo, TodoState};
use app::TodoApp;
pub use error::Error;
use todo::commands::{CreateTodo, RenameTodo};

const MENU: &[&str] = &["New todo", "List todos", "List archived todos", "Quit"];

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut app = TodoApp::new();

    let mut term = Term::stdout();
    let theme = ColorfulTheme::default();

    loop {
        term.clear_screen()?;
        writeln!(term, "{}", app.summary()?)?;

        let selection = Select::with_theme(&theme)
            .with_prompt("Action:")
            .default(0)
            .items(MENU)
            .interact_on_opt(&term)?;

        match selection {
            Some(0) => new_todo(&mut app, &mut term, &theme).await?,
            Some(1) => list_todos(&mut app, &mut term, &theme).await?,
            Some(2) => list_archive(&mut app, &mut term, &theme).await?,
            _ => break,
        }
    }

    Ok(())
}

async fn new_todo(app: &mut TodoApp, term: &mut Term, theme: &ColorfulTheme) -> Result<(), Error> {
    let name: String = Input::with_theme(theme)
        .with_prompt("New todo:")
        .interact_text_on(term)?;

    let name = name.trim();

    if !name.is_empty() {
        app.execute(CreateTodo {
            id: Id(Uuid::new_v4()),
            name: name.into(),
        })
        .await
    } else {
        Ok(())
    }
}

async fn list_todos(
    app: &mut TodoApp,
    term: &mut Term,
    theme: &ColorfulTheme,
) -> Result<(), Error> {
    term.clear_screen()?;
    writeln!(term, "Todos:")?;

    let mut todos = app.list_visible_todos()?;
    for (index, todo) in todos.iter().enumerate() {
        let state = if let TodoState::New = todo.state {
            '☐'
        } else {
            '🗹'
        };
        writeln!(term, "{:>2}. {state} {}", index + 1, todo.name)?;
    }

    let selection: String = Input::with_theme(theme)
        .with_prompt("Enter number of todo to edit or press <enter> to return")
        .allow_empty(true)
        .interact_text_on(term)?;

    if let Ok(index) = selection.parse::<usize>() {
        if (0..todos.len()).contains(&(index - 1)) {
            edit_todo(todos.remove(index - 1), app, term, theme).await?;
        }
    }

    Ok(())
}

async fn edit_todo(
    todo: Todo,
    app: &mut TodoApp,
    term: &mut Term,
    theme: &ColorfulTheme,
) -> Result<(), Error> {
    let state_action = match todo.state {
        TodoState::New => "Check",
        TodoState::Done { .. } => "Archive",
        TodoState::Archived { .. } => unreachable!(),
    };

    let selection = Select::with_theme(theme)
        .with_prompt(&format!(r#"Edit "{}" (press <esc> to return)"#, todo.name))
        .default(0)
        .item("Rename")
        .item(state_action)
        .interact_on_opt(term)?;

    match selection {
        Some(0) => rename(todo, app, term, theme).await,
        Some(1) => next_state(todo, app).await,
        _ => Ok(()),
    }
}

async fn rename(
    todo: Todo,
    app: &mut TodoApp,
    term: &mut Term,
    theme: &ColorfulTheme,
) -> Result<(), Error> {
    let new_name: String = Input::with_theme(theme)
        .with_prompt("New name:")
        .with_initial_text(todo.name)
        .interact_text_on(term)?;
    let new_name = new_name.trim();

    if new_name.is_empty() {
        Ok(())
    } else {
        app.execute(RenameTodo {
            id: todo.id,
            name: new_name.into(),
        })
        .await
    }
}

async fn next_state(todo: Todo, app: &mut TodoApp) -> Result<(), Error> {
    if matches!(todo.state, TodoState::New) {
        app.execute(CheckTodo {
            id: todo.id,
            date: OffsetDateTime::now_utc(),
        })
        .await
    } else {
        app.execute(ArchiveTodo {
            id: todo.id,
            date: OffsetDateTime::now_utc(),
        })
        .await
    }
}

async fn list_archive(
    app: &mut TodoApp,
    term: &mut Term,
    theme: &ColorfulTheme,
) -> Result<(), Error> {
    term.clear_screen()?;
    writeln!(term, "Archive:")?;

    let todos = app.list_archived_todos()?;
    for (index, todo) in todos.iter().enumerate() {
        writeln!(term, "{:>2}. {}", index + 1, todo.name)?;
    }

    let selection = Select::with_theme(theme)
        .with_prompt("Action:")
        .default(0)
        .items(&["Clean archived todos", "Return"])
        .interact_on_opt(term)?;

    match selection {
        Some(0) => app.execute(DeleteArchivedTodos).await,
        _ => Ok(()),
    }
}
