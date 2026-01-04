use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::{Repository, BranchType};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{io, time::Duration};

struct App {
    branches: Vec<String>,
    current_branch_index: usize,
    repo_path: String,
    state: ListState,
    should_quit: bool,
    message: Option<String>,
    is_error: bool,
}

impl App {
    fn new() -> Result<Self> {
        let repo = Repository::open_from_env().context("Failed to open git repository. Are you in a git directory?")?;
        let branches = Self::get_branches(&repo)?;
        let current = Self::get_current_branch(&repo).unwrap_or_default();

        let mut state = ListState::default();
        let current_index = branches.iter().position(|b| b == &current).unwrap_or(0);
        state.select(Some(current_index));

        let repo_path = repo.path().parent().unwrap().to_string_lossy().into_owned();

        Ok(Self {
            branches,
            current_branch_index: current_index,
            repo_path,
            state,
            should_quit: false,
            message: None,
            is_error: false,
        })
    }

    fn get_branches(repo: &Repository) -> Result<Vec<String>> {
        let branches = repo.branches(Some(BranchType::Local))?;
        let mut branch_names = Vec::new();

        for b in branches {
            let (branch, _) = b?;
            if let Some(name) = branch.name()? {
                branch_names.push(name.to_string());
            }
        }
        branch_names.sort();
        Ok(branch_names)
    }

    fn get_current_branch(repo: &Repository) -> Result<String> {
        let head = repo.head()?;
        Ok(head.shorthand().unwrap_or("").to_string())
    }

    fn checkout_branch(&mut self) -> Result<()> {
        if let Some(index) = self.state.selected() {
            if let Some(branch_name) = self.branches.get(index) {
                let repo = Repository::open_from_env()?;

                // First look up the branch
                let (object, reference) = repo.revparse_ext(branch_name)?;

                // Define checkout options - safe checkout
                let mut opts = git2::build::CheckoutBuilder::new();
                opts.safe(); // Fails if local changes would be overwritten

                match repo.checkout_tree(&object, Some(&mut opts)) {
                    Ok(_) => {
                        // If checkout tree succeeded, move HEAD
                        if let Some(reference) = reference {
                             match repo.set_head(reference.name().unwrap()) {
                                Ok(_) => {
                                    self.message = Some(format!("Switched to branch '{}'", branch_name));
                                    self.is_error = false;
                                    return Ok(());
                                },
                                Err(e) => {
                                    self.message = Some(format!("Failed to set HEAD: {}", e));
                                    self.is_error = true;
                                }
                            }
                        } else {
                             // This handles detached HEAD state if for some reason we got no reference
                             // But for branches we expect a reference.
                             self.message = Some(format!("Checked out '{}' (detached)", branch_name));
                             self.is_error = false;
                        }
                    },
                    Err(e) => {
                        self.message = Some(format!("Failed to checkout: {}", e));
                        self.is_error = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.branches.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.branches.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = match App::new() {
        Ok(app) => app,
        Err(e) => {
            // Restore terminal before printing error
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    if let Some(msg) = app.message {
        if app.is_error {
             eprintln!("Error: {}", msg);
        } else {
             println!("{}", msg);
        }
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => {
                            app.checkout_branch()?;
                            // Optional: quit after successful checkout
                             if !app.is_error {
                                 return Ok(());
                             }
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // List
            Constraint::Length(3), // Status/Help
        ])
        .split(frame.area());

    // Title
    let title = Paragraph::new(format!("Git Checkout - {}", app.repo_path))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title, main_layout[0]);

    // Branches List
    let items: Vec<ListItem> = app.branches
        .iter()
        .map(|b| {
            let style = if let Ok(repo) = Repository::open_from_env() {
                if let Ok(head) = repo.head() {
                     if head.shorthand().unwrap_or("") == b {
                         Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                     } else {
                         Style::default()
                     }
                } else { Style::default() }
            } else { Style::default() };

            ListItem::new(Line::from(vec![Span::styled(format!("  {}", b), style)]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Branches"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, main_layout[1], &mut app.state);

    // Status/Help
    let status_text = if let Some(msg) = &app.message {
        if app.is_error {
             Span::styled(msg, Style::default().fg(Color::Red))
        } else {
             Span::styled(msg, Style::default().fg(Color::Green))
        }
    } else {
        Span::raw("Press 'q' to quit, Enter to checkout, j/k to navigate")
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, main_layout[2]);
}
