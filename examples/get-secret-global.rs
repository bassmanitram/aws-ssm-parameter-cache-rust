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
    let parameter_name = "service/parameter";

    match CACHE
        .get() // get cache from the global scope
        .await
        .lock() // acquire cache lock
        .unwrap()
        .get_parameter(parameter_name.to_string())
        .send()
        .await
    {
        Ok(parameter_value) => {
            println!(
                "Successfully retrieved parameter {}: {}",
                parameter_name, parameter_value
            );
        }
        // e.g. ResourceNotFoundException: SSM can't find the specified parameter.
        Err(e) => println!("ERROR: Error getting parameter '{}'. {}", parameter_name, e),
    }
}
