use dalbrack::{
    Pos, TITLE,
    action::{Action, AvailableActions, toggle_explored},
    input::map_event_in_game_state,
    map::{
        Map,
        builders::{BuildConfig, BuildMap, CellularAutomata, MapBuilder},
        fov::{FovRange, LightSource},
    },
    player::Player,
    state::State,
};
use rand::Rng;
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color};

const DXY: u32 = 25;
const W: i32 = 70;
const H: i32 = 40;
const CFG: BuildConfig = BuildConfig { populated: true };

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let (pos, map) =
        CellularAutomata::walled_cities().new_map(W as usize, H as usize, CFG, &mut state);
    let builder = MapBuilder::from(CellularAutomata::walled_cities);

    state.set_map(map);
    state.world.insert_one(state.e_map, builder).unwrap();

    state.e_player = state.world.spawn(
        Player::new_base_bundle(pos, FovRange(30), &state)
            .add(LightSource {
                range: 18,
                color: Color::RGB(80, 50, 20),
            })
            .build(),
    );

    state.update_fov()?;
    state.update_light_map()?;
    state.update_ui()?;

    while state.running {
        let event = state.ui.wait_event();

        match map_event_in_game_state(&event, &state) {
            Some(action) => state.action_queue.push_back(action),
            None => {
                if let Some(action) = map_other_events(&event) {
                    state.action_queue.push_back(action);
                }
            }
        }

        state.tick()?;
    }

    Ok(())
}

pub fn map_other_events(event: &Event) -> Option<Action> {
    let action = match *event {
        Event::KeyDown {
            keycode: Some(k),
            repeat: false,
            ..
        } => match k {
            Keycode::Space => toggle_explored.into(),

            Keycode::R => Action::from(move |state: &mut State<'_>| {
                let lights: Vec<_> = state
                    .world
                    .query::<&LightSource>()
                    .without::<&Player>()
                    .iter()
                    .map(|(e, _)| e)
                    .collect();

                let mobs: Vec<_> = state
                    .world
                    .query::<&AvailableActions>()
                    .without::<&Player>()
                    .iter()
                    .map(|(e, _)| e)
                    .collect();

                for entity in lights.into_iter().chain(mobs) {
                    state.world.despawn(entity)?;
                }

                let builder = state
                    .world
                    .query_one_mut::<&MapBuilder>(state.e_map)
                    .unwrap();

                let (pos, map) = builder.get().new_map(W as usize, H as usize, CFG, state);
                state.set_map(map);
                Player::warp(pos, state);

                state.update_fov()?;
                state.update_light_map()?;

                Ok(())
            }),

            _ => return None,
        },

        // Left click to place random light
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Left,
            x,
            y,
            ..
        } => Action::from(move |state: &mut State<'_>| {
            let mut rng = rand::rng();
            let color = Color::RGB(
                rng.random_range(60..150),
                rng.random_range(60..150),
                rng.random_range(60..150),
            );

            state.world.spawn((
                Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32),
                state.tile_with_color("star", color),
                LightSource {
                    range: rng.random_range(5..12),
                    color,
                },
            ));

            Ok(())
        }),

        // Middle click to flip between floor and wall
        Event::MouseButtonDown {
            mouse_btn: MouseButton::Middle,
            x,
            y,
            ..
        } => Action::from(move |state: &mut State<'_>| {
            let pos = Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32);
            let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
            map.tiles[pos] = if map.tiles[pos] == 0 { 1 } else { 0 };

            Ok(())
        }),

        _ => return None,
    };

    Some(action)
}
