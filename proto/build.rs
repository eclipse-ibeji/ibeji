// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("consumer.proto")?;
    tonic_build::compile_protos("provider.proto")?;
    tonic_build::compile_protos("digitaltwin.proto")?;
    Ok(())
}
