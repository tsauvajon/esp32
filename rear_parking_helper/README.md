# Project

I have two main goals:
- measure the temperature over time, in my garage
- build a rear parking helper, measuring car position and showing direction/position with LEDs

Why two different things in a single project?
1. Cost and simplicity: it use a single ESP32 for two distinct projects
2. The temperature export via WiFi is resource hungry: we can run it only when the rear parking helper detects something, to save on energy. 

### Rear parking helper
I want to use an addressable LED strip, that gradually lights more LEDs as I get closer to the back wall. When I get in the final position, the LEDs blink.
I chose a WS2812B RGB LED strip (60 Pixels/m) for a good balance between cost and quality. I'll cut two small strips out of it, I think.

To detect distance, I'm using an ST1099 ultrasound sensor that I already had around. Ideally I'd get a second ultrasound sensor for side alignment too, but that's for later.

###
For the temperature, I'm using an SHT31 sensor. The garage is quite remote from home and has no direct connection of any type. It's also buried in concrete, so signals don't reach. I bought a couple of CC1101 433MHz transmitters/receivers for remote transmission to my home server, but couldn't make them work.

So instead, I'm going to store the sensor data, compacted in memory. When the ESP32 detects movement (through the ultrasound sensor), it spawns a web server, killed after a few minutes of inactivity (to save on energy), as well as a WiFi access point. By browsing the ESP32's IP address at http://192.168.2.1, I can (1) download all the sensor data as a file, (2) clear the existing sensor data - better than automatically clearing it on download, to help with testing and also prevent errors from being fatal, and (3) see the current memory usage and number of records, so that I know how often I need to collect the data.

Why an access point, and not connect the ESP to my phone? -> it's going to be more future proof, I don't want to re-flash the ESP32 whenever I change my phone's hotspot name or password.

With the downloaded data on my phone, I'll go home and upload it to some kind of Prometheus server or InfluxDb, or something similar, to display in Grafana.

# TODO

- [ ] Replace the single LED by an addressable LED strip
- [ ] Instead of binding proximity to LED brightness, make it a more visually appealing and user-friendly pattern. Grow then blink sounds good.
- [ ] Bring the web server from the standalone project to this project
- [ ] Bring the temperature data from the standalone project to this project
- [ ] Store the sensor data locally - run a measurement every 15 minutes
- [ ] Instead of a single measure, measure a few times and average the results
- [ ] Return the stored data (for example as CSV) through a new server endpoint
- [ ] Show the number of stored records in the /stats endpoint
- [ ] Allow clearing the data through a new server endpoint
- [ ] Setup the power wires for safe use
- [ ] Find a good way to organise wires and solder everything. Ideally, long cable for the SHT31 so it can go in the next room, or at least close to the floor. ESP32 either in-between the LED strips, or maybe inside the back room. Ultrasound where it makes sense and works well with the car.
