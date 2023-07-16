use crate::ocr::MarkupBox;
use colorsys::{Hsl, Rgb};
use image::{DynamicImage, GenericImage};
use imageproc::rect::Rect;

fn safe_put_pixel(
    origin_image: &mut DynamicImage,
    x: u32,
    y: u32,
    color: image::Rgba<u8>,
) -> anyhow::Result<()> {
    if x < origin_image.width() && y < origin_image.height() {
        origin_image.put_pixel(x, y, color);
    }
    Ok(())
}

pub struct ImageMarkupDecorator {}

impl ImageMarkupDecorator {
    pub fn new() -> Self {
        ImageMarkupDecorator {}
    }

    pub fn markup_recognition(
        &self,
        origin_image: &DynamicImage,
        markups: &Vec<MarkupBox>,
    ) -> anyhow::Result<DynamicImage> {
        let mut result = origin_image.clone();

        // darken the area not in the markups
        imageproc::map::map_pixels_mut(&mut result, |x, y, it| {
            if is_pixel_in_boxes(x, y, markups) {
                it
            } else {
                let rgb = Rgb::from((it[0], it[1], it[2]));
                let mut hsl: Hsl = rgb.as_ref().into();
                hsl.set_lightness(hsl.lightness() * 0.2);
                hsl.set_saturation(0.0);
                let rgb: Rgb = hsl.into();
                image::Rgba([rgb.red() as u8, rgb.green() as u8, rgb.blue() as u8, it[3]])
            }
        });

        for markup in markups {
            self.draw_box(&mut result, markup, image::Rgba([255, 255, 0, 128]), 4)?;
        }
        Ok(result)
    }

    fn draw_box(
        &self,
        origin_image: &mut DynamicImage,
        markup: &MarkupBox,
        color: image::Rgba<u8>,
        border: i32,
    ) -> anyhow::Result<()> {
        // top
        let rect_top = Rect::at(
            markup.left as i32 - border as i32,
            markup.top as i32 - border as i32,
        )
        .of_size(markup.width + border as u32 * 2, border as u32);
        imageproc::drawing::draw_filled_rect_mut(origin_image, rect_top, color);

        // bottom
        let rect_bottom = Rect::at(
            markup.left as i32 - border as i32,
            markup.top as i32 + markup.height as i32,
        )
        .of_size(markup.width + border as u32 * 2, border as u32);
        imageproc::drawing::draw_filled_rect_mut(origin_image, rect_bottom, color);

        // left
        let rect_left = Rect::at(
            markup.left as i32 - border as i32,
            markup.top as i32 - border as i32,
        )
        .of_size(border as u32, markup.height + border as u32 * 2);
        imageproc::drawing::draw_filled_rect_mut(origin_image, rect_left, color);

        // right
        let rect_right = Rect::at(
            markup.left as i32 + markup.width as i32,
            markup.top as i32 - border as i32,
        )
        .of_size(border as u32, markup.height + border as u32 * 2);
        imageproc::drawing::draw_filled_rect_mut(origin_image, rect_right, color);

        Ok(())
    }
}

fn is_pixel_in_box(x: u32, y: u32, box_: &MarkupBox) -> bool {
    x >= box_.left && x < box_.left + box_.width && y >= box_.top && y < box_.top + box_.height
}

fn is_pixel_in_boxes(x: u32, y: u32, boxes: &Vec<MarkupBox>) -> bool {
    for box_ in boxes {
        if is_pixel_in_box(x, y, box_) {
            return true;
        }
    }
    false
}
