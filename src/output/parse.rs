use std::collections::VecDeque;
use std::fmt::Debug;
use base64::Engine;
use base64::engine::general_purpose;
use solana_sdk::pubkey::Pubkey;
use std::fmt::Display;

#[derive(Debug)]
pub enum DebugNodeType {
    Primitive,
    Complex,
}

#[derive(Debug)]
pub struct DebugNode {
    pub node_type: DebugNodeType,
    pub name: String,
    pub full_type: String,
    pub value: String,
    pub children: Vec<DebugNode>,
}

#[derive(Debug)]
pub struct LineVars {
    pub line_num: usize,
    // Use a Vec to retain the order
    pub nodes: Vec<DebugNode>,
}

#[derive(Debug, Clone)]
struct OutputParseError(String);
impl std::fmt::Display for OutputParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to parse output: {}", self.0)
    }
}
impl std::error::Error for OutputParseError {}

static BASE64_ENGINE: general_purpose::GeneralPurpose = general_purpose::STANDARD;

pub fn parse_program_output(output: Vec<String>) -> Result<Vec<LineVars>, Box<dyn std::error::Error>> {
    let cleaned = clean_program_output(output);
    let mut result: Vec<LineVars> = Vec::new();
    let mut it = cleaned.into_iter();
    while let Some(line) = it.next() {
        if !line.starts_with("-.!;LINE_START") {
            continue;
        }
        let split: Vec<&str> = line.split(';').collect();
        if split.len() != 3 {
            Err(OutputParseError(format!("Invalid line: {}", line)))?
        }
        let line_num: usize = split[2].parse()?;
        let mut line_block: VecDeque<String> = VecDeque::new();
        loop {
            match it.next() {
                Some(line_2) => {
                    if line_2 == "-.!;LINE_END" {
                        break;
                    }
                    line_block.push_back(line_2);
                }
                _ => Err(OutputParseError("LINE_END not found".into()))?
            }
        }
        let line_nodes = parse_line_vars_nodes(line_block)?;
        result.push(LineVars {
            line_num,
            nodes: line_nodes,
        })
    }
    /*
    for r in &result {
        eprintln!("LineVars: line_num: {}, nodes length: {}", r.line_num, r.nodes.len());
    }
     */

    Ok(result)
}

fn clean_program_output(output: Vec<String>) -> Vec<String> {
    let mut result = vec![];
    for line in output {
        if !(line.starts_with("Program log:") || line.starts_with("Program data:")) {
            continue;
        }
        let line = line.replacen("Program log: ", "", 1).replacen("Program data: ", "", 1);
        result.push(line);
    }
    result
}

fn parse_line_vars_nodes(mut input: VecDeque<String>) -> Result<Vec<DebugNode>, OutputParseError> {
    //dbg!(&input);
    let mut result = Vec::new();
    while !input.is_empty() {
        result.push(consume_debug_node(&mut input)?);
    }
    Ok(result)
}

fn consume_debug_node(lines: &mut VecDeque<String>) -> Result<DebugNode, OutputParseError> {
    match lines.pop_front() {
        None => Err(OutputParseError("Not enough lines".into()))?,
        Some(v) => {
            if v != "START_NODE" {
                Err(OutputParseError(format!("Invalid value: {}", v)))?
            }
        }
    }

    let node_type = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
    let node_type = match node_type.as_str() {
        "complex" => DebugNodeType::Complex,
        "primitive" => DebugNodeType::Primitive,
        _ => Err(OutputParseError(format!("Invalid node type: {}", node_type)))?,
    };
    let name = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
    let full_type = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
    let ser_type = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;

    let value = match ser_type.as_str() {
        "not_implemented" => "[not implemented]".to_string(),
        "int" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            let byte_arr: [u8; 16] = decoded.try_into().map_err(|_| OutputParseError("Decode error".into()))?;
            let integer = i128::from_le_bytes(byte_arr);
            integer.to_string()
        }
        "uint" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            let byte_arr: [u8; 16] = decoded.try_into().map_err(|_| OutputParseError("Decode error".into()))?;
            let integer = u128::from_le_bytes(byte_arr);
            integer.to_string()
        }
        "bool" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            if decoded.len() != 1 {
                Err(OutputParseError("Decode error: Invalid length".into()))?
            }
            let byte = decoded[0];
            let bool_val: bool = byte == 1;
            bool_val.to_string()
        }
        "str" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            format!(r#""{}""#, data_line)
        }
        "str_ident" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            data_line
        }
        "error_str" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            format!(r#"Error: {}"#, data_line)
        }
        "no_data" => {
            "".to_string()
            //"[empty]".to_string()
        }
        "rc_meta" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            let byte_arr: [u8; 16] = decoded.try_into().map_err(|_| OutputParseError("Decode error".into()))?;
            let integer = u128::from_le_bytes(byte_arr);
            let strong_count = integer.to_string();

            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            let byte_arr: [u8; 16] = decoded.try_into().map_err(|_| OutputParseError("Decode error".into()))?;
            let integer = u128::from_le_bytes(byte_arr);
            let weak_count = integer.to_string();

            format!("strong_count={}, weak_count={}", strong_count, weak_count)
        }
        "array_len" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            let byte_arr: [u8; 16] = decoded.try_into().map_err(|_| OutputParseError("Decode error".into()))?;
            let integer = u128::from_le_bytes(byte_arr);
            let len = integer.to_string();

            format!("len={}", len)
        }
        "pubkey" => {
            let data_line = lines.pop_front().ok_or(OutputParseError("Not enough lines".into()))?;
            let decoded = BASE64_ENGINE.decode(data_line).map_err(|_| OutputParseError("Decode error".into()))?;
            let byte_arr: [u8; 32] = decoded.try_into().map_err(|_| OutputParseError("Decode error".into()))?;
            let pubkey = Pubkey::from(byte_arr);

            pubkey.to_string()
        }
        x => {
            Err(OutputParseError(format!("Unimplemented: {}", x)))?
        }
    };

    let mut children = Vec::<DebugNode>::new();
    let mut inc_index = 0;
    while lines[0] == "START_NODE" {
        let mut child = consume_debug_node(lines)?;
        if child.name == "-inc-index" {
            child.name = inc_index.to_string();
            inc_index += 1
        }
        children.push(child);
    }

    match lines.pop_front() {
        None => Err(OutputParseError("Not enough lines".into()))?,
        Some(v) => {
            if v != "END_NODE" {
                Err(OutputParseError(format!("Invalid value: {}", v)))?
            }
        }
    }

    let node = DebugNode {
        node_type,
        name,
        full_type,
        value,
        children,
    };
    Ok(node)
}
