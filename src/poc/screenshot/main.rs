use screenshots::Screen;
use std::{fs, time::Instant};
fn main() {
    let start = Instant::now();
    let screens = Screen::all().unwrap();

    for screen in screens {
        println!("capturer {screen:?}");
        let image = screen.capture().unwrap();
        let buffer = image.to_png().unwrap();
        fs::write(format!("target/{}.png", screen.display_info.id), buffer).unwrap();
    }

    println!("time elapsed: {:?}", start.elapsed());
}
