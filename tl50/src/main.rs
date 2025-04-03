// Simple demo of manually blinking green/off, to show interface, timing,
// and recovery of USB FTDI disconnect/reconnect.

use std::env;
use tokio::time::{sleep, Duration};

use tl50::{TL50ActorHandle, Color, Intensity};

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    // Supply serial device path (e.g., /dev/ttyUSB0)
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Must specify device path!");
    }

    let port_name: String = args[1].clone();
    let port_speed = 19200;

    // Set rate that tl50 checks for connection status and asserts state
    let tl50_loop_time = Duration::from_secs(1);

    // Handle to the TL50 actor, passes events to the actor
    let mut handle = TL50ActorHandle::new(port_name, port_speed, tl50_loop_time);

    // Set time between command changes (should be > tl50_loop_time)
    let main_loop_time = Duration::from_secs(5);
    loop {
        println!("Green light on!");
        handle.steady(Color::Green, Intensity::High).await;
        sleep(main_loop_time).await;

        println!("Light off!");
        handle.off().await;
        sleep(main_loop_time).await;
    }
}
