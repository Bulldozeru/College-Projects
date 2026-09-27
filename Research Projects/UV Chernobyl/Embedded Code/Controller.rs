#![no_std]
#![no_main] 

use defmt::info;
use embassy_executor::Spawner;
// use embassy_stm32::exti::ExtiInput;
use embassy_stm32::{Peri, bind_interrupts, exti::{self, ExtiInput}, gpio::{AnyPin, Level, Output, Speed}, interrupt, mode::Async, peripherals::{self, PE15}};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::gpio::{Input, Pull};

// Exti15 PE15 30 Min Button
// Exti14 PF14 60 Min Button
// Exti13 PE13 STOP Button
// PB0 PWM Tim3Ch3 D33


bind_interrupts!(struct Irqs {

    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Configure PE15 with EXTI channel 15
    let button = exti::ExtiInput::new(p.PE15, p.EXTI15, Pull::None, Irqs);
    // let mut led = Output::new(p.PE12, Level::Low, Speed::Medium);
    // spawner.spawn(buttonTask(button).unwrap());
    spawner.spawn(buttonTask(button).unwrap()); //(buttonTask(p.PE15));
    
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }

}


#[embassy_executor::task]
async fn buttonTask(mut button: ExtiInput<'static, Async>) {

        
        button.wait_for_falling_edge().await; 
        info!("ciao");
        button.wait_for_rising_edge().await;   
        info!("ola");

}    
