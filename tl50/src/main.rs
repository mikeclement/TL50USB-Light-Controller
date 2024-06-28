#![allow(dead_code)]

// Enums for the bitfields in the TL50 Advanced Segment Indication Command,
// and a struct with all the needed fields.

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

struct AdvSegInd {
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

// Enum of commands we can send to the TL50, and encoder functions for each

use bytes::{BufMut, BytesMut};

enum TL50CommandEnum {
    EnAdvSegMode,
    ChAdvSegInd(AdvSegInd),
}

fn encode_en_adv_seg_mode(
    buf: &mut BytesMut) -> Result<(), io::Error> {
    // Note we do not accrue messages, so that we can handle serial errors
    buf.clear();
    buf.reserve(8);

    buf.put_u8(0xF4);
    buf.put_u8(0x41);
    buf.put_u8(0xC7);
    buf.put_u8(0x01);
    buf.put_u8(0x00);
    buf.put_u8(0x01);
    buf.put_u8(0x01);
    buf.put_u8(0xFE);

    println!("EnAdvSegMode: {:02X?}", buf);

    Ok(())
}

fn encode_ch_adv_seg_ind(
    item: AdvSegInd,
    buf: &mut BytesMut) -> Result<(), io::Error> {
    // Note we do not accrue messages, so that we can handle serial errors
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

    // Checksum
    let byte_sum: u16 = buf.iter().map(|&b| b as u16).sum();
    let ones_comp = byte_sum ^ 0xffff;
    buf.put_u8((ones_comp & 0x00ff) as u8);
    buf.put_u8(((ones_comp & 0xff00) >> 8) as u8);

    println!("ChAdvSegInd: {:02X?}", buf);

    Ok(())
}

// Serialization is implemented using tokio::codec, where a struct is
// encoded into bytes during the send operation. For now, only the encode
// side is implemented and only for select messages; however, the TL50
// does also emit response messages.
//
// The documentation for the serial protocol can be found here:
// https://info.bannerengineering.com/cs/groups/public/documents/literature/218025.pdf

use std::io;
use futures::{stream::StreamExt, SinkExt};
use tokio_util::codec::{Decoder, Encoder};

struct TL50Codec;

impl Decoder for TL50Codec {
    type Item = ();
    type Error = io::Error;

    fn decode(&mut self, _: &mut BytesMut)
        -> Result<Option<Self::Item>, Self::Error> {
        Ok(None)
    }
}

impl Encoder<TL50CommandEnum> for TL50Codec {
    type Error = io::Error;

    fn encode(&mut self,
        item: TL50CommandEnum,
        buf: &mut BytesMut) -> Result<(), io::Error> {
        match item {
            TL50CommandEnum::EnAdvSegMode => {
                return encode_en_adv_seg_mode(buf);
            }
            TL50CommandEnum::ChAdvSegInd(cmd) => {
                return encode_ch_adv_seg_ind(cmd, buf);
            }
        }
    }
}

// Since the TL50 is a USB device that may be unplugged and replugged at any
// time, we want to be robust and reconnect as needed. TL50Driver attempts to
// detect when an error occurs and to reconnect to the serial device.
//
// Note there are other ways to achieve this effect, which might be better:
//  1. Monitor the underlying system for device events, or for the special
//     file to disappear and reappear
//  2. Implement the decoder above and actually handle incoming messages
//     from the TL50, resetting if there's an error or timeout

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

    // If port isn't set up, try to open and report if there's an open port
    fn check_port_and_open(&mut self) -> bool {
        if let Some(_) = self.port {
            return true;
        }

        let path: &str = &*self.port_path;
        let port = tokio_serial::new(path, self.port_speed)
            .open_native_async();
        match port {
            Ok(p) => {
                println!("Successfully opened {:?}", self.port_path);
                self.port = Some(p);
                return true;
            },
            Err(e) => {
                println!("Error opening {:?}: {:?}", self.port_path, e);
                return false;
            }
        }
    }

    // Returns true if port is open and write succeeds
    async fn send_command(&mut self, command: TL50CommandEnum) -> bool {
        // If necessary, attempt to set up a serial connection
        if !self.check_port_and_open() {
            return false;
        }

        // If we've gotten to here, there should be a working serial port
        // We'll still catch errors while sending, since there could be a
        // race with the USB being unplugged
        let mut write_succeeded: bool = true;
        if let Some(port) = &mut self.port {
            // This line is the magic that invokes the encoder above
            let (mut writer, _) = TL50Codec.framed(port).split();
            if let Err(err) = writer.send(command).await {
                println!("Error writing: {:?}", err);
                write_succeeded = false;
            }
        }

        // If the write failed, try reconnecting to the serial device next time
        if !write_succeeded {
            self.port = None;
        }

        write_succeeded
    }

    // Enable Advanced Segment Mode
    async fn adv_seg_mode(&mut self) -> bool {
        self.send_command(TL50CommandEnum::EnAdvSegMode).await
    }

    // Turn light off
    async fn off(&mut self) -> bool {
        let cmd = AdvSegInd {
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

        self.send_command(TL50CommandEnum::ChAdvSegInd(cmd)).await
    }

    // Set light to a steady color and intensity
    async fn steady(&mut self, color: Color, intensity: Intensity) -> bool {
        let cmd = AdvSegInd {
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

        self.send_command(TL50CommandEnum::ChAdvSegInd(cmd)).await
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
