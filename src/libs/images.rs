use bevy::prelude::Image;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat,
};
use image::ImageReader;

pub fn load_image(path: String) -> Image {
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();

    let width = img.width();
    let height = img.height();

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        img.into_raw(), // converts to Vec<u8>
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

pub mod image_gen {
    use bevy::prelude::Image;
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{
        Extent3d, TextureDimension, TextureFormat,
    };
    use wyrand::WyRand;

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

        pub fn add_color(&mut self, color: Colorant) {
            self.colorants.push(color);
            self.total_weight += color.weight as u64;
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

        pub fn draw_block(&self, rand: &mut WyRand, size: u32) -> Image {
            let mut colors = Colors::new(size, size);
            for _ in 0..(size * size) {
                colors.add_color(self.pick_color(rand));
            }
            colors.to_image()
        }

        pub fn draw_ball(&self, rand: &mut WyRand, radius: u32) -> Image {
            let size = radius * 2;
            let mut colors = Colors::new(size, size);
            let radius2 = radius * radius;
            let mut x = 0;
            let mut y = 0;
            while y < radius {
                let x2 = x * x;
                let y2 = y * y;
                let distance2 = x2 + y2;
                if distance2 < radius2 {
                    colors.add_color(self.pick_color(rand));
                } else {
                    x += 1;
                    y = 0;
                }
                y += 1;
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

        pub fn new_tight(
            red: u8,
            green: u8,
            blue: u8,
            alpha: u8,
            weight: u8,
        ) -> Colorant {
            Colorant {
                red,
                green,
                blue,
                alpha,
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

        pub fn new_random(weight: u8) -> Colorant {
            Colorant {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255,
                weight,
                looseness: 255,
                alpha_looseness: 0,
            }
        }

        pub fn pick(&self, rand: &mut WyRand) -> Color {
            let red: u8;
            let green: u8;
            let blue: u8;
            let alpha: u8;
            if self.looseness == 0 {
                red = self.red;
                green = self.green;
                blue = self.blue;
            } else {
                red = (rand.rand() % (self.looseness as u64 + 1)) as u8;
                green = (rand.rand() % (self.looseness as u64 + 1)) as u8;
                blue = (rand.rand() % (self.looseness as u64 + 1)) as u8;
            }
            if self.alpha_looseness == 0 {
                alpha = self.alpha;
            } else {
                alpha = (rand.rand() % (self.alpha_looseness as u64 + 1)) as u8;
            }
            Color::new(red, green, blue, alpha)
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
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
            )
        }
    }
}
