use resvg::render;
use std::{Result};
use std::io::Cursor;
use tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree};
use worker::*;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;

pub async fn og_image(_ctx: RouteContext<()>) -> Result<Vec<u8>>{
    // Read in the svg template we have
    let template = match liquid::ParserBuilder::with_stdlib().build() {
        Ok(file) => file,
        Err(e) => return Err(e.to_string()),
    };

    let parsed_file = match template.parse(include_str!("../assets/template.svg")) {
        Ok(file) => file,
        Err(e) => return Error::Json(e.to_string()),
    };

    // Create a new pixmap buffer to render to
    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).ok_or("Pixmap allocation error")?;

    // Use default settings
    let _client = reqwest::Client::new();

    let options = Options {
        ..Default::default()
    };

    let globals = liquid::object!({ "text": "hello"});

    let html = match parsed_file.render(&globals) {
        Ok(parse_file_html) => parse_file_html,
        Err(e) => return Ok(Response::error(e.to_string(), 400).unwrap()),
    };

    // Build our string into a svg tree
    let tree = match Tree::from_str(&html, &options.to_ref()) {
        Ok(t) => t,
        Err(e) => return Ok(Response::error(e.to_string(), 400).unwrap()),
    };

    render(
        &tree,
        usvg::FitTo::Original,
        Transform::default(),
        pixmap.as_mut(),
    );

    let mut new_image = match pixmap.encode_png() {
        Ok(img) => img,
        _ => return Ok(Response::error("Error loading image from memory", 400)),
    };

    let image = match image::load_from_memory(&new_image) {
        Ok(value) => value,
        _ => return Ok(Response::error("Error loading image from memory", 400)),
    };

    image
        .write_to(&mut Cursor::new(&mut new_image), image::ImageFormat::Png)
        .expect("Error writing image");

    Ok(new_image)
}
