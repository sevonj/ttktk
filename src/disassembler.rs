// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! TTK-91 Disassembly module.
//!
use crate::instructions::OpCode;

fn second2string(m: i32, ri: i32, addr: i32) -> String {
    let mut str = String::new();
    let mut m = m;
    if addr == 0 && ri != 0 {
        m += 1;
    }
    match m {
        0 => str += "=",
        1 => str += " ",
        2 => str += "@",
        3 => {
            // @(R1) results to this
            return format!("@({})", reg2string(ri));
        }
        _ => str += "wtf‽",
    }
    if ri == 0 {
        str += &addr.to_string();
        return str;
    }
    if addr != 0 {
        str += &addr.to_string();
        str += "(";
    }
    str += reg2string(ri).as_str();
    if addr != 0 {
        str += ")";
    }
    str
}

fn reg2string(r: i32) -> String {
    match r {
        6 => "SP".into(),
        7 => "FP".into(),
        _ => format!("R{}", r),
    }
}

fn rj2string(r: i32) -> String {
    match r {
        6 => "SP, ".into(),
        7 => "FP, ".into(),
        _ => format!("R{}, ", r),
    }
}

pub fn disassemble_instruction(input_instr: i32) -> String {

    // Split the value
    let opcode;
    match OpCode::try_from(input_instr >> 24) {
        Ok(value) => opcode = value,
        Err(_) => return "N/A".into()
    }
    let rj = (input_instr >> 21) & 0x7;
    let mut mode = (input_instr >> 19) & 0x3;
    let ri = (input_instr >> 16) & 0x7;
    // these casts catch the sign
    let addr = (input_instr & 0xffff) as i16 as i32;

    // Reverse the potential addressing mode offset.
    mode -= opcode.get_default_mode();

    // Construct return string
    let mut retstr = format!("{:width$}", opcode.to_string(), width=6);
    match opcode.get_operand_count() {
        // No operands, just opcode
        0 => return retstr,
        // 1 operand
        1 => if opcode.is_op2_only() {
            // 1 operand, second only
            retstr += second2string(mode, ri, addr).as_str();
        } else {
            // 1 operand, first only
            retstr += rj2string(rj).as_str();
        },
        // Both operands
        2 => {
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        _ => panic!("This should not be possible: '{}'", input_instr)
    }
    return retstr;
    /*
    match FromPrimitive::from_i32(input_instr >> 24) {
        Some(OpCode::NOP) => retstr += "NOP",
        Some(OpCode::STORE) => {
            retstr += "STORE ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::LOAD) => {
            retstr += "LOAD  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::IN) => {
            retstr += "IN    ";
            retstr += rj2string(rj).as_str();
            println!("IN mode: {}", mode);
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::OUT) => {
            retstr += "OUT   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::ADD) => {
            retstr += "ADD   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::SUB) => {
            retstr += "SUB   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::MUL) => {
            retstr += "MUL   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::DIV) => {
            retstr += "DIV   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::MOD) => {
            retstr += "MOD   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::AND) => {
            retstr += "AND   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::OR) => {
            retstr += "OR    ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::XOR) => {
            retstr += "XOR   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::SHL) => {
            retstr += "SHL   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::SHR) => {
            retstr += "SHR   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::NOT) => {
            retstr += "NOT   ";
            retstr += rj2string(rj).as_str();
        }
        Some(OpCode::SHRA) => {
            retstr += "SHRA  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::COMP) => {
            retstr += "COMP  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::JUMP) => {
            retstr += "JUMP  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNEG) => {
            retstr += "JNEG  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JZER) => {
            retstr += "JZER  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JPOS) => {
            retstr += "JPOS ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNNEG) => {
            retstr += "JNNEG ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNZER) => {
            retstr += "JNZER ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNPOS) => {
            retstr += "JNPOS ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JLES) => {
            retstr += "JLES  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JEQU) => {
            retstr += "JEQU  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JGRE) => {
            retstr += "JGRE  ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNLES) => {
            retstr += "JNLES ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNEQU) => {
            retstr += "JNEQU ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::JNGRE) => {
            retstr += "JNGRE ";
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::CALL) => {
            retstr += "CALL  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode + 1, ri, addr).as_str();
        }
        Some(OpCode::EXIT) => {
            retstr += "EXIT  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::PUSH) => {
            retstr += "PUSH  ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::POP) => {
            retstr += "POP   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::PUSHR) => {
            retstr += "PUSHR ";
            retstr += rj2string(rj).as_str();
        }
        Some(OpCode::POPR) => {
            retstr += "POPR  ";
            retstr += rj2string(rj).as_str();
        }
        Some(OpCode::IEXIT) => {
            retstr += "IEXIT ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::SVC) => {
            retstr += "SVC   ";
            retstr += rj2string(rj).as_str();
            retstr += second2string(mode, ri, addr).as_str();
        }
        Some(OpCode::HLT) => retstr += "HLT   ",
        Some(OpCode::HCF) => retstr += "HCF   ",

        None => retstr += "N/A",
    }

    retstr
    */
}
