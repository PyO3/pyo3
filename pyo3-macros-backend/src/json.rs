//! JSON-related utilities

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write as _;

pub enum JsonValue {
    String(Cow<'static, str>),
    Number(i16),
    Array(Vec<JsonValue>),
    Object(HashMap<&'static str, JsonValue>),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => {
                f.write_char('"')?;
                write_escaped_json_string(value, f)?;
                f.write_char('"')
            }
            Self::Number(value) => value.fmt(f),
            JsonValue::Array(values) => {
                f.write_char('[')?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    value.fmt(f)?;
                }
                f.write_char(']')
            }
            JsonValue::Object(key_values) => {
                f.write_char('{')?;
                for (i, (key, value)) in key_values.iter().enumerate() {
                    if i > 0 {
                        f.write_char(',')?;
                    }
                    f.write_char('"')?;
                    write_escaped_json_string(key, f)?;
                    f.write_char('"')?;
                    f.write_char(':')?;
                    value.fmt(f)?;
                }
                f.write_char('}')
            }
        }
    }
}

pub fn escape_json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    write_escaped_json_string(value, &mut output).unwrap();
    output
}

fn write_escaped_json_string(value: &str, output: &mut impl fmt::Write) -> fmt::Result {
    for c in value.chars() {
        match c {
            '\\' => output.write_str("\\\\"),
            '"' => output.write_str("\\\""),
            '\x08' => output.write_str("\\b"),
            '\x0C' => output.write_str("\\f"),
            '\n' => output.write_str("\\n"),
            '\r' => output.write_str("\\r"),
            '\t' => output.write_str("\\t"),
            c @ '\0'..='\x1F' => {
                write!(output, "\\u{:0>4x}", u32::from(c))
            }
            c => output.write_char(c),
        }?;
    }
    Ok(())
}
