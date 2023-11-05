# AWS SSM Parameter Rust Caching Client

<!---
![CI](https://github.com/bassmanitram/aws-ssm-parameter-cache-rust/actions/workflows/ci.yml/badge.svg)
-->

## DISCLAIMER

This is an out-and-out rip-off of Adam Quigly's work on [aws-secretsmanager-cache-rust](https://github.com/adamjq/aws-secretsmanager-cache-rust).
In time, Adam may wish to merge my entirely dodgy sed-editted facimile of his excellent work and
thus remove the stain on the world that this un-ashamed grasp for reflected glory represents.

In the mean time, I bow to his generosity of spirit allowing me to conterfeit his art for my own purposes!

# Adam's readme, adjusted to read for SSM parameters

This crate provides a client for in-process caching of Parameters from AWS SSM for Rust applications. 
It's heavily inspired by the [AWS Secrets Manager Go Caching Client](https://github.com/aws/aws-secretsmanager-caching-go) 
and the [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust).

The client internally uses an LRU (least-recently used) caching scheme that provides 
O(1) insertions and O(1) lookups for cached values.

## Getting started

To use this client you must have:
- A Rust development environment
- An Amazon Web Services (AWS) account to access Parameters stored in AWS SSM and use AWS SDK for Rust.

## Usage

The following sample demonstrates how to get started using the client:

```rust
use aws_sdk_ssm::Client;
use aws_ssm_parameter_cache::ParameterCache;

#[tokio::main]
async fn main() {
    // instantiate an AWS SSM client using the AWS Rust SDK
    let aws_config = aws_config::from_env().load().await;
    let client = Client::new(&aws_config);
    
    let mut cache = ParameterCache::new(client);

    match cache.get_ssm_parameter_string("YOUR_PARAMETER_ID".to_string()).send().await {
        Ok(parameter_value) => {
            // use SSM parameter value
        }
        // e.g. ResourceNotFoundException: SSM can't find the specified parameter.
        Err(e) => println!("ERROR: {}", e),
    }
}
```

### Forcing cache refreshes

If a parameter has been rotated since the last value was fetched and cached, and hasn't expired in the cache, it's necessary to force a cache refresh for the value by calling AWS and updating the value.

This can be done with `force_refresh()`, for example:

```rust
    match cache
        .get_ssm_parameyet_string("YOUR_PARAMETER_ID".to_string())
        .force_refresh()
        .send()
        .await
```

## Cache Configuration

- `max_cache_size usize` The maximum number of secrets to maintain in the cache 
before evicting the least frequently accessed
- `cache_item_ttl u128` The number of nanoseconds a cached parameter will be considered 
valid before the parameter value requires a refresh. Refreshing happens synchronously.

```rust
use aws_sdk_ssm::Client;
use aws_ssm_parameter_cache::{CacheConfig, ParameterCache};
use std::time;

#[tokio::main]
async fn main() {
    let aws_config = aws_config::from_env().load().await;
    let client = Client::new(&aws_config);

    // cache configuration with 30 second expiry time and maximum 1000 secrets
    let cache_config = CacheConfig::new()
        .cache_item_ttl(time::Duration::from_secs(30).as_nanos())
        .max_cache_size(1000);

    let mut cache = ParameterCache::new_with_config(client, cache_config);
}
```

## Global Caching

Certain cloud environments like AWS Lambda encourage initializing clients in the global scope to avoid initialization for
each function invocation. This can be achieved using the `lazy_static` crate, for example: 

```rust
use async_once::AsyncOnce;
use aws_sdk_ssm::Client;
use aws_ssm_parameter_cache::ParameterCache;
use lazy_static::lazy_static;
use std::sync::Mutex;

// store the cache in the global scope - useful for runtime environments like AWS Lambda
lazy_static! {
    static ref CACHE: AsyncOnce<Mutex<ParameterCache>> = AsyncOnce::new(async {
        Mutex::new(ParameterCache::new(Client::new(
            &aws_config::from_env().load().await,
        )))
    });
}

#[tokio::main]
async fn main() {
    // use cache
}
```

## Development

### Linting

The project uses [rustfmt](https://github.com/rust-lang/rustfmt) and [clippy](https://github.com/rust-lang/rust-clippy) for 
formatting and linting. Follow the instructions to install `rustfmt` and `clippy` and run:

```bash
cargo fix
```

### Tests

Run unit tests locally with:
```bash
cargo test
```

## License

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) or the [MIT license](https://opensource.org/licenses/MIT), at your option. Files in the project may not be copied, modified, or distributed except according to those terms.
