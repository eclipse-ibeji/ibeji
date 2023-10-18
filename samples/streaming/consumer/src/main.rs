// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod streaming_consumer_config;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use image::{DynamicImage, io::Reader as ImageReader};
use log::{info, LevelFilter, warn};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::view_image;
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::StreamRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use sdl2::render::WindowCanvas;
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
        let image: DynamicImage = image_reader.decode()?;

        // Display the image
        // println!("image width = {}   image height = {}", image_rgb.width(), image_rgb.height());
        // imageproc::window::display_image("My Image", &image_rgb, image_rgb.width(), image_rgb.height());

        let mut sdl_context = sdl2::init().unwrap();

        let mut canvas: WindowCanvas = view_image::create_canvas(&mut sdl_context, "Image", 800, 600);

        let resized_image = view_image::resize_image_to_fit_in_canvas(image, &canvas);

        view_image::render_image_to_canvas(&resized_image, &mut canvas);

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
