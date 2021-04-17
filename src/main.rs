#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m::{asm::bkpt, iprint, iprintln, peripheral::ITM, peripheral::NVIC};
use cortex_m_rt::entry;

use cortex_m::interrupt::{free, Mutex};

use stm32f3_discovery::stm32f3xx_hal;
use stm32f3xx_hal::{delay::Delay, gpio, pac, pac::TIM4, pac::interrupt, prelude::*, pwm, serial::Serial, time, time::{MegaHertz, MonoTimer}, timer::{ Event, Timer }};

fn initialize_tim4(rcc : &pac::rcc::RegisterBlock, clocks : stm32f3xx_hal::rcc::Clocks, gpiob : &pac::gpiob::RegisterBlock, tim4 : &pac::tim4::RegisterBlock) {   
    /* Pins:
       PB6 -> TIM4_CH1 (Input capture)
       PB7 -> TIM4_CH2 -- not used --
       PB8 -> TIM4_CH3 (Ouput PWM)
       PB9 -> TIM4_CH4 -- not used --
    */    

    rcc.ahbenr.modify(|_, w| w.iopben().set_bit());
    gpiob.moder.modify(|_, w| 
        w.moder6().alternate()
        .moder8().alternate()
    );
    gpiob.afrl.modify(|_, w| w.afrl6().af2());
    gpiob.afrh.modify(|_, w| w.afrh8().af2());

    const APB1_CLK : u32 = 8_000_000;
    const TIM_1MHZ : u32 = 1_000_000;    

    // Так мы получим частоту равную 1MHz вне зависимости от системной частоты
    const PSC : u16 = (APB1_CLK / TIM_1MHZ - 1) as u16;
    // У нас есть тики 1МГц каждую секунду. Мы хотим каждые 50мс, что соответствует 20Гц или 50_000
    const PWM_PERIOD : u16 = (TIM_1MHZ / 20) as u16;
    // Период равен ARR + 1
    const ARR : u16 = PWM_PERIOD - 1;
    // Ширина импульса 10мкс(0.01мс)
    // 50_000 = 50мс
    // х      = 0.01мс
    // х = 50_000 * 0.01мс / 50мс = 100
    const CCR : u16 = 100;
    
    struct Tim4Settings {
        psc : u16,
        arr : u16,
        ccr : u16
    }

    impl Tim4Settings {

        // хочу бля "период в мс", "ширина импульса в мс"

        fn calc(apb1 : time::Hertz, f : time::Hertz) -> Tim4Settings {
            const TIM_1MHZ : MegaHertz = 1.mhz();

            /*
            1. 50ms -> 0.05s
            2. 1 / 0.05s = 20Hz
            3. PSC = TIM_CLK / 1MHz - 1
            4. ARR = 1MHz / 20Hz - 1
            */

            let (psc, arr) = (
                (apb1.0 / TIM_1MHZ.0 - 1) as u16,
                ((TIM_1MHZ.0 / f.0) - 1) as u16,
                
            );

            let 

            Tim4Settings {

            }
        }
    }


    clocks.pclk1();



    rcc.apb1enr.modify(|_, w| w.tim4en().set_bit());
    rcc.apb1rstr.modify(|_, w| w.tim4rst().set_bit());
    rcc.apb1rstr.modify(|_, w| w.tim4rst().clear_bit());

    tim4.cr1.modify(|_, w| w.arpe().set_bit());
    tim4.egr.write(|w| w.ug().set_bit());
    tim4.psc.write(|w| w.psc().bits(PSC));
    tim4.arr.write(|w| w.arr().bits(ARR));

    // CH3(PB8) Ouput PWM

    // CCR
    tim4.ccr3.write(|w| w.ccr().bits(CCR));

    // enable CH3
    tim4.ccer.modify(|_, w| w.cc3e().set_bit());

    // set CH3 to PWM1 Mode with enabled preload 
    tim4.ccmr2_output().modify(|_, w| 
        w.oc3m().pwm_mode1().oc3pe().set_bit());
    
    // CH1(PB6) + CH2(PB7) Input Capture

    // filter(disabled) -> psc(0) -> selection(TI1)
    tim4.ccmr1_input().modify(|_, w| 
        w.ic1f().no_filter()
        .cc1s().ti1());

    // enable CH1. capture both signal fronts
    tim4.ccer.modify(|_, w| 
        w.cc1e().set_bit() /* enable CH1 */
        .cc1np().set_bit().cc1p().set_bit() /* both signal fronts */);

    // capture/compare 1 interrupt enable
    tim4.dier.modify(|_, w| w.cc1ie().enabled());

    // tim4.ccmr2_input().modify(|_, w| w.i cc3s().t)

    tim4.cr1.modify(|_, w| w.cen().set_bit());    
}



fn main() {
    println!("Hello, world!");
}
