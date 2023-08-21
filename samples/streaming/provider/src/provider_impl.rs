// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// use digital_twin_model::sdv_v1 as sdv;
use core::iter::Iterator;
use log::warn;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::digital_twin_provider_server::DigitalTwinProvider;
use samples_protobuf_data_access::sample_grpc::v1::digital_twin_provider::{
    GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse,
    SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
    StreamRequest, StreamResponse,    
};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::pin::Pin;
use std::vec::Vec;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tokio::time::{sleep, Duration};
use tonic::{Request, Response, Status};
use uuencode::uuencode;

#[derive(Debug, Default)]
struct ImageFileIterator {
    image_directory: String,
    image_filenames: Vec<String>,
    current_index: usize,
}

impl ImageFileIterator {
    pub fn new(image_directory: &str, image_filenames: Vec<String>) -> Self {
        Self {
            image_directory: image_directory.to_string(),
            image_filenames,
            current_index: 0
        }
    }

    fn read_image_file(&self, filename: &str) -> Result<String, std::io::Error> {
        let filepath = Path::new(&self.image_directory).join(filename);
        println!("About to read_image from '{}'", filepath.display());
        let mut file = File::open(filepath)?;
        println!("About to setup the buf_reader");
        println!("About to read_string");
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).expect("Unable to read");        
        println!("About to setup the uuencode");
        let encoded = uuencode(filename, &file_content);
        println!("Encode: {encoded}");
        Ok(encoded)
    }    
}

impl Iterator for ImageFileIterator {
    type Item = StreamResponse;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.image_filenames.len() ;
        if len == 0 {
            return None
        }

        let current = &self.image_filenames[self.current_index];

        self.current_index = (self.current_index + 1) % len;

        let uuencoded_content = self.read_image_file(current).ok()?;
        let result = StreamResponse { message: uuencoded_content };

        Some(result)
    }
}

#[derive(Debug, Default)]
pub struct ProviderImpl {
    image_directory: String
}

impl ProviderImpl {
    pub fn new(image_directory: &str) -> Self {
        Self {
            image_directory: image_directory.to_string(),
        }
    }

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
    /// * `request` - OpenStream request.
    async fn stream(
        &self,
        request: Request<StreamRequest>,    
    ) -> Result<Response<Self::StreamStream>, Status> {
        let image_filenames = self.get_filenames_from_image_directory().map_err(|_| Status::internal("get filenames failed"))?;

        let image_file_iterator = ImageFileIterator::new(&self.image_directory, image_filenames);

        let mut stream = Box::pin(tokio_stream::iter(image_file_iterator).throttle(Duration::from_millis(200)));

        // spawn and channel are required if you want handle "disconnect" functionality
        // the `out_stream` will not be polled after client disconnect
        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
                sleep(Duration::from_secs(1)).await;
            }
            println!("\tclient disconnected");
        });        
/*
        // creating infinite stream with requested message
        let repeat = std::iter::repeat(StreamResponse {
            message: request.into_inner().message,
        });
        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));

        // spawn and channel are required if you want handle "disconnect" functionality
        // the `out_stream` will not be polled after client disconnect
        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
            }
            println!("\tclient disconnected");
        });
*/

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::StreamStream
        ))
    }     
}

#[cfg(test)]
mod provider_impl_tests {
    use super::*;
    use uuid::Uuid;

/*
    #[tokio::test]
    async fn subscribe_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let vehicle = Arc::new(Mutex::new(Vehicle::new()));
        let provider_impl = ProviderImpl { subscription_map: subscription_map.clone(), vehicle };

        let first_id = String::from("one-id");
        let second_id = String::from("two-id");
        let first_uri = String::from("http://first.com:9000"); // Devskim: ignore DS137138
        let second_uri = String::from("http://second.com:9000"); // Devskim: ignore DS137138
        let third_uri = String::from("http://third.com:9000"); // Devskim: ignore DS137138

        let first_request = tonic::Request::new(SubscribeRequest {
            entity_id: first_id.clone(),
            consumer_uri: first_uri.clone(),
        });
        let first_result = provider_impl.subscribe(first_request).await;
        assert!(first_result.is_ok());

        let second_request = tonic::Request::new(SubscribeRequest {
            entity_id: first_id.clone(),
            consumer_uri: second_uri.clone(),
        });
        let second_result = provider_impl.subscribe(second_request).await;
        assert!(second_result.is_ok());

        let third_request = tonic::Request::new(SubscribeRequest {
            entity_id: second_id.clone(),
            consumer_uri: third_uri.clone(),
        });
        let third_result = provider_impl.subscribe(third_request).await;
        assert!(third_result.is_ok());

        // This block controls the lifetime of the lock.
        {
            let lock: MutexGuard<HashMap<String, HashSet<String>>> = subscription_map.lock();

            let first_get_result = lock.get(&first_id);
            assert!(first_get_result.is_some());
            let first_value = first_get_result.unwrap();
            assert_eq!(first_value.len(), 2);
            assert!(first_value.contains(&first_uri));
            assert!(first_value.contains(&second_uri));

            let second_get_result = lock.get(&second_id);
            assert!(second_get_result.is_some());
            let second_value = second_get_result.unwrap();
            assert_eq!(second_value.len(), 1);
            assert!(second_value.contains(&third_uri));
        }
    }

    #[tokio::test]
    async fn invoke_test() {
        let subscription_map = Arc::new(Mutex::new(HashMap::new()));
        let vehicle = Arc::new(Mutex::new(Vehicle::new()));
        let provider_impl = ProviderImpl { subscription_map, vehicle };

        let entity_id = String::from("one-id");
        let consumer_uri = String::from("bogus uri");

        let response_id = Uuid::new_v4().to_string();
        let payload = String::from("some-payload");

        let request =
            tonic::Request::new(InvokeRequest { entity_id, consumer_uri, response_id, payload });
        let result = provider_impl.invoke(request).await;
        assert!(result.is_ok());

        // Note: this test does not check that the response has successfully been sent.
    }
*/
}
