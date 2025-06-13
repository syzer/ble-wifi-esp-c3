use anyhow::Result;
use anyhow::anyhow;
use esp_idf_svc::{
    hal::{
        gpio::{InterruptType, PinDriver, Pull},
        peripherals::Peripherals,
        task::notification::Notification,
    },
    sys::esp_random,
};

mod rgb_driver;
use rgb_driver::{RGB8, WS2812RMT};
use std::num::NonZeroU32;
use getrandom::getrandom;

fn main() -> Result<()> {
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
    let mut col_rgb: [u8; 3] = [0u8; 3];
    if let Err(e) = getrandom(&mut col_rgb) {
        return Err(anyhow!("RNG failed: {:?}", e));
    }

    let brightness = 10; // Adjust brightness level (1-255)
    let colour = RGB8::new(
        (col_rgb[0] as u32 * brightness / 255) as u8,
        (col_rgb[1] as u32 * brightness / 255) as u8,
        (col_rgb[2] as u32 * brightness / 255) as u8,
    );

    // Push colour to the LED
    if let Err(e) = led.set_pixel(colour) {
        return Err(anyhow!("LED write error: {:?}", e));
    }

    Ok(())
}
