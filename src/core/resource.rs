#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use derive_more::derive::{AsRef, Deref, Display as DeriveDisplay};
use std::fmt::Display;

#[doc(hidden)]
#[must_use]
pub const fn __template_arity(template: &str) -> usize {
    let bytes = template.as_bytes();
    let mut index = 0;
    let mut arity = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                    index += 2;
                } else {
                    let name_start = index + 1;
                    let name_end = placeholder_end(bytes, name_start);

                    if !placeholder_seen_before(bytes, index, name_start, name_end) {
                        arity += 1;
                    }

                    index = name_end + 1;
                }
            }
            b'}' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'}' {
                    index += 2;
                } else {
                    panic!("invalid template literal");
                }
            }
            _ => {
                index += 1;
            }
        }
    }

    arity
}

#[doc(hidden)]
#[must_use]
pub const fn __template_has_escapes(template: &str) -> bool {
    let bytes = template.as_bytes();
    let mut index = 0;

    while index + 1 < bytes.len() {
        if (bytes[index] == b'{' || bytes[index] == b'}') && bytes[index] == bytes[index + 1] {
            return true;
        }

        index += 1;
    }

    false
}

#[doc(hidden)]
#[must_use]
pub fn __normalize_template(template: &str) -> String {
    let bytes = template.as_bytes();
    let mut index = 0;
    let mut normalized = String::with_capacity(template.len());

    while index < bytes.len() {
        match bytes[index] {
            b'{' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                normalized.push('{');
                index += 2;
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                normalized.push('}');
                index += 2;
            }
            _ => {
                normalized.push(bytes[index] as char);
                index += 1;
            }
        }
    }

    normalized
}

const fn placeholder_end(bytes: &[u8], start: usize) -> usize {
    assert!(
        start < bytes.len() && is_identifier_start(bytes[start]),
        "invalid template literal"
    );

    let mut index = start + 1;

    while index < bytes.len() && bytes[index] != b'}' {
        assert!(
            is_identifier_continue(bytes[index]),
            "invalid template literal"
        );

        index += 1;
    }

    assert!(
        !(index >= bytes.len() || bytes[index] != b'}'),
        "invalid template literal"
    );

    index
}

const fn placeholder_seen_before(
    bytes: &[u8],
    current_open: usize,
    current_start: usize,
    current_end: usize,
) -> bool {
    let mut index = 0;

    while index < current_open {
        match bytes[index] {
            b'{' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                    index += 2;
                } else {
                    let candidate_start = index + 1;
                    let candidate_end = placeholder_end(bytes, candidate_start);

                    if bytes_eq(
                        bytes,
                        candidate_start,
                        candidate_end,
                        current_start,
                        current_end,
                    ) {
                        return true;
                    }

                    index = candidate_end + 1;
                }
            }
            b'}' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'}' {
                    index += 2;
                } else {
                    panic!("invalid template literal");
                }
            }
            _ => {
                index += 1;
            }
        }
    }

    false
}

const fn bytes_eq(
    bytes: &[u8],
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    let left_len = left_end - left_start;

    if left_len != right_end - right_start {
        return false;
    }

    let mut offset = 0;

    while offset < left_len {
        if bytes[left_start + offset] != bytes[right_start + offset] {
            return false;
        }

        offset += 1;
    }

    true
}

const fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

const fn is_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn placeholder_end_runtime(bytes: &[u8], start: usize) -> Option<usize> {
    if start >= bytes.len() || !is_identifier_start(bytes[start]) {
        return None;
    }

    let mut index = start + 1;

    while index < bytes.len() && bytes[index] != b'}' {
        if !is_identifier_continue(bytes[index]) {
            return None;
        }

        index += 1;
    }

    (index < bytes.len() && bytes[index] == b'}').then_some(index)
}

#[derive(Debug)]
#[doc(hidden)]
pub enum I18NRenderPart {
    Literal(Box<str>),
    Argument(usize),
}

fn push_literal(parts: &mut Vec<I18NRenderPart>, literal: &mut String) {
    if !literal.is_empty() {
        parts.push(I18NRenderPart::Literal(
            std::mem::take(literal).into_boxed_str(),
        ));
    }
}

#[doc(hidden)]
#[must_use]
pub fn __build_render_plan(template: &str) -> Box<[I18NRenderPart]> {
    let bytes = template.as_bytes();
    let mut parts = Vec::new();
    let mut literal = String::with_capacity(template.len());
    let mut placeholders = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                literal.push('{');
                index += 2;
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                literal.push('}');
                index += 2;
            }
            b'{' => {
                let Some(name_end) = placeholder_end_runtime(bytes, index + 1) else {
                    literal.push('{');
                    index += 1;
                    continue;
                };
                let name = &template[index + 1..name_end];
                push_literal(&mut parts, &mut literal);
                let render_index = placeholders
                    .iter()
                    .position(|candidate| *candidate == name)
                    .unwrap_or_else(|| {
                        let current = placeholders.len();
                        placeholders.push(name);
                        current
                    });
                parts.push(I18NRenderPart::Argument(render_index));
                index = name_end + 1;
            }
            _ => {
                literal.push(bytes[index] as char);
                index += 1;
            }
        }
    }

    push_literal(&mut parts, &mut literal);
    parts.into_boxed_slice()
}

/// A struct representing an internationalization (i18n) dynamic resource.
#[derive(Debug, Default, AsRef, Deref, DeriveDisplay)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[display("{}", display_text)]
#[doc(hidden)]
pub struct I18NDynamicResourceValue {
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    template: &'static str,
    /// Template text with escaped braces resolved.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    #[as_ref(forward)]
    #[deref(forward)]
    display_text: &'static str,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    render_plan: &'static [I18NRenderPart],
}

impl I18NDynamicResourceValue {
    #[doc(hidden)]
    #[must_use]
    pub const fn new_static(
        display_text: &'static str,
        template: &'static str,
        render_plan: &'static [I18NRenderPart],
    ) -> Self {
        Self {
            template,
            display_text,
            render_plan,
        }
    }

    /// Invokes the dynamic resource with user-provided arguments.
    ///
    /// # Arguments
    /// * `args` - Positional arguments for the inferred placeholders.
    ///   Each item is rendered through `ToString`.
    ///
    /// # Returns
    /// A string representing the localized resource.
    ///
    #[must_use]
    pub fn with<S, const N: usize>(&self, args: &[S; N]) -> String
    where
        S: Display,
    {
        let mut output = String::with_capacity(self.display_text.len());

        for part in self.render_plan {
            match part {
                I18NRenderPart::Literal(text) => output.push_str(text),
                I18NRenderPart::Argument(index) => {
                    if let Some(value) = args.get(*index) {
                        output.push_str(&value.to_string());
                    }
                }
            }
        }

        output
    }
}

impl PartialEq for I18NDynamicResourceValue {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
    }
}

impl Eq for I18NDynamicResourceValue {}

impl PartialEq<str> for I18NDynamicResourceValue {
    fn eq(&self, other: &str) -> bool {
        self.display_text == other
    }
}
