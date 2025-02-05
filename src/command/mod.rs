pub(self) mod basic;
pub(self) mod command_type;

pub use command_type::Command;
use toml_edit::Document;

pub fn parse(command: Document) -> Command {

    todo!()
}
