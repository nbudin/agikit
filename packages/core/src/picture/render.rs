use std::collections::VecDeque;

use picture_pen_macros::picture_pen_mask;

use crate::{
    color_palettes::ColorPalette,
    picture::{
        Picture, PictureCommand, PictureCoordinate, PictureCornerStep, PictureCornerStepAxis,
        PicturePenSettings, PicturePenShape,
    },
};

pub struct PicturePenMask {
    pub origin: PictureCoordinate,
    pub width: u8,
    pub height: u8,
    pub mask: &'static [bool],
}

pub static RECTANGLE_MASKS: &[PicturePenMask] = &[
    picture_pen_mask!("*"),
    picture_pen_mask!(
        r#"
XX
X*
XX
"#
    ),
    picture_pen_mask!(
        r#"
XXX
XXX
X*X
XXX
XXX
    "#
    ),
    picture_pen_mask!(
        r#"
XXXX
XXXX
XXXX
XX*X
XXXX
XXXX
XXXX
      "#
    ),
    picture_pen_mask!(
        r#"
XXXXX
XXXXX
XXXXX
XXXXX
XX*XX
XXXXX
XXXXX
XXXXX
XXXXX
      "#
    ),
    picture_pen_mask!(
        r#"
XXXXXX
XXXXXX
XXXXXX
XXXXXX
XXXXXX
XXX*XX
XXXXXX
XXXXXX
XXXXXX
XXXXXX
XXXXXX
      "#
    ),
    picture_pen_mask!(
        r#"
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXX*XXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
XXXXXXX
      "#
    ),
    picture_pen_mask!(
        r#"
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXX*XXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
XXXXXXXX
      "#
    ),
];

pub static CIRCLE_MASKS: &[PicturePenMask] = &[
    picture_pen_mask!("*"),
    picture_pen_mask!(
        r#"
XX
X*
XX
  "#
    ),
    picture_pen_mask!(
        r#"
 X
XXX
X*X
XXX
 X
      "#
    ),
    picture_pen_mask!(
        r#"
 XX
 XX
XXXX
XX*X
XXXX
 XX
 XX
      "#
    ),
    picture_pen_mask!(
        r#"
  X
 XXX
XXXXX
XXXXX
XX*XX
XXXXX
XXXXX
 XXX
  X
      "#
    ),
    picture_pen_mask!(
        r#"
  XX
 XXXX
 XXXX
 XXXX
XXXXXX
XXX*XX
XXXXXX
 XXXX
 XXXX
 XXXX
  XX
      "#
    ),
    picture_pen_mask!(
        r#"
  XXX
 XXXXX
 XXXXX
 XXXXX
XXXXXXX
XXXXXXX
XXX*XXX
XXXXXXX
XXXXXXX
 XXXXX
 XXXXX
 XXXXX
  XXX
      "#
    ),
    picture_pen_mask!(
        r#"
   XX
  XXXX
 XXXXXX
 XXXXXX
 XXXXXX
XXXXXXXX
XXXXXXXX
XXXX*XXX
XXXXXXXX
XXXXXXXX
 XXXXXX
 XXXXXX
 XXXXXX
  XXXX
   XX
      "#
    ),
];

// from http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_7_-_Picture_Resources#ss7.1
pub static PEN_TEXTURE_PATTERNS: [u8; 32] = [
    0x20, 0x94, 0x02, 0x24, 0x90, 0x82, 0xa4, 0xa2, 0x82, 0x09, 0x0a, 0x22, 0x12, 0x10, 0x42, 0x14,
    0x91, 0x4a, 0x91, 0x11, 0x08, 0x12, 0x25, 0x10, 0x22, 0xa8, 0x14, 0x24, 0x00, 0x50, 0x24, 0x04,
];

