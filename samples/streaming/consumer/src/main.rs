// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod streaming_consumer_config;

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};

use image::io::Reader as ImageReader;
use log::{info, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::StreamRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use show_image::{ImageView, ImageInfo, create_window, WindowProxy};
use std::error::Error;
use std::io::Cursor;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

/// Stream images from the server and display them in the provided window.
///
/// # Arguments
/// * `client` - The client connection to the service that will transfer the stream.
/// * `number_of_images` - The number of images that we will stream.
/// * `window` - The window where the streamed images will be shown.
async fn stream_images(
    client: &mut DigitalTwinProviderClient<Channel>,
    entity_id: &str,
    number_of_images: usize,
    window: &mut WindowProxy,
) -> Result<(), Box<dyn Error>> {
    let stream =
        client.stream(StreamRequest { entity_id: entity_id.to_string() }).await?.into_inner();

    // The stream is infinite, so we will just take number_of_images elements and then disconnect.
    let mut stream = stream.take(number_of_images);
    while let Some(item) = stream.next().await {
        let media_content = item.unwrap().media.unwrap().media_content;
        let image_reader = ImageReader::new(Cursor::new(media_content)).with_guessed_format()?;
        let image = image_reader.decode()?;
        let image_data = image.as_bytes().to_vec();
        let image_view =
            ImageView::new(ImageInfo::rgb8(image.width(), image.height()), &image_data);
        window.set_image("some file", image_view)?;
    }

    // The stream is dropped when we exit the function and the disconnect info is sent to the server.

    Ok(())
}

#[tokio::main]
#[show_image::main]
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

    // Create a window with default options and display the image.
    let mut window = create_window("image", Default::default())?;

    let mut client = DigitalTwinProviderClient::connect(provider_uri.clone()).await.unwrap();
    stream_images(
        &mut client,
        sdv::camera::feed::ID,
        settings.number_of_images.into(),
        &mut window,
    )
    .await?;

    info!("The Consumer has completed.");

    Ok(())
}
