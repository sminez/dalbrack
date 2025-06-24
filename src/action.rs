//! Actions that can be executed by the main game loop
use crate::state::State;
use hecs::Entity;
use std::{collections::VecDeque, fmt::Debug};

#[derive(Debug, Default)]
pub struct AvailableActions(pub VecDeque<Box<dyn ActionProvider>>);

impl AvailableActions {
    pub fn next_action(&mut self, entity: Entity, state: &State<'_>) -> Option<Action> {
        let actions: Vec<Action> = self
            .0
            .iter_mut()
            .flat_map(|p| p.available_actions(entity, state))
            .flatten()
            .collect();

        self.0.retain(|p| p.retain());

        // FIXME: this will always return the first action at the moment. Really actions should
        // have weights and this should be a weighted select from the set
        actions.into_iter().next()
    }

    pub fn push<P>(&mut self, provider: P)
    where
        P: ActionProvider,
    {
        self.0.push_back(Box::new(provider));
    }
}

impl<P: ActionProvider> From<P> for AvailableActions {
    fn from(p: P) -> Self {
        AvailableActions(VecDeque::from([Box::new(p) as Box<dyn ActionProvider>]))
    }
}

/// Something that can provider actions for the entity it is attached to
pub trait ActionProvider: Debug + Send + Sync + 'static {
    /// The actions currently available from this provider for the given entity
    fn available_actions(&mut self, entity: Entity, state: &State<'_>) -> Option<Vec<Action>>;

    fn retain(&self) -> bool;

    fn into_single_action(mut self, entity: Entity, state: &State<'_>) -> Action
    where
        Self: Sized,
    {
        match self.available_actions(entity, state) {
            Some(mut actions) => actions.remove(0),
            None => Action::noop(),
        }
    }
}

/// An action that can be executed against the game state
#[allow(clippy::type_complexity)]
pub struct Action(pub Box<dyn Fn(&mut State<'_>) -> anyhow::Result<()>>);

impl Action {
    pub fn noop() -> Self {
        Self::from(Box::new(|_: &mut State<'_>| Ok(())))
    }
}

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
    state.ui.dxy -= 5;

    Ok(())
}

pub fn toggle_explored(state: &mut State<'_>) -> anyhow::Result<()> {
    let map = state.mapset.current_mut();
    if map.explored.len() == map.tiles.len() {
        map.clear_explored();
    } else {
        map.explore_all();
    }

    Ok(())
}
