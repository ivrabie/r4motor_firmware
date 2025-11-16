#![no_std]
#![no_main]
#![warn(unused_variables)]
#![warn(unused_imports)]
#![warn(dead_code)]


use defmt::*;
use embassy_executor::Spawner;

use embassy_stm32::{
    peripherals,
    gpio::{Level, Output, OutputType, Speed}, spi::{Config as SpiConfig ,Spi}, time::khz, timer::{
        qei::{Qei, Config as QeiConfig},
        simple_pwm::{PwmPin, SimplePwm},
        GeneralInstance4Channel,
    }
};

use embassy_time::{Ticker, Duration};
use {defmt_rtt as _, panic_probe as _};

mod motor;
use motor::*;

const MOTOR_COUNT_PER_REV: u16 = 330; 



#[embassy_executor::task]
async fn motor_control_task(
    mut motor1: Motor<'static, peripherals::TIM1, peripherals::TIM2>,
    mut motor2: Motor<'static, peripherals::TIM1, peripherals::TIM3>,
    mut motor3: Motor<'static, peripherals::TIM1, peripherals::TIM4>,
    mut motor4: Motor<'static, peripherals::TIM1, peripherals::TIM5>
) {

    let mut ticker = Ticker::every(Duration::from_millis(10));
    motor1.init();
    motor2.init();
    motor3.init();
    motor4.init();
    loop {
        ticker.next().await;
    }

}

#[embassy_executor::task]
async fn spi_task(mut spi: Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Slave>) {
    // SPI task implementation
    let mut count: u8 = 0;
    let mut spi_write_buffer = [0u8; 5];
    loop {
        let mut spi_read_buffer = [0u8; 1];
        
        for i in spi_write_buffer.iter_mut() {
            *i = count;
        }
        count = count.wrapping_add(1);

        spi.read(&mut spi_read_buffer).await.unwrap();
        info!("SPI Received: {:x}", spi_read_buffer[0]);
        spi.write(&spi_write_buffer).await.unwrap();
        info!("Data sent {:?}", spi_write_buffer);
    }
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // DC driver 1 configuration
    let mut m1_ins_a = Output::new(p.PB2, Level::Low, Speed::Low);
    let mut m1_ins_b = Output::new(p.PB10, Level::Low, Speed::Low);
    let mut m2_ins_a = Output::new(p.PB12, Level::Low, Speed::Low);
    let mut m2_ins_b = Output::new(p.PB13, Level::Low, Speed::Low);

    // DC driver 2 configuration
    let mut m3_ins_a = Output::new(p.PC14, Level::Low, Speed::Low);
    let mut m3_ins_b = Output::new(p.PC15, Level::Low, Speed::Low);
    let mut m4_ins_a = Output::new(p.PB0, Level::Low, Speed::Low);
    let mut m4_ins_b = Output::new(p.PB1, Level::Low, Speed::Low);

    // Driver control state
    let mut vcc_gpio = Output::new(p.PC13, Level::High, Speed::Low);
    let mut standby_gpio = Output::new(p.PA12, Level::Low, Speed::Low);

    // PWM configuration
    let ch1_pin = PwmPin::new(p.PA8, OutputType::PushPull);
    let ch2_pin = PwmPin::new(p.PA9, OutputType::PushPull);
    let ch3_pin = PwmPin::new(p.PA10, OutputType::PushPull);
    let ch4_pin = PwmPin::new(p.PA11, OutputType::PushPull);
    let pwm = SimplePwm::new(
        p.TIM1,
        Some(ch1_pin),
        Some(ch2_pin),
        Some(ch3_pin),
        Some(ch4_pin),
        khz(50),
        Default::default(),
    );
    // Split PWM channels
    let simple_pwm_channels = pwm.split();
    let mut mot1_pwm = simple_pwm_channels.ch1;
    let mut mot2_pwm = simple_pwm_channels.ch2;
    let mut mot3_pwm = simple_pwm_channels.ch3;
    let mut mot4_pwm = simple_pwm_channels.ch4;

    mot1_pwm.enable();
    mot2_pwm.enable();
    mot3_pwm.enable();
    mot4_pwm.enable();


    // Quadrature configuration

    let qei_config = QeiConfig::default();
    let qei_motor1 = Qei::new(p.TIM2, p.PA15, p.PB3, qei_config);
    let qei_motor2 = Qei::new(p.TIM3, p.PA6, p.PA7, qei_config);
    let qei_motor3 = Qei::new(p.TIM4, p.PB6, p.PB7, qei_config);
    let qei_motor4 = Qei::new(p.TIM5, p.PA0, p.PA1, qei_config);


    // SPI configuration (Slave mode)
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = khz(5000);
    let mut spi = Spi ::new_slave(
        p.SPI1,
        p.PA5,  // SCK
        p.PB5,  // MOSI
        p.PB4,  // MISO
        p.PA4,  // CS
        p.DMA2_CH3,
        p.DMA2_CH2,
        spi_config,
    );

    let motor1 = Motor::new(
        mot1_pwm,
        m1_ins_a,
        m1_ins_b,
        qei_motor1,
        MOTOR_COUNT_PER_REV
    );

    let motor2 = Motor::new(
        mot2_pwm,
        m2_ins_a,
        m2_ins_b,
        qei_motor2,
        MOTOR_COUNT_PER_REV
    );

    let motor3 = Motor::new(
        mot3_pwm,
        m3_ins_a,
        m3_ins_b,
        qei_motor3,
        MOTOR_COUNT_PER_REV
    );

    let motor4 = Motor::new(
        mot4_pwm,
        m4_ins_a,
        m4_ins_b,
        qei_motor4,
        MOTOR_COUNT_PER_REV
    );

    spawner.spawn(motor_control_task(
        motor1,
        motor2,
        motor3,
        motor4
    ).unwrap());
    spawner.spawn(spi_task(spi).unwrap());

    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        ticker.next().await;
    }
}
    