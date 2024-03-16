//! TTKTK - TTK-91 ToolKit
//! SPDX-License-Identifier: MPL-2.0
//!
//! TiToMachine k91 assembler - Instruction parsing module.
//!
use std::collections::HashMap;
use std::str::FromStr;
use num_traits::ToPrimitive;
use crate::compiler::{Statement, str_to_builtin_const, str_to_integer};

#[derive(Copy, Clone)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
}

#[derive(Copy, Clone)]
pub enum OpCode {
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

impl FromStr for Register {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "R0" => Ok(Register::R0),
            "R1" => Ok(Register::R1),
            "R2" => Ok(Register::R2),
            "R3" => Ok(Register::R3),
            "R4" => Ok(Register::R4),
            "R5" => Ok(Register::R5),
            "R6" | "SP" => Ok(Register::R6),
            "R7" | "FP" => Ok(Register::R7),
            _ => Err(format!("{} is not a register.", s))
        }
    }
}

impl FromStr for OpCode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "NOP" => Ok(OpCode::NOP),
            "STORE" => Ok(OpCode::STORE),
            "LOAD" => Ok(OpCode::LOAD),
            "IN" => Ok(OpCode::IN),
            "OUT" => Ok(OpCode::OUT),
            "ADD" => Ok(OpCode::ADD),
            "SUB" => Ok(OpCode::SUB),
            "MUL" => Ok(OpCode::MUL),
            "DIV" => Ok(OpCode::DIV),
            "MOD" => Ok(OpCode::MOD),
            "AND" => Ok(OpCode::AND),
            "OR" => Ok(OpCode::OR),
            "XOR" => Ok(OpCode::XOR),
            "SHL" => Ok(OpCode::SHL),
            "SHR" => Ok(OpCode::SHR),
            "NOT" => Ok(OpCode::NOT),
            "SHRA" => Ok(OpCode::SHRA),
            "COMP" => Ok(OpCode::COMP),
            "JUMP" => Ok(OpCode::JUMP),
            "JNEG" => Ok(OpCode::JNEG),
            "JZER" => Ok(OpCode::JZER),
            "JPOS" => Ok(OpCode::JPOS),
            "JNNEG" => Ok(OpCode::JNNEG),
            "JNZER" => Ok(OpCode::JNZER),
            "JNPOS" => Ok(OpCode::JNPOS),
            "JLES" => Ok(OpCode::JLES),
            "JEQU" => Ok(OpCode::JEQU),
            "JGRE" => Ok(OpCode::JGRE),
            "JNLES" => Ok(OpCode::JNLES),
            "JNEQU" => Ok(OpCode::JNEQU),
            "JNGRE" => Ok(OpCode::JNGRE),
            "CALL" => Ok(OpCode::CALL),
            "EXIT" => Ok(OpCode::EXIT),
            "PUSH" => Ok(OpCode::PUSH),
            "POP" => Ok(OpCode::POP),
            "PUSHR" => Ok(OpCode::PUSHR),
            "POPR" => Ok(OpCode::POPR),
            "SVC" => Ok(OpCode::SVC),
            // Extended
            "IEXIT" => Ok(OpCode::IEXIT),
            "HLT" => Ok(OpCode::HLT),
            "HCF" => Ok(OpCode::HCF),
            _ => return Err(format!("{} is not an instruction.", s)),
        }
    }
}

