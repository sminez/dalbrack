//! The global state of the game
use crate::{
    FRAME_LEN_MS, Pos,
    action::{Action, AvailableActions},
    map::{
        Map, MapSet,
        fov::{Fov, FovRange, LightMap, LightSource, Opacity},
    },
    player::Player,
    rng::RngHandle,
    tileset::{Tile, TileSet},
    ui::{Bork, Box, DisplayMode, LOGICAL_W, MAP_H, Sdl2UI, UI_H, palette},
};
use hecs::{Entity, World};
use sdl2::{event::WindowEvent, pixels::Color, rect::Rect};
use std::{
    collections::{HashMap, VecDeque},
    thread::sleep,
    time::{Duration, Instant},
};

pub mod mode;
pub use mode::{GameMode, LocalMap};

pub struct State<'a> {
    pub rng: RngHandle,
    pub world: World,
    pub e_player: Entity,
    pub mapset: MapSet,
    pub ui: Sdl2UI<'a>,
    pub ts: TileSet<'a>,
    pub running: bool,
    pub action_queue: VecDeque<Action>,
    pub log: Vec<String>,
    pub t_last_frame: Instant,
    pub tick: usize,
}

impl<'a> State<'a> {
    pub fn init(mode: DisplayMode, window_title: &str) -> anyhow::Result<Self> {
        let ts = TileSet::default();
        let ui = Sdl2UI::init(mode, window_title)?;
        let mut world = World::new();
        let e_player = world.spawn(());
        let mapset = MapSet::new();

        Ok(State {
            rng: RngHandle::new(),
            world,
            e_player,
            mapset,
            ui,
            ts,
            running: true,
            action_queue: VecDeque::new(),
            log: Vec::new(),
            t_last_frame: Instant::now(),
            tick: 0,
        })
    }

    pub fn run_mode<M: GameMode>(&mut self, mode: M) -> anyhow::Result<()> {
        use sdl2::event::Event;

        mode.init(self)?;

        while self.running {
            let event = self.ui.wait_event();
            match event {
                Event::Window {
                    win_event: WindowEvent::SizeChanged(w, h) | WindowEvent::Resized(w, h),
                    ..
                } => {
                    self.ui.resize(w as u32, h as u32);
                    self.update_ui()?;
                    continue;
                }

                Event::MouseMotion { .. } => continue,

                _ => (),
            }

            if let Some(action) = mode.action_for_input_event(&event, self) {
                self.action_queue.push_back(action);
            };

            self.tick_with(&mode)?;
        }

        Ok(())
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        self.log.push(msg.into());
    }

    pub fn bork(&mut self, mut pos: Pos, msg: impl Into<String>) -> Entity {
        if pos.y == 0 {
            pos.y = 2;
        } else {
            pos.y -= 1;
        }
        if pos.x == 0 {
            pos.x = 2;
        } else {
            pos.x -= 1;
        }

        self.world.spawn((Bork {
            pos,
            msg: msg.into(),
            fg: palette::IBM_WHITE,
            bg: palette::FOREST_BG,
            from_tick: self.tick,
        },))
    }

    pub fn tick_with<M: GameMode>(&mut self, mode: &M) -> anyhow::Result<()> {
        let mut rendered = false;

        while let Some(action) = self.action_queue.pop_front() {
            self.tick += 1;
            action.run(self)?;
            mode.after_action(self)?;
            self.wait_for_frame();
            mode.update_ui(self)?;
            rendered = true;
        }

        while let Some(action) = self.next_player_action() {
            self.tick += 1;
            action.run(self)?;
            self.run_actor_actions()?;
            mode.after_action(self)?;
            self.wait_for_frame();
            mode.update_ui(self)?;
            rendered = true;
        }

        if !rendered && self.need_frame() {
            self.wait_for_frame();
            mode.update_ui(self)?;
        }

        Ok(())
    }

