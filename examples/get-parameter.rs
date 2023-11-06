use aws_sdk_ssm::Client;
use aws_ssm_parameter_cache::ParameterCache;

#[tokio::main]
async fn main() {
    let aws_config = aws_config::from_env().load().await;
    let client = Client::new(&aws_config);
    let mut cache = ParameterCache::new(client);

    let parameter_name = "service/parameter";

    match cache.get_parameter(parameter_name).send().await {
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
