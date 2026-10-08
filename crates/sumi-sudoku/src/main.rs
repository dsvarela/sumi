mod app;
mod cli;
#[cfg(test)]
mod colour;
mod config;
#[cfg(test)]
mod fixtures;
mod game;
mod input;
mod menu;
mod settings;
mod theme;
mod ui;

fn main() -> std::process::ExitCode {
    cli::run()
}
