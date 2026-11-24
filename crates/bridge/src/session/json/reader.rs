//! Reads one flat JSON object: string keys, and values that are numbers, booleans, `null`,
//! strings or arrays of numbers. Nested objects and arrays of anything else are rejected.

use std::collections::HashMap;
use std::str;

const HEX_RADIX: u32 = 16;
const UNICODE_ESCAPE_DIGITS: usize = 4;

/// One member's value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum JsonValue {
    Number(f64),
    Bool(bool),
    Null,
    /// An array of numbers; a `null` element reads as NaN, which is how `JSON.stringify` wrote it.
    Numbers(Vec<f64>),
    Text(String),
}

fn is_number_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'-' | b'+' | b'.' | b'e' | b'E')
}

/// A read position in the line.
struct Cursor<'line> {
    bytes: &'line [u8],
    position: usize,
}

impl<'line> Cursor<'line> {
    fn new(line: &'line str) -> Self {
        Self {
            bytes: line.as_bytes(),
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self
            .bytes
            .get(self.position)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.position += 1;
        }
    }

    /// The next byte after any whitespace, without consuming it.
    fn peek(&mut self) -> Option<u8> {
        self.skip_whitespace();
        self.bytes.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.position).copied()?;
        self.position += 1;
        Some(byte)
    }

    /// Consumes `wanted` if it comes next (after any whitespace).
    fn eat(&mut self, wanted: u8) -> bool {
        let is_next = self.peek() == Some(wanted);
        if is_next {
            self.position += 1;
        }
        is_next
    }

    fn expect(&mut self, wanted: u8) -> Option<()> {
        self.eat(wanted).then_some(())
    }

    fn word(&mut self, word: &str) -> Option<()> {
        let end = self.position.checked_add(word.len())?;
        let is_match = self.bytes.get(self.position..end) == Some(word.as_bytes());
        is_match.then(|| self.position = end)
    }

    fn number(&mut self) -> Option<f64> {
        self.skip_whitespace();
        let start = self.position;
        while self
            .bytes
            .get(self.position)
            .copied()
            .is_some_and(is_number_byte)
        {
            self.position += 1;
        }
        let text = str::from_utf8(self.bytes.get(start..self.position)?).ok()?;
        let is_json_number = text.starts_with(|first: char| first == '-' || first.is_ascii_digit());
        if !is_json_number {
            return None;
        }
        text.parse().ok()
    }

    /// A number, or `null` read as NaN.
    fn nullable_number(&mut self) -> Option<f64> {
        let is_null = self.peek() == Some(b'n');
        if is_null {
            return self.word("null").map(|()| f64::NAN);
        }
        self.number()
    }

    fn numbers(&mut self) -> Option<Vec<f64>> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        if self.eat(b']') {
            return Some(values);
        }
        loop {
            values.push(self.nullable_number()?);
            if !self.eat(b',') {
                return self.expect(b']').map(|()| values);
            }
        }
    }

    fn string(&mut self) -> Option<String> {
        self.expect(b'"')?;
        let mut text = Vec::new();
        loop {
            match self.advance()? {
                b'"' => return String::from_utf8(text).ok(),
                b'\\' => self.escape(&mut text)?,
                byte => text.push(byte),
            }
        }
    }

    /// One escape after its backslash; a lone UTF-16 surrogate (`\ud83d` without its pair) is
    /// rejected.
    fn escape(&mut self, text: &mut Vec<u8>) -> Option<()> {
        let decoded = match self.advance()? {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => self.unicode_escape()?,
            _ => return None,
        };
        let mut encoded = [0; 4];
        text.extend_from_slice(decoded.encode_utf8(&mut encoded).as_bytes());
        Some(())
    }

    fn unicode_escape(&mut self) -> Option<char> {
        let end = self.position.checked_add(UNICODE_ESCAPE_DIGITS)?;
        let digits = str::from_utf8(self.bytes.get(self.position..end)?).ok()?;
        self.position = end;
        char::from_u32(u32::from_str_radix(digits, HEX_RADIX).ok()?)
    }

    fn value(&mut self) -> Option<JsonValue> {
        match self.peek()? {
            b'"' => self.string().map(JsonValue::Text),
            b't' => self.word("true").map(|()| JsonValue::Bool(true)),
            b'f' => self.word("false").map(|()| JsonValue::Bool(false)),
            b'n' => self.word("null").map(|()| JsonValue::Null),
            b'[' => self.numbers().map(JsonValue::Numbers),
            _ => self.number().map(JsonValue::Number),
        }
    }

    /// The members after the opening brace, up to and including the closing one.
    fn members(&mut self, members: &mut HashMap<String, JsonValue>) -> Option<()> {
        loop {
            let key = self.string()?;
            self.expect(b':')?;
            let value = self.value()?;
            members.insert(key, value);
            if !self.eat(b',') {
                return self.expect(b'}');
            }
        }
    }
}

/// The members of the one flat object `line` holds, surrounding whitespace allowed; `None` when
/// the line is anything else. A repeated key keeps its last value, as `JSON.parse` does.
pub(crate) fn parse_flat_object(line: &str) -> Option<HashMap<String, JsonValue>> {
    let mut cursor = Cursor::new(line);
    cursor.expect(b'{')?;
    let mut members = HashMap::new();
    if !cursor.eat(b'}') {
        cursor.members(&mut members)?;
    }
    let is_at_end = cursor.peek().is_none();
    is_at_end.then_some(members)
}
