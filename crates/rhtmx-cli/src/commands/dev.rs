use anyhow::Result;
use colored::Colorize;

pub fn execute(port: u16) -> Result<()> {
    println!("{}", "Starting development server...".green().bold());
    println!();
    println!("Server running at: {}", format!("http://localhost:{}", port).cyan());
    println!("Press Ctrl+C to stop");
    println!();

    // TODO: Implement dev server
    // This will reuse the existing rhtmx-server with hot reload
    // Point it at the merged directory (.rhtmx/merged/)

    println!("{}", "⚠ Dev server not yet implemented".yellow());
    println!("Coming soon: Will run rhtmx-server with hot reload");

    Ok(())
}
