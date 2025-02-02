mod command_type;
mod basic;

pub use command_type::Command;
use toml_edit::Document;

pub fn parse(command: Document) -> Command {

    todo!()
}
