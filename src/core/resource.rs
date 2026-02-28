#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use derive_more::derive::{AsRef, Deref, Display as DeriveDisplay};
use std::fmt::{Display, Write};

#[doc(hidden)]
#[must_use]
pub const fn __template_is_valid(template: &str) -> bool {
    let bytes = template.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                    index += 2;
                } else if let Some(name_end) = placeholder_end_checked(bytes, index + 1) {
                    index = name_end + 1;
                } else {
                    return false;
                }
            }
            b'}' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'}' {
                    index += 2;
                } else {
                    return false;
                }
            }
            _ => {
                index += 1;
            }
        }
    }

    true
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
                } else if let Some(name_end) = placeholder_end_checked(bytes, index + 1) {
                    let name_start = index + 1;

                    if !placeholder_seen_before(bytes, index, name_start, name_end) {
                        arity += 1;
                    }

                    index = name_end + 1;
                } else {
                    index += 1;
                }
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                index += 2;
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
pub const fn __template_slot_count(template: &str) -> usize {
    let bytes = template.as_bytes();
    let mut index = 0;
    let mut count = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                    index += 2;
                } else if let Some(name_end) = placeholder_end_checked(bytes, index + 1) {
                    count += 1;
                    index = name_end + 1;
                } else {
                    index += 1;
                }
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                index += 2;
            }
            _ => {
                index += 1;
            }
        }
    }

    count
}

#[doc(hidden)]
#[must_use]
pub const fn __normalized_template_len(template: &str) -> usize {
    let bytes = template.as_bytes();
    let mut index = 0;
    let mut length = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                length += 1;
                index += 2;
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                length += 1;
                index += 2;
            }
            b'{' => {
                if let Some(name_end) = placeholder_end_checked(bytes, index + 1) {
                    length += name_end + 1 - index;
                    index = name_end + 1;
                } else {
                    length += 1;
                    index += 1;
                }
            }
            _ => {
                length += 1;
                index += 1;
            }
        }
    }

    length
}

#[doc(hidden)]
#[must_use]
pub const fn __build_normalized_template<const N: usize>(template: &str) -> [u8; N] {
    let bytes = template.as_bytes();
    let mut normalized = [0; N];
    let mut source_index = 0;
    let mut output_index = 0;

    while source_index < bytes.len() {
        match bytes[source_index] {
            b'{' if source_index + 1 < bytes.len() && bytes[source_index + 1] == b'{' => {
                if output_index < N {
                    normalized[output_index] = b'{';
                }
                source_index += 2;
                output_index += 1;
            }
            b'}' if source_index + 1 < bytes.len() && bytes[source_index + 1] == b'}' => {
                if output_index < N {
                    normalized[output_index] = b'}';
                }
                source_index += 2;
                output_index += 1;
            }
            b'{' => {
                if let Some(name_end) = placeholder_end_checked(bytes, source_index + 1) {
                    while source_index <= name_end {
                        if output_index < N {
                            normalized[output_index] = bytes[source_index];
                        }
                        source_index += 1;
                        output_index += 1;
                    }
                } else {
                    if output_index < N {
                        normalized[output_index] = b'{';
                    }
                    source_index += 1;
                    output_index += 1;
                }
            }
            b'}' => {
                if output_index < N {
                    normalized[output_index] = b'}';
                }
                source_index += 1;
                output_index += 1;
            }
            _ => {
                if output_index < N {
                    normalized[output_index] = bytes[source_index];
                }
                source_index += 1;
                output_index += 1;
            }
        }
    }

    normalized
}

#[doc(hidden)]
#[must_use]
pub const fn __utf8<const N: usize>(bytes: &[u8; N]) -> &str {
    match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => "",
    }
}

const fn placeholder_end_checked(bytes: &[u8], start: usize) -> Option<usize> {
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

    if index >= bytes.len() || bytes[index] != b'}' {
        return None;
    }

    Some(index)
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
                } else if let Some(candidate_end) = placeholder_end_checked(bytes, index + 1) {
                    let candidate_start = index + 1;

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
                } else {
                    index += 1;
                }
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                index += 2;
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

