#![allow(dead_code)]

// Definition of TL50 message, and a codec to do the encoding work

enum Color {
    Green = 0x00,
    Red = 0x01,
    Orange = 0x02,
    Amber = 0x03,
    Yellow = 0x04,
    LimeGreen = 0x05,
    SpringGreen = 0x06,
    Cyan = 0x07,
    SkyBlue = 0x08,
    Blue = 0x09,
    Violet = 0x0a,
    Magenta = 0x0b,
    Rose = 0x0c,
    White = 0x0d,
}

enum Intensity {
    High = 0x00,
    Low = 0x01,
    Medium = 0x02,
    Off = 0x03,
}

enum Animation {
    Off = 0x00,
    Steady = 0x01,
    Flash = 0x02,
    TwoColorFlash = 0x03,
    HalfHalf = 0x04,
    HalfHalfRotate = 0x05,
    Chase = 0x06,
    IntensitySweep = 0x07,
}

enum Speed {
    Standard = 0x00,
    Fast = 0x01,
    Slow = 0x02,
}

enum Pattern {
    Normal = 0x00,
    Strobe = 0x01,
    ThreePulse = 0x02,
    Sos = 0x03,
    Random = 0x04,
}

enum Rotation {
    CounterClockwise = 0x00,
    Clockwise = 0x01,
}

enum Audible {
    Off = 0x00,
    Steady = 0x01,
    Pulsed = 0x02,
    Sos = 0x03,
}

struct TL50Message {
    color1: Color,
    intensity1: Intensity,
    animation: Animation,
    speed: Speed,
    pattern: Pattern,
    color2: Color,
    intensity2: Intensity,
    rotation: Rotation,
    audible: Audible,
}

use std::io;
use bytes::{BufMut, BytesMut};
use futures::{stream::StreamExt, SinkExt};
use tokio_util::codec::{Decoder, Encoder};

struct TL50Codec;

impl Decoder for TL50Codec {
    type Item = TL50Message;
    type Error = io::Error;

    fn decode(&mut self, _: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(None)
    }
}

impl Encoder<TL50Message> for TL50Codec {
    type Error = io::Error;

    fn encode(&mut self, item: TL50Message, buf: &mut BytesMut) -> Result<(), io::Error> {
        // https://info.bannerengineering.com/cs/groups/public/documents/literature/218025.pdf
        buf.clear();
        buf.reserve(38);

        // Fixed header bytes
        buf.put_u8(0xf4);
        buf.put_u8(0x41);
        buf.put_u8(0xc1);
        buf.put_u8(0x1f);
        buf.put_u8(0x00);
    
        buf.put_u8((item.color1 as u8 & 0xf)
            | ((item.intensity1 as u8 & 0x7) << 4));

        buf.put_u8((item.animation as u8 & 0x7)
            | ((item.speed as u8 & 0x3) << 3)
            | ((item.pattern as u8 & 0x7) << 5));

        buf.put_u8((item.color2 as u8 & 0xf)
            | ((item.intensity2 as u8 & 0x7) << 4)
            | ((item.rotation as u8 & 0x1) << 7));

        // 8..34 are always zeroes
        for _ in 8..35 {
            buf.put_u8(0x00);
        }

        buf.put_u8(item.audible as u8);

        let byte_sum: u16 = buf.iter().map(|&b| b as u16).sum();
        let ones_comp = byte_sum ^ 0xffff;
        buf.put_u8((ones_comp & 0x00ff) as u8);
        buf.put_u8(((ones_comp & 0xff00) >> 8) as u8);

        println!("Encoded: {:02X?}", buf);

        Ok(())
    }
}

// Serial connection management

use tokio_serial;
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;

struct TL50Driver {
    port_path: String,
    port_speed: u32,
    port: Option<SerialStream>,
}

impl TL50Driver {
    fn new(path: String, speed: u32) -> Self {
        Self {
            port_path: path,
            port_speed: speed,
            port: None,
        }
    }

    async fn send_command(&mut self, command: TL50Message) {
        if let None = self.port {
            let path: &str = &*self.port_path;
            let port = tokio_serial::new(path, self.port_speed).open_native_async();
            match port {
                Ok(p) => {
                    println!("Successfully opened {:?}", self.port_path);
                    self.port = Some(p);
                },
                Err(e) => {
                    println!("Error opening {:?}: {:?}", self.port_path, e);
                    return;
                }
            }
        }

        // If we've gotten to here, there should be a working serial port
        let mut write_succeeded: bool = true;
        if let Some(port) = &mut self.port {
            let (mut writer, _) = TL50Codec.framed(port).split();
            if let Err(err) = writer.send(command).await {
                println!("Error writing: {:?}", err);
                write_succeeded = false;
            }
        }

        if !write_succeeded {
            self.port = None;
        }
    }

    async fn off(&mut self) {
        let message = TL50Message {
            color1: Color::Green,
            intensity1: Intensity::Off,
            animation: Animation::Steady,
            speed: Speed::Standard,
            pattern: Pattern::Normal,
            color2: Color::Green,
            intensity2: Intensity::High,
            rotation: Rotation::CounterClockwise,
            audible: Audible::Off
        };

        self.send_command(message).await;
    }

    async fn steady(&mut self, color: Color, intensity: Intensity) {
        let message = TL50Message {
            color1: color,
            intensity1: intensity,
            animation: Animation::Steady,
            speed: Speed::Standard,
            pattern: Pattern::Normal,
            color2: Color::Green,
            intensity2: Intensity::High,
            rotation: Rotation::CounterClockwise,
            audible: Audible::Off
        };

        self.send_command(message).await;
    }
}

// Main

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let port_name: String = "/dev/tty.usbserial-FT791P1N".to_owned();
    let port_speed = 19200;
    let delay = 1;

    let mut tl50 = TL50Driver::new(port_name, port_speed);

    tokio::spawn(async move {
        loop {
            tl50.steady(Color::Green, Intensity::High).await;
            sleep(Duration::from_secs(delay)).await;

            tl50.off().await;
            sleep(Duration::from_secs(delay)).await;
        }
    });

    loop {}
}
