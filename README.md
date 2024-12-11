# mqtt-to-dawarich

A simple daemon that bridges the gap between Owntracks' MQTT connection and [Dawarich](https://github.com/Freika/dawarich).

## Why

I initially set up Owntracks using MQTT and don't want to migrate over to HTTP.

Also, utilising MQTT means I can fan-out the data as I see fit, for example, to Homeassistant and now Dawarich.

## How it works

- Listens to the Owntracks MQTT topic
- Converts MQTT message to API requests for Dawarich to emulate the Owntracks HTTP connection

## Usage

### Daemon

Clone the repo and run:

```bash
cargo build --release
./target/release/mqtt-to-dawarich
```

### Docker

See docker-compose.yml.example

## Environment Variables

| Variable | Description | Default Value |
| --- | --- | --- |
| `DAWARICH_BASE_URL` | The URL of the Dawarich API endpoint | `127.0.0.1` |
| `DAWARICH_PORT` | The port of the Dawarich API endpoint | `3000` |
| `DAWARICH_API_KEY` | The Dawarich API key, found http://<DAWARICH_BASE_URL>:<DAWARICH_PORT>/users/edit |  |
| `MQTT_BROKER_URL` | The URL of the MQTT broker | `127.0.0.1` |
| `MQTT_BROKER_PORT` | The port of the MQTT broker | `1883` |
| `MQTT_USERNAME` | The username for the MQTT broker |  |
| `MQTT_PASSWORD` | The password for the MQTT broker |  |
| `MQTT_TOPIC` | The topic which Owntracks send the message to |  |
| `MQTT_KEEP_ALIVE_DURATION` | MQTT keep alive setting (seconds) | `30` |
| `RUST_LOG` | Log level | `error` |

### Note

There are minimal logs, recommend to use info to confirm the MQTT and Dawarich URL connection settings.
