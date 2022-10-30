use resvg::render;
use worker::*;
use std::io::BufRead;
use std::sync::Arc;
use std::{error::Error, ops::Deref, time::Instant};
use std::{fs, io};
use tiny_skia::{Pixmap, Transform};
use usvg::{ImageHrefResolver, ImageKind, Options, Tree};

const WIDTH: usize =  1200;
const HEIGHT: usize = 630;

struct Tracer {
    start: Instant,
    latest: Instant,
}

impl Tracer {
    pub fn new() -> Self {
        let start = Instant::now();
        Self {
            latest: start,
            start,
        }
    }

    pub fn log(&mut self, event: &str) {
        if cfg!(feature = "tracing") {
            eprintln!(
                "Event: {:<15} ({:>9.3?} since last, {:>9.3?} since start)",
                event,
                self.latest.elapsed(),
                self.start.elapsed()
            );
            self.latest = Instant::now();
        }
    }
}

pub fn og_image(ctx: RouteContext<()>) -> Result<(), Box<dyn Error>> {
    // Read in the svg template we have
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(include_str!("../assets/demo-text-with-image.svg"))
        .unwrap();

    let stdin = io::stdin();
    let text = stdin.lock().lines().next().unwrap().unwrap();

    let mut tracer = Tracer::new();

    // Create a new pixmap buffer to render to
    let mut pixmap = Pixmap::new(WIDTH, HEIGHT).ok_or("Pixmap allocation error")?;

    // Use default settings
    let mut options = Options {
        image_href_resolver: ImageHrefResolver {
            resolve_string: Box::new(move |path: &str, _| {
                let response = reqwest::blocking::get(path).ok()?;
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|hv| hv.to_str().ok())?
                    .to_owned();
                let image_buffer = response.bytes().ok()?.into_iter().collect::<Vec<u8>>();
                match content_type.as_str() {
                    "image/png" => Some(ImageKind::PNG(Arc::new(image_buffer))),
                    "image/jpg" => Some(ImageKind::JPEG(Arc::new(image_buffer))),
                    "image/gif" => Some(ImageKind::GIF(Arc::new(image_buffer))),
                    "image/svg+xml" => Tree::from_data(&image_buffer, &Options::default().to_ref())
                        .ok()
                        .map(ImageKind::SVG),
                    _ => None,
                }
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    options
        .fontdb
        .load_font_data(include_bytes!("../assets/Inter.ttf").to_vec());

    let globals = liquid::object!({ "text": text });

    let svg = template.render(&globals).unwrap();

    tracer.log("rendering");

    // Build our string into a svg tree
    let tree = Tree::from_str(&svg, &options.to_ref())?;

    render(
        &tree,
        usvg::FitTo::Original,
        Transform::default(),
        pixmap.as_mut(),
    );

    tracer.log("rendering");

    // Encode our pixmap buffer into a webp image
    let encoded_buffer =
        webp::Encoder::new(pixmap.data(), webp::PixelLayout::Rgba, WIDTH, HEIGHT).encode_lossless();
    let result = encoded_buffer.deref();

 let mut new_image =
                        Vec::with_capacity(WIDTH * HEIGHT);

                    image
                        .write_to(&mut Cursor::new(&mut new_image), image_transform_format)
                        .expect("Error writing image");

                    let mut headers = worker::Headers::new();
                    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
                    let _ = headers.set("Content-Type", &image_transform_format_header);
                    let _ = headers.set("Cache-Control", "max-age=2629746");


    Ok(result)
}
