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
use uuencode::uudecode;

async fn streaming(
    client: &mut DigitalTwinProviderClient<Channel>,
    entity_id: &str,
    num: usize,
    window: &mut WindowProxy,
) -> Result<(), Box<dyn Error>> {
    let stream =
        client.stream(StreamRequest { entity_id: entity_id.to_string() }).await?.into_inner();

    // The stream is infinite, so take just num elements and then disconnect.
    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        let content = item.unwrap().content;
        if let Some((contents, filename)) = uudecode(&content) {
            let image_reader = ImageReader::new(Cursor::new(contents)).with_guessed_format()?;
            let image = image_reader.decode()?;
            let image_data = image.as_bytes().to_vec();
            let image_view =
                ImageView::new(ImageInfo::rgb8(image.width(), image.height()), &image_data);
            window.set_image(filename, image_view)?;
        }
    }

    // The stream is droped here and the disconnect info is sent to the server.

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
    streaming(&mut client, sdv::camera::feed::ID, settings.number_of_images.into(), &mut window)
        .await?;

    info!("The Consumer has completed.");

    Ok(())
}
