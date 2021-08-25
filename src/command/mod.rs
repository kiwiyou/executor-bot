mod arg;
pub mod handler;
pub use arg::*;

#[derive(Clone)]
pub enum ExecutorCommand {
    Help,
    Run,
}

macro_rules! arg_parser {
    ($(($label:literal, $enum:expr)),+) => {
        CommandMatcher::new(&[$($label),+], &[$($enum),+])
    };
}

pub fn init_parser() -> CommandMatcher<ExecutorCommand> {
    arg_parser!(
        ("help", ExecutorCommand::Help),
        ("eval", ExecutorCommand::Run)
    )
}
