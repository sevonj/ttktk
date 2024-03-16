//! TTKTK - TTK-91 ToolKit
//! TiToMachine compiler.
//!
//!
use std::collections::HashMap;
use num_traits::ToPrimitive;

#[allow(dead_code)] // TODO: Not checked for anymore. Should be checked for symbol names.
const FORBIDDEN_CHARS: [char; 6] = [
    '(', ')', // parentheses
    '@', '=', // mode signs
    '-', // minus
    ':', // what was colon used for, again?
];

#[derive(PartialEq)]
enum Keyword {
    Directive,
    Constant,
    Data,
    Code,
    Register,
    None,
}

#[derive(Copy, Clone)]
enum Instruction {
    // Standard
    NOP = 0x00,
    STORE = 0x01,
    LOAD = 0x02,
    IN = 0x03,
    OUT = 0x04,
    ADD = 0x11,
    SUB = 0x12,
    MUL = 0x13,
    DIV = 0x14,
    MOD = 0x15,
    AND = 0x16,
    OR = 0x17,
    XOR = 0x18,
    SHL = 0x19,
    SHR = 0x1A,
    NOT = 0x1B,
    SHRA = 0x1C,
    COMP = 0x1F,
    JUMP = 0x20,
    JNEG = 0x21,
    JZER = 0x22,
    JPOS = 0x23,
    JNNEG = 0x24,
    JNZER = 0x25,
    JNPOS = 0x26,
    JLES = 0x27,
    JEQU = 0x28,
    JGRE = 0x29,
    JNLES = 0x2A,
    JNEQU = 0x2B,
    JNGRE = 0x2C,
    CALL = 0x31,
    EXIT = 0x32,
    PUSH = 0x33,
    POP = 0x34,
    PUSHR = 0x35,
    POPR = 0x36,
    SVC = 0x70,

    // Extended
    IEXIT = 0x39,
    HLT = 0x71,
    HCF = 0x72,
}

pub fn compile(source: String) -> Result<String, String> {

    // Start address. Zero if none.
    let mut org: Option<usize> = None;

    // Dictionary of const names and their valuers.
    let const_symbols: HashMap<String, i16>;

    // Dictionaries of labels and their offsets in their respective segments.
    let data_symbols: HashMap<String, usize>;
    let code_symbols: HashMap<String, usize>;

    // These contain source processed into integers.
    let data_segment: Vec<i32>;
    let code_segment: Vec<i32>;

    // Source code distilled into "Statement" structs.
    let mut statements;
    match code_to_statements(&source) {
        Ok(val) => statements = val,
        Err(e) => return Err(e)
    }

    // Guard: Multiple definition
    match assert_no_multiple_definition(&statements) {
        Ok(()) => {}
        Err(e) => return Err(e)
    }

    // Get directives
    for statement in &statements {
        if statement.statement_type != Keyword::Directive {
            continue;
        }
        let keyword = statement.words[0].as_str();
        match keyword {
            "ORG" => {
                if org != None {
                    return Err(format!("Found 'ORG' on line {}, but it's already defined!", statement.line));
                }
                org = Some(parse_org_directive(statement)?);
            }
            _ => return Err(format!("Compiler made an error on line {}: {} is not a directive.", statement.line, keyword))
        }
    }

    // Get constants
    match parse_const_statements(&statements) {
        Ok(result) => const_symbols = result,
        Err(e) => return Err(e)
    }

    // Get variables
    match parse_data_statements(&mut statements) {
        Ok((seg, symbols)) => {
            data_segment = seg;
            data_symbols = symbols;
        }
        Err(e) => return Err(e)
    }

    // Get code
    code_symbols = parse_code_symbols(&statements);
    match parse_code_statements(&mut statements, org, &const_symbols, &data_symbols, &code_symbols) {
        Ok(seg) => {
            code_segment = seg;
        }
        Err(e) => return Err(e)
    }

    let binary;
    match build_b91(
        code_segment,
        data_segment,
        code_symbols,
        data_symbols,
        org,
    ) {
        Ok(result) => binary = result,
        Err(e) => return Err(e)
    }
    Ok(binary)
}

