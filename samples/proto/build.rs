// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("sample_grpc/v1/digital_twin_consumer.proto")?;
    tonic_build::compile_protos("sample_grpc/v1/digital_twin_provider.proto")?;
    Ok(())
}
