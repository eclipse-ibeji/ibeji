// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod streaming_consumer_config;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use image::{DynamicImage, io::Reader as ImageReader, imageops::{FilterType, resize}};
use image::buffer::ConvertBuffer;
use log::{info, LevelFilter, warn};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::StreamRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use std::error::Error;
use std::io::Cursor;
use tokio_stream::StreamExt;
use tonic::transport::Channel;


// use image::GrayImage;


/// Stream images from the server and display them in the provided window.
///
/// # Arguments
/// * `client` - The client connection to the service that will transfer the stream.
/// * `number_of_images` - The number of images that we will stream.
async fn stream_images(
    client: &mut DigitalTwinProviderClient<Channel>,
    entity_id: &str,
    number_of_images: usize,
) -> Result<(), Box<dyn Error>> {
/*
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window_width = 1920; // 640; // 800;
    let window_height = 1080; // 480; // 600;

    let mut window = video_subsystem
        .window("Streamed Image", window_width, window_height)
        .resizable()
        .allow_highdpi()
        .build()
        .unwrap();

    window.set_minimum_size(150, 150).unwrap();

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    let texture_creator = canvas.texture_creator();
*/
    let stream =
        client.stream(StreamRequest { entity_id: entity_id.to_string() }).await?.into_inner();

    // The stream is infinite, so we will just take number_of_images elements and then disconnect.
    let mut stream = stream.take(number_of_images);
    while let Some(item) = stream.next().await {
        let opt_media = item.unwrap().media;
        if opt_media.is_none() {
            warn!("No media value present, so ignoring this item.");
            continue;
        }
        let media_content = opt_media.unwrap().media_content;
        let image_reader = ImageReader::new(Cursor::new(media_content)).with_guessed_format()?;
        let mut image: DynamicImage = image_reader.decode()?;
        // let mut image_rgb = image.to_rgb8();

        // Display the image
        // println!("image width = {}   image height = {}", image_rgb.width(), image_rgb.height());
        // imageproc::window::display_image("My Image", &image_rgb, image_rgb.width(), image_rgb.height());

        // https://github.com/image-rs/imageproc/blob/master/src/window.rs

        // image_rgb = image_rgb.convert();

        let window_width: u32 = 800; // image.width();
        let window_height: u32 = 600; // image.height();

        let width_scale = window_width as f32 / image.width() as f32;
        let height_scale = window_height as f32 / image.height() as f32;
        let scale: f32 = height_scale.min(width_scale);
        let width = (scale * image.width() as f32) as u32;
        let height = (scale * image.height() as f32) as u32;

        // image = image.grayscale();
        image = image.resize(width, height, FilterType::Triangle);

        let mut resized_image = image.to_rgb8();
        // esized_image = resized_image.convert();

        // let resized_image = resize(&image_rgb, width, height, FilterType::Triangle);
        let (image_width, image_height) = resized_image.dimensions();
        println!("image_width = {image_width}   image_height = {image_height}");
        let mut buffer = resized_image.into_raw();
/** */

let sdl_context = sdl2::init().unwrap();

let video_subsystem = sdl_context.video().unwrap();


// let window_width = image_width; // 640; // 800;
// let window_height = image_height; // 480; // 600;

// let window_width = 1920; // 640; // 800;
// let window_height = 1080; // 480; // 600;

let mut window = video_subsystem
    .window("Streamed Image", window_width, window_height)
    .position_centered()
    .allow_highdpi()
    .build()
    .unwrap();

// window.set_minimum_size(150, 150).unwrap();

let mut canvas = window
    .into_canvas()
    // .software()
    .build()
    .unwrap();

canvas.set_draw_color(Color::RGB(255, 255, 255));

let texture_creator = canvas.texture_creator();

/** */
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

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    // The stream is dropped when we exit the function and the disconnect info is sent to the server.

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    let settings = crate::streaming_consumer_config::load_settings();

    let invehicle_digital_twin_uri = retrieve_invehicle_digital_twin_uri(
        settings.invehicle_digital_twin_uri,
        settings.chariott_uri,
    )
    .await
    .unwrap();

    // Retrieve the provider URI.
    let provider_endpoint_info = discover_digital_twin_provider_using_ibeji(
        &invehicle_digital_twin_uri,
        sdv::camera::feed::ID,
        digital_twin_protocol::GRPC,
        &[digital_twin_operation::STREAM.to_string()],
    )
    .await
    .unwrap();
    let provider_uri = provider_endpoint_info.uri;
    info!("The provider URI for the Cabin Camera Feed property's provider is {provider_uri}");

    let mut client = DigitalTwinProviderClient::connect(provider_uri.clone()).await.unwrap();
    stream_images(
        &mut client,
        sdv::camera::feed::ID,
        settings.number_of_images.into(),
    )
    .await?;

    info!("The Consumer has completed.");

    Ok(())
}
