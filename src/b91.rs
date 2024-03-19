// SPDX-FileCopyrightText: 2024 sevonj
//
// SPDX-License-Identifier: MPL-2.0

//! TTKTK - TTK-91 ToolKit
//!
//! Module for compiled .b91 files.
//!
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::{FromStr, Lines};

/// Representation of a .b91 file. Useful for loading compiled files.
/// You can construct this from .b91 file contents with [from_str](#method.from_str).
#[derive(Clone)]
pub struct B91 {
    /// Code segment struct
    pub code_segment: B91Segment,
    /// Data segment struct
    pub data_segment: B91Segment,
    /// Symbol table dictionary: <symbol, value>.
    pub symbol_table: HashMap<String, i32>,
}

/// Represents either the data segment, or code segment.
#[derive(Clone)]
pub struct B91Segment {
    /// First address in this segment
    pub start: usize,
    /// Last address in this segment
    pub end: usize,
    /// Segment contents
    pub content: Vec<i32>,
}

#[derive(PartialEq, Debug)]
pub enum B91ParseError {
    End,
    IncorrectID,
    InvalidSection(String),
    RepeatSection(String),
    SectionMissing(String),
    SegmentOffsetParseError(String),
    NegativeSegmentSize(String),
    SymbolParseError(String),
}

impl Display for B91ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            B91ParseError::End => {
                write!(f, "Unexpected end of string.")
            }
            B91ParseError::IncorrectID => {
                write!(f, "Incorrect ID. Expected '___b91___'")
            }
            B91ParseError::InvalidSection(section) => {
                write!(f, "Unknown section: '{section}'")
            }
            B91ParseError::RepeatSection(section) => {
                write!(f, "Repeat section: '{section}'")
            }
            B91ParseError::SectionMissing(section) => {
                write!(f, "Section missing: '{section}'")
            }
            B91ParseError::SegmentOffsetParseError(line) => {
                write!(f, "Failed to parse segment offsets: '{line}")
            }
            B91ParseError::NegativeSegmentSize(line) => {
                write!(f, "Negative segment size: '{line}")
            }
            B91ParseError::SymbolParseError(line) => {
                write!(f, "Failed to parse symbol: '{line}")
            }
        }
    }
}

impl FromStr for B91 {
    type Err = B91ParseError;
    /// Get a [B91] from [&str].
    /// This expects the same sections that titokone outputs:
    /// - `___b91___` (must be first)
    /// - `___code___`
    /// - `___data___`
    /// - `___symboltable___`
    /// - `___end___` (must be last)
    fn from_str(b91: &str) -> Result<Self, Self::Err> {
        let mut lines = b91.lines();

        // Header: ___b91___
        match lines.next() {
            None => return Err(B91ParseError::End),
            Some(line) => {
                if line != "___b91___" {
                    return Err(B91ParseError::IncorrectID);
                }
            }
        }

        let mut code_segment: Option<B91Segment> = None;
        let mut data_segment: Option<B91Segment> = None;
        let mut symbol_table: Option<HashMap<String, i32>> = None;

        // Loop through sections
        loop {
            match lines.next() {
                Some(line) => {
                    match line {
                        "" => continue,
                        "___code___" => {
                            if code_segment.is_some() {
                                return Err(B91ParseError::RepeatSection("___code___".into()));
                            }
                            code_segment = Some(B91Segment::from_lines(&mut lines)?)
                        }
                        "___data___" => {
                            if data_segment.is_some() {
                                return Err(B91ParseError::RepeatSection("___code___".into()));
                            }
                            data_segment = Some(B91Segment::from_lines(&mut lines)?)
                        }
                        "___symboltable___" => {
                            if symbol_table.is_some() {
                                return Err(B91ParseError::RepeatSection("___code___".into()));
                            }
                            symbol_table = Some(parse_symbol_table(&mut lines)?);
                            break;
                        }
                        // Symboltable doesn't have a length, so we're using ___end___ as terminator
                        // "___end___" => break,
                        _ => return Err(B91ParseError::InvalidSection(line.into()))
                    }
                }
                None => return Err(B91ParseError::End),
            }
        }
        if code_segment.is_none() {
            return Err(B91ParseError::SectionMissing("___code___".into()));
        }
        if data_segment.is_none() {
            return Err(B91ParseError::SectionMissing("___data___".into()));
        }
        if symbol_table.is_none() {
            return Err(B91ParseError::SectionMissing("___symboltable___".into()));
        }

        Ok(B91 {
            code_segment: code_segment.unwrap(),
            data_segment: data_segment.unwrap(),
            symbol_table: symbol_table.unwrap(),
        })
    }
}

