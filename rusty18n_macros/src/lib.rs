use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Error, Index, LitInt, LitStr};

struct TemplateSpec {
    display_text: String,
    render_format: String,
    expected_args: usize,
    render_indices: Vec<usize>,
}

#[proc_macro]
pub fn __i18n_build_resource(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);

    match parse_template(&literal) {
        Ok(spec) => build_tokens(&literal, spec),
        Err(error) => error.to_compile_error().into(),
    }
}

fn parse_template(literal: &LitStr) -> syn::Result<TemplateSpec> {
    let template = literal.value();
    let bytes = template.as_bytes();
    let mut display_text = String::with_capacity(template.len());
    let mut render_format = String::with_capacity(template.len());
    let mut placeholders = Vec::new();
    let mut render_indices = Vec::new();
    let mut index = 0;
    let mut text_start = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                push_text_segment(
                    &template,
                    text_start,
                    index,
                    &mut display_text,
                    &mut render_format,
                );
                display_text.push('{');
                render_format.push_str("{{");
                index += 2;
                text_start = index;
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                push_text_segment(
                    &template,
                    text_start,
                    index,
                    &mut display_text,
                    &mut render_format,
                );
                display_text.push('}');
                render_format.push_str("}}");
                index += 2;
                text_start = index;
            }
            b'{' => {
                push_text_segment(
                    &template,
                    text_start,
                    index,
                    &mut display_text,
                    &mut render_format,
                );

                let placeholder_end = find_placeholder_end(bytes, index, literal)?;
                let name = &template[index + 1..placeholder_end];

                validate_identifier(name, literal)?;

                let placeholder_index = placeholders
                    .iter()
                    .position(|candidate: &String| candidate == name)
                    .unwrap_or_else(|| {
                        placeholders.push(name.to_string());
                        placeholders.len() - 1
                    });

                display_text.push('{');
                display_text.push_str(name);
                display_text.push('}');
                render_format.push_str("{}");
                render_indices.push(placeholder_index);

                index = placeholder_end + 1;
                text_start = index;
            }
            b'}' => {
                return Err(Error::new_spanned(literal, "invalid template literal"));
            }
            _ => index += 1,
        }
    }

    push_text_segment(
        &template,
        text_start,
        template.len(),
        &mut display_text,
        &mut render_format,
    );

    Ok(TemplateSpec {
        display_text,
        render_format,
        expected_args: placeholders.len(),
        render_indices,
    })
}

fn push_text_segment(
    template: &str,
    start: usize,
    end: usize,
    display_text: &mut String,
    render_format: &mut String,
) {
    if start < end {
        let segment = &template[start..end];
        display_text.push_str(segment);
        render_format.push_str(segment);
    }
}

fn find_placeholder_end(
    bytes: &[u8],
    opening_brace: usize,
    literal: &LitStr,
) -> syn::Result<usize> {
    let mut index = opening_brace + 1;

    while index < bytes.len() && bytes[index] != b'}' {
        if bytes[index] == b'{' {
            return Err(Error::new_spanned(literal, "invalid template literal"));
        }

        index += 1;
    }

    if index >= bytes.len() || index == opening_brace + 1 {
        Err(Error::new_spanned(literal, "invalid template literal"))
    } else {
        Ok(index)
    }
}

fn validate_identifier(identifier: &str, literal: &LitStr) -> syn::Result<()> {
    let mut bytes = identifier.bytes();
    let Some(first) = bytes.next() else {
        return Err(Error::new_spanned(literal, "invalid template literal"));
    };

    if !(first == b'_' || first.is_ascii_alphabetic()) {
        return Err(Error::new_spanned(literal, "invalid template literal"));
    }

    if bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric()) {
        Ok(())
    } else {
        Err(Error::new_spanned(literal, "invalid template literal"))
    }
}

fn build_tokens(literal: &LitStr, spec: TemplateSpec) -> TokenStream {
    let display_text = LitStr::new(&spec.display_text, literal.span());
    let template = LitStr::new(&literal.value(), literal.span());
    let expected_args = LitInt::new(&spec.expected_args.to_string(), Span::call_site());
    let render_format = LitStr::new(&spec.render_format, literal.span());
    let render_indices = spec
        .render_indices
        .into_iter()
        .map(Index::from)
        .collect::<Vec<_>>();

    let render = if render_indices.is_empty() {
        quote! {
            (|_: &[::std::string::String]| {
                ::std::format!(#render_format)
            }) as fn(&[::std::string::String]) -> ::std::string::String
        }
    } else {
        quote! {
            (|args: &[::std::string::String]| {
                ::std::format!(#render_format, #(args[#render_indices].as_str()),*)
            }) as fn(&[::std::string::String]) -> ::std::string::String
        }
    };

    quote! {
        (#display_text, #template, #expected_args, #render)
    }
    .into()
}
