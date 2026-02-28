#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use derive_more::derive::{AsRef, Deref, Display as DeriveDisplay};
use impl_trait_for_tuples::impl_for_tuples;
use std::fmt::Display;

/// Converts user-provided dynamic arguments into positional `String`s.
///
/// This enables ergonomic calls such as:
/// `dynamic.with((1, "name", 3.5))`
/// for resources that internally render templates such as:
/// `"Hello {name}, total {count}"`.
pub trait IntoDynamicResourceArgs {
    /// Converts `self` into the positional arguments expected by the dynamic resource.
    fn into_dynamic_resource_args(self) -> Vec<String>;
}

#[impl_for_tuples(0, 16)]
#[tuple_types_no_default_trait_bound]
impl IntoDynamicResourceArgs for Tuple {
    for_tuples!( where #( Tuple: Display )* );

    fn into_dynamic_resource_args(self) -> Vec<String> {
        Vec::from([for_tuples!( #( self.Tuple.to_string() ),* )])
    }
}

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

fn render_template(template: &str, args: &[String]) -> String {
    let bytes = template.as_bytes();
    let mut output = String::with_capacity(template.len());
    let mut placeholders = Vec::with_capacity(args.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                output.push('{');
                index += 2;
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                output.push('}');
                index += 2;
            }
            b'{' => {
                let Some(name_end) = placeholder_end_runtime(bytes, index + 1) else {
                    output.push('{');
                    index += 1;
                    continue;
                };
                let name = &template[index + 1..name_end];
                let render_index = placeholders
                    .iter()
                    .position(|candidate| *candidate == name)
                    .unwrap_or_else(|| {
                        let current = placeholders.len();
                        placeholders.push(name);
                        current
                    });

                if let Some(value) = args.get(render_index) {
                    output.push_str(value.as_str());
                } else {
                    output.push('{');
                    output.push_str(name);
                    output.push('}');
                }
                index = name_end + 1;
            }
            _ => {
                output.push(bytes[index] as char);
                index += 1;
            }
        }
    }

    output
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
}

impl I18NDynamicResourceValue {
    #[doc(hidden)]
    #[must_use]
    pub const fn new_static(display_text: &'static str, template: &'static str) -> Self {
        Self {
            template,
            display_text,
        }
    }

    /// Invokes the dynamic resource with user-provided arguments.
    ///
    /// # Arguments
    /// * `args` - Arguments that can be converted into positional strings.
    ///   Each tuple item must implement `Display`.
    ///
    /// # Returns
    /// A string representing the localized resource.
    ///
    #[must_use]
    pub fn with<T>(&self, args: T) -> String
    where
        T: IntoDynamicResourceArgs,
    {
        render_template(self.template, &args.into_dynamic_resource_args())
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
