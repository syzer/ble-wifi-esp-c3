#![no_main]
#![feature(impl_trait_in_assoc_type)]

use esp_println::println;  
use anyhow::{Result, anyhow};

use esp_idf_svc::{
    hal::{
        gpio::{InterruptType, PinDriver, Pull},
        peripherals::Peripherals,
        task::notification::Notification,
    },
    // sys::esp_random,
};

mod rgb_driver;
use rgb_driver::{RGB8, WS2812RMT};
// use std::num::NonZeroU32;
// use num_traits::NonZeroU32;
use core::num::NonZeroU32;

use getrandom::getrandom;
use embassy_executor::Spawner;

// use esp_idf_svc::hal::rmt::WS2812RMT;
// use esp_hal::{clock::ClockControl, peripherals::Peripherals, system::SystemControl};


// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     println!("No panic 🐕, just a loop!");
//     loop {};
// }

// #[embassy_executor::main]
// #[esp_hal_embassy::main]


#[esp_hal::main]
async fn main(_spawner: Spawner) -> anyhow::Result<()> {

// async fn main(_spawner: embassy_executor::Spawner) -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let mut led = WS2812RMT::new(peripherals.pins.gpio2, peripherals.rmt.channel0)?;

    // Configures the button
    let mut button = PinDriver::input(peripherals.pins.gpio9)?;
    button.set_pull(Pull::Up)?;
    button.set_interrupt_type(InterruptType::PosEdge)?;

    // Configures the notification
    let notification = Notification::new();
    let notifier = notification.notifier();

    // Subscribe and create the callback
    // Safety: make sure the `Notification` object is not dropped while the subscription is active
    unsafe {
        button.subscribe(move || {
            notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
        })?;
    }

    loop {
        // Enable interrupt and wait for new notificaton
        button.enable_interrupt()?;
        notification.wait(esp_idf_svc::hal::delay::BLOCK);
        println!("Button pressed!");
        // Generates random rgb values and sets them in the led.
        let _ = random_light(&mut led);
    }
}

#[allow(unused)]
pub fn random_light(led: &mut WS2812RMT) -> Result<()> {
    // Fill a 3-byte buffer with random data
    let mut buf = [0u8; 3];
    if let Err(e) = getrandom(&mut buf) {
        return Err(anyhow!("RNG failed: {:?}", e));
    }

    let brightness = 8; // Adjust brightness level (1-255)

     // Reduce brightness by dividing by 8 (or adjust divisor for different brightness levels)
    let colour = RGB8::new(
        (buf[0] as u32 * brightness / 255) as u8,
        (buf[1] as u32 * brightness / 255) as u8,
        (buf[2] as u32 * brightness / 255) as u8,
    );


    // Push colour to the LED
    if let Err(e) = led.set_pixel(colour) {
        return Err(anyhow!("LED write error: {:?}", e));
    }

    Ok(())
}
