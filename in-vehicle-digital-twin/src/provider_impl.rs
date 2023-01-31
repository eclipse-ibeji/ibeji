// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use log::info;
use proto::provider::provider_server::Provider;
use proto::provider::{
    GetRequest, GetResponse, InvokeRequest, InvokeResponse, SetRequest, SetResponse,
    SubscribeRequest, SubscribeResponse, UnsubscribeRequest, UnsubscribeResponse,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct ProviderImpl {}

#[tonic::async_trait]
impl Provider for ProviderImpl {
    /// Subscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Subscribe request.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<SubscribeResponse>, Status> {
        info!("Got a subscribe request: {:?}", request);
        // TODO - provide subscribe functionality
        let response = SubscribeResponse {};

        Ok(Response::new(response))
    }

    /// Unsubscribe implementation.
    ///
    /// # Arguments
    /// * `request` - Unsubscribe request.
    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeResponse>, Status> {
        info!("Got an unsubscribe request: {:?}", request);
        // TODO - provide unsubscribe functionality
        let response = UnsubscribeResponse {};

        Ok(Response::new(response))
    }

    /// Get implementation.
    ///
    /// # Arguments
    /// * `request` - Get request.
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        info!("Got a get request: {:?}", request);
        // TODO - provide get functionality
        let response = GetResponse {};

        Ok(Response::new(response))
    }

    /// Set implementation.
    ///
    /// # Arguments
    /// * `request` - Set request.
    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        info!("Got a set request: {:?}", request);
        // TODO - provide set functionality
        let response = SetResponse {};

        Ok(Response::new(response))
    }

    /// Invoke implementation.
    ///
    /// # Arguments
    /// * `request` - Invoke request.
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        info!("Got an invoke request: {:?}", request);
        // TODO - provide set functionality
        let response = InvokeResponse {};

        Ok(Response::new(response))
    }
}
