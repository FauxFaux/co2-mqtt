## co2-mqtt

A script to read from a common "co2 monitor" and write to MQTT.

This meter looks like
this [CO2Mini Indoor Air Quality Monitor - RAD-0301](https://www.co2meter.com/collections/desktop/products/co2mini-co2-indoor-air-quality-monitor):

![co2meter.com branded co2 monitor](docs/monitor.png)


### Setup

* Have a working `mqtt` broker such as [mosquitto](https://mosquitto.org/).
* Install [`docs/90-co2mini.rules`](docs/90-co2mini.rules) to `/etc/udev/rules.d/90-co2mini.rules`.
* `sudo udevadm control --reload-rules && sudo udevadm trigger` or reboot (hee hee).
* This should make `/dev/co2mini0`, or `/dev/co2mini1` or.. exist.


### Usage

1) Set `MQTT_URL='mqtt://192.168.33.37:1883/?client_id=co2-mqtt'`, the `client_id` is apparently required,
2) Set `CO2_DEVICE=/dev/co2mini0` to the name of your device, found in `/dev`,
3) Optionally set `CO2_NAME=lounge`, if you want this value set reasonably in the topic, otherwise it will be `monitor`,
4) Optionally set `RUST_LOG=debug`, if you want to see what is happening,
5) `cargo run`


### Outputs

It will write all observed values, sensical, known or not, to the broker:

```
publishing to:co2/lounge/SENSOR/co2           payload:{"publishTime":"2022-10-04T15:13:08.190989669Z","value":716.0}
publishing to:co2/lounge/SENSOR/unknown_0x57  payload:{"publishTime":"2022-10-04T15:13:09.143043186Z","value":7674.0}
publishing to:co2/lounge/SENSOR/unknown_0x56  payload:{"publishTime":"2022-10-04T15:13:09.223001692Z","value":10890.0}
publishing to:co2/lounge/SENSOR/unknown_0x41  payload:{"publishTime":"2022-10-04T15:13:10.607013457Z","value":0.0}
publishing to:co2/lounge/SENSOR/unknown_0x43  payload:{"publishTime":"2022-10-04T15:13:10.751041453Z","value":3045.0}
publishing to:co2/lounge/SENSOR/temp          payload:{"publishTime":"2022-10-04T15:13:10.831031623Z","value":22.975000000000023}
publishing to:co2/lounge/SENSOR/unknown_0x6d  payload:{"publishTime":"2022-10-04T15:13:12.975072481Z","value":3214.0}
publishing to:co2/lounge/SENSOR/unknown_0x6e  payload:{"publishTime":"2022-10-04T15:13:13.055034069Z","value":20414.0}
```

The "key" is in the topic name, e.g. `co2` (in ppm), or `temp` (in C).

Unknown values are unknown! Exciting. There is no documentation for any of these.
Hopefully they're not my GPS coordinates.

The document format is:
```json
{
  "publishTime": "2022-10-04T15:13:13.055034069Z",
  "value": 20414.0
}
```

That is, the time that the event was read from the wire, and the value itself.

Unknown values are not processed, i.e. `20414.0` means we saw `0x4f` `0xbe` on the wire.


### Operating

There is no error handling. Errors will cause the application to exit,
if the device stops talking, the script will hang forever. This should be fixed.


### Prior art

* A [more complete script](https://github.com/FauxFaux/neohub-mqtt#example-stack) explaining why you might use this. 
* https://github.com/lambdalisue/rs-co2-mini-monitor (rust cli)
* https://github.com/maddindeiss/co2-monitor (nodejs)
* https://github.com/jerr0328/co2mini (python)
* [Blog post on reverse engineering the old protocol](https://hackaday.io/project/5301-reverse-engineering-a-low-cost-usb-co-monitor).


### Contributing

Github PRs or issues, please.


### License

MIT OR Apache-2.0
