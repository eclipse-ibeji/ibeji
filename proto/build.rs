// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("consumer.proto")?;
    tonic_build::compile_protos("provider.proto")?;
    tonic_build::compile_protos("digitaltwin.proto")?;
    Ok(())
}
