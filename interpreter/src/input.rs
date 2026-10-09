use crate::{
    ConcreteHeap,
    java_class::Method,
    java_types::{
        ConcreteSimpleVal, ConcreteStackVal, ConcreteVal, HeapType, HeapValue, SimpleType,
        SimpleValue, StackValue, Type, Value,
    },
    state::Heap,
};
use std::collections::HashMap;

pub fn parse_input(method: &Method, input: &str) -> (Vec<ConcreteStackVal>, ConcreteHeap) {
    let types = &method.id.params;
    let mut heap = Heap { heap: vec![] };

    let trimmed = input.trim();
    let inner = trimmed
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or_else(|| panic!("expected input wrapped in parens, got `{input}`"));

    let parts = split_top_level(inner);

    assert_eq!(
        parts.len(),
        types.len(),
        "argument count mismatch for `{input}`: expected {}, got {}",
        types.len(),
        parts.len()
    );

    let values = types
        .iter()
        .zip(parts.into_iter())
        .map(|(t, part)| parse_input_value(t, part, &mut heap))
        .collect();

    (values, heap)
}

/// Splits a comma-separated list at the top level only, ignoring commas
/// nested inside `[...]` or `'...'` (so array/char/string literals stay intact).
fn split_top_level(s: &str) -> Vec<&str> {
    if s.trim().is_empty() {
        return vec![];
    }

    let mut parts = vec![];
    let mut depth = 0i32;
    let mut in_quote = false;
    let mut start = 0usize;

    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        match *b as char {
            '\'' => in_quote = !in_quote,
            '[' if !in_quote => depth += 1,
            ']' if !in_quote => depth -= 1,
            ',' if !in_quote && depth == 0 => {
                parts.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(s[start..].trim());
    parts
}

fn parse_input_value(ty: &Type, input: &str, heap: &mut ConcreteHeap) -> ConcreteStackVal {
    match ty {
        Type::S(s) => parse_simple_value(*s, input).to_stack_value(),
        Type::H(_) if input == "null" => StackValue::Ref(None),
        Type::H(h) => parse_heap_ref(h, input, heap),
    }
}

/// Parses a *simple* literal directly into a SimpleValue (so array elements
/// keep their proper variant, e.g. Char instead of being widened to Int).
/// Sub-int types are stored in an i32, as `SimpleValue` uses `I` for all of them.
fn parse_simple_value(ty: SimpleType, input: &str) -> ConcreteSimpleVal {
    match ty {
        SimpleType::Int => SimpleValue::Int(
            input
                .parse()
                .unwrap_or_else(|_| panic!("invalid int literal `{input}`")),
        ),
        SimpleType::Float => SimpleValue::Float(
            input
                .parse()
                .unwrap_or_else(|_| panic!("invalid float literal `{input}`")),
        ),
        SimpleType::Byte => SimpleValue::Byte(
            input
                .parse::<i8>()
                .unwrap_or_else(|_| panic!("invalid byte literal `{input}`")) as i32,
        ),
        SimpleType::Short => SimpleValue::Short(
            input
                .parse::<i16>()
                .unwrap_or_else(|_| panic!("invalid short literal `{input}`")) as i32,
        ),
        SimpleType::Char => SimpleValue::Char(parse_char_literal(input) as i32),
        SimpleType::Boolean => {
            let b: bool = input
                .parse()
                .unwrap_or_else(|_| panic!("invalid bool literal `{input}`"));
            SimpleValue::Boolean(b as i32)
        }
        SimpleType::Ref => match input {
            "null" => SimpleValue::Ref(None),
            _ => unimplemented!("non-null reference literal `{input}` isn't supported here"),
        },
    }
}

fn parse_char_literal(input: &str) -> u16 {
    let inner = input.trim_matches('\'');
    inner
        .chars()
        .next()
        .unwrap_or_else(|| panic!("empty char literal `{input}`")) as u16
}

fn parse_heap_ref(ty: &HeapType, input: &str, heap: &mut ConcreteHeap) -> ConcreteStackVal {
    match ty {
        HeapType::Array { ty } => parse_array(*ty, input, heap),
        HeapType::Class { name } if name == "java/lang/String" => parse_string(input, heap),
        HeapType::Class { name } => {
            panic!("don't know how to parse a literal for class `{name}`")
        }
    }
}

/// Parses `[I:1, 2, 3]`, `[C:'h', 'e']`, `[I:]`, etc.
fn parse_array(ty: SimpleType, input: &str, heap: &mut ConcreteHeap) -> ConcreteStackVal {
    let stripped = input
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or_else(|| panic!("invalid array literal `{input}`, expected `[...]`"));

    let (_tag, rest) = stripped
        .split_once(':')
        .unwrap_or_else(|| panic!("invalid array literal `{input}`, missing `:`"));

    let values: Vec<ConcreteSimpleVal> = split_top_level(rest)
        .into_iter()
        .map(|elem| parse_simple_value(ty, elem))
        .collect();

    let idx = heap.heap.len() as u32;
    heap.heap.push(HeapValue::Array { ty: ty, values });
    StackValue::Ref(Some(idx))
}

/// Parses `s'hello'` into a heap-allocated `java/lang/String` object
/// whose `value` field is a char array (embedded directly for simplicity).
fn parse_string(input: &str, heap: &mut ConcreteHeap) -> ConcreteStackVal {
    let content = input
        .strip_prefix("s'")
        .and_then(|s| s.strip_suffix('\''))
        .unwrap_or_else(|| panic!("invalid string literal `{input}`, expected `s'...'`"));

    let chars: Vec<ConcreteSimpleVal> = content
        .encode_utf16()
        .map(|c| SimpleValue::Char(c as i32))
        .collect();

    let mut fields: HashMap<String, ConcreteVal> = HashMap::new();
    fields.insert(
        "value".to_string(),
        Value::H(HeapValue::Array {
            ty: SimpleType::Char,
            values: chars,
        }),
    );

    let idx = heap.heap.len() as u32;
    heap.heap.push(HeapValue::Object {
        name: "java/lang/String".to_string(),
        fields,
    });
    StackValue::Ref(Some(idx))
}
