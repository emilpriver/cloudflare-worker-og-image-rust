use resvg::render;
use std::io::Cursor;
use tiny_skia::{Pixmap, Transform};
use usvg::{Options, Tree};
use worker::*;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;

pub async fn og_image(req: Request) -> Result<Vec<u8>> {
    // Read in the svg template we have
    let template = match liquid::ParserBuilder::with_stdlib().build() {
        Ok(file) => file,
        Err(e) => return Err(Error::BindingError(e.to_string())),
    };

    let parsed_file = match template.parse(include_str!("../assets/template.svg")) {
        Ok(file) => file,
        Err(e) => return Err(Error::BindingError(e.to_string())),
    };

    // Create a new pixmap buffer to render to
    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).ok_or("Pixmap allocation error")?;

    // Use default settings
    let _client = reqwest::Client::new();

    let mut options = Options {
        ..Default::default()
    };

    options
        .fontdb
        .load_font_data(include_bytes!("../assets/OpenSans-VariableFont_wdth,wght.ttf").to_vec());

    let parsed_url = match req.url() {
        Ok(url) => url,
        _ => return Err(Error::BindingError("Can't parse url".to_string())),
    };

    let mut text_query_value = None;
    for (k, v) in parsed_url.query_pairs() {
        if k == "text" {
            text_query_value = Some(v.to_string());
        }
    }

    // if we miss "text" query parameter, then return error
    if text_query_value == None {
        return Err(Error::BindingError(
            "Missing 'text' query parameter".to_string(),
        ));
    }

    let globals = liquid::object!({ "text": text_query_value });

    let html = match parsed_file.render(&globals) {
        Ok(parse_file_html) => parse_file_html,
        Err(e) => return Err(Error::BindingError(e.to_string())),
    };

    // Build our string into a svg tree
    let tree = match Tree::from_str(&html, &options.to_ref()) {
        Ok(t) => t,
        Err(e) => return Err(Error::BindingError(e.to_string())),
    };

    render(
        &tree,
        usvg::FitTo::Original,
        Transform::default(),
        pixmap.as_mut(),
    );

    let mut new_image = match pixmap.encode_png() {
        Ok(img) => img,
        Err(e) => return Err(Error::BindingError(e.to_string())),
    };

    let image = match image::load_from_memory(&new_image) {
        Ok(value) => value,
        Err(e) => return Err(Error::BindingError(e.to_string())),
    };

    image
        .write_to(&mut Cursor::new(&mut new_image), image::ImageFormat::Jpeg)
        .expect("Error writing image");

    Ok(new_image)
}
