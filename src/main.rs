#![no_std]
#![no_main]

use bsp::{
    entry,
    hal::{prelude::_rphal_pio_PIOExt, rosc::RingOscillator, Timer},
};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

use rp2040_project_template::GameState;
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use smart_leds::{brightness, SmartLedsWrite, RGB8};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let mut ws = ws2812_pio::Ws2812::new(
        pins.gpio22.into_mode(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    let mut ring_oscillator = RingOscillator::new(pac.ROSC).initialize();

    let mut game_state = GameState::new(&mut ring_oscillator);

    loop {
        match game_state.player {
            rp2040_project_template::Player::P1 => info!("1"),
            rp2040_project_template::Player::P2 => info!("2"),
        }
        ws.write(brightness(game_state.leds().into_iter(), 10))
            .unwrap();
        game_state.tick(&mut ring_oscillator);
        delay.delay_ms(1000);
    }
}
