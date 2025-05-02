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

pub fn parse_program_output(output: Vec<String>) -> Result<Vec<LineVars>, Box<dyn std::error::Error>> {
    let cleaned = clean_program_output(output);
    let mut result: Vec<LineVars> = Vec::new();
    let mut it = cleaned.into_iter();
    while let Some(line) = it.next() {
        if !line.starts_with("-.!;LINE_START") {
            continue;
        }
        let split: Vec<&str> =  line.split(';').collect();
        if split.len() != 3 {
            Err(format!("Invalid line: {}", line))?
        }
        let line_num: usize = split[2].parse()?;
        let mut line_block: Vec<String> = vec![];
        loop {
            match it.next() {
                Some(line_2) => {
                    if line_2 == "-.!;LINE_END" {
                        break;
                    }
                    line_block.push(line_2);
                },
                None => Err("LINE_END not found")?
            }
        }
        result.push(LineVars {
            line_num,
            nodes: vec![]
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
        if ! (line.starts_with("Program log:") || line.starts_with("Program data:")) {
            continue;
        }
        let line = line.replacen("Program log: ", "", 1).replacen("Program data: ", "", 1);
        result.push(line);
    }
    result
}
