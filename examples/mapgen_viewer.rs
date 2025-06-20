use dalbrack::{
    TITLE,
    map::builders::{BspDungeon, BuildMap, CellularAutomata},
    state::State,
};
use sdl2::{event::Event, keyboard::Keycode};
use std::time::Instant;

const DXY: u32 = 25;
const W: i32 = 60;
const H: i32 = 40;
const FRAME_LEN: u128 = 100;

macro_rules! set {
    ($builder:expr, $new:expr, $maps:expr, $state:expr) => {{
        $builder = Box::new($new);
        $maps = $builder.trace_build(W as usize, H as usize, &$state);
        $maps.reverse();
    }};
}

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let mut builder = Box::new(CellularAutomata::default()) as Box<dyn BuildMap>;
    let mut maps = builder.trace_build(W as usize, H as usize, &state);
    maps.reverse();

    if let Some(map) = maps.pop() {
        state.set_map(map);
    }

    let mut t1 = Instant::now();
    update(&mut state)?;

    loop {
        let mut need_render = false;

        if let Some(event) = state.ui.wait_event_timeout(FRAME_LEN as u32) {
            need_render = true;
            match event {
                Event::Quit { .. } => return Ok(()),

                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => match k {
                    Keycode::Num1 => set!(builder, BspDungeon, maps, state),
                    Keycode::Num2 => set!(builder, CellularAutomata::simple(), maps, state),
                    Keycode::Num3 => set!(builder, CellularAutomata::rogue_basin(), maps, state),
                    Keycode::Num4 => set!(builder, CellularAutomata::diamoeba(), maps, state),
                    Keycode::Num5 => set!(builder, CellularAutomata::invertamaze(), maps, state),
                    Keycode::Num6 => set!(builder, CellularAutomata::walled_cities(), maps, state),
                    Keycode::Num7 => set!(builder, CellularAutomata::corrosion(), maps, state),
                    Keycode::Num8 => set!(builder, CellularAutomata::ice_balls(), maps, state),
                    Keycode::Num9 => set!(builder, CellularAutomata::coagulations(), maps, state),

                    Keycode::R => {
                        maps = builder.trace_build(W as usize, H as usize, &state);
                        maps.reverse();
                    }

                    Keycode::RightBracket => state.ui.dxy += 5,
                    Keycode::LeftBracket => state.ui.dxy -= 5,

                    Keycode::Q | Keycode::Escape => return Ok(()),

                    _ => need_render = false,
                },

                _ => need_render = false,
            }
        }

        let t2 = Instant::now();
        if t2.duration_since(t1).as_millis() >= FRAME_LEN {
            t1 = t2;
            if let Some(map) = maps.pop() {
                state.set_map(map);
                need_render = true;
            }
        }

        if need_render {
            update(&mut state)?;
        }
    }
}

fn update(state: &mut State<'_>) -> anyhow::Result<()> {
    state.ui.clear();
    state.blit_all()?;
    state.ui.render()?;

    Ok(())
}
