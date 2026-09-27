#![no_main]
#![no_std]

use defmt::info;
use embassy_executor::Spawner;
// use embassy_stm32::exti::ExtiInput;

use embassy_stm32::{Peri, bind_interrupts, peripherals};
use exti::ExtiInput;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};
    use embassy_stm32::gpio::{Input, Pull};

static SIG: Signal<CriticalSectionRawMutex, u32> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(Default::default());



    // let pin = ::new(p.PH1, Pull::Up);
    let mut button = exti::ExtiInput::new(p.PC0, p.EXTI0, Pull::None);
    spawner.spawn(buttonTask(button)).unwrap();
    spawner.spawn(convertor()).unwrap();
    loop {
        Timer::after_millis(800).await;
    }
    // let mut button = exti::ExtiInput::new(peripherals.PA8, peripherals.EXTI8, Pull::None);
}

#[embassy_executor::task]
async fn convertor() {
    loop {
        let x = SIG.wait().await; // Wait until signaled

        if x < 60 {
            info!(".");
        } else if x < 80 {
            info!("-");
        }
    }
}

#[embassy_executor::task]
async fn buttonTask(mut button: ExtiInput<'static>) {
    loop {
        button.wait_for_falling_edge().await;
        let a = Instant::now();
        button.wait_for_rising_edge().await;
        let b = Instant::now();
        let c = b - a;
        let aplha = c.as_millis();
        match u32::try_from(aplha) {
            Ok(_alpha) => (),
            Err(_) => info!("Value exceeded u32::MAX!"),
        }

        SIG.signal(aplha.try_into().unwrap_or(u32::MAX));
    }
}

/*use embassy_sync::signal::Signal;

static SIG: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn waiter() {
    SIG.wait().await; // Wait until signaled
    defmt::info!("Signal received!");
}

#[embassy_executor::task]
async fn trigger() {
    SIG.signal(()); // Notify the waiting task
} */