/// This will Find all relevant source code lines, and break them into "Statements"
fn code_to_statements(source: &String) -> Result<Vec<Statement>, String> {
    let mut statements: Vec<Statement> = Vec::new();

    for (i, text) in source.lines().enumerate() {
        let mut text = text.to_owned();

        let statement_type: Keyword;
        let line = i + 1;
        let label: Option<String>;
        let comment: Option<String>;

        // Get comment and remove it from the text line
        match text.split_once(';') {
            Some((before, after)) => {
                comment = Some(after.to_string());
                text = before.to_owned();
            }
            None => comment = None,
        }

        // Split the text line into words
        text = text.replace(",", " ");
        let mut words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
        if words.is_empty() {
            continue;
        }

        // Get label and remove it from keywords
        if str_to_keyword_type(&words[0]) == Keyword::None {
            label = Some(words[0].to_owned());
            words.remove(0);
        } else {
            label = None
        }

        // Find the statement's type by looking at the first word.
        let keyword_string = words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        match str_to_keyword_type(keyword) {
            Keyword::None => {
                return Err(format!("Unknown keyword '{}' on line {}\n{}", keyword, line, text));
            }
            Keyword::Register => {
                return Err(format!("Unexpected register '{}' on line {}\n{}", keyword, line, text));
            }
            Keyword::Directive => statement_type = Keyword::Directive,
            Keyword::Data => statement_type = Keyword::Data,
            Keyword::Constant => statement_type = Keyword::Constant,
            Keyword::Code => statement_type = Keyword::Code,
        }

        // Make a statement.
        statements.push(Statement {
            statement_type,
            words,
            line: i + 1,
            label,
            comment,
        })
    }
    return Ok(statements);
}

fn parse_org_directive(statement: &Statement) -> Result<usize, String> {
    let value;
    let keyword_string = statement.words[0].to_uppercase();
    let keyword = keyword_string.as_str();
    let line = statement.line;

    // Guard: Label
    if !statement.label.is_none() {
        return Err(format!("You can't label a compiler directive! '{}' on line {}", keyword, line));
    }

    // Guard: Incorrect number of words
    match statement.words.len() {
        2 => (), // expected amount
        1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
        _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
    }

    // Get value
    match str_to_integer(&statement.words[1]) {
        Ok(val) => value = val,
        Err(e) => return Err(format!("Can't parse value on line {}: {}", line, e))
    }

    // Guard: Value out of range
    if value < 0 {
        return Err(format!("You tried to offset the program to a negative address! '{}' on line {}", keyword, line));
    }

    // Ok.
    Ok(value as usize)
}

/// This will create a dictionary of all constants (keyword EQU).
/// Note: Do check for multiple definitions _before_ this.
fn parse_const_statements(statements: &Vec<Statement>) -> Result<HashMap<String, i16>, String> {
    let mut consts: HashMap<String, i16> = HashMap::new();
    for statement in statements {
        if statement.statement_type != Keyword::Constant {
            continue;
        }
        let keyword_string = statement.words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        let line = statement.line;
        let value;

        // Guard: Keyword sanity check
        if keyword != "EQU" {
            return Err(format!("Line {}: '{}' is not 'EQU'. This is compiler's fault, not yours. Please file an issue.", line, keyword));
        }

        // Guard: No label
        if statement.label.is_none() {
            return Err(format!("Constants require a name! '{}' on line {}", keyword, line));
        }
        let label = statement.label.clone().unwrap();

        // Guard: Incorrect number of words
        match statement.words.len() {
            2 => (), // expected amount
            1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
            _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
        }

        // Get value
        match str_to_integer(&statement.words[1]) {
            Ok(val) => value = val,
            Err(e) => return Err(format!("Error parsing value on line {}: {}", line, e))
        }

        // Guard: Value out of range
        if value < i16::MIN as i32 {
            return Err(format!("Value out of range on line {}. Got {}, but minimum is {}. Note that constants are 16-bit only.", line, value, i16::MIN));
        } else if value > i16::MAX as i32 {
            return Err(format!("Value out of range on line {}. Got {}, but maximum is {}. Note that constants are 16-bit only.", line, value, i16::MAX));
        }

        // Done
        consts.insert(label, value as i16);
    }
    return Ok(consts);
}

/// Creates data segment and data symbols
fn parse_data_statements(
    statements: &mut Vec<Statement>)
    -> Result<(Vec<i32>, HashMap<String, usize>), String>
{
    let mut data_segment = Vec::new();
    let mut data_symbols = HashMap::new();

    for statement in statements {
        if statement.statement_type != Keyword::Data {
            continue;
        }

        let keyword_string = statement.words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        let line = statement.line;
        let value;

        // Guard: Word count
        match statement.words.len() {
            2 => (), // expected amount
            1 => return Err(format!("No value given for '{}' on line {}", keyword, line)),
            _ => return Err(format!("Too many words for '{}' on line {}", keyword, line)),
        }

        // Get value
        match str_to_integer(&statement.words[1]) {
            Ok(val) => value = val,
            Err(e) => return Err(format!("Error parsing value on line {}: {}", line, e))
        }

        match keyword {
            // Data Constant - store a value
            "DC" => {
                // Add symbol, if labeled
                if let Some(label) = &statement.label {
                    data_symbols.insert(label.clone(), data_segment.len());
                }

                // Push data
                data_segment.push(value);
            }
            // Data Segment - allocate space
            "DS" => {
                // Guard: out of range
                if value < 0 {
                    return Err(format!("You tried to allocate a negative number of addresses! '{}' on line {}", keyword, line));
                } else if value == 0 {
                    return Err(format!("You tried to allocate a zero addresses! '{}' on line {}", keyword, line));
                }

                // Add symbol, if labeled
                if let Some(label) = &statement.label {
                    data_symbols.insert(label.clone(), data_segment.len());
                }

                // Push data
                for _ in 0..value {
                    data_segment.push(0);
                }
            }
            _ => return Err(format!("Error: '{}' on line {} is not a variable keyword. This is compiler's fault, not yours. Please file an issue.", keyword, line)),
        }
    }
    Ok((data_segment, data_symbols))
}