impl OpCode {
    pub fn get_operand_count(&self) -> usize {
        match self {
            OpCode::NOP => 0,
            OpCode::STORE => 2,
            OpCode::LOAD => 2,
            OpCode::IN => 2,
            OpCode::OUT => 2,
            OpCode::ADD => 2,
            OpCode::SUB => 2,
            OpCode::MUL => 2,
            OpCode::DIV => 2,
            OpCode::MOD => 2,
            OpCode::AND => 2,
            OpCode::OR => 2,
            OpCode::XOR => 2,
            OpCode::SHL => 2,
            OpCode::SHR => 2,
            OpCode::NOT => 1,
            OpCode::SHRA => 2,
            OpCode::COMP => 2,
            OpCode::JUMP => 1,
            OpCode::JNEG => 2,
            OpCode::JZER => 2,
            OpCode::JPOS => 2,
            OpCode::JNNEG => 2,
            OpCode::JNZER => 2,
            OpCode::JNPOS => 2,
            OpCode::JLES => 1,
            OpCode::JEQU => 1,
            OpCode::JGRE => 1,
            OpCode::JNLES => 1,
            OpCode::JNEQU => 1,
            OpCode::JNGRE => 1,
            OpCode::CALL => 2,
            OpCode::EXIT => 2,
            OpCode::PUSH => 2,
            OpCode::POP => 2,
            OpCode::PUSHR => 1,
            OpCode::POPR => 1,
            OpCode::SVC => 2,
            // Extended
            OpCode::IEXIT => 2,
            OpCode::HLT => 0,
            OpCode::HCF => 0,
        }
    }
    pub fn get_default_mode(&self) -> i32 {
        match self {
            OpCode::NOP => 1,
            OpCode::STORE => 0,
            OpCode::LOAD => 1,
            OpCode::IN => 1,
            OpCode::OUT => 1,
            OpCode::ADD => 1,
            OpCode::SUB => 1,
            OpCode::MUL => 1,
            OpCode::DIV => 1,
            OpCode::MOD => 1,
            OpCode::AND => 1,
            OpCode::OR => 1,
            OpCode::XOR => 1,
            OpCode::SHL => 1,
            OpCode::SHR => 1,
            OpCode::NOT => 1,
            OpCode::SHRA => 1,
            OpCode::COMP => 1,
            OpCode::JUMP => 0,
            OpCode::JNEG => 0,
            OpCode::JZER => 0,
            OpCode::JPOS => 0,
            OpCode::JNNEG => 0,
            OpCode::JNZER => 0,
            OpCode::JNPOS => 0,
            OpCode::JLES => 0,
            OpCode::JEQU => 0,
            OpCode::JGRE => 0,
            OpCode::JNLES => 0,
            OpCode::JNEQU => 0,
            OpCode::JNGRE => 0,
            OpCode::CALL => 0,
            OpCode::EXIT => 1,
            OpCode::PUSH => 1,
            OpCode::POP => 1,
            OpCode::PUSHR => 1,
            OpCode::POPR => 1,
            OpCode::SVC => 1,
            // Extended
            OpCode::IEXIT => 1,
            OpCode::HLT => 1,
            OpCode::HCF => 1,
        }
    }

    /// Some jumps use op2 but not op1.
    pub fn is_op2_only(&self) -> bool {
        match self {
            OpCode::NOP => false,
            OpCode::STORE => false,
            OpCode::LOAD => false,
            OpCode::IN => false,
            OpCode::OUT => false,
            OpCode::ADD => false,
            OpCode::SUB => false,
            OpCode::MUL => false,
            OpCode::DIV => false,
            OpCode::MOD => false,
            OpCode::AND => false,
            OpCode::OR => false,
            OpCode::XOR => false,
            OpCode::SHL => false,
            OpCode::SHR => false,
            OpCode::NOT => false,
            OpCode::SHRA => false,
            OpCode::COMP => false,
            OpCode::JUMP => true,
            OpCode::JNEG => false,
            OpCode::JZER => false,
            OpCode::JPOS => false,
            OpCode::JNNEG => false,
            OpCode::JNZER => false,
            OpCode::JNPOS => false,
            OpCode::JLES => true,
            OpCode::JEQU => true,
            OpCode::JGRE => true,
            OpCode::JNLES => true,
            OpCode::JNEQU => true,
            OpCode::JNGRE => true,
            OpCode::CALL => false,
            OpCode::EXIT => false,
            OpCode::PUSH => false,
            OpCode::POP => false,
            OpCode::PUSHR => false,
            OpCode::POPR => false,
            OpCode::SVC => false,
            // Extended
            OpCode::IEXIT => false,
            OpCode::HLT => false,
            OpCode::HCF => false,
        }
    }
}

