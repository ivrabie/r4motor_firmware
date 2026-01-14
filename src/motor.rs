#![no_std]
use defmt::*;
use embassy_stm32::{
    gpio::{Level, Output},
    timer::{GeneralInstance4Channel, 
    qei::{Qei, Direction },
    simple_pwm::SimplePwmChannel},
};

pub enum MDirection {
    Stop,
    Forward,
    Backward,
}

pub struct Motor<'a, S: GeneralInstance4Channel, Q: GeneralInstance4Channel> {
    pub pwm: SimplePwmChannel<'a, S>,
    pub ins_a: Output<'a>,
    pub ins_b: Output<'a>,
    pub qei: Qei<'a, Q>,
    pub count_per_rev: u16,
    pub last_count: u16,
    pub last_time: u64,
}


impl<'a, S: GeneralInstance4Channel, Q: GeneralInstance4Channel> Motor<'a, S, Q> {
    pub fn new(
        pwm: SimplePwmChannel<'a, S>,
        ins_a: Output<'a>,
        ins_b: Output<'a>,
        qei: Qei<'a, Q>,
        count_per_rev: u16,
    ) -> Self {
        Self {
            pwm,
            ins_a,
            ins_b,
            qei,
            count_per_rev: count_per_rev,
            last_count: 0,
            last_time: embassy_time::Instant::now().as_millis(),
        }
    }

    pub fn init(&mut self)
    {
        self.ins_a.set_low();
        self.ins_b.set_low();
        self.pwm.enable();
    }

    pub fn set_direction(&mut self, dir:MDirection)
    {
        self.ins_a.set_low();
        self.ins_b.set_low();
        match dir {
            MDirection::Forward => {
                self.ins_a.set_high();
                self.ins_b.set_low();
            }
            MDirection::Backward => {
                self.ins_a.set_low();
                self.ins_b.set_high();
            }
            MDirection::Stop => {
                self.ins_a.set_low();
                self.ins_b.set_low();
            }
        }
    }

    pub fn set_pwm_duty(&mut self, duty:u8)
    {
        self.pwm.set_duty_cycle_percent(duty);
    }

    pub fn get_direction(&mut self) -> MDirection
    {
        if self.ins_a.is_set_low() && self.ins_b.is_set_low() && 
           self.pwm.current_duty_cycle() == 0 {
            MDirection::Stop
        } else {
            match self.qei.read_direction() {
                Direction::Upcounting => MDirection::Forward,
                Direction::Downcounting => MDirection::Backward,
            }
        }
    }

    pub fn get_count(&mut self) -> u16
    {
        self.qei.count()
    }

    pub fn calculate_rpm(&mut self, interval_ms:u32) -> f32
    {
        let count = self.qei.count() as f32;
        let last_count = self.last_count as f32;
        let interval_ms = interval_ms as f32;
        let count_per_rev = self.count_per_rev as f32;
        let rpm = (count - last_count) / count_per_rev / (interval_ms / 60000.0);
        self.last_count = count as u16;
        rpm
    }


    pub fn ctrl_loop(&mut self){
        let current_time  =  embassy_time::Instant::now().as_millis();
        let interval_ms = (current_time - self.last_time) as u32;
        let rpm = self.calculate_rpm(interval_ms) as u32;
        info!("Current RPM: {}", rpm);
        self.last_time = current_time;
    }

}
