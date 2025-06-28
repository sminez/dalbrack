use dalbrack::{
    Pos, TITLE,
    map::{
        builders::{BuildConfig, BuildMap, Forest},
        fov::{FovRange, LightSource},
    },
    player::Player,
    state::{LocalMap, State},
    ui::{Box, DisplayMode, palette},
};
use sdl2::pixels::Color;

const W: i32 = 60;
const SCREEN_H: i32 = 40;
const H: i32 = SCREEN_H - 5;
const CFG: BuildConfig = BuildConfig { populated: true };

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DisplayMode::FullScreen, TITLE)?;
    // let mut state = State::init(DisplayMode::Fixed(W as u32, SCREEN_H as u32, 16), TITLE)?;
    let (pos, map) = Forest::default().new_map(W as usize, H as usize, CFG, &mut state);

    // This needs to be a first class thing in the UI rather than directly spawning here
    let white = palette::IBM_WHITE;
    state.world.spawn((Box::new(0, H, W - 1, 4, white),));
    state.world.spawn((
        Pos::new(1, H + 1),
        String::from("You enter the woods of Dalbrack"),
        white,
    ));

    state.set_map(map);

    state.e_player = state.world.spawn(
        Player::new_base_bundle(pos, FovRange(75), &state)
            .add(LightSource {
                range: 12,
                // color: Color::RGB(80, 50, 20),
                color: Color::RGB(178, 111, 45),
            })
            .build(),
    );

    state.run_mode(LocalMap)?;

    Ok(())
}
