#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;

use embassy_stm32::{
    gpio::{Level, Output, OutputType, Speed}, spi::{Config as SpiConfig ,Spi}, time::khz, timer::{
        qei::{Qei, Config as QeiConfig},
        simple_pwm::{PwmPin, SimplePwm},
    }
};
use embassy_time::{Ticker, Duration};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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


    let mut ticker = Ticker::every(Duration::from_millis(10)); 
    loop {
        // info!("high");
        let mut spi_read_buffer = [0u8; 1];
        let spi_write_buffer = [1u8, 2u8, 3u8, 4u8, 5u8];
        m1_ins_a.set_high();
        m1_ins_b.set_high();
        m2_ins_a.set_high();
        m2_ins_b.set_high();
        m3_ins_a.set_high();
        m3_ins_b.set_high();
        m4_ins_a.set_high();
        m4_ins_b.set_high();
        spi.read(&mut spi_read_buffer).await.unwrap();
        info!("SPI Received: {:x}", spi_read_buffer[0]);
        spi.write(&spi_write_buffer).await.unwrap();
        info!("Data sent {:?}", spi_write_buffer);
        vcc_gpio.set_high();
        standby_gpio.set_high();

        // info!("Motor1 count: {}", qei_motor1.count());
        // info!("Motor2 count: {}", qei_motor2.count());
        // info!("Motor3 count: {}", qei_motor3.count());
        // info!("Motor4 count: {}", qei_motor4.count());

        mot1_pwm.set_duty_cycle_percent(40);
        mot2_pwm.set_duty_cycle_percent(50);
        mot3_pwm.set_duty_cycle_percent(60);
        mot4_pwm.set_duty_cycle_percent(70);
        // Timer::after_millis(5).await;

        ticker.next().await;

        // info!("low");

        mot1_pwm.set_duty_cycle_percent(5);
        mot2_pwm.set_duty_cycle_percent(10);
        mot3_pwm.set_duty_cycle_percent(15);
        mot4_pwm.set_duty_cycle_percent(20);

        m1_ins_a.set_low();
        m1_ins_b.set_low();
        m2_ins_a.set_low();
        m2_ins_b.set_low();
        m3_ins_a.set_low();
        m3_ins_b.set_low();
        m4_ins_a.set_low();
        m4_ins_b.set_low();
        vcc_gpio.set_low();
        standby_gpio.set_low();

        ticker.next().await;
    }
}
