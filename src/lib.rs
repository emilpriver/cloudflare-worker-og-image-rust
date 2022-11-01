use worker::*;

mod image;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |request, _ctx| async {
            let image = match image::og_image(request).await {
                Ok(img) => img,
                Err(e) => {
                    console_log!("{}", e);

                    return Response::error("Error generating og image", 405);
                }
            };

            let mut headers = worker::Headers::new();
            let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
            let _ = headers.set("Content-Type", "image/png");
            let _ = headers.set("Cache-Control", "max-age=2629746");

            let body = ResponseBody::Body(image);

            Ok(Response::from_body(body).unwrap().with_headers(headers))
        })
        .run(req, env)
        .await
}