pub fn parse_instruction(
    statement: Statement,
    org: Option<usize>,
    const_symbols: &HashMap<String, i32>,
    code_symbols: &HashMap<String, i32>,
    data_symbols: &HashMap<String, i32>,
    code_size: usize,
) -> Result<i32, String>
{
    let org = org.unwrap_or(0);
    let mut words = statement.words.clone();
    let keyword_string = statement.words[0].to_uppercase();
    let keyword = keyword_string.as_str();

    // Remove oper keyword
    words.remove(0);
    let line = statement.line;

    // Get opcode
    let opcode;
    match OpCode::from_str(keyword) {
        Ok(op) => opcode = op,
        Err(e) => return Err(format!("Line {}: {}", line, e))
    }

    // Assert correct number of operands
    if words.len() != opcode.get_operand_count() {
        return Err(format!("Line {}: Invalid number of operands for {}. Expected {}, but got {}", line, keyword, opcode.get_operand_count(), words.len()));
    }

    // Get operand words
    let op1: String;
    let op2: String;

    match words.len() {
        0 => {
            op1 = String::new();
            op2 = String::new();
        }
        1 => {
            // Some jumps use op2 but not op1. Swap them, if appropriate.
            if opcode.is_op2_only() {
                op1 = "R0".to_string();
                op2 = words[0].clone();
            } else {
                op1 = words[0].to_uppercase();
                op2 = String::new();
            }
        }
        2 => {
            op1 = words[0].to_uppercase();
            op2 = words[1].clone();
        }
        _ => panic!("Line {}: wtf, word count is '{}'. '{:?}'", line, words.len(), words)
    }

    // Get first register
    let rj;
    match Register::from_str(op1.as_str()) {
        Ok(register) => rj = register,
        Err(e) => return Err(format!("Line {}: {}", line, e))
    }

    // Parse op2: Ri, mode, addr
    let mode;
    let ri;
    let addr: i32;

    if op2.is_empty() {
        mode = opcode.get_default_mode();
        ri = Register::R0;
        addr = 0;
    } else if let Ok(parsed) = parse_op2(op2.as_str()) {

        // Mode
        mode = opcode.get_default_mode() + parsed.mode;

        // Register
        ri = parsed.register;

        // Address
        if parsed.addr.as_str() == "" {
            // (is empty)
            addr = 0;
        } else if let Ok(val) = str_to_builtin_const(&parsed.addr) {
            // (is builtin const)
            addr = val;
        } else if let Some(val) = const_symbols.get(&parsed.addr) {
            // (is const)
            addr = val.to_i32().unwrap();
        } else if let Some(offset) = data_symbols.get(&parsed.addr) {
            // (is variable)
            addr = (org + code_size).to_i32().unwrap() + offset;
        } else if let Some(offset) = code_symbols.get(&parsed.addr) {
            // (is code label)
            addr = (org).to_i32().unwrap() + offset;
        } else if let Ok(val) = str_to_integer(parsed.addr.as_str()) {
            // (is number)
            addr = val;
        } else {
            return Err(format!("Line {}: invalid address: {}", line, parsed.addr));
        }
    } else {
        return Err(format!("Line {}: Couldn't parse second operand: {}", line, op2));
    }

    if addr < i16::MIN as i32 && addr > u16::MAX as i32 {
        return Err(format!("Line {}: Address: {} is out of range", line, addr));
    }

    let mut value;
    value = (opcode as i32) << 24;
    value += (rj as i32) << 21;
    value += mode << 19;
    value += (ri as i32) << 16;
    value += addr & 0xffff;
    Ok(value)
}


/// Used by parse_op2()
struct Op2 {
    pub mode: i32,
    pub addr: String,
    pub register: Register,
}

/// Parse second operand: "=123(R2)"
fn parse_op2(input_str: &str) -> Result<Op2, String> {
    let mut mode: i32 = 0;
    let mut addr = String::new();
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

    // We're done already: Second operand text is a register with no address.
    if let Ok(register) = Register::from_str(text.as_str()) {

        // Do not allow negative direct register addressing "-R1"
        if addr.as_str() == "-" {
            return Err(format!("Negative direct register addressing '{}' is not allowed. The minus sign only affects address portion.", input_str));
        }

        return Ok(Op2 {
            mode: mode - 1, // Register only decrements because of direct reg addressing
            addr,
            register,
        });
    }

    let register;
    // Second operand _contains_ register in parentheses
    if let Some((before_open, after_open)) = text.split_once('(') {
        match after_open.split_once(')') {
            Some((register_string, after_close)) => {
                register = Register::from_str(register_string)?;

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
    } else {
        register = Register::R0;
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
    use crate::compiler::Keyword;
    use super::*;
    /*
    Addressing modes require some careful testing.

        Code                    Desired effect  Examples, explanation
        - "=" prefix            decrement mode
        - "@" prefix            increment mode
        - no address            decrement mode  "R1", "R0"
        - no addr, neg reg      illegal         "-R1", "-R0"    Titokone refuses to compile, thinking it's label.
        - @@0                   illegal         Would get mode 2 on store command. Is illegal and should be illegal.
        - ==0                   illegal         Would get decrement by 2 Is illegal and should be illegal.

        Other:
        - Reg in parentheses implies presence of address 0: "(R1)" == "0(R1)"
        - Mode sign must be the first character. "=-1" is OK. "-=1" should fail.

     */
    #[test]
    /// Go through all combinations of mode signs
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
    fn test_parse_op2_mode_direct_register_neg() {
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
    fn test_parse_instruction() {
        let sym = HashMap::new();
        let sym2 = HashMap::new();
        assert_eq!(parse_instruction(dummy_statement("add r1 =0"), None, &sym, &sym2, &sym2, 0).unwrap(), 287309824);
    }

    fn dummy_statement(text: &str) -> Statement {
        Statement {
            statement_type: Keyword::Code,
            label: None,
            words: text.split_whitespace().map(str::to_string).collect(),
            line: 0,
            comment: None,
        }
    }
}