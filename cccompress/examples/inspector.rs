//!
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    widgets::Paragraph,
};
use cccompress::{algorithm::*, Result};

fn main() -> Result<()> {
    // Generate data to be displayed by using the test data.
    let tree = build_tree()?;

    // startup: Enable raw mode for the terminal, giving us fine control over user input
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    // Initialize the terminal backend using crossterm
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // Main application loop
    loop {
        // Render the UI
        terminal.draw(|f| {
            f.render_widget(Paragraph::new(format!("{}", tree.0)), f.size());
        })?;

        // Check for user input every 250 milliseconds
        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            // If a key event occurs, handle it
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match key.code {
                        crossterm::event::KeyCode::Char('h') => move_screen(Direction::Left),
                        crossterm::event::KeyCode::Char('j') => move_screen(Direction::Down),
                        crossterm::event::KeyCode::Char('k') => move_screen(Direction::Up),
                        crossterm::event::KeyCode::Char('l') => move_screen(Direction::Right),
                        crossterm::event::KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }
    }

    // shutdown down: reset terminal back to original state
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn build_tree() -> Result<CtBinaryTree> {
    let content = std::fs::read_to_string("loremipsum.txt")?;
    let spec = CharSpectrum::from_stream(&content);
    Ok(CtBinaryTree::try_from(spec)?)
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn move_screen(dir: Direction) {
    todo!();
}
