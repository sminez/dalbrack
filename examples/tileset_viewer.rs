use hecs::Entity;
use risky_endevours::{Pos, state::State, tileset::TileSet};
use sdl2::{event::Event, keyboard::Keycode};

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(1080, 1000, 50, "Risky Endevours")?;
    state.ts = TileSet::df_rde()?;

    let mut col: u16 = 0;
    let mut row: u16 = 0;

    let e_tile = state.world.spawn((Pos::new(1, 1),));
    let e_coords = state.world.spawn((Pos::new(1, 3),));
    let e_ident = state.world.spawn((Pos::new(1, 4),));

    update(e_tile, e_coords, e_ident, row, col, &mut state)?;

    loop {
        match state.ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Right => col += 1,
                Keycode::Left => col = col.saturating_sub(1),
                Keycode::Up => row = row.saturating_sub(1),
                Keycode::Down => row += 1,

                Keycode::Space => state.ui.toggle_debug_bg(),
                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        update(e_tile, e_coords, e_ident, row, col, &mut state)?;
    }
}

fn update(
    e_tile: Entity,
    e_coords: Entity,
    e_ident: Entity,
    row: u16,
    col: u16,
    state: &mut State<'_>,
) -> anyhow::Result<()> {
    let coords = format!("({row},{col})");
    let tile = state.ts.ibm437_tile(row, col);
    let ident = match state.ts.tile_name(tile) {
        Some(ident) => ident.to_string(),
        None => String::new(),
    };

    state.world.insert_one(e_tile, tile)?;
    state.world.insert_one(e_coords, coords)?;
    state.world.insert_one(e_ident, ident)?;

    state.ui.clear();
    state.blit_all()?;
    state.ui.render()?;

    Ok(())
}
