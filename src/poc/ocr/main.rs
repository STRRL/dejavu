use image::io::Reader as ImageReader;
use rusty_tesseract::{Args, Image};

fn main() {
    // or instantiate Image from a DynamicImage
    let dynamic_image = ImageReader::open("./static/screenshot.png")
        .unwrap()
        .decode()
        .unwrap();
    let img = Image::from_dynamic_image(&dynamic_image).unwrap();
    let default_args = Args::default();
    let output = rusty_tesseract::image_to_data(&img, &default_args).unwrap();
    println!("The String output is: {:?}", output);
}