// from http://agiwiki.sierrahelp.com/index.php?title=AGI_Specifications:_Chapter_7_-_Picture_Resources#ss7.1
pub static PEN_TEXTURE_START_POSITIONS: [u8; 120] = [
    0x00, 0x18, 0x30, 0xc4, 0xdc, 0x65, 0xeb, 0x48, 0x60, 0xbd, 0x89, 0x04, 0x0a, 0xf4, 0x7d, 0x6d,
    0x85, 0xb0, 0x8e, 0x95, 0x1f, 0x22, 0x0d, 0xdf, 0x2a, 0x78, 0xd5, 0x73, 0x1c, 0xb4, 0x40, 0xa1,
    0xb9, 0x3c, 0xca, 0x58, 0x92, 0x34, 0xcc, 0xce, 0xd7, 0x42, 0x90, 0x0f, 0x8b, 0x7f, 0x32, 0xed,
    0x5c, 0x9d, 0xc8, 0x99, 0xad, 0x4e, 0x56, 0xa6, 0xf7, 0x68, 0xb7, 0x25, 0x82, 0x37, 0x3a, 0x51,
    0x69, 0x26, 0x38, 0x52, 0x9e, 0x9a, 0x4f, 0xa7, 0x43, 0x10, 0x80, 0xee, 0x3d, 0x59, 0x35, 0xcf,
    0x79, 0x74, 0xb5, 0xa2, 0xb1, 0x96, 0x23, 0xe0, 0xbe, 0x05, 0xf5, 0x6e, 0x19, 0xc5, 0x66, 0x49,
    0xf0, 0xd1, 0x54, 0xa9, 0x70, 0x4b, 0xa4, 0xe2, 0xe6, 0xe5, 0xab, 0xe4, 0xd2, 0xaa, 0x4c, 0xe3,
    0x06, 0x6f, 0xc6, 0x4a, 0x75, 0xa3, 0x97, 0xe1,
];

pub static DEFAULT_PEN_SETTINGS: PicturePenSettings = PicturePenSettings::new()
    .with_shape(PicturePenShape::Rectangle)
    .with_size(0)
    .with_splatter(false);

pub struct PixelBuffer<Pixel: Clone> {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Pixel>,
}

impl<Pixel: Clone> PixelBuffer<Pixel> {
    pub fn new(width: usize, height: usize, color: Pixel) -> Self {
        Self {
            width,
            height,
            buffer: vec![color; width * height],
        }
    }

    pub fn get_pixel(&self, pos: &PictureCoordinate) -> Option<Pixel> {
        self.buffer
            .get(pos.y as usize * self.width + pos.x as usize)
            .cloned()
    }

    pub fn set_pixel(&mut self, pos: &PictureCoordinate, value: &Pixel) {
        self.buffer[pos.y as usize * self.width + pos.x as usize] = value.clone();
    }

    // ported from http://agiwiki.sierrahelp.com/index.php?title=Picture_Resource_(AGI)
    pub fn draw_line(&mut self, from: &PictureCoordinate, to: &PictureCoordinate, color: Pixel) {
        let height = to.y as isize - from.y as isize;
        let width = to.x as isize - from.x as isize;
        let add_x = if height == 0 {
            0.0
        } else {
            width as f64 / height.abs() as f64
        };
        let add_y = if width == 0 {
            0.0
        } else {
            height as f64 / width.abs() as f64
        };
        let mut x = from.x as f64;
        let mut y = from.y as f64;

        if width.abs() > height.abs() {
            let add_x = if width == 0 {
                0.0
            } else {
                width as f64 / width.abs() as f64
            };
            while x as u8 != to.x {
                self.set_pixel(
                    &PictureCoordinate {
                        x: direction_biased_round(x, add_x) as u8,
                        y: direction_biased_round(y, add_y) as u8,
                    },
                    &color,
                );
                x += add_x;
                y += add_y;
            }
        } else {
            let add_y = if height == 0 {
                0.0
            } else {
                height as f64 / height.abs() as f64
            };
            while y as u8 != to.y {
                self.set_pixel(
                    &PictureCoordinate {
                        x: direction_biased_round(x, add_x) as u8,
                        y: direction_biased_round(y, add_y) as u8,
                    },
                    &color,
                );
                x += add_x;
                y += add_y;
            }
        }

        self.set_pixel(to, &color);
    }

