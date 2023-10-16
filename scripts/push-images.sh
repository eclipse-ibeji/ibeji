#!/bin/bash

CONTAINER_REGISTRY=agritechacr001
IMAGE_VERSION=0.1.1

# Login to Container Registry
az acr login --name ${CONTAINER_REGISTRY}

# Tag images
docker tag ibeji-agritechnica-dt:${IMAGE_VERSION} ${CONTAINER_REGISTRY}.azurecr.io/ibeji-agritechnica-dt:${IMAGE_VERSION}
docker tag ibeji-agritechnica-provider:${IMAGE_VERSION} ${CONTAINER_REGISTRY}.azurecr.io/ibeji-agritechnica-provider:${IMAGE_VERSION}
docker tag ibeji-agritechnica-consumer:${IMAGE_VERSION} ${CONTAINER_REGISTRY}.azurecr.io/ibeji-agritechnica-consumer:${IMAGE_VERSION}

# Push images
docker push ${CONTAINER_REGISTRY}.azurecr.io/ibeji-agritechnica-dt:${IMAGE_VERSION}
docker push ${CONTAINER_REGISTRY}.azurecr.io/ibeji-agritechnica-provider:${IMAGE_VERSION}
docker push ${CONTAINER_REGISTRY}.azurecr.io/ibeji-agritechnica-consumer:${IMAGE_VERSION}