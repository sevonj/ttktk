// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! TTK-91 Disassembly module.
//!
use crate::instructions::{OpCode, Register};

/// Disassemble instruction (extended)
/// Returns "N/A" if failed.
pub fn disassemble_instruction(input_instr: i32) -> String {

    // Get opcode
    let opcode;
    match OpCode::try_from(input_instr >> 24) {
        Ok(value) => opcode = value,
        Err(_) => return "N/A".into()
    }

    // Get registers
    let rj = Register::try_from((input_instr >> 21) & 0x7).unwrap();
    let ri = Register::try_from((input_instr >> 16) & 0x7).unwrap();

    // Get address. These casts catch the sign.
    let addr = (input_instr & 0xffff) as i16 as i32;

    // Get addressing mode
    let mut mode = (input_instr >> 19) & 0x3;
    // Undo mode offset from opcode.
    mode -= opcode.get_default_mode();
    // Undo mode offset from direct register addressing.
    if addr == 0 && ri != Register::R0 {
        mode += 1;
    }
    // Mode 2 means @(R1) an is only valid on instructions with default mode of 0.
    if mode == 2 && opcode.get_default_mode() == 1 {
        return "N/A".into();
    }
    if mode == 3 {
        return "N/A".into();
    }


    // Return string
    let oper = format!("{:width$}", opcode.to_string(), width = 5);
    let op2 = op2_to_string(mode, ri, addr);

    match opcode.get_operand_count() {
        0 => return format!("{oper}"),
        1 => return if opcode.is_op2_only() {
            format!("{oper} {op2}")
        } else {
            format!("{oper} {rj}")
        },
        2 => return format!("{oper} {rj}, {op2}"),
        _ => panic!("This should not be possible: '{}'", input_instr)
    }
}

/// Disassemble instruction (classic)
/// Same as the other one, but refuses to recognize extended instructions.
pub fn disassemble_instruction_classic(input_instr: i32) -> String {
    let opcode;
    match OpCode::try_from(input_instr >> 24) {
        Ok(value) => opcode = value,
        Err(_) => return "N/A".into()
    }
    if !opcode.is_classic_isa() {
        return "N/A".into();
    }
    disassemble_instruction(input_instr)
}

fn op2_to_string(mode: i32, ri: Register, addr: i32) -> String {
    // -1 is only valid on instructions with default mode 1.
    // 2 is only valid on instructions with default mode 2.
    // 2 results in an @ sign _and_ parentheses.
    let m = match mode {
        -1 => "=",
        0 => " ",
        1 => "@",
        2 => "@",
        _ => "‽",
    };

    if ri == Register::R0 {
        if mode == 2 {
            if addr == 0 {
                format!("{m}(R0)")
            } else {
                format!("{m}{addr}(R0)")
            }
        } else {
            format!("{m}{addr}")
        }
    } else if addr == 0 {
        if mode == 2 {
            format!("{m}({ri})")
        } else {
            format!("{m}{ri}")
        }
    } else {
        format!("{m}{addr}({ri})")
    }
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

    #[test]
    fn test_disassemble_mode_0() {
        assert_eq!(disassemble_instruction(287309824).as_str(), "ADD   R1, =0");
        assert_eq!(disassemble_instruction(18874368).as_str(), "STORE R1,  0");
    }

    #[test]
    fn test_disassemble_mode_1() {
        assert_eq!(disassemble_instruction(287834112).as_str(), "ADD   R1,  0");
        assert_eq!(disassemble_instruction(19398656).as_str(), "STORE R1, @0");
    }

    #[test]
    fn test_disassemble_mode_2() {
        assert_eq!(disassemble_instruction(288358400).as_str(), "ADD   R1, @0");
        assert_eq!(disassemble_instruction(19922944).as_str(), "STORE R1, @(R0)");
        assert_eq!(disassemble_instruction(19922945).as_str(), "STORE R1, @1(R0)");
    }

    #[test]
    fn test_disassemble_mode_3() {
        // "ADD   R1, ‽0"
        assert_eq!(disassemble_instruction(288882688).as_str(), "N/A");
        // "STORE R1, ‽0"
        assert_eq!(disassemble_instruction(20447232).as_str(), "N/A");
    }
}