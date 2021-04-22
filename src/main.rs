#![no_main]
#![no_std]

#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964
extern crate panic_itm; // panic handler

use core::cell::RefCell;

use cortex_m::{asm::bkpt, iprint, iprintln};
use cortex_m_rt::entry;

use cortex_m::interrupt::{free, Mutex};

use stm32f3_discovery::stm32f3xx_hal;
use stm32f3xx_hal::{
    delay::Delay,
    gpio, pac,
    pac::interrupt,
    pac::TIM4,
    prelude::*,
    pwm,
    serial::Serial,
    time,
    time::{MegaHertz, MonoTimer},
    timer::{Event, Timer},
};

struct HcSr04Measure {
    measure1: Option<u16>,
    measure2: Option<u16>,
    distance: Option<u16>,
}

impl HcSr04Measure {
    const fn new() -> HcSr04Measure {
        HcSr04Measure {
            measure1: None,
            measure2: None,
            distance: None
        }
    }

    fn get_distance(&self) -> Option<u16> {
        self.distance
    }

    fn save_measure(&mut self, measure: u16) {
        // shiet
        if measure < 4000 {
            match (self.measure1, self.measure2) {
                (None, None) => self.measure1 = Some(measure),
                (Some(m1), None) => {
                    self.measure2 = Some(measure);
                    let m2 = self.measure2.unwrap();
                    self.distance = Some((m2 - m1) * 17 / 1000);
                }
                (Some(_), Some(_)) => {
                    self.measure1 = Some(measure);
                    self.measure2 = None;
                }
                (None, Some(_)) => panic!(""),
            }
        }
    }
}

static DP_TIM4: Mutex<RefCell<Option<(pac::TIM4, HcSr04Measure)>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn TIM4() {
    free(|cs| {
        let mut tim4_ref = DP_TIM4.borrow(cs).borrow_mut();
        let (tim4, hcsr04) = tim4_ref.as_mut().unwrap();

        let crr1 = tim4.ccr1.read().ccr().bits();

        hcsr04.save_measure(crr1);

        // front?
        // disable CH3
        tim4.ccer.modify(|_, w| w.cc3e().clear_bit());

        pac::NVIC::unpend(pac::Interrupt::TIM4);
    });
}

#[entry]
fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // let mut flash = dp.FLASH.constrain();
    // let mut rcc = dp.RCC.constrain();
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);

    /* stm32f3 pins:
       PB6 -> TIM4_CH1 (Input capture)
       PB7 -> TIM4_CH2 -- not used --
       PB8 -> TIM4_CH3 (Ouput PWM)
       PB9 -> TIM4_CH4 -- not used --
    */

    /* gpiob oscillation */
    dp.RCC.ahbenr.modify(|_, w| w.iopben().set_bit());

    /* PB6, PB8 AF2 mode */
    dp.GPIOB
        .moder
        .modify(|_, w| w.moder6().alternate().moder8().alternate());
    dp.GPIOB.afrl.modify(|_, w| w.afrl6().af2());
    dp.GPIOB.afrh.modify(|_, w| w.afrh8().af2());

    // configure timer to tick every 1us(1MHz)
    // example (pclk1 == 8MHz):
    // psc = 8_000_000 / 1_000_000 - 1 = 7
    // clocks.pclk1().0
    const APB1_CLK: u32 = 8_000_000;
    const TIM_1MHZ: u32 = 1_000_000;
    let psc: u16 = (APB1_CLK / TIM_1MHZ - 1) as u16;
    // timer period 50ms(20Hz)
    // period = 1_000_000 / 20 = 50_000
    const PWM_PERIOD: u16 = (TIM_1MHZ / 20) as u16;
    const ARR: u16 = PWM_PERIOD - 1;
    // we need impulse width 10us
    // 50_000 = 50ms
    // х      = 0.01ms
    // х = 50_000 * 0.01ms / 50ms = 100
    const CCR3: u16 = 100;

    /* TIM4 oscillation */
    dp.RCC.apb1enr.modify(|_, w| w.tim4en().set_bit());
    dp.RCC.apb1rstr.modify(|_, w| w.tim4rst().set_bit());
    dp.RCC.apb1rstr.modify(|_, w| w.tim4rst().clear_bit());

    /* TIM4 clock settings */
    dp.TIM4.cr1.modify(|_, w| w.arpe().set_bit());
    dp.TIM4.egr.write(|w| w.ug().set_bit());
    dp.TIM4.psc.write(|w| w.psc().bits(psc));
    dp.TIM4.arr.write(|w| w.arr().bits(ARR));

    // CH3(PB8) Ouput PWM

    // set up CCR
    dp.TIM4.ccr3.write(|w| w.ccr().bits(CCR3));

    // set CH3 to PWM1 Mode with enabled preload
    dp.TIM4
        .ccmr2_output()
        .modify(|_, w| w.oc3m().pwm_mode1().oc3pe().set_bit());

    // enable CH3
    dp.TIM4.ccer.modify(|_, w| w.cc3e().set_bit());

    // CH1(PB6) Input Capture

    // filter(disabled) -> psc(0) -> selection(TI1)
    dp.TIM4
        .ccmr1_input()
        .modify(|_, w| w.ic1f().no_filter().cc1s().ti1());

    // enable CH1. capture both signal fronts
    dp.TIM4
        .ccer
        .modify(|_, w| w.cc1e().set_bit().cc1np().set_bit().cc1p().set_bit());

    // capture/compare CH1 interrupt enable
    dp.TIM4.dier.modify(|_, w| w.cc1ie().enabled());

    // enable TIM4 interrupts
    pac::NVIC::unpend(pac::Interrupt::TIM4);
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIM4);
    }

    // enable timer
    dp.TIM4.cr1.modify(|_, w| w.cen().set_bit());

    free(|cs| {
        DP_TIM4
            .borrow(cs)
            .replace(Some((dp.TIM4, HcSr04Measure::new())));
    });

    loop {
        let mut v: Option<u16> = None;
        free(|cs| {
            let tim4_ref = DP_TIM4.borrow(cs).borrow();
            let (_, hcsr04) = tim4_ref.as_ref().unwrap();

            v = hcsr04.get_distance();
        });

        if let Some(distance) = v {
            iprintln!(&mut cp.ITM.stim[0], "distance: {0}", distance)
        }
    }
}
