use crate::cache;
use crate::config::Provider;
use crate::error::WeatherError;
use crate::weather::normalizer::WeatherNormalizer;
use crate::weather::provider::WeatherProvider;
use crate::weather::types::{WeatherData, WeatherLocation, WeatherUnits};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct WeatherClient {
    provider: Arc<dyn WeatherProvider>,
    cache: Arc<RwLock<Option<CachedWeather>>>,
    cache_duration: Duration,
}

struct CachedWeather {
    data: WeatherData,
    fetched_at: Instant,
}

impl WeatherClient {
    pub fn new(provider: Arc<dyn WeatherProvider>, cache_duration: Duration) -> Self {
        Self {
            provider,
            cache: Arc::new(RwLock::new(None)),
            cache_duration,
        }
    }

    pub async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
        provider: Provider,
    ) -> Result<WeatherData, WeatherError> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref()
                && cached.fetched_at.elapsed() < self.cache_duration
            {
                return Ok(cached.data.clone());
            }
        }

        if let Some(cached_data) =
            cache::load_cached_weather(location.latitude, location.longitude, provider).await
            && std::env::var("CACHE_DISABLED").is_err()
        // Should've done this sooner
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedWeather {
                data: cached_data.clone(),
                fetched_at: Instant::now(),
            });
            return Ok(cached_data);
        }

        let response = self.provider.get_current_weather(location, units).await?;

        let data = WeatherNormalizer::normalize(response);

        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedWeather {
                data: data.clone(),
                fetched_at: Instant::now(),
            });
        }

        cache::save_weather_cache(&data, location.latitude, location.longitude, provider);

        Ok(data)
    }

    #[allow(dead_code)]
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weather::provider::open_meteo::OpenMeteoProvider;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_invalidation() {
        let provider = Arc::new(OpenMeteoProvider::new());
        let client = WeatherClient::new(provider, Duration::from_secs(60));

        client.invalidate_cache().await;

        let cache = client.cache.read().await;
        assert!(cache.is_none());
    }
}
