//! Actions that can be executed by the main game loop
use crate::{map::Map, state::State};

/// An action that can be executed against the game state
#[allow(clippy::type_complexity)]
pub struct Action(pub Box<dyn Fn(&mut State<'_>) -> anyhow::Result<()>>);

impl Action {
    pub fn run(self, state: &mut State<'_>) -> anyhow::Result<()> {
        (self.0)(state)
    }
}

impl<F> From<F> for Action
where
    F: Fn(&mut State<'_>) -> anyhow::Result<()> + 'static,
{
    fn from(f: F) -> Self {
        Action(Box::new(f))
    }
}

/// Quit the game
pub fn quit(state: &mut State<'_>) -> anyhow::Result<()> {
    state.running = false;

    Ok(())
}

pub fn zoom_in(state: &mut State<'_>) -> anyhow::Result<()> {
    state.ui.dxy += 5;

    Ok(())
}

pub fn zoom_out(state: &mut State<'_>) -> anyhow::Result<()> {
    state.ui.dxy += 5;

    Ok(())
}

pub fn toggle_explored(state: &mut State<'_>) -> anyhow::Result<()> {
    let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
    if map.explored.len() == map.tiles.len() {
        map.clear_explored();
    } else {
        map.explore_all();
    }

    Ok(())
}
