use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

struct Origin {
    pub x: usize,
    pub y: usize,
}

#[proc_macro]
pub fn picture_pen_mask(input: TokenStream) -> TokenStream {
    let mask_definition = parse_macro_input!(input as LitStr).value();
    let lines: Vec<&str> = mask_definition.lines().collect();
    let height = lines.len();
    let width = lines
        .iter()
        .map(|line| line.len())
        .max()
        .unwrap_or_else(|| lines[0].len());
    let mut mask = Vec::with_capacity(width * height);
    let mut origin: Option<Origin> = None;

    for (y, line) in lines.iter().enumerate() {
        let mut x = 0;
        let line_chars: Vec<_> = line.chars().collect();
        while x < width {
            let pixel = line_chars.get(x).unwrap_or(&' ');
            match pixel {
                'X' => mask.push(true),
                '*' => {
                    origin = Some(Origin { x, y });
                    mask.push(true);
                }
                ' ' => mask.push(true),
                _ => panic!("Unknown character {:?} found in mask definition", pixel),
            }

            x += 1;
        }
    }

    let Some(origin) = origin else {
        panic!("Origin not found in mask definition");
    };

    let x = origin.x as u8;
    let y = origin.y as u8;
    let width = width as u8;
    let height = height as u8;

    quote! {
        PicturePenMask {
            origin: PictureCoordinate {
                x: #x,
                y: #y
            },
            width: #width,
            height: #height,
            mask: &[#(#mask),*],
        }
    }
    .into()
}