/// Before actually parsing the code, we need to know possible code labels the code might reference.
fn parse_code_symbols(statements: &Vec<Statement>) -> HashMap<String, usize> {
    let mut code_symbols = HashMap::new();
    let mut code_offset = 0;
    for statement in statements {
        if statement.statement_type != Keyword::Code {
            continue;
        }
        // Add symbol, if labeled
        if let Some(label) = &statement.label {
            code_symbols.insert(label.clone(), code_offset);
        }
        code_offset += 1;
    }
    code_symbols
}

fn get_code_segment_size(statements: &Vec<Statement>) -> usize {
    let mut code_offset = 0;
    for statement in statements {
        if statement.statement_type != Keyword::Code {
            continue;
        }
        code_offset += 1;
    }
    code_offset
}

/// Creates code segment and code symbols
fn parse_code_statements(
    statements: &mut Vec<Statement>,
    org: Option<usize>,
    const_symbols: &HashMap<String, i16>,
    data_symbols: &HashMap<String, usize>,
    code_symbols: &HashMap<String, usize>,
) -> Result<Vec<i32>, String>
{
    let mut code_segment = Vec::new();
    let code_size = get_code_segment_size(statements);

    for statement in statements {
        if statement.statement_type != Keyword::Code {
            continue;
        }

        let org = org.unwrap_or(0);
        let mut words = statement.words.clone();
        let keyword_string = statement.words[0].to_uppercase();
        let keyword = keyword_string.as_str();
        // Remove oper keyword
        words.remove(0);
        let line = statement.line;
        let mut value: i32;

        // indirect memory access.
        // Normally addressing mode goes like this:
        // "=" => 0,
        // " " => 1,
        // "@" => 2
        // with some instructions indirect addressing is disabled
        // "=" is not allowed,
        // " " => 0,
        // "@" => 1

        // Operands. Most instructions use the defaults, but not all.
        let mut op1: String = "".to_string();
        let mut op2: String = "".to_string();

        if words.len() >= 1 {
            op1 = words[0].clone();
        }
        if words.len() >= 2 {
            op2 = words[1].clone();
        }

        let opcode: i32;
        let ri: i32;
        let mut mode: i32 = 1;
        let rj: i32;
        let addr: i32;

        match keyword {
            "NOP" => {
                assert_op_count(words.len(), 0, line)?;
                opcode = Instruction::NOP as i32;
            }
            "STORE" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::STORE as i32;
                mode -= 1; // no indirect
            }
            "LOAD" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::LOAD as i32;
            }
            "IN" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::IN as i32;
            }
            "OUT" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::OUT as i32;
            }
            "ADD" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::ADD as i32;
            }
            "SUB" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SUB as i32;
            }
            "MUL" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::MUL as i32;
            }
            "DIV" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::DIV as i32;
            }
            "MOD" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::MOD as i32;
            }
            "AND" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::AND as i32;
            }
            "OR" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::OR as i32;
            }
            "XOR" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::XOR as i32;
            }
            "SHL" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SHL as i32;
            }
            "SHR" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SHR as i32;
            }
            "NOT" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::NOT as i32;
            }
            "SHRA" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SHRA as i32;
            }
            "COMP" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::COMP as i32;
            }
            "JUMP" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JUMP as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNEG" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNEG as i32;
                mode -= 1; // no indirect
            }
            "JZER" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JZER as i32;
                mode -= 1; // no indirect
            }
            "JPOS" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JPOS as i32;
                mode -= 1; // no indirect
            }
            "JNNEG" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNNEG as i32;
                mode -= 1; // no indirect
            }
            "JNZER" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNZER as i32;
                mode -= 1; // no indirect
            }
            "JNPOS" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::JNPOS as i32;
                mode -= 1; // no indirect
            }
            "JLES" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JLES as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JEQU" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JEQU as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JGRE" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JGRE as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNLES" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JNLES as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNEQU" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JNEQU as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "JNGRE" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::JNGRE as i32;
                // Special case: op2 is used, but op1 isn't.
                op2 = op1;
                op1 = "".to_string();
                mode -= 1; // no indirect
            }
            "CALL" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::CALL as i32;
                mode -= 1; // no indirect
            }
            "EXIT" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::EXIT as i32;
            }
            "PUSH" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::PUSH as i32;
            }
            "POP" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::POP as i32;
            }
            "PUSHR" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::PUSHR as i32;
            }
            "POPR" => {
                assert_op_count(words.len(), 1, line)?;
                opcode = Instruction::POPR as i32;
            }
            "SVC" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::SVC as i32;
            }
            // Extended
            "IEXIT" => {
                assert_op_count(words.len(), 2, line)?;
                opcode = Instruction::IEXIT as i32;
            }
            "HLT" => {
                assert_op_count(words.len(), 0, line)?;
                opcode = Instruction::HLT as i32;
            }
            "HCF" => {
                assert_op_count(words.len(), 0, line)?;
                opcode = Instruction::HCF as i32;
            }
            _ => return Err(format!("Compiler error: {} is not an instruction. Please file an issue.", keyword)),
        }

        op1 = op1.to_uppercase();

        // Parse op1: Rj
        if op1.is_empty() {
            rj = 0;
        } else if let Ok(val) = str_to_register(op1.as_str()) {
            rj = val
        } else {
            return Err(format!("Line {}: Invalid register '{}' for first operand!", line, op1));
        }

        // Parse op2: Ri, mode, addr
        if op2.is_empty() {
            ri = 0;
            addr = 0;
        } else if let Ok(parsed) = parse_op2(op2.as_str()) {
            // Mode
            mode += parsed.mode;
            // Register
            match str_to_register(parsed.register.as_str()) {
                Ok(val) => ri = val,
                Err(_) => return Err(format!("Line {}: Invalid register '{}' for second operand!", line, op2)),
            }
            // Address (is empty)
            if parsed.addr.as_str() == "" {
                addr = 0;
            }
            // Address (is builtin const)
            else if let Ok(val) = str_to_builtin_const(&parsed.addr) {
                addr = val;
            }
            // Address (is const)
            else if let Some(val) = const_symbols.get(&parsed.addr) {
                addr = val.to_i32().unwrap();
            }
            // Address (is variable)
            else if let Some(offset) = data_symbols.get(&parsed.addr) {
                addr = (org + code_size + offset).to_i32().unwrap();
            }
            // Address (is code)
            else if let Some(offset) = code_symbols.get(&parsed.addr) {
                addr = (org + offset).to_i32().unwrap();
            }
            // Address (is number)
            else if let Ok(val) = str_to_integer(parsed.addr.as_str()) {
                addr = val;
            }
            // Address (is invalid)
            else {
                return Err(format!("Line {}: invalid address: {}", line, parsed.addr));
            }
        } else {
            return Err(format!("Line {}: Couldn't parse second operand: {}", line, op2));
        }

        value = opcode << 24;
        value += rj << 21;
        value += mode << 19;
        value += ri << 16;
        match i16::try_from(addr) {
            Ok(val) => value += (val as i32) & 0xffff,
            Err(_) => {
                match u16::try_from(addr) {
                    Ok(val) => value += (val as i32) & 0xffff,
                    Err(_) => return Err(format!("Compiler error: can't fit addr '{}' into 16 bits on line {}", addr, line))
                }
            }
        }

        code_segment.push(value);
    }
    Ok(code_segment)
}

