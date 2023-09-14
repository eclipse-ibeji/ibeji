# Ibeji Core Common

## gRPC Interceptor

Tower allows you to build a stack of layers and inject them in front of the services running in the Tonic web server.
Each of these tower layers can have a tower service implementation that examines and/or manipulates incoming http requests and outgoing http responses.

Tower does not provide support for gRPC specific http messages, so we needed to provide those capabilities.



    let sample_layer: Option<GrpcInterceptorLayer> =
        Some(GrpcInterceptorLayer::new(SampleGrpcInterceptor::sample_grpc_interceptor_factory));

    let layer = ServiceBuilder::new().option_layer(sample_layer);

    let invehicle_digital_twin_impl = invehicle_digital_twin_impl::InvehicleDigitalTwinImpl {
        entity_access_info_map: Arc::new(RwLock::new(HashMap::new())),
    };

    // Setup the HTTP server.
    Server::builder()
        .layer(layer)
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr)
        .await?;

