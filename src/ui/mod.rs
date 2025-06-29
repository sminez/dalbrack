use crate::Pos;
use anyhow::anyhow;
use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    event::Event,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext},
};

mod color;

pub use color::{ColorExt, palette};

pub const LOGICAL_W: u32 = 75;
pub const LOGICAL_H: u32 = 50;
pub const UI_H: u32 = 6;
pub const MAP_W: u32 = LOGICAL_W;
pub const MAP_H: u32 = LOGICAL_H - UI_H;

pub enum DisplayMode {
    Fixed(u32, u32, u32),
    FullScreen,
}

pub struct Sdl2UI<'a> {
    w: u32,
    h: u32,
    pub dxy: u32,
    target: Option<Rect>,
    _ctx: Sdl,
    _video_ss: VideoSubsystem,
    canvas: Canvas<Window>,
    pub buf: Surface<'a>,
    tc: TextureCreator<WindowContext>,
    evts: EventPump,
    bg: Color,
}

impl<'a> Sdl2UI<'a> {
    pub fn init(mode: DisplayMode, window_title: &str) -> anyhow::Result<Self> {
        let ctx = sdl2::init().map_err(|e| anyhow!("{e}"))?;
        let video_ss = ctx.video().map_err(|e| anyhow!("{e}"))?;

        let (mut canvas, w, h, dxy) = match mode {
            DisplayMode::FullScreen => {
                let mut win = video_ss
                    .window(window_title, LOGICAL_W, LOGICAL_H)
                    .resizable()
                    .fullscreen_desktop()
                    .build()?;
                let idx = win.display_index().map_err(|e| anyhow!("{e}"))?;
                let mode = video_ss
                    .current_display_mode(idx)
                    .map_err(|e| anyhow!("{e}"))?;
                let dxy = mode.h as u32 / LOGICAL_H;
                win.set_size(LOGICAL_W * dxy, LOGICAL_H * dxy).unwrap();

                let canvas = win.into_canvas().target_texture().present_vsync().build()?;

                (canvas, LOGICAL_W * dxy, LOGICAL_H * dxy, dxy)
            }

            DisplayMode::Fixed(w, h, dxy) => {
                let win = video_ss
                    .window(window_title, w * dxy, h * dxy)
                    .position_centered()
                    .build()?;

                let canvas = win.into_canvas().target_texture().present_vsync().build()?;

                (canvas, w * dxy, h * dxy, dxy)
            }
        };

        let tc = canvas.texture_creator();
        let evts = ctx.event_pump().map_err(|e| anyhow!("{e}"))?;

        let buf = Surface::new(w, h, PixelFormatEnum::ARGB8888).map_err(|e| anyhow!("{e}"))?;

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.present();

        Ok(Self {
            w,
            h,
            dxy,
            target: None,
            _ctx: ctx,
            _video_ss: video_ss,
            canvas,
            buf,
            tc,
            evts,
            bg: Color::MAGENTA, // so its obvious when its not been set
        })
    }

    pub fn set_bg(&mut self, color: Color) {
        self.bg = color;
    }

    pub fn resize(&mut self, w: u32, _h: u32) {
        let offset = (w - LOGICAL_W * self.dxy) as i32 / 2;
        self.target = if offset > 0 {
            Some(Rect::new(offset, 0, self.w, self.h))
        } else {
            None
        };
    }

    pub fn map_click(&self, x: i32, y: i32) -> Pos {
        Pos::new(
            (x - self.target.map(|r| r.x).unwrap_or_default()) / self.dxy as i32,
            y / self.dxy as i32,
        )
    }

    /// Poll for currently pending events.
    ///
    /// Window resize events are handled internally.
    /// Returns None if no events are pending.
    pub fn poll_event(&mut self) -> Option<Event> {
        self.evts.poll_event()
    }

    /// Block and wait for the next event.
    ///
    /// Window resize events are handled internally.
    pub fn wait_event(&mut self) -> Event {
        self.evts.wait_event()
    }

    pub fn wait_event_timeout(&mut self, ms: u32) -> Option<Event> {
        self.evts.wait_event_timeout(ms)
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(self.bg);
        self.canvas.clear();
        self.buf.fill_rect(None, self.bg).unwrap();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let tx = self.buf.as_texture(&self.tc)?;
        self.canvas
            .copy(&tx, None, self.target)
            .map_err(|e| anyhow!("unable to copy buffer to canvas: {e}"))?;
        self.canvas.present();

        Ok(())
    }
}

#[derive(Debug)]
pub struct Box {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: Color,
}

impl Box {
    pub fn new(x: u32, y: u32, w: u32, h: u32, color: Color) -> Self {
        Self {
            x: x as i32,
            y: y as i32,
            w: w as i32,
            h: h as i32,
            color,
        }
    }
}
