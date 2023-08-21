// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use digital_twin_model::sdv_v1 as sdv;
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use samples_common::constants::{digital_twin_operation, digital_twin_protocol};
use samples_common::consumer_config;
use samples_common::utils::{
    discover_digital_twin_provider_using_ibeji, retrieve_invehicle_digital_twin_uri,
};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::StreamRequest;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_client::DigitalTwinProviderClient;
use show_image::{ImageView, ImageInfo, create_window, WindowProxy};
use std::error::Error;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use uuencode::uudecode;

use image::io::Reader as ImageReader;

use std::io::Cursor;

async fn streaming(
    client: &mut DigitalTwinProviderClient<Channel>,
    num: usize,
    window: &mut WindowProxy,
) {
    let stream = client.stream(StreamRequest { message: "foo".into() }).await.unwrap().into_inner();

    // stream is infinite - take just 5 elements and then disconnect
    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        let message = item.unwrap().message;
        println!("\treceived: {}", message);
        if let Some((contents, filename)) = uudecode(&message) {
            if let loaded_image_result =
                ImageReader::new(Cursor::new(contents)).with_guessed_format()
            {
                if let decoded_image_result = loaded_image_result.unwrap().decode() {
                    let decoded_image = decoded_image_result.unwrap();
                    let pixel_data = decoded_image.to_bytes();
                    let width = decoded_image.width();
                    let height = decoded_image.height();
                    // let (width, height) = pixel_data.dimensions();
                    let image = ImageView::new(ImageInfo::rgb8(width, height), &pixel_data);
                    window.set_image(filename, image);
                }
            }
        }
    }
    // stream is droped here and the disconnect info is send to server
}

#[tokio::main]
#[show_image::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging.
    Builder::new().filter(None, LevelFilter::Info).target(Target::Stdout).init();

    info!("The Consumer has started.");

    let settings = consumer_config::load_settings();

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
    info!("The provider URI for the AmbientAirTemperature property's provider is {provider_uri}");

    // show_image - https://docs.rs/show-image/latest/show_image/

    // Create a window with default options and display the image.
    let mut window = create_window("image", Default::default())?;

    let mut client = DigitalTwinProviderClient::connect(provider_uri.clone()).await.unwrap();
    streaming(&mut client, 20, &mut window).await;

    info!("The Consumer has completed.");

    Ok(())
}
