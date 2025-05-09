use colored::*;
use crate::output::parse::{DebugNode, DebugNodeType};

const PRINT_MAX_CHILDREN: usize = 15;

/*
pub fn print_debug_node(node: &DebugNode, indent: usize) {
    let node_type = match node.node_type {
        DebugNodeType::Complex => "-",
        DebugNodeType::Primitive => "-",
    };

    let indent_str = "   ".repeat(indent);

    let value_str = match node.value.len() {
        0 => "".to_string(),
        _ => format!("({})", node.value)
    };

    println!("{}{} {}: {} {}", indent_str, node_type, node.name, node.full_type, value_str);
    //println!("{}{} {}: {}", indent_str, node_type, node.name, node.full_type);

    for child in node.children.iter() {
        print_debug_node(child, indent + 1);
    }
}
 */

pub fn print_debug_node_colored(node: &DebugNode, indent: usize) {

    let node_type = match node.node_type {
        DebugNodeType::Complex => "▶".bright_blue(),
        DebugNodeType::Primitive => "•".green(),
    };

    let indent_str = "  ".repeat(indent);

    let name_str = node.name.bold().bright_yellow();

    let type_str = match node.full_type.len() {
        0 => "".to_string(),
        _ => format!("({})", node.full_type)
    }.italic().cyan();

    let value_str = match node.value.len() {
        0 => "".to_string(),
        _ => node.value.clone()
    }.bright_purple();

    let gap_str = if value_str.len() > 0 { " ".to_string() } else { "".to_string() };

    println!("{}{} {}: {}{}{}", indent_str, node_type, name_str, value_str, gap_str, type_str);

    for (i, child) in node.children.iter().enumerate() {
        if i < PRINT_MAX_CHILDREN {
            print_debug_node_colored(child, indent + 1);
        } else {
            let indent_str = "  ".repeat(indent + 1);
            let node_type = "•".green();
            println!("{}{} [...]", indent_str, node_type);
            break
        }
    }
}