use dalbrack::{
    TITLE,
    map::{
        builders::{BuildMap, Forest},
        fov::{FovRange, LightSource},
    },
    player::Player,
    state::{LocalMap, State},
    ui::{DisplayMode, MAP_H, MAP_W},
};
use sdl2::pixels::Color;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DisplayMode::FullScreen, TITLE)?;
    // let mut state = State::init(DisplayMode::Fixed(W as u32, SCREEN_H as u32, 16), TITLE)?;

    let (pos, map) = Forest::default().new_map(
        MAP_W as usize,
        MAP_H as usize,
        Default::default(),
        &mut state,
    );
    state.set_map(map);
    state.log("You enter the woods of Dalbrack, in search of the Snoot");
    state.log("Where could it be?...");

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
