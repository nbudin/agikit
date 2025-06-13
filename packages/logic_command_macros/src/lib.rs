use std::{env, path::Path};

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;
use syn::{
    Ident, LitBool, LitInt, LitStr,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[derive(Serialize, Deserialize, AsRefStr)]
enum AGICommandArgType {
    Number,
    Variable,
    Flag,
    Message,
    Object,
    Item,
    String,
    Word,
    CtrlCode,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AGICommand {
    pub opcode: u8,
    pub name: String,
    pub arg_types: Vec<AGICommandArgType>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCommand {
    pub opcode: u8,
    pub name: String,
    pub arg_types: Vec<AGICommandArgType>,
    #[serde(default)]
    pub var_args: bool,
}

struct ProjectRelativePath {
    path: String,
    span: proc_macro2::Span,
}

impl Parse for ProjectRelativePath {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let path: LitStr = input.parse()?;
        let project_dir = env::var("CARGO_MANIFEST_DIR")
            .map_err(|_| syn::Error::new(path.span(), "CARGO_MANIFEST_DIR not set"))?;
        let file_path = Path::new(&project_dir).join(path.value());
        file_path.exists().then(|| ()).expect(
            format!(
                "The provided path {} does not exist or is not a valid file path",
                file_path.display()
            )
            .as_str(),
        );
        Ok(ProjectRelativePath {
            path: file_path.to_string_lossy().into(),
            span,
        })
    }
}

impl ProjectRelativePath {
    fn span(&self) -> proc_macro2::Span {
        self.span.clone()
    }
}

impl AsRef<Path> for &ProjectRelativePath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.path)
    }
}

#[proc_macro]
pub fn include_agi_commands(input: TokenStream) -> TokenStream {
    let json_path = parse_macro_input!(input as ProjectRelativePath);
    let json_content =
        std::fs::read_to_string(&json_path).expect("Failed to read AGI commands JSON file");
    let commands: Vec<AGICommand> =
        serde_json::from_str(&json_content).expect("Failed to parse AGI commands JSON");

    let command_tokens = commands
        .into_iter()
        .map(|cmd| {
            let opcode = LitInt::new(&cmd.opcode.to_string(), json_path.span());
            let name = LitStr::new(&cmd.name, json_path.span());
            let arg_types = cmd
                .arg_types
                .iter()
                .map(|arg| {
                    let ident = Ident::new(arg.as_ref(), json_path.span());
                    quote!(AGICommandArgType::#ident).to_token_stream()
                })
                .collect::<Vec<_>>();

            quote!(
                AGICommand { opcode: #opcode, name: #name.to_string(), arg_types: vec![#(#arg_types),*] }
            )
            .to_token_stream()
        })
        .collect::<Vec<_>>();

    quote! { vec![#(#command_tokens),*] }.into()
}

#[proc_macro]
pub fn include_test_commands(input: TokenStream) -> TokenStream {
    let json_path = parse_macro_input!(input as ProjectRelativePath);

    let json_content =
        std::fs::read_to_string(&json_path).expect("Failed to read test commands JSON file");
    let commands: Vec<TestCommand> =
        serde_json::from_str(&json_content).expect("Failed to parse test commands JSON");

    let command_tokens = commands
        .into_iter()
        .map(|cmd| {
            let opcode = LitInt::new(&cmd.opcode.to_string(), json_path.span());
            let name = LitStr::new(&cmd.name, json_path.span());
            let arg_types = cmd
                .arg_types
                .iter()
                .map(|arg| {
                    let ident = Ident::new(arg.as_ref(), json_path.span());
                    quote!(AGICommandArgType::#ident).to_token_stream()
                })
                .collect::<Vec<_>>();
            let var_args = LitBool::new(cmd.var_args, json_path.span());

            quote!(
                TestCommand { opcode: #opcode, name: #name.to_string(), arg_types: vec![#(#arg_types),*], var_args: #var_args }
            )
            .to_token_stream()
        })
        .collect::<Vec<_>>();

    quote! { vec![#(#command_tokens),*] }.into()
}