    pub fn plot_with_pen(
        &mut self,
        pos: &PictureCoordinate,
        pen: PicturePenSettings,
        texture: Option<u8>,
        color: Pixel,
    ) {
        let mask = match pen.shape() {
            PicturePenShape::Rectangle => &RECTANGLE_MASKS[pen.size() as usize],
            PicturePenShape::Circle => &CIRCLE_MASKS[pen.size() as usize],
        };
        let mut mask_on_pixel_count: usize = 0; // texture bitmap only affects masked-on pixels; only count those
        let texture_start_position = texture.and_then(|texture| {
            if pen.splatter() {
                Some(PEN_TEXTURE_START_POSITIONS[texture as usize])
            } else {
                None
            }
        });

        for (index, mask_on) in mask.mask.iter().enumerate() {
            if !*mask_on {
                continue;
            }

            if let Some(texture_start_position) = texture_start_position {
                // yes, mod 255, per the AGI spec.  Lance Ewing thinks it was a bug in AGI itself
                let texture_position =
                    (texture_start_position as usize + mask_on_pixel_count) % 255;
                let texture_byte = PEN_TEXTURE_PATTERNS[texture_position / 8];
                let texture_bit = texture_byte & (1 << (texture_position % 8));
                mask_on_pixel_count += 1;
                if texture_bit == 0 {
                    continue;
                }
            }

            let mask_x = index as isize % mask.width as isize;
            let mask_y = index as isize / mask.width as isize;
            let logical_x = mask_x - mask.origin.x as isize;
            let logical_y = mask_y - mask.origin.y as isize;
            let pixel_pos = PictureCoordinate {
                x: (logical_x + pos.x as isize) as u8,
                y: (logical_y + pos.y as isize) as u8,
            };
            self.set_pixel(&pixel_pos, &color);
        }
    }
}

impl<Pixel: Clone + Into<usize>> PixelBuffer<Pixel> {
    pub fn to_rgba_data(&self, color_palette: &ColorPalette) -> Vec<u8> {
        self.buffer
            .iter()
            .cloned()
            .flat_map(|pixel| color_palette.colors[pixel.into()])
            .collect()
    }
}

struct FloodFillTarget<'a, Pixel: Clone> {
    pub buffer: &'a mut PixelBuffer<Pixel>,
    pub color: Pixel,
}

struct FloodFillCheckBuffer<Pixel: Clone> {
    pub target_index: usize,
    pub background_color: Pixel,
}

fn flood_fill<'a, Pixel: Clone + PartialEq>(
    start_position: &PictureCoordinate,
    targets: &mut [FloodFillTarget<'a, Pixel>],
    check_buffers: &[FloodFillCheckBuffer<Pixel>],
) {
    let mut queue: VecDeque<PictureCoordinate> = VecDeque::from([start_position.clone()]);
    let width = targets[0].buffer.width;
    let height = targets[0].buffer.height;
    let mut visited = PixelBuffer::new(width, height, false);

    while queue.len() > 0 {
        let current_position = queue.pop_front().unwrap();
        if visited.get_pixel(&current_position).unwrap() {
            continue;
        } else {
            visited.set_pixel(&current_position, &true);
        }

        let is_background_pixel = check_buffers.iter().all(|check_buffer| {
            let check_color = targets[check_buffer.target_index]
                .buffer
                .get_pixel(&current_position)
                .unwrap();
            check_color == check_buffer.background_color
        });

        if is_background_pixel {
            for target in targets.iter_mut() {
                target.buffer.set_pixel(&current_position, &target.color);
            }
            if current_position.x > 0 {
                queue.push_back(PictureCoordinate {
                    x: current_position.x - 1,
                    y: current_position.y,
                });
            }
            if current_position.y > 0 {
                queue.push_back(PictureCoordinate {
                    x: current_position.x,
                    y: current_position.y - 1,
                });
            }
            if current_position.x as usize + 1 < width {
                queue.push_back(PictureCoordinate {
                    x: current_position.x + 1,
                    y: current_position.y,
                });
            }
            if current_position.y as usize + 1 < height {
                queue.push_back(PictureCoordinate {
                    x: current_position.x,
                    y: current_position.y + 1,
                });
            }
        }
    }
}

