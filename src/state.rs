//! The global state of the game
use crate::{
    FRAME_LEN_MS, Pos,
    action::{Action, AvailableActions},
    data_files::parse_color_palette,
    map::{
        Map,
        fov::{Fov, FovRange, LightMap, LightSource, Opacity},
    },
    player::Player,
    tileset::{Tile, TileSet},
    ui::Sdl2UI,
};
use hecs::{Entity, Ref, World};
use sdl2::{pixels::Color, rect::Rect};
use std::{
    collections::{HashMap, VecDeque},
    thread::sleep,
    time::{Duration, Instant},
};

pub struct State<'a> {
    pub world: World,
    pub e_player: Entity,
    pub e_map: Entity,
    pub ui: Sdl2UI<'a>,
    pub ts: TileSet<'a>,
    pub palette: HashMap<String, Color>,
    pub running: bool,
    pub action_queue: VecDeque<Action>,
    pub last_tick: Instant,
}

impl<'a> State<'a> {
    pub fn init(w: u32, h: u32, dxy: u32, window_title: &str) -> anyhow::Result<Self> {
        let ts = TileSet::default();
        let palette = parse_color_palette()?;
        let bg = *palette.get("black").unwrap();
        let c_hidden = *palette.get("hidden").unwrap();
        let ui = Sdl2UI::init(w, h, dxy, window_title, bg, c_hidden)?;
        let mut world = World::new();
        let e_player = world.spawn(());
        let e_map = world.spawn(());

        Ok(State {
            world,
            e_player,
            e_map,
            ui,
            ts,
            palette,
            running: true,
            last_tick: Instant::now(),
            action_queue: VecDeque::new(),
        })
    }

    pub fn tick_with(
        &mut self,
        update_fn: impl Fn(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut rendered = false;

        while let Some(action) = self.action_queue.pop_front() {
            action.run(self)?;

            self.update_fov()?;
            self.update_light_map()?;

            self.wait_for_frame();
            (update_fn)(self)?;
            rendered = true;
        }

        while let Some(action) = self.next_player_action() {
            action.run(self)?;
            self.run_actor_actions()?;

            self.update_fov()?;
            self.update_light_map()?;

            self.wait_for_frame();
            (update_fn)(self)?;
            rendered = true;
        }

        if !rendered && self.need_frame() {
            self.wait_for_frame();
            (update_fn)(self)?;
        }

        Ok(())
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        self.tick_with(Self::update_ui)
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
        let delta = t_now.duration_since(self.last_tick).as_millis() as u64;
        delta >= FRAME_LEN_MS
    }

    fn wait_for_frame(&mut self) {
        let t_now = Instant::now();
        let delta = t_now.duration_since(self.last_tick).as_millis() as u64;
        if delta < FRAME_LEN_MS {
            sleep(Duration::from_millis(FRAME_LEN_MS - delta));
        }

        self.last_tick = Instant::now();
    }

    pub fn update_ui(&mut self) -> anyhow::Result<()> {
        self.ui.clear();
        self.blit_all()?;
        self.ui.render()?;

        Ok(())
    }

    pub fn tile_with_named_color(&self, ident: &str, color: &str) -> Tile {
        let mut tile = self.ts.tile(ident).unwrap();
        tile.color = *self.palette.get(color).unwrap();

        tile
    }

    pub fn tile_with_color(&self, ident: &str, color: Color) -> Tile {
        let mut tile = self.ts.tile(ident).unwrap();
        tile.color = color;

        tile
    }

    pub fn set_map(&mut self, map: Map) {
        self.world
            .insert_one(self.e_map, map)
            .expect("e_map to be valid");
    }

    pub fn current_map(&self) -> Option<Ref<Map>> {
        self.world.get::<&Map>(self.e_map).ok()
    }

    /// This will no-op rather than error if we are missing the correct player components
    /// or if the map is missing.
    pub fn update_fov(&mut self) -> anyhow::Result<()> {
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

        let map = match self.world.query_one_mut::<&mut Map>(self.e_map) {
            Ok(map) => map,
            Err(_) => return Ok(()),
        };

        let fov = Fov::new(map, &objects, pos, range);
        self.world.insert_one(self.e_player, fov)?;

        Ok(())
    }

    pub fn update_light_map(&mut self) -> anyhow::Result<()> {
        let light_map = {
            let fov = match self.world.get::<&Fov>(self.e_player) {
                Ok(fov) => fov,
                Err(_) => return Ok(()),
            };
            let mut map = self.world.get::<&mut Map>(self.e_map).unwrap();
            let mut sources = self.world.query::<(&Pos, &LightSource)>();
            let light_map = LightMap::from_sources(
                &map,
                &fov,
                sources.iter().map(|(_, s)| s),
                self.ui.c_hidden,
            );

            for p in fov.points.iter() {
                if light_map.points.contains_key(p) {
                    let idx = map.pos_idx(*p);
                    map.explored.insert(idx);
                }
            }

            light_map
        };

        self.world.insert_one(self.e_map, light_map)?;

        Ok(())
    }

    pub fn blit_all(&mut self) -> anyhow::Result<()> {
        self.blit_map()?;
        self.blit_tiles()?;
        self.blit_text()?;

        Ok(())
    }

    pub fn blit_map(&mut self) -> anyhow::Result<()> {
        let fov = self.world.get::<&Fov>(self.e_player).ok();
        let map = match self.world.get::<&mut Map>(self.e_map) {
            Ok(map) => map,
            Err(_) => return Ok(()), // no map to render
        };
        let fov_and_light_map = match self.world.get::<&LightMap>(self.e_map) {
            Ok(lm) => fov.map(|fov| (fov, lm)),
            Err(_) => None,
        };

        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
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

    pub fn blit_tiles(&mut self) -> anyhow::Result<()> {
        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;

        for (_entity, (pos, tile)) in self.world.query::<(&Pos, &Tile)>().iter() {
            if let Ok(fov) = self.world.get::<&Fov>(self.e_player) {
                if !fov.points.contains(pos) {
                    continue;
                }
            };

            r.x = pos.x * dxy;
            r.y = pos.y * dxy;
            self.ts.blit_tile(tile, r, &mut self.ui.buf)?;
        }

        Ok(())
    }

    pub fn blit_text(&mut self) -> anyhow::Result<()> {
        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;

        let mut buf = [0; 4];
        for (_entity, (pos, s)) in self.world.query_mut::<(&Pos, &String)>() {
            r.x = pos.x * dxy;
            r.y = pos.y * dxy;

            for ch in s.chars() {
                let ident = ch.encode_utf8(&mut buf);
                let tile = self.ts.tile(ident).unwrap();
                self.ts.blit_tile(&tile, r, &mut self.ui.buf)?;
                r.x += dxy;
            }
        }

        Ok(())
    }
}
