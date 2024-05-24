#![allow(dead_code)]

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

        println!("{:02X?}", buf);

        Ok(())
    }
}

//-------------------------

use tokio::time::{sleep, Duration};
use tokio_serial;
use tokio_serial::SerialPortBuilderExt;

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let port_name = "/dev/tty.usbserial-FT791P1N";
    let port_speed = 19200;

    let port = tokio_serial::new(port_name, port_speed).open_native_async()?;

    let (mut writer, _) = TL50Codec.framed(port).split();

    tokio::spawn(async move {
        loop {
            let message_on = TL50Message {
                color1: Color::Green,
                intensity1: Intensity::High,
                animation: Animation::Steady,
                speed: Speed::Standard,
                pattern: Pattern::Normal,
                color2: Color::Green,
                intensity2: Intensity::High,
                rotation: Rotation::CounterClockwise,
                audible: Audible::Off
            };

            let mut write_result = writer.send(message_on).await;
            sleep(Duration::from_secs(2)).await;
            match write_result {
                Ok(_) => (),
                Err(err) => println!("{:?}", err),
            }

            let message_off = TL50Message {
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

            write_result = writer.send(message_off).await;
            sleep(Duration::from_secs(2)).await;
            match write_result {
                Ok(_) => (),
                Err(err) => println!("{:?}", err),
            }
        }
    });

    loop {}
}