#[derive(Clone, Copy, Debug, Default)]
#[doc(hidden)]
pub struct I18NRenderSlot {
    pub placeholder_start: usize,
    pub placeholder_end: usize,
    pub argument_index: usize,
}

impl I18NRenderSlot {
    const EMPTY: Self = Self {
        placeholder_start: 0,
        placeholder_end: 0,
        argument_index: 0,
    };
}

#[derive(Debug)]
#[doc(hidden)]
pub struct I18NRenderPlan<const N: usize> {
    pub slots: [I18NRenderSlot; N],
}

#[doc(hidden)]
#[must_use]
pub const fn __build_render_plan<const N: usize>(template: &str) -> I18NRenderPlan<N> {
    let bytes = template.as_bytes();
    let mut slots = [I18NRenderSlot::EMPTY; N];
    let mut unique_starts = [0; N];
    let mut unique_ends = [0; N];
    let mut unique_len = 0;
    let mut slot_index = 0;
    let mut index = 0;
    let mut render_index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                    index += 2;
                    render_index += 1;
                } else if let Some(name_end) = placeholder_end_checked(bytes, index + 1) {
                    let name_start = index + 1;
                    let placeholder_len = name_end + 1 - index;
                    let mut argument_index = 0;
                    let mut found = false;

                    while argument_index < unique_len {
                        if bytes_eq(
                            bytes,
                            unique_starts[argument_index],
                            unique_ends[argument_index],
                            name_start,
                            name_end,
                        ) {
                            found = true;
                            break;
                        }
                        argument_index += 1;
                    }

                    if !found && unique_len < N {
                        unique_starts[unique_len] = name_start;
                        unique_ends[unique_len] = name_end;
                        argument_index = unique_len;
                        unique_len += 1;
                    }

                    if slot_index < N {
                        slots[slot_index] = I18NRenderSlot {
                            placeholder_start: render_index,
                            placeholder_end: render_index + placeholder_len,
                            argument_index,
                        };
                        slot_index += 1;
                    }

                    index = name_end + 1;
                    render_index += placeholder_len;
                } else {
                    index += 1;
                    render_index += 1;
                }
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                index += 2;
                render_index += 1;
            }
            _ => {
                index += 1;
                render_index += 1;
            }
        }
    }

    I18NRenderPlan { slots }
}

/// A struct representing an internationalization (i18n) dynamic resource.
#[derive(Debug, Default, AsRef, Deref, DeriveDisplay)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[display("{}", display_text)]
#[doc(hidden)]
pub struct I18NDynamicResourceValue<const N: usize> {
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    template: &'static str,
    /// Template text with escaped braces resolved.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    #[as_ref(forward)]
    #[deref(forward)]
    display_text: &'static str,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    render_plan: &'static [I18NRenderSlot],
}

impl<const N: usize> I18NDynamicResourceValue<N> {
    #[doc(hidden)]
    #[must_use]
    pub const fn new_static(
        display_text: &'static str,
        template: &'static str,
        render_plan: &'static [I18NRenderSlot],
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
    ///   Each item is rendered directly through `Display`.
    ///
    /// # Returns
    /// A string representing the localized resource.
    #[must_use]
    pub fn with<S>(&self, args: &[S; N]) -> String
    where
        S: Display,
    {
        let mut output = String::with_capacity(self.display_text.len());
        let mut cursor = 0;

        for slot in self.render_plan {
            output.push_str(&self.display_text[cursor..slot.placeholder_start]);

            if write!(&mut output, "{}", &args[slot.argument_index]).is_err() {
                return output;
            }

            cursor = slot.placeholder_end;
        }

        output.push_str(&self.display_text[cursor..]);

        output
    }
}

impl<const N: usize> PartialEq for I18NDynamicResourceValue<N> {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
    }
}

impl<const N: usize> Eq for I18NDynamicResourceValue<N> {}

impl<const N: usize> PartialEq<str> for I18NDynamicResourceValue<N> {
    fn eq(&self, other: &str) -> bool {
        self.display_text == other
    }
}