pub struct RenderedPicture {
    pub visual_buffer: PixelBuffer<u8>,
    pub priority_buffer: PixelBuffer<u8>,
}

impl RenderedPicture {
    pub fn draw_line(
        &mut self,
        from: &PictureCoordinate,
        to: &PictureCoordinate,
        visual_color: Option<u8>,
        priority_color: Option<u8>,
    ) {
        if let Some(visual_color) = visual_color {
            self.visual_buffer.draw_line(from, to, visual_color);
        }
        if let Some(priority_color) = priority_color {
            self.priority_buffer.draw_line(from, to, priority_color);
        }
    }

    pub fn flood_fill(
        &mut self,
        start_position: &PictureCoordinate,
        visual_color: Option<u8>,
        priority_color: Option<u8>,
    ) {
        if let Some(visual_color) = visual_color {
            let mut targets: Vec<FloodFillTarget<u8>> = vec![FloodFillTarget {
                buffer: &mut self.visual_buffer,
                color: visual_color,
            }];
            if let Some(priority_color) = priority_color {
                targets.push(FloodFillTarget {
                    buffer: &mut self.priority_buffer,
                    color: priority_color,
                });
            }
            flood_fill(
                start_position,
                &mut targets,
                &[FloodFillCheckBuffer {
                    target_index: 0,
                    background_color: 15,
                }],
            );
        } else if let Some(priority_color) = priority_color {
            flood_fill(
                start_position,
                &mut [FloodFillTarget {
                    buffer: &mut self.priority_buffer,
                    color: priority_color,
                }],
                &[FloodFillCheckBuffer {
                    target_index: 0,
                    background_color: 4,
                }],
            );
        }
    }

    pub fn plot_with_pen(
        &mut self,
        pos: &PictureCoordinate,
        pen: PicturePenSettings,
        texture: Option<u8>,
        visual_color: Option<u8>,
        priority_color: Option<u8>,
    ) {
        if let Some(visual_color) = visual_color {
            self.visual_buffer
                .plot_with_pen(pos, pen, texture, visual_color);
        }
        if let Some(priority_color) = priority_color {
            self.priority_buffer
                .plot_with_pen(pos, pen, texture, priority_color);
        }
    }

    pub fn draw_corner_steps(
        &mut self,
        start_position: &PictureCoordinate,
        steps: &[PictureCornerStep],
        visual_color: Option<u8>,
        priority_color: Option<u8>,
    ) {
        let mut last_point = start_position.clone();
        for step in steps.iter() {
            let point = match step.axis {
                PictureCornerStepAxis::X => PictureCoordinate {
                    x: step.position,
                    y: last_point.y,
                },
                PictureCornerStepAxis::Y => PictureCoordinate {
                    x: last_point.x,
                    y: step.position,
                },
            };
            self.draw_line(&last_point, &point, visual_color, priority_color);
            last_point = point;
        }
    }
}

// ported from http://agiwiki.sierrahelp.com/index.php?title=Picture_Resource_(AGI)
pub fn direction_biased_round(number: f64, direction: f64) -> f64 {
    if direction < 0.0 {
        if number - number.floor() <= 0.501 {
            number.floor()
        } else {
            number.ceil()
        }
    } else {
        if number - number.floor() < 0.499 {
            number.floor()
        } else {
            number.ceil()
        }
    }
}

