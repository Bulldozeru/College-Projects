#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::{bind_interrupts, interrupt};
use embassy_sync::pubsub::{WaitResult};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::Timer;
use panic_probe as _;
use embassy_sync::signal::Signal;
use embassy_futures::select::{select, Either};


static START_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static STOP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static channel: PubSubChannel<CriticalSectionRawMutex, i32, 4, 4, 4> = PubSubChannel::<CriticalSectionRawMutex, i32, 4, 4, 4>::new();

bind_interrupts!(
    pub struct Irqs{
        EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

static PUB: PubSubChannel<CriticalSectionRawMutex, &'static str, 4, 2, 8> = PubSubChannel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut button = ExtiInput::new(p.PE15, p.EXTI15, Pull::Down, Irqs);
    let mut rst = ExtiInput::new(p.PB10, p.EXTI10, Pull::Down, Irqs);
    
    let mut counter = 0;

    let mut led = Output::new(p.PB0, Level::Low, Speed::Low);
    let mut program: &'static mut i32;
    
    spawner.spawn(program30(button).unwrap());
    spawner.spawn(killSwitch(rst).unwrap());
    spawner.spawn(executor(led).unwrap());
    info!("Press the USER button...");



}

#[embassy_executor::task]
async fn program30(mut button: ExtiInput<'static, Async>) {

    let presa = channel.publisher().unwrap();

    loop {

        button.wait_for_rising_edge().await;
        info!("Pressed!");
        START_SIGNAL.signal(());
        presa.publish(30).await;
    }
}

async fn program60(mut button: ExtiInput<'static, Async>) {

    let presa = channel.publisher().unwrap();

    loop {

        button.wait_for_rising_edge().await;
        info!("Pressed!");
        presa.publish(60).await;
    }
}

#[embassy_executor::task]
async fn killSwitch(mut button: ExtiInput<'static, Async>) {

    loop {

        button.wait_for_rising_edge().await;

        STOP_SIGNAL.signal(());
        info!("kira");
    }
}

#[embassy_executor::task]
async fn executor(mut led: Output<'static> ) { 


    loop {
        let mut sapacaraiber = channel.subscriber().unwrap();
        START_SIGNAL.wait().await;
        let message: WaitResult<i32> = sapacaraiber.next_message().await;
        
        let mut counter: i32 = 0;
        let mut runtime = 0;
        match message {

            WaitResult::Message(30) => runtime = 210000,
            WaitResult::Message(60) => runtime = 420000,
            _ => runtime = 0
        }

        info!("Oscillations");
        let loop_future = async {
            while counter <  runtime {
                
                Timer::after_millis(2).await;
                led.set_high();
                Timer::after_millis(2).await;
                led.set_low();
                counter += 1;
            }
        };

        let killSwitch = STOP_SIGNAL.wait();

        match select(loop_future, killSwitch).await {
            Either::First(_) => {}
            Either::Second(_) => { led.set_low(); }
        }
    }

}

