#!/bin/bash

IMAGE_VERSION=0.1.1

cd ..

# Build Ibeji Agritechnica Digital Twin image
docker build -t ibeji-agritechnica-dt:${IMAGE_VERSION} -f Dockerfile.ibeji.dt .

# Build Ibeji Agritechnica Digital Twin Property Provider image
docker build -t ibeji-agritechnica-provider:${IMAGE_VERSION} -f Dockerfile.ibeji.provider .

# Build Ibeji Agritechnica Digital Twin Property Consumer image
docker build -t ibeji-agritechnica-consumer:${IMAGE_VERSION} -f Dockerfile.ibeji.consumer .