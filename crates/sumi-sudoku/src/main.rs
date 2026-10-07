mod app;
mod cli;
#[cfg(test)]
mod fixtures;
mod game;
mod input;
mod theme;
mod ui;

fn main() -> std::process::ExitCode {
    cli::run()
}
