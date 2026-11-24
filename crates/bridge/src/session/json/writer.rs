//! Writes a flat JSON object the way JavaScript's `JSON.stringify` does.

use numeric::format::js_number;

/// A flat JSON object written member by member, in insertion order.
#[derive(Clone, Debug, Default)]
pub(crate) struct JsonObject {
    text: String,
}

/// A number as `JSON.stringify` writes it: JavaScript's `String(number)`, but `null` when it is
/// not finite.
fn json_number(value: f64) -> String {
    if !value.is_finite() {
        return "null".to_owned();
    }
    js_number(value)
}

impl JsonObject {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Starts a member: the separator, then `"key":`. Keys are plain ASCII names, never escaped.
    fn key(&mut self, key: &str) {
        if !self.text.is_empty() {
            self.text.push(',');
        }
        self.text.push('"');
        self.text.push_str(key);
        self.text.push_str("\":");
    }

    pub(crate) fn number(&mut self, key: &str, value: f64) {
        self.key(key);
        self.text.push_str(&json_number(value));
    }

    pub(crate) fn numbers(&mut self, key: &str, values: &[f64]) {
        self.key(key);
        let items: Vec<String> = values.iter().copied().map(json_number).collect();
        self.text.push('[');
        self.text.push_str(&items.join(","));
        self.text.push(']');
    }

    pub(crate) fn boolean(&mut self, key: &str, is_set: bool) {
        self.key(key);
        self.text.push_str(if is_set { "true" } else { "false" });
    }

    /// The finished object, without a trailing newline.
    pub(crate) fn finish(self) -> String {
        format!("{{{}}}", self.text)
    }
}
