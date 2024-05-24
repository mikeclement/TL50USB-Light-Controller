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

fn build(color1: Color, intensity1: Intensity, animation: Animation,
    speed: Speed, pattern: Pattern, color2: Color, intensity2: Intensity,
    rotation: Rotation, audible: Audible) -> [u8; 38] {
    
    // https://info.bannerengineering.com/cs/groups/public/documents/literature/218025.pdf
    let mut b: [u8; 38] = [0; 38];

    // Fixed header bytes
    b[0] = 0xf4;
    b[1] = 0x41;
    b[2] = 0xc1;
    b[3] = 0x1f;
    b[4] = 0x00;
    
    b[5] = (color1 as u8 & 0xf)
        | ((intensity1 as u8 & 0x7) << 4);

    b[6] = (animation as u8 & 0x7)
        | ((speed as u8 & 0x3) << 3)
        | ((pattern as u8 & 0x7) << 5);

    b[7] = (color2 as u8 & 0xf)
        | ((intensity2 as u8 & 0x7) << 4)
        | ((rotation as u8 & 0x1) << 7);

    // 8..34 are always zeroes

    b[35] = audible as u8;

    let byte_sum: u16 = b.iter().map(|&b| b as u16).sum();
    let ones_comp = byte_sum ^ 0xffff;
    b[36] = (ones_comp & 0x00ff) as u8;
    b[37] = ((ones_comp & 0xff00) >> 8) as u8;

    b
}

fn main() {
    let v = build(Color::Red, Intensity::High, Animation::Steady, Speed::Standard,
        Pattern::Normal, Color::Green, Intensity::High, Rotation::CounterClockwise,
        Audible::Off);
    println!("build() produced: {:02X?}", v);
}
