use std::{fs, path::Path, sync::Arc};

use skrifa::{raw::FileRef, FontRef, MetadataProvider};
use vello::{
    kurbo::{Affine, Rect},
    peniko::{Blob, Brush, BrushRef, Fill, Font, StyleRef},
    Glyph, Scene,
};

use crate::math::{Bounds2u, Vec2u};

pub struct Drawer {
    monospace_font: Arc<Font>,

    scene: Scene,
}

impl Drawer {
    pub fn init() -> Self {
        // TODO: Have backup font bundled, otherwise not everyone has this font
        let monospace_font_path = if cfg!(target_os = "windows") {
            Path::new(r"C:\Windows\Fonts\CascadiaCode.ttf")
        } else if cfg!(target_os = "macos") {
            Path::new("/Library/Fonts/cascadia_code.ttf")
        } else {
            panic!(
                "Unrecognized OS, cannot find font folder, should implement bundled backup font!"
            );
        };
        let monospace_font_bytes = fs::read(monospace_font_path).unwrap_or_else(|_| {
            panic!("Cannot find {monospace_font_path:?}, should implement bundled backup font!")
        });

        Self {
            monospace_font: Arc::new(Font::new(Blob::new(Arc::new(monospace_font_bytes)), 0)),
            scene: Scene::new(),
        }
    }

    pub fn reset(&mut self) {
        // we will re-use scene, instead of reallocating a new scene every frame
        self.scene.reset();
    }

    pub fn scene_ref(&self) -> &Scene {
        &self.scene
    }

    fn draw_text_impl<'b, 's, T: AsRef<str>>(
        &mut self,
        font: &Font,
        size: f32,
        variations: &[(&str, f32)],
        brush: impl Into<BrushRef<'b>>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        style: impl Into<StyleRef<'s>>,
        text: T,
    ) {
        let brush = brush.into();
        let style = style.into();

        let font_ref = to_font_ref(font).expect("valid font");
        let axes = font_ref.axes();
        let var_loc = axes.location(variations.iter().copied());

        self.scene
            .draw_glyphs(font)
            .font_size(size)
            .transform(transform)
            .glyph_transform(glyph_transform)
            .normalized_coords(bytemuck::cast_slice(var_loc.coords()))
            .brush(brush)
            .hint(false)
            .draw(
                style,
                get_glyphs(font, size, variations, text).0.into_iter(),
            );
    }

    pub fn draw_monospace_text<T: AsRef<str>>(
        &mut self,
        size: f32,
        brush: Brush,
        transform: Affine,
        text: T,
    ) {
        self.draw_text_impl(
            &self.monospace_font.clone(),
            size,
            &[],
            &brush,
            transform,
            None,
            Fill::NonZero,
            text.as_ref(),
        );
    }

    pub fn get_monospace_text_width<T: AsRef<str>>(&mut self, size: f32, text: T) -> Vec2u {
        get_glyphs(&self.monospace_font.clone(), size, &[], text)
            .1
            .size
    }

    pub fn draw_rect(&mut self, brush: Brush, bounds: Bounds2u) {
        let rect = Rect::new(
            bounds.pos.x as f64,
            bounds.pos.y as f64,
            (bounds.pos.x + bounds.size.x) as f64,
            (bounds.pos.y + bounds.size.y) as f64,
        );

        self.scene
            .fill(Fill::NonZero, Affine::IDENTITY, &brush, None, &rect);
    }
}

fn to_font_ref(font: &Font) -> Option<FontRef<'_>> {
    match FileRef::new(font.data.as_ref()).ok()? {
        FileRef::Font(font) => Some(font),
        FileRef::Collection(collection) => collection.get(font.index).ok(),
    }
}

fn get_glyphs<T: AsRef<str>>(
    font: &Font,
    font_size: f32,
    variations: &[(&str, f32)],
    text: T,
) -> (Vec<Glyph>, Bounds2u) {
    let font_ref = to_font_ref(font).expect("valid font");
    let axes = font_ref.axes();
    let font_size = skrifa::instance::Size::new(font_size);
    let var_loc = axes.location(variations.iter().copied());
    let charmap = font_ref.charmap();
    let metrics = font_ref.metrics(font_size, &var_loc);
    let line_height = metrics.ascent - metrics.descent + metrics.leading;
    let glyph_metrics = font_ref.glyph_metrics(font_size, &var_loc);

    let mut current_x = 0_f32;
    let mut current_y = 0_f32;

    let mut max_x = 0_f32;
    let mut max_y = 0_f32;

    let glyphs = text
        .as_ref()
        .chars()
        .filter_map(|ch| {
            max_y = max_y.max(current_y + line_height);

            if ch == '\n' {
                current_y += line_height;
                current_x = 0.0;
                return None;
            }

            let gid = charmap.map(ch).unwrap_or_default();
            let advance = glyph_metrics.advance_width(gid).unwrap_or_default();
            let x = current_x;
            current_x += advance;
            max_x = max_x.max(current_x);
            Some(Glyph {
                id: gid.to_u32(),
                x,
                // glyph's origin is bottom-left, unlike the normal convention of top-left...
                y: current_y + metrics.ascent + metrics.descent + 1.0,
            })
        })
        .collect();

    (
        glyphs,
        Bounds2u {
            pos: Vec2u::ZERO,
            size: Vec2u {
                x: max_x as u64,
                y: max_y as u64,
            },
        },
    )
}
