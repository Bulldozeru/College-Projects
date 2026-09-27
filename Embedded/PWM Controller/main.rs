#![no_main]
#![no_std]

use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::{Duration, Timer};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let servo_pin = PwmPin::new(p.PA0, OutputType::PushPull);

    // 50 Hz PWM (20 ms period)
    let mut pwm = SimplePwm::new(
        p.TIM2,
        Some(servo_pin),
        None,
        None,
        None,
        hz(50),
        Default::default(),
    );

    // Enable the PWM channel
    let mut ch1 = pwm.ch1();
    ch1.enable();

    const MIN_PERIOD_US: u32 = 500;
    const MAX_PERIOD_US: u32 = 2500;
    const PERIOD_US: u32 = 20000;

    let min_value = (MIN_PERIOD_US * 1000) / PERIOD_US;
    let max_value = (MAX_PERIOD_US * 1000) / PERIOD_US;

    // Main loop to move the servo back and forth
    loop {
        // At 50 Hz, T = 20 ms. A 2.5 ms pulse means 2.5/20 = 12.5% duty cycle.
        ch1.set_duty_cycle_fraction(max_value as u16, 1000); // sets the servo to set servo to ~180°.
        // Wait 1 second before moving back
        Timer::after(Duration::from_secs(1)).await;
        // A 0.5 ms pulse means 0.5/20 = 2.5% duty cycle.
        ch1.set_duty_cycle_fraction(min_value as u16, 1000); // sets the servo to set servo to ~0°.
        // Wait 1 second before repeating
        Timer::after(Duration::from_secs(1)).await;
    }
}

// Set duty cycle to 50%