/// This is a shortcut to make the assertion a oneliner in parse_code_statements.
fn assert_op_count(n: usize, m: usize, ln: usize) -> Result<(), String> {
    if n != m {
        return Err(format!("Line {}: Too many terms!", ln));
    }
    Ok(())
}

/// Go through statements and check if same label comes up more than once.
fn assert_no_multiple_definition(statements: &Vec<Statement>) -> Result<(), String> {
    let mut failed = false;
    let mut definitions: HashMap<String, Vec<usize>> = HashMap::new();
    // Collect all definitions
    for statement in statements {
        if let Some(label) = &statement.label {
            if definitions.contains_key(label) {
                // Defined already! mark failed add this line to the entry.
                failed = true;
                definitions.get_mut(label).unwrap().push(statement.line);
            } else {
                // First definition: create an entry that contains this line.
                let mut vec: Vec<usize> = Vec::new();
                vec.push(statement.line);
                definitions.insert(label.clone(), vec);
            }
        }
    }
    // Failure: Construct an error message
    if failed {
        let mut err_mgs = "Multiple definitions:".to_string();
        for (label, lines) in definitions {
            if lines.len() > 1 {
                err_mgs += format!("\n    {} on lines: {:?}", label, lines).as_str()
            }
        }
        return Err(err_mgs);
    }
    // Success
    Ok(())
}

