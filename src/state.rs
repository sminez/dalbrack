//! The global state of the game
use crate::{
    Pos,
    data_files::parse_color_palette,
    map::{
        Map,
        fov::{Fov, FovRange, LightMap, LightSource},
    },
    tileset::{Tile, TileSet},
    ui::Sdl2UI,
};
use hecs::{Entity, World};
use sdl2::{pixels::Color, rect::Rect};
use std::collections::HashMap;

pub struct State<'a> {
    pub world: World,
    pub e_player: Entity,
    pub e_map: Entity,
    pub ui: Sdl2UI<'a>,
    pub ts: TileSet<'a>,
    pub palette: HashMap<String, Color>,
}

impl<'a> State<'a> {
    pub fn init(w: u32, h: u32, dxy: u32, window_title: &str) -> anyhow::Result<Self> {
        let ts = TileSet::default();
        let palette = parse_color_palette()?;
        let bg = *palette.get("black").unwrap();
        let ui = Sdl2UI::init(w, h, dxy, window_title, bg)?;
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
        })
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        self.update_fov()?;
        self.update_light_map()?;

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

    /// This will no-op rather than error if we are missing the correct player components
    /// or if the map is missing.
    pub fn update_fov(&mut self) -> anyhow::Result<()> {
        let (pos, range) = match self.world.query_one_mut::<(&Pos, &FovRange)>(self.e_player) {
            Ok((pos, range)) => (*pos, *range),
            Err(_) => return Ok(()),
        };

        let map = match self.world.query_one_mut::<&mut Map>(self.e_map) {
            Ok(map) => map,
            Err(_) => return Ok(()),
        };

        let fov = Fov::new(map, pos, range);
        self.world.insert_one(self.e_map, fov)?;

        Ok(())
    }

    pub fn update_light_map(&mut self) -> anyhow::Result<()> {
        let light_map = {
            let mut q = match self.world.query_one::<(&mut Map, &Fov)>(self.e_map) {
                Ok(q) => q,
                Err(_) => return Ok(()),
            };

            let (map, fov) = match q.get() {
                Some((map, fov)) => (map, fov),
                None => return Ok(()),
            };

            let mut sources = self.world.query::<(&Pos, &LightSource)>();
            let light_map = LightMap::from_sources(map, fov, sources.iter().map(|(_, s)| s));

            for p in fov.points.iter() {
                if light_map.points.contains_key(p) {
                    map.explored.insert(map.pos_idx(*p));
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
        let (map, fov_and_light_map) = if self
            .world
            .satisfies::<(&Map, &Fov, &LightMap)>(self.e_map)?
        {
            self.world
                .query_one_mut::<(&mut Map, &Fov, &LightMap)>(self.e_map)
                .map(|(map, fov, light_map)| (map, Some((fov, light_map))))?
        } else {
            match self.world.query_one_mut::<&mut Map>(self.e_map) {
                Ok(map) => (map, None),
                Err(_) => return Ok(()), // no map to render
            }
        };

        let mut r = Rect::new(0, 0, self.ui.dxy, self.ui.dxy);
        let dxy = self.ui.dxy as i32;
        r.x = 0;
        r.y = 0;

        // FIXME: this needs to be stored in the light map once its written
        let black = *self.palette.get("hidden").unwrap();

        for (y, line) in map.cells.chunks(map.w).enumerate() {
            for (x, tile_idx) in line.iter().enumerate() {
                r.x = x as i32 * dxy;
                r.y = y as i32 * dxy;
                let mut tile = map.tile_defs[*tile_idx];

                if let Some((fov, light_map)) = fov_and_light_map.as_ref() {
                    let p = Pos::new(x as i32, y as i32);
                    if fov.points.contains(&p) {
                        tile.t.color = light_map
                            .apply_light_level(p, tile.t.color)
                            .unwrap_or(black);
                    } else {
                        tile.t.color = black;
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
            if let Some(fov) = self.world.query_one::<&Fov>(self.e_map).unwrap().get() {
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
