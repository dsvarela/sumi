mod app;
mod cli;
mod game;
mod input;
mod theme;
mod ui;

fn main() -> std::process::ExitCode {
    cli::run()
}
