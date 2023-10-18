// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use image::{DynamicImage, imageops::FilterType};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use sdl2::surface::Surface;

pub fn create_canvas(sdl_context: &mut Sdl, window_title: &str, window_width: u32, window_height: u32) -> WindowCanvas{
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(window_title, window_width, window_height)
        .position_centered()
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .unwrap();

    // Set the background color to black.
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    canvas
}

pub fn resize_image_to_fit_in_canvas(image: DynamicImage, canvas: &WindowCanvas) -> DynamicImage {
    let (window_width, window_height): (u32, u32) = canvas.output_size().unwrap();

    let width_scale = window_width as f32 / image.width() as f32;
    let height_scale = window_height as f32 / image.height() as f32;
    let scale: f32 = height_scale.min(width_scale);

    let resized_image_width = (scale * image.width() as f32) as u32;
    let resized_image_height = (scale * image.height() as f32) as u32;

    image.resize(resized_image_width, resized_image_height, FilterType::Triangle)
}

pub fn render_image_to_canvas(image: & DynamicImage, canvas: &mut WindowCanvas ) {
    let rgb_image = image.to_rgb8();

    let mut buffer = rgb_image.into_raw();

    let texture_creator = canvas.texture_creator();

    let image_width = image.width();
    let image_height = image.height();

    // The pitch is the width of the texture times the size of a single pixel in bytes.
    // Since we are using 24 bit pixels (RGB24), we need to mutiple the width by 3.
    let image_pitch: u32 = image_width * 3;

    let surface = Surface::from_data(
        &mut buffer,
        image_width,
        image_height,
        image_pitch,
        PixelFormatEnum::RGB24
    ).unwrap();

    let texture = texture_creator.create_texture_from_surface(surface).unwrap();

    canvas.clear();
    canvas.copy(&texture, None, Rect::new(0, 0, image_width, image_height)).unwrap();
    canvas.present();
}