fn build_b91(
    code_segment: Vec<i32>,
    data_segment: Vec<i32>,
    code_symbols: HashMap<String, usize>,
    data_symbols: HashMap<String, usize>,
    org: Option<usize>,
) -> Result<String, String>
{
    let org = org.unwrap_or(0);
    let code_size = code_segment.len();
    let fp_start: i32 = (org + code_size) as i32 - 1; // fp_start can be -1 if code_size == 0
    let data_start = code_size + org;
    let sp_start = fp_start + data_segment.len() as i32;

    let mut return_str = "___b91___\n".to_string();

    // --- Code segment
    return_str += "___code___\n";
    // Code start and FP
    return_str += format!("{} {}\n", org.to_string(), fp_start.to_string()).as_str();
    // Actual code
    for i in code_segment {
        return_str += format!("{}\n", i.to_string()).as_str();
    }

    // --- Data segment
    return_str += "___data___\n";
    // Data start and SP
    return_str += format!("{} {}\n", data_start.to_string(), sp_start.to_string()).as_str();
    // Actual data
    for i in data_segment {
        return_str += format!("{}\n", i.to_string()).as_str();
    }

    // --- Symbol table
    return_str += "___symboltable___\n";
    // Variables:
    for (label, offset) in data_symbols {
        let addr = data_start + offset;
        return_str += format!("{} {}\n", label, addr).as_str();
    }
    // Code labels
    for (label, offset) in code_symbols {
        let addr = org + offset;
        return_str += format!("{} {}\n", label, addr).as_str();
    }

    // --- End
    return_str += "___end___\n";

    Ok(return_str)
}

fn str_to_keyword_type(keyword: &str) -> Keyword {
    let keyword_string = keyword.to_uppercase();
    let keyword = keyword_string.as_str();

    if let Ok(_) = str_to_opcode(keyword) {
        return Keyword::Code;
    }
    if let Ok(_) = str_to_register(keyword) {
        return Keyword::Register;
    }
    if keyword == "EQU" {
        return Keyword::Constant;
    }
    if keyword == "DS" || keyword == "DC" {
        return Keyword::Data;
    }
    if keyword == "ORG" {
        return Keyword::Directive;
    }
    Keyword::None
}

fn str_to_register(keyword: &str) -> Result<i32, String> {
    match keyword.to_uppercase().as_str() {
        "R0" | "" => Ok(0),
        "R1" => Ok(1),
        "R2" => Ok(2),
        "R3" => Ok(3),
        "R4" => Ok(4),
        "R5" => Ok(5),
        "R6" | "SP" => Ok(6),
        "R7" | "FP" => Ok(7),
        _ => Err(format!("{} is not a register.", keyword)),
    }
}

fn str_to_opcode(keyword: &str) -> Result<i32, String> {
    let value;
    match keyword.to_uppercase().as_str() {
        "NOP" => value = Instruction::NOP,
        "STORE" => value = Instruction::STORE,
        "LOAD" => value = Instruction::LOAD,
        "IN" => value = Instruction::IN,
        "OUT" => value = Instruction::OUT,
        "ADD" => value = Instruction::ADD,
        "SUB" => value = Instruction::SUB,
        "MUL" => value = Instruction::MUL,
        "DIV" => value = Instruction::DIV,
        "MOD" => value = Instruction::MOD,
        "AND" => value = Instruction::AND,
        "OR" => value = Instruction::OR,
        "XOR" => value = Instruction::XOR,
        "SHL" => value = Instruction::SHL,
        "SHR" => value = Instruction::SHR,
        "NOT" => value = Instruction::NOT,
        "SHRA" => value = Instruction::SHRA,
        "COMP" => value = Instruction::COMP,
        "JUMP" => value = Instruction::JUMP,
        "JNEG" => value = Instruction::JNEG,
        "JZER" => value = Instruction::JZER,
        "JPOS" => value = Instruction::JPOS,
        "JNNEG" => value = Instruction::JNNEG,
        "JNZER" => value = Instruction::JNZER,
        "JNPOS" => value = Instruction::JNPOS,
        "JLES" => value = Instruction::JLES,
        "JEQU" => value = Instruction::JEQU,
        "JGRE" => value = Instruction::JGRE,
        "JNLES" => value = Instruction::JNLES,
        "JNEQU" => value = Instruction::JNEQU,
        "JNGRE" => value = Instruction::JNGRE,
        "CALL" => value = Instruction::CALL,
        "EXIT" => value = Instruction::EXIT,
        "PUSH" => value = Instruction::PUSH,
        "POP" => value = Instruction::POP,
        "PUSHR" => value = Instruction::PUSHR,
        "POPR" => value = Instruction::POPR,
        "SVC" => value = Instruction::SVC,

        "IEXIT" => value = Instruction::IEXIT,
        "HLT" => value = Instruction::HLT,
        "HCF" => value = Instruction::HCF,
        _ => return Err(format!("{} is not an instruction.", keyword)),
    }
    Ok(value as i32)
}

