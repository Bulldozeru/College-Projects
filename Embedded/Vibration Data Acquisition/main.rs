#![no_main]
#![no_std]

// EEPROM ADDR RANGE 0x0000 0x7FFF

use defmt::{error, info, warn};

use defmt_rtt as _;
use eeprom24x::Eeprom24x;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Pull;
use embassy_stm32::i2c::I2c;
use embassy_stm32::{bind_interrupts, exti, i2c, peripherals};
use embassy_time::{Duration, Timer};
use panic_probe as _;

use defmt_rtt as _;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    spi::{Config, Mode, Phase, Polarity, Spi},
    time::Hertz,
};

// use panic_probe as _;
const MPU_WHO_AM_I: u8 = 0x75;
const MPU_WHO_AM_I_VALUE: u8 = 0x68; // Expected value for MPU-6000
const MPU_USER_CTRL: u8 = 0x6A;
const MPU_PWR_MGMT_1: u8 = 0x6B;
const MPU_ACCEL_XOUT_H: u8 = 0x3B;
const MPU_READ: u8 = 0x80;

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());

    let buttonGrabber = exti::ExtiInput::new(peripherals.PA0, peripherals.EXTI0, Pull::None); // Sw 1
    let buttonReader = exti::ExtiInput::new(peripherals.PA1, peripherals.EXTI1, Pull::None); // Sw 2
    spawner.spawn(dataGrabber(buttonGrabber)).unwrap();
    spawner.spawn(dataReader(buttonReader)).unwrap();

    loop {
        info!("Standing by...");
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn dataReader(mut button: ExtiInput<'static>) {
    button.wait_for_falling_edge().await;
    let sensorData = [0u8; 4096];

    eeReader(sensorData, 0x0000);
    info!("EEPROM READER -> {}", &sensorData);
}
#[embassy_executor::task]
async fn eeReader(mut sensorData: [u8; 4096], readAddr: u32) {
    let peripherals = embassy_stm32::init(Default::default());
    let sda = peripherals.PB7;
    let scl = peripherals.PB6;

    let i2c = I2c::new(
        peripherals.I2C1,
        scl,
        sda,
        Irqs,
        peripherals.GPDMA1_CH0,
        peripherals.GPDMA1_CH1,
        Default::default(),
    );

    let mut eeprom = Eeprom24x::new_24x256(i2c, eeprom24x::SlaveAddr::default());

    // let mut buffer = [0u8; readingSpace];
    let data = eeprom.read_data(readAddr, &mut sensorData);
    match data {
        Ok(_) => info!("{:x}", &sensorData),
        Err(_) => info!(""),
    }
}

#[embassy_executor::task]
async fn eeWriter(writeAddr: u32, sensorData: [u8; 4096]) {
    let peripherals = embassy_stm32::init(Default::default());
    let sda = peripherals.PB7;
    let scl = peripherals.PB6;

    let i2c = I2c::new(
        peripherals.I2C1,
        scl,
        sda,
        Irqs,
        peripherals.GPDMA1_CH0,
        peripherals.GPDMA1_CH1,
        Default::default(),
    );

    let mut eeprom = Eeprom24x::new_24x256(i2c, eeprom24x::SlaveAddr::default());
    eeprom
        .write_page(writeAddr, &sensorData)
        .expect("Write Fault");
    Timer::after_millis(5).await;
}

#[embassy_executor::task]
async fn dataGrabber(mut button: ExtiInput<'static>) {
    let peripherals = embassy_stm32::init(Default::default());
    info!("Device started");
    button.wait_for_falling_edge().await;
    // Create the SPI bus configuration
    let mut config = Config::default();
    config.frequency = Hertz(1_000_000);

    // MPU6000 requires SPI Mode 3 (CPOL=1, CPHA=1)
    config.mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };

    let mut spi = Spi::new(
        peripherals.SPI1,
        peripherals.PA5, // SCK
        peripherals.PA7, // MOSI
        peripherals.PA6, // MISO
        peripherals.GPDMA1_CH0,
        peripherals.GPDMA1_CH1,
        config,
    );

    // We use the PA8 pin as CS
    let mut cs = Output::new(peripherals.PA8, Level::High, Speed::VeryHigh);

    // Disable I2C interface
    let disable_i2c_buf: [u8; 2] = [MPU_USER_CTRL, 0x10];
    cs.set_low();
    if let Err(e) = spi.write(&disable_i2c_buf).await {
        error!("Failed to disable I2C interface: {:?}", e);
    }
    cs.set_high();

    // Wake up MPU6000, and set the clock source to be synced with gyroscope, as
    // recommended by the manual.
    let wake_buf: [u8; 2] = [MPU_PWR_MGMT_1, 0x01];
    cs.set_low();
    if let Err(e) = spi.write(&wake_buf).await {
        error!("Failed to wake up MPU-6000: {:?}", e);
    }
    cs.set_high();

    // Yield execution to the executor for 100ms to allow the sensor to stabilize
    Timer::after(Duration::from_millis(100)).await;

    // Verify WHO_AM_I
    let command = [MPU_WHO_AM_I | MPU_READ, 0x00];
    let mut rx = [0u8; 2];

    cs.set_low();
    let res = spi.transfer(&mut rx, &command).await;
    cs.set_high();

    match res {
        Err(e) => {
            error!("SPI transfer failed during WHO_AM_I check: {:?}", e);
        }
        Ok(_) => {
            let who_am_i_value = rx[1];
            info!("Sensor's WHO_AM_I register is 0x{:02X}", who_am_i_value);

            if who_am_i_value == MPU_WHO_AM_I_VALUE {
                info!("Sensor is MPU-6000");
            } else {
                warn!("This is not an MPU-6000 sensor. Expected 0x68.");
            }
        }
    }

    let accel_scale: f32 = 9.81 / 16384.0;
    // let gyro_scale: f32 = 1.0 / 131.0;


    let mut aqData = [0u8; 4096];
    let mut n = 0;
    loop {
        let mut tx_buf = [0u8; 15];
        let mut rx_buf = [0u8; 15];
        tx_buf[0] = MPU_ACCEL_XOUT_H | MPU_READ;
        n = n + 4;
        cs.set_low();
        let transfer_result = spi.transfer(&mut rx_buf, &tx_buf).await;
        cs.set_high();
        match transfer_result {
            Ok(_) => {
                // Parse raw 16-bit signed values (big-endian) skipping the first dummy byte
                let ax_raw = i16::from_be_bytes([rx_buf[1], rx_buf[2]]) as f32;
                let ay_raw = i16::from_be_bytes([rx_buf[3], rx_buf[4]]) as f32;
                let az_raw = i16::from_be_bytes([rx_buf[5], rx_buf[6]]) as f32;

                // let gx_raw = i16::from_be_bytes([rx_buf[9], rx_buf[10]]) as f32;
                // let gy_raw = i16::from_be_bytes([rx_buf[11], rx_buf[12]]) as f32;
                // let gz_raw = i16::from_be_bytes([rx_buf[13], rx_buf[14]]) as f32;

                // Apply scaling
                let ax = ax_raw * accel_scale;
                let ay = ay_raw * accel_scale;
                let az = az_raw * accel_scale;
                // let gx = (gx_raw * gyro_scale * 100000.0) as i32;
                // let gy = (gy_raw * gyro_scale * 100000.0) as i32;
                // let gz = (gz_raw * gyro_scale * 100000.0) as i32;

                aqData[n..n + 4].clone_from_slice(&ax.to_be_bytes());
                aqData[n + 4..n + 8].clone_from_slice(&ay.to_be_bytes());
                aqData[n + 8..n + 12].clone_from_slice(&az.to_be_bytes());

                if n == 4084 {
                    eeWriter(0x0000, aqData);
                }
            }

            Err(e) => {
                info!("Error {}", e)
            }
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
