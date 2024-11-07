use bevy::prelude::Image;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat,
};
use image::ImageReader;

pub fn load_image(path: &String) -> Image {
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();
    Image::new(
        Extent3d {
            width: img.width(),
            height: img.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        img.into_raw(), // converts to Vec<u8>
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

pub mod image_gen {
    use std::collections::HashMap;

    use bevy::asset::Handle;
    use bevy::ecs::prelude::Resource;
    use bevy::prelude::Image;
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{
        Extent3d, TextureDimension, TextureFormat,
    };
    use wyrand::WyRand;

    use crate::resource::rune;

    // For images that have already been generated.
    #[derive(Default, Resource)]
    pub struct GeneratedImageAssets(pub HashMap<String, Handle<Image>>);

    impl GeneratedImageAssets {
        pub fn insert(&mut self, uid: String, image: &Handle<Image>) {
            self.0.insert(uid, image.clone());
        }

        pub fn get(&self, uid: &String) -> Option<Handle<Image>> {
            match self.0.get(uid) {
                Some(handle) => Some(handle.clone()),
                None => None,
            }
        }
    }

    pub struct ColorPalette {
        pub colorants: Vec<Colorant>,
        pub total_weight: u64,
    }

    impl ColorPalette {
        pub fn new() -> ColorPalette {
            ColorPalette {
                colorants: Vec::new(),
                total_weight: 0,
            }
        }

        pub fn add_colorant(&mut self, color: Colorant) -> &mut Self {
            self.colorants.push(color);
            self.total_weight += color.weight as u64;
            self
        }

        pub fn adjust_alpha_looseness(
            &self,
            alpha_looseness: u8,
        ) -> ColorPalette {
            let mut new_palette = ColorPalette::new();
            for colorant in &self.colorants {
                new_palette.add_colorant(
                    colorant.adjust_alpha_looseness(alpha_looseness),
                );
            }
            new_palette
        }

        pub fn pick(&self, rand: &mut WyRand) -> Colorant {
            let mut pick = rand.rand() % self.total_weight;
            for color in &self.colorants {
                if pick < color.weight as u64 {
                    return *color;
                }
                pick -= color.weight as u64;
            }
            panic!("ColorPalette::pick: should never get here");
        }

        pub fn pick_color(&self, rand: &mut WyRand) -> Color {
            self.pick(rand).pick(rand)
        }

        // simply draw a pixel for each coordinate
        pub fn draw_block(&self, rand: &mut WyRand, size: u32) -> Image {
            let mut colors = Colors::new(size, size);
            for _ in 0..(size * size) {
                colors.add_color(self.pick_color(rand));
            }
            colors.to_image()
        }

        // draw a non-transparent pixel for each coordinate within a radius
        // draw a fully transparent pixel for each coordinate outside the radius
        pub fn draw_ball(&self, rand: &mut WyRand, size: u32) -> Image {
            let radius = size / 2;
            let radius2 = (radius * radius) as i32;
            let mut colors = Colors::new(size, size);
            for x in 0..size {
                for y in 0..size {
                    let x = x as i32 - radius as i32;
                    let y = y as i32 - radius as i32;
                    let distance2 = x * x + y * y;
                    if distance2 < radius2 {
                        colors.add_color(self.pick_color(rand));
                    } else {
                        colors.add_color(Color::new_clear());
                    }
                }
            }
            colors.to_image()
        }

        // draw a triangle with a rounded top
        // (written by claude)
        pub fn draw_powder(&self, rand: &mut WyRand, size: u32) -> Image {
            let radius = size / 2;
            let radius2 = radius * radius;
            let mut colors = Colors::new(size, size);

            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - radius as f32;
                    let dy = y as f32 - radius as f32;

                    let vertical_factor = (y as f32 / size as f32).powf(0.3);
                    let width_multiplier = 0.6 + vertical_factor * 0.8;

                    let squeeze_factor =
                        1.0 + (1.0 - vertical_factor).powf(0.7) * 1.2;
                    let adjusted_dx = dx * squeeze_factor;

                    let adjusted_dy = if dy > 0.0 {
                        dy * (1.0 + (y as f32 / size as f32).powf(2.0) * 0.2)
                    } else {
                        dy
                    };

                    let adjusted_distance2 =
                        adjusted_dx * adjusted_dx + adjusted_dy * adjusted_dy;

                    if adjusted_distance2
                        < (radius2 as f32 * width_multiplier * width_multiplier)
                            as f32
                    {
                        colors.add_color(self.pick_color(rand));
                    } else {
                        colors.add_color(Color::new_clear());
                    }
                }
            }
            colors.to_image()
        }

        // draw four irregularly overlapping circles
        // (written by claude)
        pub fn draw_lump(&self, rand: &mut WyRand, size: u32) -> Image {
            let radius = size / 2;
            let small_radius = (radius as f32 * 0.6) as u32;
            let small_radius2 = small_radius * small_radius;
            let mut colors = Colors::new(size, size);

            // Generate four random centers using u64 and converting to appropriate range
            let mut centers = Vec::with_capacity(4);
            for _ in 0..4 {
                let offset_x =
                    (rand.rand() as f32 / u64::MAX as f32 - 0.5) * 0.8;
                let offset_y =
                    (rand.rand() as f32 / u64::MAX as f32 - 0.5) * 0.8;
                centers.push((offset_x, offset_y));
            }

            for y in 0..size {
                for x in 0..size {
                    let mut in_shape = false;

                    for &(offset_x, offset_y) in &centers {
                        let center_x =
                            radius as f32 + (radius as f32 * offset_x);
                        let center_y =
                            radius as f32 + (radius as f32 * offset_y);

                        let dx = x as f32 - center_x;
                        let dy = y as f32 - center_y;
                        let distance2 = (dx * dx + dy * dy) as u32;

                        if distance2 < small_radius2 {
                            in_shape = true;
                            break;
                        }
                    }

                    if in_shape {
                        colors.add_color(self.pick_color(rand));
                    } else {
                        colors.add_color(Color::new_clear());
                    }
                }
            }
            colors.to_image()
        }

        pub fn draw_shovel_head(&self, rand: &mut WyRand, size: u32) -> Image {
            let radius = size / 2;
            let radius2 = radius * radius;
            let mut colors = Colors::new(size, size);

            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - radius as f32;
                    let dy = y as f32 - radius as f32;

                    // Make shape wider at bottom, narrower at top
                    let vertical_factor = (y as f32 / size as f32).powf(0.5); // More rounded at bottom
                    let width_multiplier = 0.8 + vertical_factor * 0.4; // Varies from 0.8 at top to 1.2 at bottom

                    // Horizontal squeezing that increases toward top
                    let squeeze_factor = 1.0 + (1.0 - vertical_factor) * 0.5;
                    let adjusted_dx = dx * squeeze_factor;

                    // Calculate adjusted distance for the shape
                    let adjusted_distance2 =
                        adjusted_dx * adjusted_dx + dy * dy;

                    if adjusted_distance2
                        < (radius2 as f32 * width_multiplier * width_multiplier)
                            as f32
                    {
                        colors.add_color(self.pick_color(rand));
                    } else {
                        colors.add_color(Color::new_clear());
                    }
                }
            }
            colors.to_image()
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Colorant {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub alpha: u8,
        // how often this color should appear in the ColorPalette
        pub weight: u8,
        // how much the color varies; 0 is none and 255 is all
        // (128 is all if t he colors are all 127 or 128)
        pub looseness: u8,
        pub alpha_looseness: u8,
    }

    impl Colorant {
        pub fn new(
            red: u8,
            green: u8,
            blue: u8,
            alpha: u8,
            weight: u8,
            looseness: u8,
            alpha_looseness: u8,
        ) -> Colorant {
            Colorant {
                red,
                green,
                blue,
                alpha,
                weight,
                looseness,
                alpha_looseness,
            }
        }

        pub fn new_tight(red: u8, green: u8, blue: u8, weight: u8) -> Colorant {
            Colorant {
                red,
                green,
                blue,
                alpha: 255,
                weight,
                looseness: 0,
                alpha_looseness: 0,
            }
        }

        pub fn new_loose(
            red: u8,
            green: u8,
            blue: u8,
            looseness: u8,
            weight: u8,
        ) -> Colorant {
            Colorant {
                red,
                green,
                blue,
                alpha: 255,
                weight,
                looseness,
                alpha_looseness: 0,
            }
        }

        pub fn adjust_alpha_looseness(&self, alpha_looseness: u8) -> Colorant {
            Colorant {
                alpha_looseness,
                ..*self
            }
        }

        pub fn pick(&self, rand: &mut WyRand) -> Color {
            let red: u8;
            let green: u8;
            let blue: u8;
            if self.looseness == 0 {
                red = self.red;
                green = self.green;
                blue = self.blue;
            } else {
                red = Self::random_of_color(self.red, rand, self.looseness);
                green = Self::random_of_color(self.green, rand, self.looseness);
                blue = Self::random_of_color(self.blue, rand, self.looseness);
            }
            let alpha: u8;
            if self.alpha_looseness == 0 {
                alpha = self.alpha;
            } else {
                alpha = Self::random_of_color(
                    self.alpha,
                    rand,
                    self.alpha_looseness,
                );
            }
            Color::new(red, green, blue, alpha)
        }

        fn random_of_color(base: u8, rand: &mut WyRand, looseness: u8) -> u8 {
            let r = rand.rand() % (looseness as u64 + 1);
            if r < looseness as u64 / 2 {
                if r > base as u64 {
                    0
                } else {
                    base - r as u8
                }
            } else {
                if r + base as u64 > 255 {
                    255
                } else {
                    base + r as u8
                }
            }
        }
    }

    pub struct Color {
        pub red: u8,
        pub green: u8,
        pub blue: u8,
        pub alpha: u8,
    }

    impl Color {
        pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
            Color {
                red,
                green,
                blue,
                alpha,
            }
        }

        pub fn new_clear() -> Color {
            Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            }
        }
    }

    pub struct Colors {
        pub bytes: Vec<u8>,
        pub width: u32,
        pub height: u32,
    }

    impl Colors {
        pub fn new(width: u32, height: u32) -> Colors {
            Colors {
                bytes: Vec::with_capacity((width * height) as usize * 4),
                width,
                height,
            }
        }

        pub fn add_color(&mut self, color: Color) {
            self.bytes.push(color.red);
            self.bytes.push(color.green);
            self.bytes.push(color.blue);
            self.bytes.push(color.alpha);
        }

        pub fn to_image(&self) -> Image {
            // let data = colors
            // .iter()
            // .flat_map(|color| {
            // vec![color.red, color.green, color.blue, color.alpha]
            // })
            // .collect::<Vec<u8>>();
            Image::new(
                Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                self.bytes.clone(),
                // TextureFormat::Rgba8Unorm,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
        }
    }

    pub const RUNE_SIZE: usize = 50;

    pub fn draw_rune(r: rune::Rune) -> Image {
        let bits: Vec<Vec<bool>> = rune::rune_to_pixels(&r);
        let height = bits.len();
        let width = bits[0].len();
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                data.extend_from_slice(&[
                    0,
                    0,
                    0,
                    if bits[y][x] { 255 } else { 0 },
                ]);
            }
        }

        Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
    }
}