fn str_to_builtin_const(sym: &str) -> Result<i32, String> {
    match sym {
        "SHRT_MAX" => Ok(32767),
        "SHRT_MIN" => Ok(-32768),
        "USHRT_MAX" => Ok(65535),
        "INT_MAX" => Ok(2147483647),
        "INT_MIN" => Ok(-2147483648),
        "UINT_MAX" => Ok(-1),

        "CRT" => Ok(0),
        "KBD" => Ok(1),
        "RTC" => Ok(2),
        "HALT" => Ok(11),
        "READ" => Ok(12),
        "WRITE" => Ok(13),
        "TIME" => Ok(14),
        "DATE" => Ok(15),
        _ => Err(format!("{} is not a builtin constant.", sym)),
    }
}

fn str_to_integer(input_string: &str) -> Result<i32, String> {
    let mut num_string: String = input_string.to_lowercase();

    // Catch minus sign
    let minus = input_string.starts_with('-');
    if minus {
        num_string = input_string.chars().into_iter().skip(1).collect();
    }

    // Catch Bin/Oct/Hex prefix
    let prefix: String = num_string.chars().into_iter().take(2).collect();
    let radix = match prefix.as_str() {
        "0b" => 2,
        "0o" => 8,
        "0x" => 16,
        _ => 10,
    };
    if radix != 10 {
        num_string = num_string.chars().skip(2).collect();
    }

    // u32 and then cast to i32 because from_str_radix doesn't seem to understand two's complement
    let value: i32;
    match u32::from_str_radix(num_string.as_str(), radix) {
        Ok(int) => value = int as i32,
        Err(e) => return Err(format!("{}: '{}'", e, input_string)),
    }

    match minus {
        true => Ok(-value),
        false => Ok(value),
    }
}

/// One of the first things that happens to a line of code is to be organized into this struct.
/// Statement holds the code as Vec<String>, and knows some high-level information and metadata
/// about it.
struct Statement {
    pub statement_type: Keyword,
    pub label: Option<String>,
    //
    pub words: Vec<String>,
    // Remaining keywords after label
    pub line: usize,
    #[allow(dead_code)] // Comments will be used eventually.
    pub comment: Option<String>,
}

/// Used by parse_op2()
struct Op2 {
    pub mode: i32,
    pub addr: String,
    pub register: String,
}

