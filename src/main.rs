#![no_std]
#![no_main]

use bsp::{
    entry,
    hal::{prelude::_rphal_pio_PIOExt, rosc::RingOscillator, Timer},
};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::timer::CountDown;
use fugit::{Duration, ExtU32};
use panic_probe as _;

use rp2040_project_template::{Direction, GameState, Player};
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

    let button_a = pins.gpio6.into_pull_up_input();
    let button_b = pins.gpio7.into_pull_up_input();
    let button_c = pins.gpio8.into_pull_up_input();
    let button_d = pins.gpio9.into_pull_up_input();

    loop {
        match game_state.player {
            rp2040_project_template::Player::P1 => info!("driver - player 1"),
            rp2040_project_template::Player::P2 => info!("driver - player 2"),
        }

        let timeout = game_state.tick(&mut ring_oscillator);
        ws.write(brightness(game_state.leds().into_iter(), 255))
            .unwrap();
        let mut count_down = timer.count_down();
        count_down.start(timeout * 1.millis());
        while !count_down.wait().is_ok() {
            if button_a.is_low().unwrap() {
                info!("a"); // 2 left
                game_state.button_push(Player::P2, Direction::CCW);
            }
            if button_b.is_low().unwrap() {
                info!("b"); // 1 left
                game_state.button_push(Player::P1, Direction::CCW);
            }
            if button_c.is_low().unwrap() {
                info!("c"); // 1 right
                game_state.button_push(Player::P1, Direction::CW);
            }
            if button_d.is_low().unwrap() {
                info!("d"); // 2 right
                game_state.button_push(Player::P2, Direction::CW);
            }
        }
    }
}
