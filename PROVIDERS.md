# weathr data providers

How to use different providers & how to impl them

## Using Providers
There are currently 2 available providers which are
- OpenMeteo (default)
- MetOffice

Currently the only way to change the provider is done via the config

### OpenMeteo Provider
By default Open Meteo is used, to change the provider you must have a config

If you want to force the use of this provider add to your config
```
[provider.OpenMeteo]
```


### [MetOffice](metoffice.gov.uk) Provider
This is the [UK Government Met Office](metoffice.gov.uk) weather provider
#### Getting your API key
[https://login.auth.metoffice.cloud/](https://login.auth.metoffice.cloud/)

#### Enabling
To enable simply add to your config once you have your API Key
```
[provider.MetOffice]
# Met Office API key
api_key = "YOUR MET OFFICE API KEY"
```

## Supplementary Providers
Currently there is 1 Sup-provider which is the US Government Astronomical Applications Department

These types of providers are meant to be small & suppliment other providers data in the event they are missing data, an example is the MetOffice provider doesn't provide any atronomical data, instead the provider will make another request to get that data

## Adding providers

### Creating the provider
There are 2 types of providers a `WeatherProvider` and a `SupplementaryWeatherProvider`, a provider can be both a supplementary provider and a "primary" provider

#### Where to place your provider
`src/weather/provider`
#### Must Haves
Your new provider must use a trait, it can be either `WeatherProvider` or `SupplementaryWeatherProvider`


### Making the primary provider useful
Add in `src/config.rs` the `Provider` enum the provider name

Then `src/app.rs` in `App::new` a match at line 139 to map the `Provider` enum to a provider, there is where you add your provider's initialisation 

### Making the supplementary provider useful
Currently supplementary are ad-hoc, the trait is useful for further improvements