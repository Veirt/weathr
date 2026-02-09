use crate::weather::provider::WeatherProviderResponse;
use crate::weather::types::{WeatherCondition, WeatherData};

pub struct WeatherNormalizer;

impl WeatherNormalizer {
    pub fn normalize(response: WeatherProviderResponse) -> WeatherData {
        let condition = Self::wmo_code_to_condition(response.weather_code);

        WeatherData {
            condition,
            temperature: response.temperature,
            apparent_temperature: response.apparent_temperature,
            humidity: response.humidity,
            precipitation: response.precipitation,
            wind_speed: response.wind_speed,
            wind_direction: response.wind_direction,
            cloud_cover: response.cloud_cover,
            pressure: response.pressure,
            visibility: response.visibility,
            is_day: response.is_day == 1,
            moon_phase: response.moon_phase,
            timestamp: response.timestamp,
        }
    }

    fn wmo_code_to_condition(code: i32) -> WeatherCondition {
        match code {
            0 => WeatherCondition::Clear,
            1 => WeatherCondition::PartlyCloudy,
            2 => WeatherCondition::PartlyCloudy,
            3 => WeatherCondition::Overcast,
            45 | 48 => WeatherCondition::Fog,
            51 | 53 | 55 => WeatherCondition::Drizzle,
            56 | 57 => WeatherCondition::FreezingRain,
            61 | 63 | 65 => WeatherCondition::Rain,
            66 | 67 => WeatherCondition::FreezingRain,
            71 | 73 | 75 => WeatherCondition::Snow,
            77 => WeatherCondition::SnowGrains,
            80..=82 => WeatherCondition::RainShowers,
            85 | 86 => WeatherCondition::SnowShowers,
            95 => WeatherCondition::Thunderstorm,
            96 | 99 => WeatherCondition::ThunderstormHail,
            _ => WeatherCondition::Clear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmo_code_mapping() {
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(0),
            WeatherCondition::Clear
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(1),
            WeatherCondition::PartlyCloudy
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(3),
            WeatherCondition::Overcast
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(45),
            WeatherCondition::Fog
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(51),
            WeatherCondition::Drizzle
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(61),
            WeatherCondition::Rain
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(71),
            WeatherCondition::Snow
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(80),
            WeatherCondition::RainShowers
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(95),
            WeatherCondition::Thunderstorm
        );
        assert_eq!(
            WeatherNormalizer::wmo_code_to_condition(99),
            WeatherCondition::ThunderstormHail
        );
    }

    #[test]
    fn test_normalize_response() {
        let response = WeatherProviderResponse {
            weather_code: 61,
            temperature: 20.5,
            apparent_temperature: 19.0,
            humidity: 75.0,
            precipitation: 2.5,
            wind_speed: 15.0,
            wind_direction: 180.0,
            cloud_cover: 85.0,
            pressure: 1013.0,
            visibility: Some(10000.0),
            is_day: 1,
            moon_phase: Some(0.5),
            timestamp: "2024-01-01T12:00".to_string(),
        };

        let data = WeatherNormalizer::normalize(response);

        assert_eq!(data.condition, WeatherCondition::Rain);
        assert_eq!(data.temperature, 20.5);
        assert!(data.is_day);
        assert_eq!(data.moon_phase, Some(0.5));
    }
}
