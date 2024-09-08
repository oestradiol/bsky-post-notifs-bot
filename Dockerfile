## Step 1: Base Image Setup
FROM rust:1.81 AS base

## Step 2: Source Code Build
FROM base AS builder
WORKDIR /app

# Copy the entire source code
COPY . .

# Build the application
# RUN rustup toolchain install nightly
# ENV RUSTFLAGS=-Z threads=8
# RUN cargo +nightly build --release --locked
RUN cargo build --release --locked

## Step 3: Production Image Setup
FROM base AS runner
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/app .
COPY --from=builder /app/.env .

# ## Step 4: User Setup
# # Creates the group and the user
# RUN addgroup --system --gid 1001 rust
# RUN adduser --system --uid 1001 posts

# # Changes the ownership of the workdir and docker sock
# RUN chown -R posts:rust ./
# RUN chown -R posts:rust /var/run

# # Changes the user to the created user
# USER posts
# RUN chmod -R 666 /var/run

## Step 6: Container Execution
CMD ["./app"]