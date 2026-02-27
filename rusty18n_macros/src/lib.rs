use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Error, Index, LitInt, LitStr};
use winnow::{
    combinator::{alt, delimited, repeat},
    error::ModalResult,
    token::take_while,
    Parser,
};

struct TemplateSpec {
    display_text: String,
    render_format: String,
    expected_args: usize,
    render_indices: Vec<usize>,
}

#[derive(Clone, Copy)]
enum TemplatePart<'a> {
    Text(&'a str),
    Escaped(char),
    Placeholder(&'a str),
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
    let mut display_text = String::with_capacity(template.len());
    let mut render_format = String::with_capacity(template.len());
    let mut placeholders = Vec::new();
    let mut render_indices = Vec::new();
    let parts = repeat::<_, _, Vec<_>, _, _>(0.., template_part)
        .parse(template.as_str())
        .map_err(|_| Error::new_spanned(literal, "invalid template literal"))?;

    for part in parts {
        match part {
            TemplatePart::Text(text) => {
                display_text.push_str(text);
                render_format.push_str(text);
            }
            TemplatePart::Escaped(ch) => {
                display_text.push(ch);
                render_format.push(match ch {
                    '{' => '{',
                    '}' => '}',
                    _ => ch,
                });
                render_format.push(ch);
            }
            TemplatePart::Placeholder(name) => {
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
            }
        }
    }

    Ok(TemplateSpec {
        display_text,
        render_format,
        expected_args: placeholders.len(),
        render_indices,
    })
}

fn template_part<'a>(input: &mut &'a str) -> ModalResult<TemplatePart<'a>> {
    alt((
        "{{".value(TemplatePart::Escaped('{')),
        "}}".value(TemplatePart::Escaped('}')),
        delimited('{', identifier, '}').map(TemplatePart::Placeholder),
        take_while(1.., |ch: char| ch != '{' && ch != '}').map(TemplatePart::Text),
    ))
    .parse_next(input)
}

fn identifier<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    (
        take_while(1..=1, is_identifier_start),
        take_while(0.., is_identifier_continue),
    )
        .take()
        .parse_next(input)
}

const fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

const fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
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
