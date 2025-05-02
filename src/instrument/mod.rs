pub mod project;
pub mod source;
pub mod fixed_serialization;
pub mod ast;
pub mod ast_general;
pub mod ast_custom_types;
pub mod custom_types;

pub use project::*;
pub use source::*;
pub use fixed_serialization::*;
pub use ast::*;
pub use ast_general::*;
pub use ast_custom_types::*;

pub fn is_hidden_path(path: &std::ffi::OsStr) -> bool {
    path.to_str().map_or(false, |s| s.starts_with('.'))
}