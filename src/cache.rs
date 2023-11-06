use std::num::NonZeroUsize;

use super::cache_item::CacheItem;
use super::config::CacheConfig;
use aws_sdk_config::error::SdkError;
use aws_sdk_ssm::operation::get_parameter::GetParameterError;
use aws_sdk_ssm::Client as SSMClient;
use lru::LruCache;

/// Client for in-process caching of parameter values from AWS SSM.
///
/// An LRU (least-recently used) caching scheme is used that provides
/// O(1) insertions and O(1) lookups for cached values.
pub struct ParameterCache {
    client: SSMClient,
    config: CacheConfig,
    cache: LruCache<String, CacheItem<String>>,
}

impl ParameterCache {
    /// Returns a new ParameterCache using the default Cache Configuration options.
    pub fn new(client: SSMClient) -> Self {
        ParameterCache::new_cache(client, CacheConfig::new())
    }

    /// Returns a new ParameterCache using a provided custom Cache Configuration.
    pub fn new_with_config(client: SSMClient, config: CacheConfig) -> Self {
        ParameterCache::new_cache(client, config)
    }

    fn new_cache(client: SSMClient, config: CacheConfig) -> Self {
        let cache = LruCache::new(
            NonZeroUsize::new(config.max_cache_size)
                .unwrap_or(NonZeroUsize::new(1).expect("Default max_cache_size must be non-zero")),
        );
        Self {
            client,
            config,
            cache,
        }
    }

    /// Returns a builder for getting parameter strings.
    ///
    /// Retrieve the parameter value with send()
    pub fn get_parameter<'a,'b>(&'a mut self, parameter_name: &'b str) -> GetParameterStringBuilder<'a,'b> {
        GetParameterStringBuilder::new(self, parameter_name)
    }
}

/// A builder for the get_parameter method.
pub struct GetParameterStringBuilder<'a,'b> {
    parameter_cache: &'a mut ParameterCache,
    parameter_name: &'b str,
    force_refresh: bool,
}

impl<'a,'b> GetParameterStringBuilder<'a,'b> {
    pub fn new(parameter_cache: &'a mut ParameterCache, parameter_name: &'b str) -> Self {
        GetParameterStringBuilder {
            parameter_cache,
            parameter_name,
            force_refresh: false,
        }
    }

    /// Forces a refresh of the parameter.
    ///
    /// Forces the parameter to be fetched from AWS and updates the cache with the fresh value.
    /// This is required when the cached parameter is out of date but not expired, for example due to rotation.
    pub fn force_refresh(mut self) -> Self {
        self.force_refresh = true;
        self
    }

    /// Fetches the parameter value from the cache.
    ///
    /// If the parameter value exists in the cache and hasn't expired it will be immediately returned.
    /// The parameter will be fetched by calling AWS SSM and updated in the cache if:
    /// - the parameter value hasn't been stored in the cache
    /// - the parameter stored in the cache but has expired
    /// - the force_refresh option was provided
    ///
    /// Values are stored in the cache with the cache_item_ttl from the CacheConfig.
    pub async fn send(&mut self) -> Result<String, SdkError<GetParameterError>> {
        if !self.force_refresh {
            if let Some(cache_item) = self.parameter_cache.cache.get(self.parameter_name) {
                if !cache_item.is_expired() {
                    return Ok(cache_item.value.clone());
                }
            }
        }

        match self.fetch_parameter().await {
            Ok(parameter_value) => {
                let cache_item = CacheItem::new(
                    parameter_value.clone(),
                    self.parameter_cache.config.cache_item_ttl,
                );
                self.parameter_cache
                    .cache
                    .put(self.parameter_name.to_string(), cache_item);
                Ok(parameter_value)
            }
            Err(e) => Err(e),
        }
    }

    async fn fetch_parameter(&mut self) -> Result<String, SdkError<GetParameterError>> {
        match self
            .parameter_cache
            .client
            .get_parameter()
            .name(self.parameter_name)
            .send()
            .await
        {
            Ok(resp) => return Ok(resp.parameter.unwrap().value.unwrap()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_config::config::{Credentials, Region};
    use aws_sdk_ssm::{Client as SSMClient, Config};

    #[test]
    fn get_parameter_builder_defaults() {
        let mock_ssm_client = get_mock_ssm_client();
        let mut parameter_cache = ParameterCache::new(mock_ssm_client);

        let builder = GetParameterStringBuilder::new(&mut parameter_cache, "service/parameter");

        assert_eq!(builder.parameter_name, "service/parameter");
        assert!(!builder.force_refresh);
    }

    #[test]
    fn get_parameter_builder_force_refresh() {
        let mock_ssm_client = get_mock_ssm_client();
        let mut parameter_cache = ParameterCache::new(mock_ssm_client);

        let builder = GetParameterStringBuilder::new(&mut parameter_cache, "service/parameter")
            .force_refresh();

        assert_eq!(builder.parameter_name, "service/parameter");
        assert!(builder.force_refresh);
    }

    // provides a mocked AWS SSM client for testing
    fn get_mock_ssm_client() -> SSMClient {
        let conf = Config::builder()
            .region(Region::new("ap-southeast-2"))
            .credentials_provider(Credentials::new("asdf", "asdf", None, None, "test"))
            .build();

        SSMClient::from_conf(conf)
    }
}
