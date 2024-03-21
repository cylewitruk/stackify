ARG USER_ID
ARG GROUP_ID
ARG STACKS_2_4_TAG_BRANCH
ARG NAKAMOTO_TAG_BRANCH

# ------------------------------------------------------------------------------
# Build stage for Bitcoin Core and other downloaded dependency binaries
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim as download-deps
ARG BITCOIN_VERSION

RUN apt update \
    && apt install -y ca-certificates wget \
    && mkdir /out

# 'dasel' - a jq-like tool for querying and updating JSON, TOML, YAML, etc.
RUN wget -O /out/dasel https://github.com/TomWright/dasel/releases/download/v2.7.0/dasel_linux_amd64

# Bitcoin Core
RUN wget https://bitcoincore.org/bin/bitcoin-core-${BITCOIN_VERSION}/bitcoin-${BITCOIN_VERSION}-x86_64-linux-gnu.tar.gz \
    && tar -zxvf bitcoin-${BITCOIN_VERSION}-x86_64-linux-gnu.tar.gz \
    && cp bitcoin-${BITCOIN_VERSION}/bin/* /out/

RUN chmod +x /out/*

# ------------------------------------------------------------------------------
# Build stage for Stacks Core dependencies and build environment
# ------------------------------------------------------------------------------
FROM rust:1.76-slim-bookworm as build-base
ARG USER_ID
ARG GROUP_ID
ARG BITCOIN_VERSION

COPY --from=download-deps /out/bitcoin-cli /usr/local/bin/bitcoin-cli

# Update and install packages
RUN apt update \
    && apt upgrade -y \
    && apt install -y build-essential libclang-dev git wget tree \
        jq libssl-dev pkg-config libfontconfig-dev libsqlite3-dev \
        gcc-multilib clang mold libsecp256k1-1 libsecp256k1-dev \
        procps

RUN rustup component add rustfmt

# Create our non-root user & group ('stacks')
RUN groupadd -r -g ${GROUP_ID} stacks \
    && useradd -r -m --uid ${GROUP_ID} -g ${GROUP_ID} stacks \
    && install -d -m 0755 -o stacks -g stacks \
        /src \
        /sccache \
        /target \
        /stacks \
        /bitcoin

USER stacks
COPY --chown=stacks:stacks --from=download-deps /out/* /home/stacks/bin/
COPY --chown=stacks:stacks ./cargo-config.toml /home/stacks/.cargo/config.toml

RUN mkdir ~/repos \
    && cd ~/repos \
    && git clone https://github.com/stacks-network/stacks-core.git \
    && git clone https://github.com/stacks-network/sbtc.git \
    && git clone https://github.com/hirosystems/clarinet.git --recursive \
    && touch ~/stacks-core-deps

#ENV RUSTC_WRAPPER /usr/local/cargo/bin/sccache
ENV CARGO_INCREMENTAL 0
ENV SCCACHE_DIRECT true
ENV SCCACHE_DIR /sccache
ENV CARGO_TARGET_DIR /target

WORKDIR /home/stacks

# ------------------------------------------------------------------------------
# Build stage for Stacks Core dependencies and build environment
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim as runtime
ARG USER_ID
ARG GROUP_ID
ARG BITCOIN_VERSION

COPY --from=download-deps /out/bitcoin-cli /usr/local/bin/bitcoin-cli
COPY --from=download-deps /out/bitcoind /usr/local/bin/bitcoind
COPY --from=download-deps /out/dasel /usr/local/bin/dasel

RUN apt update \
    && apt upgrade -y \
    && apt install -y procps sqlite3 jq

# Create our non-root user & group ('stacks')
RUN groupadd -r -g ${GROUP_ID} stacks \
    && useradd -r -m --uid ${GROUP_ID} -g ${GROUP_ID} stacks \
    && install -d -m 0755 -o stacks -g stacks \
        /src \
        /sccache \
        /target \
        /stacks \
        /stacks/logs \
        /stacks/bitcoin \
        /stacks/conf

WORKDIR /home/stacks