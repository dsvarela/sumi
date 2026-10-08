mod app;
mod cli;
mod config;
#[cfg(test)]
mod fixtures;
mod game;
mod input;
mod theme;
mod ui;

fn main() -> std::process::ExitCode {
    cli::run()
}
