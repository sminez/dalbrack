use crate::Pos;
use anyhow::anyhow;
use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    event::{Event, WindowEvent},
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext},
};

const LOGICAL_W: u32 = 60;
const LOGICAL_H: u32 = 40;

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

                (canvas, LOGICAL_W, LOGICAL_H, dxy)
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

    fn handle_resize(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::Window {
                win_event: WindowEvent::SizeChanged(w, h) | WindowEvent::Resized(w, h),
                ..
            } => {
                self.w = w as u32;
                self.h = h as u32;
                self.dxy = self.h / LOGICAL_H;
                let offset = (h as u32 - LOGICAL_H * self.dxy) as i32;
                self.target = Some(Rect::new(offset, 0, self.w, self.h));
                self.buf = Surface::new(w as u32, h as u32, PixelFormatEnum::ARGB8888).unwrap();

                None
            }

            evt => Some(evt),
        }
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
        loop {
            let event = self.evts.poll_event()?;
            let event = self.handle_resize(event);
            if event.is_some() {
                return event;
            }
        }
    }

    /// Block and wait for the next event.
    ///
    /// Window resize events are handled internally.
    pub fn wait_event(&mut self) -> Event {
        loop {
            let event = self.evts.wait_event();
            let event = self.handle_resize(event);
            if let Some(event) = event {
                return event;
            }
        }
    }

    pub fn wait_event_timeout(&mut self, ms: u32) -> Option<Event> {
        loop {
            let event = self.evts.wait_event_timeout(ms)?;
            let event = self.handle_resize(event);
            if event.is_some() {
                return event;
            }
        }
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
    pub fn new(x: i32, y: i32, w: i32, h: i32, color: Color) -> Self {
        Self { x, y, w, h, color }
    }
}

pub fn blend(color1: Color, color2: Color, perc: f32) -> Color {
    let (c1, m1, y1, k1) = to_cmyk(color1);
    let (c2, m2, y2, k2) = to_cmyk(color2);

    from_cmyk(
        c1 * perc + c2 * (1.0 - perc),
        m1 * perc + m2 * (1.0 - perc),
        y1 * perc + y2 * (1.0 - perc),
        k1 * perc + k2 * (1.0 - perc),
    )
}

fn to_cmyk(color: Color) -> (f32, f32, f32, f32) {
    let mut c = 1.0 - (color.r as f32 / 255.0);
    let mut m = 1.0 - (color.g as f32 / 255.0);
    let mut y = 1.0 - (color.b as f32 / 255.0);

    let mut k = if c < m { c } else { m };
    k = if k < y { k } else { y };

    c = (c - k) / (1.0 - k);
    m = (m - k) / (1.0 - k);
    y = (y - k) / (1.0 - k);

    (c, m, y, k)
}

fn from_cmyk(c: f32, m: f32, y: f32, k: f32) -> Color {
    let mut r = c * (1.0 - k) + k;
    let mut g = m * (1.0 - k) + k;
    let mut b = y * (1.0 - k) + k;

    r = (1.0 - r) * 255.0 + 0.5;
    g = (1.0 - g) * 255.0 + 0.5;
    b = (1.0 - b) * 255.0 + 0.5;

    Color::RGB(r as u8, g as u8, b as u8)
}
