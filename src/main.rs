mod app;
mod clipboard;
mod editor;
mod event;
mod highlight;
mod config;
mod ui;

use std::env;
use std::fs;
use std::io;

use crossterm::{
    event::{DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use event::run_app;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!("ekphos {}", VERSION);
    println!("A lightweight, fast, terminal-based markdown research tool");
    println!();
    println!("USAGE:");
    println!("    ekphos [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print help information");
    println!("    -v, --version    Print version information");
    println!("    -c, --config     Print config file path");
    println!("    -d, --dir        Print notes directory path");
    println!("    --reset          Reset config and themes to defaults");
}

fn reset_config_and_themes() {
    let config_path = config::Config::config_path();
    let themes_dir = config::Config::themes_dir();

    println!("Resetting ekphos configuration...");
    println!();

    if config_path.exists() {
        match fs::remove_file(&config_path) {
            Ok(_) => println!("  Deleted: {}", config_path.display()),
            Err(e) => eprintln!("  Failed to remove config: {}", e),
        }
    } else {
        println!("  Config file not found (skipped)");
    }

    if themes_dir.exists() {
        match fs::remove_dir_all(&themes_dir) {
            Ok(_) => println!("  Deleted: {}", themes_dir.display()),
            Err(e) => eprintln!("  Failed to remove themes: {}", e),
        }
    } else {
        println!("  Themes directory not found (skipped)");
    }

    println!();
    println!("Generating fresh defaults...");
    println!();

    let _config = config::Config::load_or_create();

    println!("  Created: {}", config_path.display());
    println!("  Created: {}", themes_dir.join("ekphos-dawn.toml").display());

    println!();
    println!("Reset complete! Configuration restored to v{} defaults.", VERSION);
}

fn main() -> io::Result<()> {
    // Handle CLI arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "-v" | "--version" => {
                println!("ekphos {}", VERSION);
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-c" | "--config" => {
                println!("{}", config::Config::config_path().display());
                return Ok(());
            }
            "-d" | "--dir" => {
                let config = config::Config::load();
                println!("{}", config.notes_path().display());
                return Ok(());
            }
            "--reset" => {
                reset_config_and_themes();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Run 'ekphos --help' for usage information");
                return Ok(());
            }
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste, EnableFocusChange)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste,
        DisableFocusChange
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}
