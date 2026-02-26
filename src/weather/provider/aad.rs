//! US Government Astronomical Applications Department

use async_trait::async_trait;
use chrono::{Local, NaiveTime};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    error::{NetworkError, WeatherError},
    weather::{
        WeatherLocation, WeatherUnits,
        provider::{
            SupplementaryProviderRequest, SupplementaryProviderResponse,
            SupplementaryWeatherProvider,
        },
    },
};

const BASE_URL: &str = "https://aa.usno.navy.mil/api/";

pub struct AADProvider;

impl Default for AADProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AADProvider {
    pub fn new() -> Self {
        Self
    }

    fn build_url(
        &self,
        wanted: &SupplementaryProviderRequest,
        location: &WeatherLocation,
    ) -> String {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let offset_seconds = now.offset().local_minus_utc();
        let offset_hours = offset_seconds / 3600;

        match wanted {
            SupplementaryProviderRequest::PhasesOfMoon => {
                format!("{BASE_URL}moon/phases/date?date={date}&nump=1")
            }
            SupplementaryProviderRequest::SunAndMoonForOneDay => {
                format!(
                    "{BASE_URL}rstt/oneday?date={date}&coords={},{}&tz={}&dst=true",
                    location.latitude, location.longitude, offset_hours
                )
            }
        }
    }

    fn convert_string_to_moon_phase(value: &str) -> f64 {
        match value {
            // New Moon
            "Waxing Crescent" => 0.15,
            "First Quarter" => 0.25,
            "Waxing Gibbous" => 0.35,
            "Full Moon" => 0.5,
            "Waning Gibbous" => 0.65,
            "Last Quarter" => 0.75,
            "Waning Crescent" => 0.85,
            _ => 0.0, // New Moon
        }
    }
}

#[async_trait]
impl SupplementaryWeatherProvider for AADProvider {
    async fn get_supplementary_weather(
        &self,
        location: &WeatherLocation,
        #[allow(unused_variables)] units: &WeatherUnits,
        wanted: SupplementaryProviderRequest,
    ) -> Result<SupplementaryProviderResponse, WeatherError> {
        let url = self.build_url(&wanted, location);

        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        let data: Value = response
            .json()
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        let now = Local::now();

        match wanted {
            SupplementaryProviderRequest::PhasesOfMoon => {
                // TODO: Consider using the Fracillum / 10
                let phase_data = &data["phasedata"];

                let phases: Vec<MoonPhase> = serde_json::from_value(phase_data.clone()).unwrap();

                let current_phase = phases.first().unwrap();

                let phase = AADProvider::convert_string_to_moon_phase(&current_phase.phase);
                Ok(SupplementaryProviderResponse::PhasesOfMoon(Some(phase)))
            }
            SupplementaryProviderRequest::SunAndMoonForOneDay => {
                let data = &data["properties"]["data"];
                let current_moon_phase =
                    Self::convert_string_to_moon_phase(data["curphase"].as_str().unwrap());
                let sun_data: Vec<SunData> =
                    serde_json::from_value(data["sundata"].clone()).unwrap();

                let sunrise = get_sun_phase(&sun_data, CelestialPhenomena::Rise)
                    .unwrap()
                    .to_chrono_time();
                let sunset = get_sun_phase(&sun_data, CelestialPhenomena::Set)
                    .unwrap()
                    .to_chrono_time();
                let current_time = now.time();

                Ok(SupplementaryProviderResponse::SunAndMoonForOneDay {
                    is_day: current_time > sunrise && current_time < sunset,
                    moon_phase: Some(current_moon_phase),
                })
            }
        }
    }

    fn get_attribution(&self) -> &'static str {
        ""
    }

    fn capabilities(&self) -> Vec<SupplementaryProviderRequest> {
        vec![SupplementaryProviderRequest::PhasesOfMoon]
    }
}

fn get_sun_phase(sun_data: &[SunData], target: CelestialPhenomena) -> Option<&SunData> {
    sun_data.iter().find(|item| item.phen == target)
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum CelestialPhenomena {
    #[serde(rename = "Begin Civil Twilight")]
    BeginCivilTwilight,
    Rise,
    #[serde(rename = "Upper Transit")]
    UpperTransit,
    Set,
    #[serde(rename = "End Civil Twilight")]
    EndCivilTwilight,
}

#[derive(Debug, Clone, Deserialize)]
struct MoonPhase {
    // day: u8,
    // month: u8,
    phase: String,
    // time: String,
    // year: u16,
}

#[derive(Debug, Clone, Deserialize)]
struct SunData {
    pub phen: CelestialPhenomena,
    time: String,
}

impl SunData {
    fn get_time(&self) -> String {
        self.time.clone().replace("  ST", "") // Unsure what ST stands for, but its not needed
    }

    fn to_chrono_time(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.get_time(), "%H:%M").unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::weather::WeatherLocation;
    use crate::weather::provider::aad::BASE_URL;

    #[test]
    fn tz_test() {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        let offset_seconds = now.offset().local_minus_utc();
        let offset_hours = offset_seconds / 3600;
        println!("{date} {offset_hours}");

        let location = WeatherLocation {
            latitude: 52.52,
            longitude: 13.41,
            elevation: None,
        };

        println!(
            "{BASE_URL}rstt/oneday?date={date}&coords={},{}&tz={}&dst=true",
            location.latitude, location.longitude, offset_hours
        );
    }

    #[test]
    fn moon_phase_validation() {
        let step = (0.15f64 * 8.0).round() as usize % 8;
        println!("{}", step);
    }
}
