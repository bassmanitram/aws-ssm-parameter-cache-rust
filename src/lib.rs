// Apache License

// Copyright (c) 2021 Adam Quigley

//! This crate provides a client for in-process caching of parameters from AWS SSM for Rust applications.
//! It is heavily inspired by the [AWS SSM Go Caching Client](https://github.com/aws/aws-secretsmanager-caching-go)
//! and the [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust).
//!
//! The client internally uses an LRU (least-recently used) caching scheme that provides
//! O(1) insertions and O(1) lookups for cached values.

//! ## Example
//! ```rust
//! use aws_sdk_ssm::Client;
//! use aws_ssm_parameter_cache::ParameterCache;
//!
//! #[tokio::main]
//! async fn main() {
//!     let aws_config = aws_config::from_env().load().await;
//!     let client = Client::new(&aws_config);
//!     let mut cache = ParameterCache::new(client);
//!
//!     let parameter_name = "service/parameter";
//!
//!     match cache.get_parameter(parameter_name).send().await {
//!         Ok(parameter_value) => {
//!             // do something
//!         }
//!         Err(e) => println!("{}", e),
//!     }
//! }
//! ```

mod cache;
mod cache_item;
mod config;
pub use cache::ParameterCache;
pub use config::CacheConfig;
