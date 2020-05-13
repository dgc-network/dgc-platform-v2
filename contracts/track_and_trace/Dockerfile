# Copyright (c) The dgc.network
# SPDX-License-Identifier: Apache-2.0

FROM hyperledger/grid-dev:v1 as grid-track_and_trace-builder

COPY contracts/track_and_trace /build/contracts/track_and_trace
COPY sdk/ /build/sdk/

# Do a release build to cache dependencies
WORKDIR /build/contracts/track_and_trace

# Build the contract
ARG REPO_VERSION
RUN sed -i -e "0,/version.*$/ s/version.*$/version\ =\ \"${REPO_VERSION}\"/" Cargo.toml
RUN cargo build --target wasm32-unknown-unknown --release

# Build a scar file
# Copy the packaging directory
COPY contracts/track_and_trace/packaging/scar/* \
     /build/contracts/track_and_trace/packaging/scar/

# Copy the contract to the packaging directory
RUN cp /build/target/wasm32-unknown-unknown/release/grid-track-and-trace-tp.wasm \
    packaging/scar

WORKDIR /build/contracts/track_and_trace/packaging/scar

# Create .scar file
RUN tar -jcvf /tmp/grid-track-and-trace_${REPO_VERSION}.scar .

# -------------=== track_and_trace submitter build ===-------------

FROM hyperledger/sawtooth-sabre-cli

COPY --from=grid-track_and_trace-builder \
     /build/target/wasm32-unknown-unknown/release/grid-track-and-trace-tp.wasm /tmp

# Copy the contract definition
COPY contracts/track_and_trace/track_and_trace.yaml /tmp

COPY --from=grid-track_and_trace-builder \
    /tmp/grid-track-and-trace*.scar /tmp