impl Picture {
    pub fn render_to(
        &self,
        rendered_picture: &mut RenderedPicture,
        starting_picture_color: Option<u8>,
        starting_priority_color: Option<u8>,
        starting_pen: PicturePenSettings,
    ) {
        let mut picture_color = starting_picture_color;
        let mut priority_color = starting_priority_color;
        let mut pen = starting_pen;

        for command in self.commands.iter() {
            match command {
                PictureCommand::SetPictureColor(command) => {
                    picture_color = Some(command.color_number);
                }
                PictureCommand::DisablePictureDraw(_) => {
                    picture_color = None;
                }
                PictureCommand::SetPriorityColor(command) => {
                    priority_color = Some(command.color_number);
                }
                PictureCommand::DisablePriorityDraw(_) => {
                    priority_color = None;
                }
                PictureCommand::AbsoluteLine(command) => {
                    if command.points.len() > 1 {
                        let mut last_point = &command.points[0];
                        for point in command.points.iter().skip(1) {
                            rendered_picture.draw_line(
                                last_point,
                                point,
                                picture_color,
                                priority_color,
                            );
                            last_point = point;
                        }
                    }
                }
                PictureCommand::RelativeLine(command) => {
                    let mut last_point = command.start_position.clone();
                    for relative_point in command.relative_points.iter() {
                        let point = PictureCoordinate {
                            x: last_point
                                .x
                                .checked_add_signed(relative_point.x_displacement().value())
                                .unwrap(),
                            y: last_point
                                .y
                                .checked_add_signed(relative_point.y_displacement().value())
                                .unwrap(),
                        };
                        rendered_picture.draw_line(
                            &last_point,
                            &point,
                            picture_color,
                            priority_color,
                        );
                        last_point = point;
                    }
                }
                PictureCommand::DrawYCorner(command) => {
                    rendered_picture.draw_corner_steps(
                        &command.start_position,
                        &command.steps,
                        picture_color,
                        priority_color,
                    );
                }
                PictureCommand::DrawXCorner(command) => {
                    rendered_picture.draw_corner_steps(
                        &command.start_position,
                        &command.steps,
                        picture_color,
                        priority_color,
                    );
                }
                PictureCommand::Fill(command) => {
                    for start_position in command.start_positions.iter() {
                        rendered_picture.flood_fill(&start_position, picture_color, priority_color);
                    }
                }
                PictureCommand::ChangePen(command) => {
                    pen = command.settings;
                }
                PictureCommand::PlotWithPen(command) => {
                    for point in command.points.iter() {
                        rendered_picture.plot_with_pen(
                            &point.position,
                            pen,
                            point.texture,
                            picture_color,
                            priority_color,
                        );
                    }
                }
                PictureCommand::End(_) => {}
            }
        }
    }

    pub fn render(&self) -> RenderedPicture {
        let visual_buffer = PixelBuffer::new(160, 168, 15u8);
        let priority_buffer = PixelBuffer::new(160, 168, 4u8);
        let mut rendered_picture = RenderedPicture {
            visual_buffer,
            priority_buffer,
        };
        self.render_to(&mut rendered_picture, None, None, DEFAULT_PEN_SETTINGS);
        rendered_picture
    }
}

#[cfg(feature = "js")]
mod js {
    use std::collections::HashMap;

    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
    use web_sys::js_sys::Uint8Array;

    use crate::{
        color_palettes::ColorPalette,
        picture::{
            Picture, PicturePenSettings,
            render::{PixelBuffer, RenderedPicture},
        },
    };

