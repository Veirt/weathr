use crate::error::WeatherError;
use crate::weather::types::{WeatherLocation, WeatherUnits};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod aad;
pub mod met_office;
pub mod open_meteo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherProviderResponse {
    pub weather_code: i32,
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub humidity: f64,
    pub precipitation: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub cloud_cover: f64,
    pub pressure: f64,
    pub visibility: Option<f64>,
    pub is_day: i32,
    pub moon_phase: Option<f64>,
    pub timestamp: String,
    pub attribution: String,
}

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
    ) -> Result<WeatherProviderResponse, WeatherError>;

    fn get_attribution(&self) -> &'static str;
}

#[async_trait]
/// This trait is used supplement a weather provider if it cannot by itself provide all data for `WeatherProviderResponse`
/// An Example would be the Met Office doesn't give Sun & Moon information
pub trait SupplementaryWeatherProvider {
    async fn get_supplementary_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
        wanted: SupplementaryProviderRequest,
    ) -> Result<SupplementaryProviderResponse, WeatherError>;

    #[allow(unused)]
    fn get_attribution(&self) -> &'static str;

    #[allow(unused)] // I want to have a way for sup-providers to add their own capabilites to a list for mix&matching if a sup-provider is unavailable
    fn capabilities(&self) -> Vec<SupplementaryProviderRequest>;
}

/// Helper macro - TODO: Remove `#[allow(dead_code)]`
macro_rules! provider_enums {
    (
        $(
            $name:ident
            $payload:tt
        ),* $(,)?
    ) => {
        pub enum SupplementaryProviderRequest {
            #[allow(dead_code)]
            $(
                $name,
            )*
        }

        pub enum SupplementaryProviderResponse {
            #[allow(dead_code)]
            $(
                $name $payload,
            )*
        }
    };

    (@expand_variant $name:ident ( $($inner:tt)* )) => {
        $name($($inner)*)
    };

    (@expand_variant $name:ident { $($inner:tt)* }) => {
        $name { $($inner)* }
    };
}

provider_enums! {
    PhasesOfMoon(Option<f64>),
    SunAndMoonForOneDay {
        is_day: bool, // Consider i32 if support for twilight is added
        moon_phase: Option<f64>
    }
}
