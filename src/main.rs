#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate flate2;
extern crate image;
extern crate reqwest;

use rocket::config::{Config, Environment};
use rocket::http::hyper::header::{Connection, ContentEncoding, Encoding};
use rocket::http::{RawStr, Status};
use rocket::response::Response;

use flate2::write::GzEncoder;
use flate2::Compression;
use image::{imageops, FilterType, ImageBuffer};

use std::io::{Cursor, Write};

const MAX_OUTPUT_SIZE: u32 = 512;

/// Resizes an input image to fill a maximum frame size whilst preserving the original image ratio.
#[get("/<img>")]
fn convert(img: &RawStr) -> Result<Response<'static>, Status> {
    let url = img.url_decode().map_err(|_| Status::BadRequest)?;
    let mut resp = reqwest::get(&url).map_err(|_| Status::BadRequest)?;

    let mut buf = Vec::new();
    resp.copy_to(&mut buf)
        .expect("Failed to load response data");

    let original = image::load_from_memory(&buf)
        .map_err(|_| Status::BadRequest)?
        .to_rgb();

    let (width, height) = (original.width(), original.height());

    // Find the optimum output size.
    let size = (width.max(height)).min(MAX_OUTPUT_SIZE) as f32;
    let scalar = (size / width as f32).min(size / height as f32);

    let (xscaled, yscaled) = (scalar * width as f32, scalar * height as f32);
    let (xoverlay, yoverlay) = ((size - xscaled) / 2.0, (size - yscaled) / 2.0);

    let resized = imageops::resize(
        &original,
        xscaled as u32,
        yscaled as u32,
        FilterType::Lanczos3,
    );

    let mut rescaled = ImageBuffer::new(size as u32, size as u32);
    imageops::overlay(&mut rescaled, &resized, xoverlay as u32, yoverlay as u32);

    let mut enc = GzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(&rescaled.into_raw())
        .map_err(|_| Status::InternalServerError)?;

    let data = enc.finish().map_err(|_| Status::InternalServerError)?;

    Response::build()
        .header(Connection::keep_alive())
        .header(ContentEncoding(vec![Encoding::Gzip]))
        .sized_body(Cursor::new(data))
        .ok()
}

fn main() {
    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8080)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .mount("/convert", routes![convert])
        .launch();
}
