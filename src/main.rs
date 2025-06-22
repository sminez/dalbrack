use dalbrack::{
    Pos, TITLE,
    action::{Action, toggle_explored},
    input::map_event_in_game_state,
    map::{
        Map,
        builders::{BspDungeon, BuildMap, CellularAutomata, MapBuilder},
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

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    // let (pos, map) = CellularAutomata::walled_cities().new_map(W as usize, H as usize, &state);
    // let builder = MapBuilder::from(CellularAutomata::walled_cities);

    let (pos, map) = BspDungeon::default().new_map(W as usize, H as usize, &mut state);
    let builder = MapBuilder::from(CellularAutomata::walled_cities);

    state.set_map(map);
    state
        .world
        .insert_one(state.e_map, builder)
        .expect("e_map to be valid");

    let player_sprite = state.tile_with_named_color("@", "white");
    state.e_player = state.world.spawn((
        Player,
        FovRange(30),
        LightSource {
            range: 12,
            color: Color::RGB(80, 50, 20),
        },
        pos,
        player_sprite,
    ));

    state.tick()?;

    while state.running {
        let event = state.ui.wait_event();

        match map_event_in_game_state(&event) {
            Some(action) => state.action_queue.push_back(action),
            None => match map_other_events(&event) {
                Some(action) => state.action_queue.push_back(action),
                None => continue,
            },
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
                let builder = state
                    .world
                    .query_one_mut::<&MapBuilder>(state.e_map)
                    .unwrap();

                let (pos, map) = builder.get().new_map(W as usize, H as usize, state);
                state.set_map(map);
                Player::set_pos(pos, state);
                let lights: Vec<_> = state
                    .world
                    .query::<&LightSource>()
                    .without::<&Player>()
                    .iter()
                    .map(|(e, _)| e)
                    .collect();

                for entity in lights.into_iter() {
                    state.world.despawn(entity)?;
                }

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
