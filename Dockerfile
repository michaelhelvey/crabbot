# I know that a multistage build could save us like 50MB but compiling the app
# on a non-al2 system is a pain because we need to compile against the exact
# glibc version that the lambda runtime supports.
FROM public.ecr.aws/lambda/provided:al2 as runner

# Add the lambda adapter that will call our axum application with API gateway requests
# See: https://github.com/awslabs/aws-lambda-web-adapter
COPY --from=public.ecr.aws/awsguru/aws-lambda-adapter:0.8.4 /lambda-adapter /opt/extensions/lambda-adapter

# Shoutout to
# https://fasterthanli.me/series/building-a-rust-service-with-nix/part-5#a-basic-dockerfile-to-build-our-service
# for some good tips for writing good Rust dockerfiles

# Install system dependencies
RUN set -eux; \
  yum update -y; \
  yum install -y \
  ca-certificates gcc pkgconfig openssl openssl-devel \
  ;

# Allow users building the image locally to switch the compiler target for their
# machine if they want.
ARG PLATFORM=aarch64-unknown-linux-gnu

# Install rustup
RUN --mount=type=cache,target=/root/.rustup \
  set -eux; \
  curl --location --fail \
  "https://static.rust-lang.org/rustup/dist/${PLATFORM}/rustup-init" \
  --output rustup-init; \
  chmod +x rustup-init; \
  ./rustup-init -y --no-modify-path --default-toolchain stable; \
  rm rustup-init;

# Add rustup to path, check that it works
ENV PATH=${PATH}:/root/.cargo/bin
RUN set -eux; \
  rustup --version;

# Copy sources and build them
WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.lock ./

# We can cache a bunch of cargo stuff to speed up builds here using BuildKit caches:
RUN --mount=type=cache,target=/root/.rustup \
  --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/root/.cargo/git \
  --mount=type=cache,target=/app/target \
  set -eux; \
  cargo build --release; \
  cp target/release/crabbot .;

ENTRYPOINT ["/app/crabbot"]
