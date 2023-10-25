# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

# syntax=docker/dockerfile:1

# Comments are provided throughout this file to help you get started.
# If you need more help, visit the Dockerfile reference guide at
# https://docs.docker.com/engine/reference/builder/

################################################################################
# Create a stage for building the application.

ARG RUST_VERSION=1.72.1
FROM docker.io/library/rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME=invehicle-digital-twin
WORKDIR /sdv

COPY ./ .

# Add Build dependencies.
RUN apt update && apt upgrade -y && apt install -y protobuf-compiler

# Check that APP_NAME argument is valid.
RUN sanitized=$(echo "${APP_NAME}" | tr -dc '^[a-zA-Z_0-9-]+$'); \
[ "$sanitized" = "${APP_NAME}" ] || { \
    echo "ARG 'APP_NAME' is invalid. APP_NAME='${APP_NAME}' sanitized='${sanitized}'"; \
    exit 1; \
}

# Build the application with the 'containerize' feature.
RUN cargo build --release -p "${APP_NAME}" --features "containerize"

# Copy the built application to working directory.
RUN cp ./target/release/"${APP_NAME}" /sdv/service

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
#
# The example below uses the debian bullseye image as the foundation for running the app.
# By specifying the "bullseye-slim" tag, it will also use whatever happens to be the
# most recent version of that tag when you build your Dockerfile. If
# reproducability is important, consider using a digest
# (e.g., debian@sha256:ac707220fbd7b67fc19b112cee8170b41a9e97f703f588b2cdbbcdcecdd8af57).
FROM docker.io/library/debian:bullseye-slim AS final

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/#user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

WORKDIR /sdv

# Copy the executable from the "build" stage.
COPY --from=build /sdv/service /sdv/

# Copy configuration for service.
COPY --from=build /sdv/container/config/standalone/ /sdv/

# Expose the port that the application listens on.
EXPOSE 5010

# What the container should run when it is started.
CMD ["/sdv/service"]