    #[wasm_bindgen(js_name = "RenderedPicture")]
    #[derive(Clone)]
    pub struct JsRenderedPicture {
        #[wasm_bindgen(js_name = "visualBuffer", getter_with_clone)]
        pub visual_buffer: Uint8Array,
        #[wasm_bindgen(js_name = "priorityBuffer", getter_with_clone)]
        pub priority_buffer: Uint8Array,
    }

    impl JsRenderedPicture {
        pub fn to_rendered_picture(
            &self,
            color_palette: &ColorPalette,
        ) -> Result<RenderedPicture, JsValue> {
            let inverted_palette = color_palette
                .colors
                .iter()
                .enumerate()
                .map(|(color_number, color)| (color as &[u8], color_number))
                .collect::<HashMap<_, _>>();

            let color_buffer_to_pixel_buffer = |color_buffer: &Uint8Array| {
                color_buffer
                    .to_vec()
                    .as_slice()
                    .chunks(4)
                    .map(|pixel| {
                        inverted_palette
                            .get(pixel)
                            .ok_or_else(|| {
                                JsValue::from_str(
                                    format!("Color {:?} is not in palette", pixel).as_str(),
                                )
                            })
                            .map(|color_number| *color_number as u8)
                    })
                    .collect::<Result<Vec<_>, _>>()
            };
            let visual_buffer = color_buffer_to_pixel_buffer(&self.visual_buffer)?;
            let priority_buffer = color_buffer_to_pixel_buffer(&self.priority_buffer)?;

            Ok(RenderedPicture {
                visual_buffer: PixelBuffer {
                    buffer: visual_buffer,
                    width: 160,
                    height: 168,
                },
                priority_buffer: PixelBuffer {
                    buffer: priority_buffer,
                    width: 160,
                    height: 168,
                },
            })
        }
    }

    #[wasm_bindgen]
    pub struct JsRenderPictureStartingFromOptions {
        #[wasm_bindgen(js_name = "rendered_picture", getter_with_clone)]
        pub rendered_picture: JsRenderedPicture,
        #[wasm_bindgen(js_name = "pictureColor")]
        pub picture_color: Option<u8>,
        #[wasm_bindgen(js_name = "priorityColor")]
        pub priority_color: Option<u8>,
        pub pen: PicturePenSettings,
    }

    #[wasm_bindgen(js_name = "renderPicture")]
    pub fn render_picture(
        picture: &Picture,
        palette: &ColorPalette,
        #[wasm_bindgen(js_name = "startingFrom")] starting_from: Option<
            JsRenderPictureStartingFromOptions,
        >,
    ) -> Result<JsRenderedPicture, JsValue> {
        let rendered = match starting_from {
            Some(starting_from) => {
                let mut starting_rendered = starting_from
                    .rendered_picture
                    .to_rendered_picture(palette)?;
                picture.render_to(
                    &mut starting_rendered,
                    starting_from.picture_color,
                    starting_from.priority_color,
                    starting_from.pen,
                );
                starting_rendered
            }
            None => picture.render(),
        };

        let visual_buffer =
            Uint8Array::new_with_length((rendered.visual_buffer.buffer.len() * 4) as u32);
        let priority_buffer =
            Uint8Array::new_with_length((rendered.priority_buffer.buffer.len() * 4) as u32);
        visual_buffer.copy_from(&rendered.visual_buffer.to_rgba_data(palette));
        priority_buffer.copy_from(&rendered.priority_buffer.to_rgba_data(palette));

        Ok(JsRenderedPicture {
            visual_buffer,
            priority_buffer,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::picture::render::RECTANGLE_MASKS;

    #[test]
    pub fn test_pen_mask_parsing() {
        let rectangle1 = &RECTANGLE_MASKS[1];
        assert_eq!(2, rectangle1.width);
        assert_eq!(3, rectangle1.height);
        assert_eq!(1, rectangle1.origin.x);
        assert_eq!(1, rectangle1.origin.y);
        for pixel in rectangle1.mask.iter() {
            assert_eq!(true, *pixel);
        }
    }
}
