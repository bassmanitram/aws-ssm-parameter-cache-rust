use aws_sdk_ssm::Client;
use aws_ssm_parameter_cache::{CacheConfig, ParameterCache};
use std::time;

#[tokio::main]
async fn main() {
    let aws_config = aws_config::from_env().load().await;
    let client = Client::new(&aws_config);

    let custom_cache_ttl = time::Duration::from_secs(30).as_nanos();
    let cache_config = CacheConfig::new().cache_item_ttl(custom_cache_ttl);

    let mut cache = ParameterCache::new_with_config(client, cache_config);

    let parameter_name = "service/parameter";

    match cache
        .get_parameter(parameter_name.to_string())
        .force_refresh() // force the value to be fetched from AWS and updated in the cache
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
