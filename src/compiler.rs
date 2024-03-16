//! TTKTK - TTK-91 ToolKit
//! SPDX-License-Identifier: MPL-2.0
//!
//! TiToMachine k91 assembler.
//!
mod instruction;

use std::collections::HashMap;
use std::str::FromStr;
use crate::compiler::instruction::{OpCode, parse_instruction, Register};

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
    let mut code_segment: Vec<i32> = Vec::new();

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
    let code_size = get_code_segment_size(&statements);
    code_symbols = parse_code_symbols(&statements);

    for statement in statements {
        if statement.statement_type == Keyword::Code {
            // TODO: symbol maps could be merged into one to make this function signature neater.
            code_segment.push(parse_instruction(
                statement,
                org,
                &const_symbols,
                &code_symbols,
                &data_symbols,
                code_size,
            )?);
        }
    }

    // Mash them together
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

    if let Ok(_) = OpCode::from_str(keyword) {
        return Keyword::Code;
    }
    if let Ok(_) = Register::from_str(keyword) {
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
    #[allow(dead_code)] // Comments will be added to the output, eventually.
    pub comment: Option<String>,
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