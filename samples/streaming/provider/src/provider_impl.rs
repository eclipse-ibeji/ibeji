// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use core::iter::Iterator;
use log::{debug, info, warn};
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProvider;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{
    GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse,
    SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
    StreamRequest, StreamResponse};
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;
use std::pin::Pin;
use std::vec::Vec;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tokio::time::Duration;
use tonic::{Request, Response, Status};
use uuencode::uuencode;

#[derive(Debug, Default)]
/// Continuously iterates through a list of files.
/// After it reaches the end of the list, it cycles back to the start of the list.
struct ImageFileIterator {
    /// The directory that contains the images.
    image_directory: String,
    /// The filenames of the images that we want to stream.
    image_filenames: Vec<String>,
    /// The index for the current image filename.
    current_image_file_index: usize,
}

impl ImageFileIterator {
    /// Create a new ImageFileIterator.
    ///
    /// # Arguments
    /// * `image_directory` - The directory that contains the images.
    /// * `image_filenames` - The filenames of the images that we want to stream.
    pub fn new(image_directory: &str, image_filenames: Vec<String>) -> Self {
        Self {
            image_directory: image_directory.to_string(),
            image_filenames,
            current_image_file_index: 0,
        }
    }

    /// Reads an image file and returns its uuencoded form.
    ///
    /// # Arguments
    /// * `filenme` -The image's filename.
    fn read_image_file(&self, filename: &str) -> Result<String, Error> {
        let filepath = Path::new(&self.image_directory).join(filename);
        debug!("Read_image from '{}'", filepath.display());
        let mut file = File::open(filepath)?;
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).expect("Unable to read");
        let encoded = uuencode(filename, &file_content);
        Ok(encoded)
    }
}

impl Iterator for ImageFileIterator {
    type Item = StreamResponse;

    /// Get the next item from the iterator.
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.image_filenames.len();
        if len == 0 {
            return None;
        }

        let current_image_filename = &self.image_filenames[self.current_image_file_index];

        // Note: We will go back to the start of the list once we past the end.
        self.current_image_file_index = (self.current_image_file_index + 1) % len;

        Some(StreamResponse { content: self.read_image_file(current_image_filename).ok()? })
    }
}

#[derive(Debug, Default)]
pub struct ProviderImpl {
    image_directory: String,
}

impl ProviderImpl {
    /// Create a new ProviderImpl.
    pub fn new(image_directory: &str) -> Self {
        Self { image_directory: image_directory.to_string() }
    }

    /// Get the filenames from the image directory.
    fn get_filenames_from_image_directory(&self) -> Result<Vec<String>, std::io::Error> {
        let mut images = vec![];
        for entry in std::fs::read_dir(&self.image_directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            images.push(entry.file_name().into_string().unwrap());
        }

        if images.is_empty() {
            panic!("No images");
        }

        Ok(images)
    }
}

#[tonic::async_trait]
impl DigitalTwinProvider for ProviderImpl {
    // Note: The name "StreamStream" is not ideal, but it is what gRPC is forcing us to use.
    //       gRPC generates the name by concatenating the rpc method name with "Stream".
    type StreamStream = Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send>>;

    /// Subscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Subscribe request.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeResponse>, Status> {
        warn!("Got a subscribe request: {request:?}");

        Err(Status::unimplemented("subscribe has not been implemented"))
    }

    /// Unsubscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Unsubscribe request.
    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeResponse>, Status> {
        warn!("Got an unsubscribe request: {request:?}");

        Err(Status::unimplemented("unsubscribe has not been implemented"))
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        warn!("Got a get request: {request:?}");

        Err(Status::unimplemented("get has not been implemented"))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        warn!("Got a set request: {request:?}");

        Err(Status::unimplemented("set has not been implemented"))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `request` - Invoke request.
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        warn!("Got an invoke request: {request:?}");

        Err(Status::unimplemented("get has not been implemented"))
    }

    /// Stream implementation.
    ///
    /// # Arguments
    /// * `request` - Stream request.
    async fn stream(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamStream>, Status> {
        let image_filenames = self
            .get_filenames_from_image_directory()
            .map_err(|err| Status::internal(format!("Get filenames failed due to: {err}")))?;

        let image_file_iterator = ImageFileIterator::new(&self.image_directory, image_filenames);

        let mut stream =
            Box::pin(tokio_stream::iter(image_file_iterator).throttle(Duration::from_secs(5)));

        // The spawn and channel are required if you want to handle disconnect functionality.
        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        debug!("The next item in the stream was successfully sent to the client.");
                    }
                    Err(err) => {
                        warn!("Failed to send the next item in the stream to the client due to: {err}");
                        break;
                    }
                }
            }
            info!("Client disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::StreamStream))
    }
}