    pub fn tick_with_fn(
        &mut self,
        f: impl Fn(&mut State<'_>) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.tick_with(&f)
    }

    fn next_player_action(&self) -> Option<Action> {
        self.world
            .get::<&mut AvailableActions>(self.e_player)
            .ok()?
            .next_action(self.e_player, self)
    }

    fn run_actor_actions(&mut self) -> anyhow::Result<()> {
        let actions: Vec<_> = self
            .world
            .query::<&mut AvailableActions>()
            .without::<&Player>()
            .iter()
            .filter_map(|(e, aa)| aa.next_action(e, self))
            .collect();

        for action in actions {
            action.run(self)?;
        }

        Ok(())
    }

    fn need_frame(&self) -> bool {
        let t_now = Instant::now();
        let delta = t_now.duration_since(self.t_last_frame).as_millis() as u64;
        delta >= FRAME_LEN_MS
    }

    fn wait_for_frame(&mut self) {
        let t_now = Instant::now();
        let delta = t_now.duration_since(self.t_last_frame).as_millis() as u64;
        if delta < FRAME_LEN_MS {
            sleep(Duration::from_millis(FRAME_LEN_MS - delta));
        }

        self.t_last_frame = Instant::now();
    }

    pub fn update_ui(&mut self) -> anyhow::Result<()> {
        self.ui.clear();
        self.blit_all()?;
        self.ui.render()?;

        Ok(())
    }

    pub fn tile_with_color(&self, ident: &str, color: Color) -> Tile {
        let mut tile = self.ts.tile(ident).unwrap();
        tile.color = color;

        tile
    }

    pub fn set_map(&mut self, map: Map) {
        self.ui.set_bg(map.bg);
        self.mapset.push(map);
        self.mapset.next();
    }

    /// This will no-op rather than error if we are missing the correct player components
    /// or if the map is missing.
    pub fn update_fov(&mut self) -> anyhow::Result<()> {
        if self.mapset.is_empty() {
            return Ok(());
        }

        match self.world.query_one_mut::<&Fov>(self.e_player) {
            Ok(fov) if !fov.dirty => return Ok(()), // nothing to compute
            _ => (),
        }

        let (pos, range) = match self.world.query_one_mut::<(&Pos, &FovRange)>(self.e_player) {
            Ok((pos, range)) => (*pos, *range),
            Err(_) => return Ok(()),
        };

        let objects: HashMap<Pos, Opacity> = self
            .world
            .query::<(&Pos, &Opacity)>()
            .iter()
            .map(|(_, (&pos, &op))| (pos, op))
            .collect();

        let map = self.mapset.current_mut();
        let fov = Fov::new(map, &objects, pos, range);
        self.world.insert_one(self.e_player, fov)?;

        Ok(())
    }

    pub fn update_light_map(&mut self) -> anyhow::Result<()> {
        if self.mapset.is_empty() {
            return Ok(());
        }

        let fov = match self.world.get::<&Fov>(self.e_player) {
            Ok(fov) => fov,
            Err(_) => return Ok(()),
        };
        let map = self.mapset.current_mut();
        let mut sources = self.world.query::<(&Pos, &LightSource)>();
        let light_map =
            LightMap::from_sources(map, &fov, sources.iter().map(|(_, s)| s), map.hidden);

        for p in fov.points.iter() {
            if light_map.points.contains_key(p) {
                let idx = map.pos_idx(*p);
                map.explored.insert(idx);
            }
        }

        self.mapset.current_mut().light_map = Some(light_map);

        Ok(())
    }

    pub fn clear_with_comp<T: hecs::Component>(&mut self) -> anyhow::Result<()> {
        let entities: Vec<_> = self
            .world
            .query::<&T>()
            .without::<&Player>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for entity in entities.into_iter() {
            self.world.despawn(entity)?;
        }

        Ok(())
    }

    pub fn blit_all(&mut self) -> anyhow::Result<()> {
        self.blit_map()?;
        self.blit_ui()?;
        self.blit_tiles()?;
        self.blit_boxes()?;
        self.blit_text()?;

        Ok(())
    }

    pub fn blit_map(&mut self) -> anyhow::Result<()> {
        if self.mapset.is_empty() {
            return Ok(()); // no map to render
        }

        let fov = self.world.get::<&Fov>(self.e_player).ok();
        let map = self.mapset.current_mut();
        let fov_and_light_map = match map.light_map.as_ref() {
            Some(lm) => fov.map(|fov| (fov, lm)),
            None => None,
        };

        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let mut t = self.ts.tile("square").unwrap();
        let dxy = self.ui.dxy as i32;
        r.x = 0;
        r.y = 0;

        for (y, line) in map.cells.chunks(map.w).enumerate() {
            for (x, tile_idx) in line.iter().enumerate() {
                r.x = x as i32 * dxy;
                r.y = y as i32 * dxy;
                let mut tile = map.tile_defs[*tile_idx];

                if let Some((fov, light_map)) = fov_and_light_map.as_ref() {
                    let p = Pos::new(x as i32, y as i32);
                    if fov.points.contains(&p) {
                        tile.t.color = light_map.apply_light_level(p, tile.t.color);

                        if let Some(c) = tile.bg.and_then(|c| light_map.apply_bg_light_level(p, c))
                        {
                            t.color = c;
                            self.ts.blit_tile(&t, r, &mut self.ui.buf)?;
                        }
                    } else {
                        tile.t.color = light_map.c_hidden;
                    }
                }

                if map.explored.contains(&map.idx(x, y)) {
                    self.ts.blit_tile(&tile.t, r, &mut self.ui.buf)?;
                }
            }
        }

        Ok(())
    }

    pub fn blit_ui(&mut self) -> anyhow::Result<()> {
        let white = palette::IBM_WHITE;

        // Outline for the recent message log
        let b = Box::new(0, MAP_H, LOGICAL_W - 1, UI_H - 1, white);
        self.ts.blit_box(&b, self.ui.dxy, &mut self.ui.buf)?;

        // Current log
        let to_skip = self.log.len().saturating_sub(UI_H as usize - 2);
        for (i, s) in self.log.iter().skip(to_skip).enumerate() {
            self.ts.blit_text(
                Pos::new(1, MAP_H as i32 + i as i32 + 1),
                s,
                white,
                self.ui.dxy,
                &mut self.ui.buf,
            )?;
        }

        // Barks
        let mut to_remove = Vec::new();
        for (e, bork) in self.world.query::<&mut Bork>().iter() {
            if self.tick - bork.from_tick >= 1 {
                to_remove.push(e);
                continue;
            }
            self.ts.blit_bork(bork, self.ui.dxy, &mut self.ui.buf)?;
        }

        for entity in to_remove.into_iter() {
            self.world.despawn(entity)?;
        }

        Ok(())
    }

    pub fn blit_tiles(&mut self) -> anyhow::Result<()> {
        let fov_and_light_map = if self.mapset.is_empty() {
            None
        } else {
            let fov = self.world.get::<&Fov>(self.e_player).ok();
            let map = self.mapset.current_mut();
            match map.light_map.as_ref() {
                Some(lm) => fov.map(|fov| (fov, lm)),
                None => None,
            }
        };

        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;

        for (_entity, (pos, tile)) in self.world.query::<(&Pos, &Tile)>().iter() {
            let mut tile = *tile;
            if let Some((fov, light_map)) = fov_and_light_map.as_ref() {
                if !fov.points.contains(pos) {
                    continue;
                }
                tile.color = light_map.apply_light_level(*pos, tile.color);
            }

            r.x = pos.x * dxy;
            r.y = pos.y * dxy;
            self.ts.blit_tile(&tile, r, &mut self.ui.buf)?;
        }

        Ok(())
    }

    pub fn blit_boxes(&mut self) -> anyhow::Result<()> {
        for (_entity, b) in self.world.query::<&Box>().iter() {
            self.ts.blit_box(b, self.ui.dxy, &mut self.ui.buf)?;
        }

        Ok(())
    }

    pub fn blit_text(&mut self) -> anyhow::Result<()> {
        for (_entity, (pos, s, color)) in self.world.query_mut::<(&Pos, &String, &Color)>() {
            self.ts
                .blit_text(*pos, s, *color, self.ui.dxy, &mut self.ui.buf)?;
        }

        Ok(())
    }
}
