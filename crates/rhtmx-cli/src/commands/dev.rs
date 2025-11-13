use anyhow::Result;
use colored::Colorize;
use std::env;
use crate::theme::ThemeManager;

pub fn execute(port: u16) -> Result<()> {
    println!("{}", "Starting development server...".green().bold());
    println!();

    // Load and merge theme with user files
    let current_dir = env::current_dir()?;
    let manager = ThemeManager::new(&current_dir);

    println!("{}", "Preparing project...".cyan());
    manager.load_and_merge(false)?;

    println!();
    println!("Server will run at: {}", format!("http://localhost:{}", port).cyan().bold());
    println!("Press Ctrl+C to stop");
    println!();

    // TODO: Implement dev server
    // This will reuse the existing rhtmx-server with hot reload
    // Point it at merged directory: manager.merged_path()

    println!("{}", "⚠ Dev server not yet fully implemented".yellow());
    println!("Theme merged successfully to: {}", manager.merged_path().display());
    println!("Next: Will integrate with rhtmx-server for hot reload");

    Ok(())
}
