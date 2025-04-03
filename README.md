# TL50USB-Light-Controller
 
Basic Python and Rust code to control a Banner engineering TL50 USB enabled
tower indicator.

Original Python code by Liam O'Brien, with changes by Mike Clement. All Rust
code by Mike Clement.

Product information:
* https://www.youtube.com/watch?v=crRyCEbCWO8&ab_channel=BannerEngineering
* https://www.bannerengineering.com/th/en/products/lighting-and-indicators/tower-lights/50mm-tower-lights-tl50-series.html

# Python code

`tl50-controller.py` is a very simple implementation of messages to control
the TL50 LED.
It is intentionally written in Python 2 as a demo for a very old software
stack, but should be trivially updatable to Python 3.

# Rust code

The Rust code is contained in the `tl50` directory. All the interesting
code is in `lib.rs`.
Everything is implemented within the Tokio framework, so it can be easily
integrated with async code.
Serialization is done using `Tokio::codec` with an encoder defined for each
message type.
An actor pattern is used to watch for USB disconnect and reconnect events and
to (re)assert state as necessary, using an "Enable Advanced Segment Mode"
message as a sort of no-op that tests the liveliness of the USB connection.

Currently, only the "off" and "steady" commands are implemented. See the
product docs or the Python code for the other commands available.

## Usage

```
cargo run -- <device-path>
```
e.g., 
```
cargo run -- /dev/tty.usbserial-FT791P1N
```