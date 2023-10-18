// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use image::{imageops::FilterType, DynamicImage};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::surface::Surface;
use sdl2::Sdl;

// This module is based on SDL2 (Simple DirectMedia Layer). It complements the sdl2 crate by providing
// methods that make it easier to render images.

/// Create a canvas with an enclosing window.
///
/// # Arguments
/// * `sdl_context` - The SDL context.
/// * `window_title` - The window's title.
/// * `window_width` - The window's width.
/// * `window_height` - The window's height.
pub fn create_canvas(
    sdl_context: &mut Sdl,
    window_title: &str,
    window_width: u32,
    window_height: u32,
) -> Result<WindowCanvas, String> {
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(window_title, window_width, window_height)
        .position_centered()
        .allow_highdpi()
        .build()
        .map_err(|err| format!("{}", err))?;

    let mut canvas = window.into_canvas().build().map_err(|err| format!("{}", err))?;

    // Set the background color to black.
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    Ok(canvas)
}

/// Resize an image to fit inside a canvas.
///
/// # Arguments
/// * `image` - The image that needs to be resized.
/// * `canvas` - The canvas that it needs to fit in.
pub fn resize_image_to_fit_in_canvas(
    image: DynamicImage,
    canvas: &WindowCanvas,
) -> Result<DynamicImage, String> {
    let (window_width, window_height): (u32, u32) = canvas.output_size()?;

    let width_scale = window_width as f32 / image.width() as f32;
    let height_scale = window_height as f32 / image.height() as f32;
    let scale: f32 = height_scale.min(width_scale);

    let resized_image_width = (scale * image.width() as f32) as u32;
    let resized_image_height = (scale * image.height() as f32) as u32;

    Ok(image.resize(resized_image_width, resized_image_height, FilterType::Triangle))
}

/// Render an image to a canvas.
///
/// # Arguments
/// * `image` - The image that we want to render.
/// * `canvas` - The canvas that will render the image.
pub fn render_image_to_canvas(
    image: &DynamicImage,
    canvas: &mut WindowCanvas,
) -> Result<(), String> {
    // Prepare the image for copying it to a surface.
    let rgb_image = image.to_rgb8();
    let mut image_buffer = rgb_image.into_raw();

    let image_width = image.width();
    let image_height = image.height();
    // The pitch is the width of the texture times the size of a single pixel in bytes.
    // Since we are using 24 bit pixels (RGB24), we need to mutiple the width by 3.
    let image_pitch: u32 = image_width * 3;

    let surface = Surface::from_data(
        &mut image_buffer,
        image_width,
        image_height,
        image_pitch,
        PixelFormatEnum::RGB24,
    )
    .map_err(|err| format!("{}", err.to_string()))?;

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.create_texture_from_surface(surface).unwrap();

    // Render the image.
    canvas.clear();
    canvas.copy(&texture, None, Rect::new(0, 0, image_width, image_height))?;
    canvas.present();

    Ok(())
}
