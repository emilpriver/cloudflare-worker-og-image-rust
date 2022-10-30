use resvg::render;
use std::io;
use std::io::BufRead;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Instant;
use tiny_skia::{Pixmap, Transform};
use usvg::{ImageHrefResolver, ImageKind, Options, Tree};
use worker::*;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 630;

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

pub async fn og_image(ctx: RouteContext<()>) -> Result<Response> {
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
    let client = reqwest::Client::new();
    
    let mut options = Options {
        image_href_resolver: ImageHrefResolver {
            resolve_string: Box::new(move |path: &str, _| async move {
                let response = client.get(path).send().await.unwrap();
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

    let globals = liquid::object!({ "text": text });

    let svg = template.render(&globals).unwrap();

    // Build our string into a svg tree
    let tree = match Tree::from_str(&svg, &options.to_ref()) {
        Ok(t) => t,
        Err(e) => return Ok(Response::error("Error creating tree", 400).unwrap()),
    };

    render(
        &tree,
        usvg::FitTo::Original,
        Transform::default(),
        pixmap.as_mut(),
    );

    tracer.log("rendering");

    let mut new_image = Vec::with_capacity(WIDTH as usize * HEIGHT as usize);

    let image = match image::load_from_memory(&pixmap.data()) {
        Ok(value) => value,
        _ => return Ok(Response::error("Error loading image from memory", 400).unwrap()),
    };

    image
        .write_to(&mut Cursor::new(&mut new_image), image::ImageFormat::Png)
        .expect("Error writing image");

    let mut headers = worker::Headers::new();
    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
    let _ = headers.set("Content-Type", "image/png");
    let _ = headers.set("Cache-Control", "max-age=2629746");

    let body = ResponseBody::Body(new_image);

    // Implicit return (learn to love it)
    Ok(Response::from_body(body).unwrap().with_headers(headers))
}