impl B91Segment {
    pub fn from_lines(lines: &mut Lines) -> Result<B91Segment, B91ParseError> {
        let start: usize;
        let end: usize;
        let mut content: Vec<i32>;

        // Get start & end
        match lines.next() {
            Some(line) => {
                // Split
                let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
                if words.len() != 2 {
                    return Err(B91ParseError::SegmentOffsetParseError(format!("words.len() != 2, '{line}")));
                }
                // Start
                match words[0].parse::<usize>() {
                    Ok(value) => start = value,
                    Err(e) => return Err(B91ParseError::SegmentOffsetParseError(format!("{e}, '{line}")))
                }
                // End
                match words[1].parse::<usize>() {
                    Ok(value) => end = value,
                    Err(e) => return Err(B91ParseError::SegmentOffsetParseError(format!("{e}, '{line}")))
                }
                // Content
                content = Vec::new();
                if start > end + 1 {
                    return Err(B91ParseError::NegativeSegmentSize(line.into()));
                }
                let length = end + 1 - start;
                for _ in 0..length {
                    match lines.next() {
                        Some(line) => {
                            // Push value to segment
                            match line.parse::<i32>() {
                                Ok(value) => content.push(value),
                                Err(e) => return Err(B91ParseError::SegmentOffsetParseError(format!("{e}, '{line}")))
                            }
                        }
                        None => return Err(B91ParseError::End),
                    }
                }
            }
            None => return Err(B91ParseError::End),
        }
        Ok(B91Segment {
            start,
            end,
            content,
        })
    }
}

fn parse_symbol_table(lines: &mut Lines) -> Result<HashMap<String, i32>, B91ParseError> {
    let mut symbol_table = HashMap::new();
    loop {
        match lines.next() {
            Some(line) => {
                // Exit
                if line == "___end___" {
                    break;
                }
                // Split
                let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
                if words.len() != 2 {
                    return Err(B91ParseError::SymbolParseError(format!("words.len() != 2, '{line}")));
                }
                // Symbol
                let key = words[0].clone();
                match words[1].parse::<i32>() {
                    Ok(value) => {
                        symbol_table.insert(key, value);
                    }
                    Err(e) => {
                        return Err(B91ParseError::SymbolParseError(format!("{e}, '{line}")));
                    }
                }
            }
            None => return Err(B91ParseError::End),
        }
    }
    Ok(symbol_table)
}

#[cfg(test)]
mod tests {
    use std::result;
    use super::*;

    #[test]
    fn test_b91_from_str_empty() {
        assert!(B91::from_str("").is_err());
    }

    #[test]
    fn test_b91_from_str_ok() {
        let input = "___b91___
___code___
0 1
524288
1891631115
___data___
2 5
2
0
0
0
___symboltable___
halt 11
const 1
array 3
variable 2
label 0
___end___";
        assert!(B91::from_str(input).is_ok());
    }

    #[test]
    fn test_b91_from_str_code() {
        let input = "___b91___
___code___
4 6
101
-202
303
___data___
0 0
0
___symboltable___
___end___";
        let result = B91::from_str(input).unwrap();

        assert_eq!(result.code_segment.content[0], 101);
        assert_eq!(result.code_segment.content[1], -202);
        assert_eq!(result.code_segment.content[2], 303);

        assert_eq!(result.code_segment.start, 4);
        assert_eq!(result.code_segment.end, 6);

        assert_eq!(result.code_segment.content.len(), 3);
    }

    #[test]
    fn test_b91_from_str_data() {
        let input = "___b91___
___code___
0 0
0
___data___
6 8
101
-202
303
___symboltable___
___end___";
        let result = B91::from_str(input).unwrap();

        assert_eq!(result.data_segment.content[0], 101);
        assert_eq!(result.data_segment.content[1], -202);
        assert_eq!(result.data_segment.content[2], 303);

        assert_eq!(result.data_segment.start, 6);
        assert_eq!(result.data_segment.end, 8);

        assert_eq!(result.data_segment.content.len(), 3);
    }

    #[test]
    fn test_b91_from_str_symbols() {
        let input = "___b91___
___code___
0 0
0
___data___
0 0
0
___symboltable___
symbol0 0
Symbol1 1
SYMBOL2 2
___end___";
        let result = B91::from_str(input).unwrap();

        assert_eq!(result.symbol_table.get("symbol0").unwrap().to_owned(), 0);
        assert_eq!(result.symbol_table.get("Symbol1").unwrap().to_owned(), 1);
        assert_eq!(result.symbol_table.get("SYMBOL2").unwrap().to_owned(), 2);

        assert_eq!(result.symbol_table.len(), 3);
    }
}