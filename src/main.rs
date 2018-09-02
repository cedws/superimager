#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate image;
extern crate reqwest;
extern crate rocket;

use std::io::Cursor;

use rocket::config::{Config, Environment};
use rocket::http::{RawStr, Status};
use rocket::response::Response;

use image::{imageops, FilterType, ImageBuffer, RgbImage};

const MAX_OUTPUT_SIZE: f32 = 512.0;

// Resizes an input image to fit a maximum frame size whilst preserving the original image ratio.
#[get("/<img>")]
fn convert(img: &RawStr) -> Result<Response<'static>, Status> {
    let url = match img.url_decode() {
        Ok(n) => n,
        Err(_) => return Err(Status::BadRequest),
    };

    let mut resp = match reqwest::get(&url) {
        Ok(n) => n,
        Err(_) => return Err(Status::BadRequest),
    };

    println!("Converting {}.", &url);

    let mut buf = vec![];
    resp.copy_to(&mut buf)
        .expect("Failed to load response data.");

    let original = match image::load_from_memory(buf.as_slice()) {
        Ok(n) => n,
        Err(_) => return Err(Status::BadRequest),
    }.to_rgb();

    // Assert that the image has valid dimensions.
    assert!(original.width() > 0);
    assert!(original.height() > 0);

    let width = original.width() as f32;
    let height = original.height() as f32;

    // Find the optimum output size.
    let size = (width.max(height)).min(MAX_OUTPUT_SIZE);
    let scalar = (size / width).min(size / height);

    // Assert that the image scalar is greater than zero.
    assert!(scalar > 0.0);

    let (xscaled, yscaled) = (scalar * width, scalar * height);
    let (xoverlay, yoverlay) = ((size - xscaled) / 2.0, (size - yscaled) / 2.0);

    let resized = imageops::resize(
        &original,
        xscaled as u32,
        yscaled as u32,
        FilterType::Lanczos3,
    );

    let mut rescaled = ImageBuffer::new(size as u32, size as u32) as RgbImage;
    imageops::overlay(&mut rescaled, &resized, xoverlay as u32, yoverlay as u32);

    // Assert that output image has a width and height of 512.
    assert_eq!(rescaled.width(), size as u32);
    assert_eq!(rescaled.height(), size as u32);

    Response::build()
        .sized_body(Cursor::new(rescaled.into_raw()))
        .ok()
}

fn main() {
    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8080)
        .finalize()
        .unwrap();

    rocket::custom(config, true)
        .mount("/convert", routes![convert])
        .launch();
}
