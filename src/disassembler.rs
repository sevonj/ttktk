// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! TTK-91 Disassembly module.
//!
use crate::instructions::{OpCode, Register};

fn second2string(m: i32, ri: Register, addr: i32) -> String {
    let mut str = String::new();
    let mut m = m;
    if addr == 0 && ri != Register::R0 {
        m += 1;
    }
    match m {
        0 => str += "=",
        1 => str += " ",
        2 => str += "@",
        3 => {
            // @(R1) results to this // Now does it?
            return format!("@({})", ri);
        }
        _ => str += "wtf‽",
    }
    if ri == Register::R0 {
        str += &addr.to_string();
        return str;
    }
    if addr != 0 {
        str += &addr.to_string();
        str += "(";
    }
    str += format!("{ri}").as_str();
    if addr != 0 {
        str += ")";
    }
    str
}

pub fn disassemble_instruction(input_instr: i32) -> String {

    // Split the value
    let opcode;
    match OpCode::try_from(input_instr >> 24) {
        Ok(value) => opcode = value,
        Err(_) => return "N/A".into()
    }
    let rj = Register::try_from((input_instr >> 21) & 0x7).unwrap();
    let mut mode = (input_instr >> 19) & 0x3;
    let ri = Register::try_from((input_instr >> 16) & 0x7).unwrap();
    // these casts catch the sign
    let addr = (input_instr & 0xffff) as i16 as i32;


    // Reverse the potential addressing mode offset.
    mode += 1;
    mode -= opcode.get_default_mode();


    // Construct return string
    let mut retstr = format!("{:width$}", opcode.to_string(), width = 6);
    match opcode.get_operand_count() {

        // No operands, just opcode
        0 => return retstr,

        // 1 operand
        1 => if opcode.is_op2_only() {
            // 1 operand, second only
            retstr += second2string(mode, ri, addr).as_str();
        } else {
            // 1 operand, first only
            retstr += format!("{rj}").as_str();
        },

        // Both operands
        2 => {
            retstr += format!("{rj}").as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        _ => panic!("This should not be possible: '{}'", input_instr)
    }
    retstr
}

#[cfg(test)]
mod tests {
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
    fn test_disassemble_instruction() {
        assert_eq!(disassemble_instruction(287309824).as_str(), "ADD   R1, =0");

        assert_eq!(disassemble_instruction(287375360).as_str(), "ADD   R1,  R1");
    }
}