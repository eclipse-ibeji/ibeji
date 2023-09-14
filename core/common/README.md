# Ibeji Core Common

## gRPC Interceptor

gRPC Interceptor is a concept that intercepts gRPC calls received by a tonic http server, so that they can be examine and modified.

gRPC Interceptor is based on the interceptor pattern. Details on the interceptor pattern can be found on [wikipedia](https://en.wikipedia.org/wiki/Interceptor_pattern).

gRPC Interceptors rely on the Tower crate's Layer construct to apply the desired behavior to the incoming requests and outgoing responses.  Tower does not provide support for gRPC specific http messages, so we have provided those capabilities in gRPC Interceptor.

These documents/code were very helpful in developing this solution:
<ul>
  <li> https://docs.rs/tower/latest/tower/trait.Layer.html
  <li> https://docs.rs/tower/latest/tower/trait.Service.html
  <li> https://stackoverflow.com/questions/68203821/prost-the-encode-method-cannot-be-invoked-on-a-trait-object
  <li> https://github.com/hyperium/tonic/blob/master/examples/src/tower/client.rs
  <li> https://github.com/hyperium/tonic/blob/master/examples/src/tower/server.rs
  <li> https://stackoverflow.com/questions/76758914/parse-grpc-orginal-body-with-tonic-prost
  <li> https://stackoverflow.com/questions/57632558/grpc-server-complaining-that-message-is-larger-than-max-size-when-its-not
  <li> https://discord.com/channels/500028886025895936/628706823104626710/1086425720709992602
  <li> https://github.com/tower-rs/tower/issues/727
  <li> https://github.com/linkerd/linkerd2-proxy/blob/0814a154ba8c8cc7af394ac3fa6f940bd01755ae/linkerd/stack/src/fail_on_error.rs#LL30-L69C2
</ul>

## Sample gRPC Interceptor

A simple gRPC Interceptor sample has been provided.  To use it with Ibeji:

Add the following use statements to main.rs:

```rust
use common::grpc_interceptor::GrpcInterceptorLayer;
use common::sample_grpc_interceptor::SampleGrpcInterceptor;
use tower::ServiceBuilder;
```

Adjust the HTTP server setup in main.rs as follows:

```rust
    let sample_layer: Option<GrpcInterceptorLayer> =
        Some(GrpcInterceptorLayer::new(SampleGrpcInterceptor::sample_grpc_interceptor_factory));

    let layer = ServiceBuilder::new().option_layer(sample_layer);

    // Setup the HTTP server.
    Server::builder()
        .layer(layer)
        .add_service(InvehicleDigitalTwinServer::new(invehicle_digital_twin_impl))
        .serve(addr)
        .await?;
```
