use image::{GenericImage, RgbImage};

use crate::ocr::MarkupBox;

fn safe_put_pixel(
    origin_image: &mut RgbImage,
    x: u32,
    y: u32,
    color: image::Rgb<u8>,
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
        origin_image: &RgbImage,
        markups: &Vec<MarkupBox>,
    ) -> anyhow::Result<RgbImage> {
        let mut result: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = origin_image.clone();
        for markup in markups {
            self.draw_box(&mut result, markup, image::Rgb([255, 0, 0]))?;
        }
        Ok(result)
    }

    fn draw_box(
        &self,
        origin_image: &mut RgbImage,
        markup: &MarkupBox,
        color: image::Rgb<u8>,
    ) -> anyhow::Result<()> {
        // draw the top edge
        for x in markup.left..(markup.left + markup.width) {
            safe_put_pixel(origin_image, x, markup.top, color)?;
            safe_put_pixel(origin_image, x, markup.top + markup.height, color)?;
        }

        // draw the left edge
        for y in markup.top..(markup.top + markup.height) {
            safe_put_pixel(origin_image, markup.left, y, color)?;
            safe_put_pixel(origin_image, markup.left + markup.width, y, color)?;
        }

        Ok(())
    }
}
