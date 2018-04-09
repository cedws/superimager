#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate image;
extern crate reqwest;
extern crate rocket;

use std::io::Cursor;

use rocket::http::{RawStr, Status};
use rocket::config::{Config, Environment};
use rocket::response::Response;

use image::{imageops, FilterType, ImageBuffer, RgbImage};

// Resizes an input image to fit a 512x512 frame whilst preserving the original image ratio.
#[get("/<img>")]
fn convert(img: &RawStr) -> Result<Response<'static>, Status> {
    let mut resp = reqwest::get(&img.url_decode().unwrap())
        .expect("Failed to load image from URL. The URL may be invalid.");

    println!("Converting {}.", img);

    let mut buf = vec![];
    resp.copy_to(&mut buf)
        .expect("Failed to load response data.");

    let original = image::load_from_memory(buf.as_slice())
        .expect("Failed to load response data as image.")
        .to_rgb();

    // Assert that the image has valid dimensions.
    assert!(original.width() > 0);
    assert!(original.height() > 0);

    let width = original.width() as f32;
    let height = original.height() as f32;

    let scalar = (512.0 / width).min(512.0 / height);

    // Assert that the image scalar is greater than zero.
    assert!(scalar > 0.0);

    let (xscaled, yscaled) = (scalar * width, scalar * height);
    let (xoverlay, yoverlay) = ((512.0 - xscaled) / 2.0, (512.0 - yscaled) / 2.0);

    let resized = imageops::resize(
        &original,
        xscaled as u32,
        yscaled as u32,
        FilterType::CatmullRom,
    );

    let mut rescaled = ImageBuffer::new(512, 512) as RgbImage;
    imageops::overlay(&mut rescaled, &resized, xoverlay as u32, yoverlay as u32);

    // Assert that output image has a width and height of 512.
    assert_eq!(rescaled.width(), 512);
    assert_eq!(rescaled.height(), 512);

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
