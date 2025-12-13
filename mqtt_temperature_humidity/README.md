# MQTT temperature + humidity sample

The firmware samples the SHT31 sensor, prints the raw measurements over the serial console, and publishes both telemetry and device status JSON payloads to an MQTT broker.

## Configuration

Build-time environment variables configure both Wi-Fi and MQTT. The easiest way to provide them is to add a `.cargo/config.toml` that looks like this:

```toml
[env]
SSID = "YourAccessPoint"
WIFI_PASSWORD = "super-secret"
MQTT_BROKER_IP = "192.168.1.10"
MQTT_BROKER_PORT = "1883"
MQTT_TOPIC_TELEMETRY = "sensors/esp32/temperature"
MQTT_TOPIC_STATUS = "sensors/esp32/status"
MQTT_CLIENT_ID = "esp32-temp-display"
```

* `MQTT_BROKER_IP` must be an IPv4 address the ESP32 can reach. DNS is not used to keep the implementation small.
* `MQTT_TOPIC_TELEMETRY` carries the telemetry payload (`{"temperature_c":..,"humidity_pct":..}`) while `MQTT_TOPIC_STATUS` carries the device status payload (`{"online":..,"rssi_dbm":..,"uptime_s":..}` and similar fields).