/// Parse second operand: "=123(R2)"
/// Note: Addressing mode in the return is _not final!_
/// This means that the mode must be _added_ to the instruction's default mode, which is usually 1,
/// and sometimes 0.
fn parse_op2(input_str: &str) -> Result<Op2, String> {
    let mut mode: i32 = 0;
    let mut addr = String::new();
    let mut register = String::new();
    //let mut chars = input_str.chars();

    let mut text = input_str.to_string();

    // Catch mode sign
    if input_str.starts_with("=") {
        mode = -1;
        text.remove(0);
    } else if input_str.starts_with("@") {
        mode = 1;
        text.remove(0);
    }

    // Catch minus sign
    if input_str.starts_with("-") {
        addr += "-";
        text.remove(0);
    }

    // Second operand text _is_ a register, no address.
    if let Ok(_) = str_to_register(text.as_str()) {
        // Do not allow negative direct register addressing "-R1"
        if addr.as_str() == "-" {
            return Err(format!("Negative direct register addressing '{}' is not allowed. The minus sign only affects address portion.", input_str));
        }
        register = text;
        return Ok(Op2 {
            mode: mode - 1, // Register only decrements because of direct reg addressing
            addr,
            register,
        });
    }

    // Second operand _contains_ register in parentheses
    if let Some((before_open, after_open)) = text.split_once('(') {
        match after_open.split_once(')') {
            Some((reg, after_close)) => {
                // We got a register string from between after_open and before_close
                register = reg.to_uppercase();

                // Err: There's stuff on both sides of the parentheses!
                if !before_open.is_empty() && !after_close.is_empty() {
                    return Err(format!("Failed to parse second operand: '{}'", input_str));
                }
                // Nothing outside parentheses; we're done
                if before_open.is_empty() && after_close.is_empty() {
                    return Ok(Op2 {
                        mode,
                        addr,
                        register,
                    });
                }
                // One side is empty and one is not.
                text = before_open.to_string() + after_close;
            }
            None => return Err("Unclosed parentheses".to_string())
        }
    }

    // _No register_ in second operand. It's just address.
    addr += text.as_str();
    Ok(Op2 {
        mode,
        addr,
        register,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(str_to_integer("0b110100").unwrap(), 52);
        assert_eq!(str_to_integer("0o64").unwrap(), 52);
        assert_eq!(str_to_integer("52").unwrap(), 52);
        assert_eq!(str_to_integer("0x34").unwrap(), 52);

        assert_eq!(str_to_integer("-0b1").unwrap(), -1);
        assert_eq!(str_to_integer("-0o1").unwrap(), -1);
        assert_eq!(str_to_integer("-1").unwrap(), -1);
        assert_eq!(str_to_integer("-0x1").unwrap(), -1);

        // Treat as unsigned if no minus sign.
        assert_eq!(str_to_integer("0b11111111111111111111111111111111").unwrap(), -1);
        assert_eq!(str_to_integer("0o37777777777").unwrap(), -1);
        assert_eq!(str_to_integer("4294967295").unwrap(), -1);
        assert_eq!(str_to_integer("0xffffffff").unwrap(), -1);
    }

    #[test]
    fn test_parse_number_incorrect() {
        assert!(str_to_integer("0xabcdefg").is_err());
        assert!(str_to_integer("0o2345678").is_err());
        assert!(str_to_integer("0b012").is_err());
        assert!(str_to_integer("0b00a").is_err());

        assert!(str_to_integer("--0b1").is_err());
        assert!(str_to_integer("-=0o1").is_err());
        assert!(str_to_integer("-@1").is_err());
        assert!(str_to_integer("-0x0x1").is_err());
        assert!(str_to_integer("-0x0o1").is_err());
        assert!(str_to_integer("-x01").is_err());

        // Treat as unsigned if no minus sign.
        assert!(str_to_integer("0b111111111111111111111111111111111").is_err());
        assert!(str_to_integer("0o377777777777").is_err());
        assert!(str_to_integer("4294967296").is_err());
        assert!(str_to_integer("0xfffffffff").is_err());
    }

    /*
    Addressing modes require some careful testing.

        Code                    Desired effect  Examples, explanation
        - "=" prefix            decrement
        - "@" prefix            increment
        - no address            decrement       "R1", "R0"
        - no addr, neg reg      unknown         "-R1", "-R0"    (Titokone refuses to compile, thinking it's label)
        - @@0                   illegal         Would get mode 2 on store command. Is illegal and should be illegal.
        - ==0                   illegal         Would get decrement by 2 Is illegal and should be illegal.

        Other:
        - Reg in parentheses implies presence of address 0: "(R1)" == "0(R1)"
        - Mode sign must be the first character. "=-1" is OK. "-=1" should fail.

     */
    #[test]
    /// Go through all combinations of mode signs
    /// This needs testing, definition, and fixing.
    fn test_parse_op2_mode_sign() {
        // Very basic
        assert_eq!(parse_op2("=0").unwrap().mode, -1);  // Immediate value
        assert_eq!(parse_op2("0").unwrap().mode, 0);    // Direct memory
        assert_eq!(parse_op2("@0").unwrap().mode, 1);   // Indirect memory

        assert_eq!(parse_op2("=1").unwrap().mode, -1);  // Immediate value
        assert_eq!(parse_op2("1").unwrap().mode, 0);    // Direct memory
        assert_eq!(parse_op2("@1").unwrap().mode, 1);   // Indirect memory

        assert_eq!(parse_op2("=55555").unwrap().mode, -1);  // Immediate value
        assert_eq!(parse_op2("55555").unwrap().mode, 0);    // Direct memory
        assert_eq!(parse_op2("@55555").unwrap().mode, 1);   // Indirect memory

        assert_eq!(parse_op2("=-55555").unwrap().mode, -1);  // Immediate value
        assert_eq!(parse_op2("-55555").unwrap().mode, 0);    // Direct memory
        assert_eq!(parse_op2("@-55555").unwrap().mode, 1);   // Indirect memory

        assert_eq!(parse_op2("=0x500").unwrap().mode, -1);  // Immediate value
        assert_eq!(parse_op2("0x500").unwrap().mode, 0);    // Direct memory
        assert_eq!(parse_op2("@0x500").unwrap().mode, 1);   // Indirect memory
    }

    #[test]
    /// Only the first character should count as a mode sign.
    fn test_parse_op2_mode_sign_first() {
        assert_eq!(parse_op2("=1").unwrap().mode, -1);
        assert_eq!(parse_op2("@1").unwrap().mode, 1);

        // The sign should not affect mode and should not be removed from the string.
        assert_eq!(parse_op2("-=1").unwrap().mode, 0);
        assert_eq!(parse_op2("-=1").unwrap().addr, "-=1");
        assert_eq!(parse_op2("-@1").unwrap().mode, 0);
        assert_eq!(parse_op2("-@1").unwrap().addr, "-@1");

        assert_eq!(parse_op2("0=1").unwrap().mode, 0);
        assert_eq!(parse_op2("0=1").unwrap().addr, "0=1");
        assert_eq!(parse_op2("0@1").unwrap().mode, 0);
        assert_eq!(parse_op2("0@1").unwrap().addr, "0@1");

        // First mode sign should count and be removed, but not the second
        assert_eq!(parse_op2("==1").unwrap().mode, -1);
        assert_eq!(parse_op2("==1").unwrap().addr, "=1");
        assert_eq!(parse_op2("@@1").unwrap().mode, 1);
        assert_eq!(parse_op2("@@1").unwrap().addr, "@1");
        assert_eq!(parse_op2("=@1").unwrap().mode, -1);
        assert_eq!(parse_op2("=@1").unwrap().addr, "@1");
        assert_eq!(parse_op2("@=1").unwrap().mode, 1);
        assert_eq!(parse_op2("@=1").unwrap().addr, "=1");
    }

    #[test]
    /// Test everything that should result in indexed addressing (mode 0)
    fn test_parse_op2_mode_direct_register() {
        // No address: should decrement mode.
        assert_eq!(parse_op2("R0").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R1").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R2").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R3").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R4").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R5").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R6").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("R7").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("SP").unwrap().mode, -1);   // Direct register
        assert_eq!(parse_op2("FP").unwrap().mode, -1);   // Direct register
    }

    #[test]
    /// Minus sign only affects address. These shouldn't pass the compiler.
    fn test_parse_op2_direct_register_neg() {
        assert!(parse_op2("-R0").is_err());
        assert!(parse_op2("-R1").is_err());
        assert!(parse_op2("-FP").is_err());
    }

    #[test]
    /// Parentheses implies address: should not decrement mode.
    /// In practice, "(R1)" should be treated the same as "0(R1)"    fn test_parse_op2_mode_indexed_implied() {
    fn test_parse_op2_mode_indexed_implied() {
        assert_eq!(parse_op2("(R0)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("(R1)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("(R2)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("(SP)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("(FP)").unwrap().mode, 0);   // Indexed addressing
    }

    #[test]
    fn test_parse_op2_mode_indexed() {
        assert_eq!(parse_op2("0(R3)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0(R4)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0(R5)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0(R6)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0(R7)").unwrap().mode, 0);   // Indexed addressing

        assert_eq!(parse_op2("0x123(R0)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("-0x123(R1)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0b101(R2)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("-0b1010(R3)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0o1234(R4)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("-0o1234(R5)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0XaAbB(R6)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0O8887(R7)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("0B1010(SP)").unwrap().mode, 0);   // Indexed addressing
        assert_eq!(parse_op2("9999(FP)").unwrap().mode, 0);   // Indexed addressing
    }

    #[test]
    /// Indexed with mode sign
    fn test_parse_op2_mode_indexed_immediate() {
        assert_eq!(parse_op2("=(R0)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=(R1)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=(R2)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=0(R0)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=0(R1)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=0(R2)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=0x123(R0)").unwrap().mode, -1);   // Indexed immediate
        assert_eq!(parse_op2("=-0x123(R1)").unwrap().mode, -1);   // Indexed immediate
    }

    #[test]
    /// Indexed with mode sign
    fn test_parse_op2_mode_indexed_indirect() {
        assert_eq!(parse_op2("@(R7)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@(SP)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@(FP)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@0(R7)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@0(SP)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@0(FP)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@0b101(R2)").unwrap().mode, 1);   // Indexed indirect
        assert_eq!(parse_op2("@-0b1010(R3)").unwrap().mode, 1);   // Indexed indirect
    }

    #[test]
    fn test_cannot_redefine_const() {}

    #[test]
    fn test_cannot_redefine_var() {}

    #[test]
    fn test_cannot_redefine_code() {}

    #[test]
    /// Every combination of redefining a keyword with another type
    fn test_cannot_redefine_mixed() {}

    #[test]
    fn test_cannot_redefine_builtin_const() {}
}