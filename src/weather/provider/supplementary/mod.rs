use async_trait::async_trait;

use crate::{error::WeatherError, weather::{WeatherLocation, WeatherUnits}};

pub mod aad;

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
