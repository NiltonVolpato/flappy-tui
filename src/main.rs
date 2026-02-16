use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{self, Color as CColor},
    terminal,
};
use fundsp::{hpc::*, prelude::*};
use rodio::{self, Sink, mixer::Mixer};
use std::io::{self, Write, stdout};
use std::time::{Duration, Instant};

// ── Sounds ──────────────────────────────────────────────────────────────────

fn play_death(mixer: &Mixer) {
    let sink = Sink::connect_new(mixer);

    // 1. Create the Frequency Ramp (400Hz to 80Hz over 0.4s)
    let freq = lfo(|t: f64| lerp11(400.0, 80.0, (t / 0.4).min(1.0)));

    // 2. Create the Gain Ramp (0.15 to 0.0 over 0.5s)
    let gain = lfo(|t: f64| lerp11(0.15, 0.0, (t / 0.5).min(1.0)));

    // 3. Connect: freq >> sawtooth * gain
    // This is the equivalent of: o.connect(g).connect(destination)
    let sound = freq >> saw() >> mul(gain);

    // 4. Wrap in a Rodio-compatible source
    // fundsp uses 44.1kHz by default
    let source = rodio::source::from_iter(sound.take(44100 * 0.5))
        .convert_samples::<f32>()
        .periodic_samples(Duration::from_secs_f32(1.0 / 44100.0), 1);

    sink.append(source);
    sink.detach(); // Play in background
}

// ── Colors ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rgb(u8, u8, u8);

impl Rgb {
    const fn lerp(a: Rgb, b: Rgb, t_256: u16) -> Rgb {
        let t = t_256 as i32;
        Rgb(
            (a.0 as i32 + (b.0 as i32 - a.0 as i32) * t / 256) as u8,
            (a.1 as i32 + (b.1 as i32 - a.1 as i32) * t / 256) as u8,
            (a.2 as i32 + (b.2 as i32 - a.2 as i32) * t / 256) as u8,
        )
    }
}

const SKY_TOP: Rgb = Rgb(70, 180, 200);
const SKY_BOT: Rgb = Rgb(190, 232, 245);
const GRASS: Rgb = Rgb(84, 168, 55);
const GRASS_LIGHT: Rgb = Rgb(110, 200, 70);
const DIRT: Rgb = Rgb(210, 185, 110);
const DIRT_DARK: Rgb = Rgb(185, 160, 90);
const PIPE_L: Rgb = Rgb(74, 122, 26);
const PIPE_M: Rgb = Rgb(100, 170, 40);
const PIPE_R: Rgb = Rgb(115, 191, 46);
const PIPE_HI: Rgb = Rgb(145, 215, 62);
const CAP_DARK: Rgb = Rgb(60, 100, 20);
const BIRD_Y: Rgb = Rgb(245, 200, 66);
const BIRD_HI: Rgb = Rgb(255, 225, 100);
const BIRD_WING: Rgb = Rgb(215, 165, 35);
const BIRD_EYE: Rgb = Rgb(255, 255, 255);
const BIRD_PUPIL: Rgb = Rgb(20, 20, 20);
const BIRD_BEAK: Rgb = Rgb(225, 75, 35);
const BIRD_BEAK_HI: Rgb = Rgb(240, 110, 50);
const HILL_FAR: Rgb = Rgb(120, 195, 75);
const HILL_NEAR: Rgb = Rgb(95, 175, 55);
const WHITE: Rgb = Rgb(255, 255, 255);
const SHADOW: Rgb = Rgb(30, 30, 30);

// ── Pixel buffer with half-block rendering ──────────────────────────────────

struct PixelBuf {
    w: usize,
    h: usize, // pixel height = terminal rows * 2
    px: Vec<Rgb>,
}

impl PixelBuf {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![SKY_TOP; w * h],
        }
    }

    fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.px.resize(w * h, SKY_TOP);
    }

    fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = c;
        }
    }

    fn get(&self, x: usize, y: usize) -> Rgb {
        self.px[y * self.w + x]
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x + dx, y + dy, c);
            }
        }
    }

    fn render(&self, out: &mut impl Write) -> io::Result<()> {
        queue!(out, cursor::MoveTo(0, 0))?;
        let rows = self.h / 2;
        let mut prev_fg = Rgb(0, 0, 0);
        let mut prev_bg = Rgb(0, 0, 0);
        let mut need_fg = true;
        let mut need_bg = true;

        for row in 0..rows {
            for col in 0..self.w {
                let top = self.get(col, row * 2);
                let bot = self.get(col, row * 2 + 1);

                if top == bot {
                    if need_bg || prev_bg != top {
                        queue!(
                            out,
                            style::SetBackgroundColor(CColor::Rgb {
                                r: top.0,
                                g: top.1,
                                b: top.2
                            })
                        )?;
                        prev_bg = top;
                        need_bg = false;
                    }
                    queue!(out, style::Print(' '))?;
                } else {
                    if need_fg || prev_fg != top {
                        queue!(
                            out,
                            style::SetForegroundColor(CColor::Rgb {
                                r: top.0,
                                g: top.1,
                                b: top.2
                            })
                        )?;
                        prev_fg = top;
                        need_fg = false;
                    }
                    if need_bg || prev_bg != bot {
                        queue!(
                            out,
                            style::SetBackgroundColor(CColor::Rgb {
                                r: bot.0,
                                g: bot.1,
                                b: bot.2
                            })
                        )?;
                        prev_bg = bot;
                        need_bg = false;
                    }
                    queue!(out, style::Print('\u{2580}'))?; // ▀
                }
            }
            if row < rows - 1 {
                queue!(out, style::ResetColor, style::Print("\r\n"))?;
                need_fg = true;
                need_bg = true;
            }
        }
        queue!(out, style::ResetColor)?;
        out.flush()
    }
}

// ── 3x5 bitmap digits ──────────────────────────────────────────────────────

#[rustfmt::skip]
const DIGITS: [[u8; 15]; 10] = [
    [1,1,1, 1,0,1, 1,0,1, 1,0,1, 1,1,1], // 0
    [0,1,0, 1,1,0, 0,1,0, 0,1,0, 1,1,1], // 1
    [1,1,1, 0,0,1, 1,1,1, 1,0,0, 1,1,1], // 2
    [1,1,1, 0,0,1, 0,1,1, 0,0,1, 1,1,1], // 3
    [1,0,1, 1,0,1, 1,1,1, 0,0,1, 0,0,1], // 4
    [1,1,1, 1,0,0, 1,1,1, 0,0,1, 1,1,1], // 5
    [1,1,1, 1,0,0, 1,1,1, 1,0,1, 1,1,1], // 6
    [1,1,1, 0,0,1, 0,1,0, 0,1,0, 0,1,0], // 7
    [1,1,1, 1,0,1, 1,1,1, 1,0,1, 1,1,1], // 8
    [1,1,1, 1,0,1, 1,1,1, 0,0,1, 1,1,1], // 9
];

fn draw_digit(buf: &mut PixelBuf, x: i32, y: i32, d: u8, fg: Rgb, shadow: bool) {
    let glyph = &DIGITS[d as usize];
    for row in 0..5 {
        for col in 0..3 {
            if glyph[row * 3 + col] == 1 {
                let px = x + col as i32;
                let py = y + row as i32;
                if shadow {
                    buf.set(px + 1, py + 1, SHADOW);
                }
                buf.set(px, py, fg);
            }
        }
    }
}

fn draw_number(buf: &mut PixelBuf, cx: i32, y: i32, n: u32, fg: Rgb) {
    let s = n.to_string();
    let total_w = s.len() as i32 * 4 - 1; // 3px per digit + 1px spacing
    let start_x = cx - total_w / 2;
    // Shadow pass
    for (i, ch) in s.chars().enumerate() {
        let d = ch as u8 - b'0';
        draw_digit(buf, start_x + i as i32 * 4, y, d, fg, true);
    }
}

// ── Game ────────────────────────────────────────────────────────────────────

struct Pipe {
    x: f64,
    gap_center: f64,
    scored: bool,
}

#[derive(PartialEq)]
enum State {
    Ready,
    Playing,
    Dying,
    Dead,
}

struct Game {
    pw: usize, // pixel width
    ph: usize, // pixel height
    bird_y: f64,
    bird_vy: f64,
    pipes: Vec<Pipe>,
    score: u32,
    best: u32,
    state: State,
    frame: u64,
    ground_x: f64,
    dead_timer: u32,
    show_hud: bool,
    // Derived
    scale: f64,
    ground_h: usize,
    pipe_w: usize,
    pipe_gap: usize,
    bird_x: f64,
    gravity: f64,
    flap_vel: f64,
    pipe_speed: f64,
    pipe_spacing: f64,
}

impl Game {
    fn new(pw: usize, ph: usize) -> Self {
        let scale = ph as f64 / 48.0;
        let ground_h = (8.0 * scale).max(6.0) as usize;
        let pipe_gap = (15.0 * scale).max(11.0) as usize;
        let pipe_w = (8.0 * scale).max(5.0).min(14.0) as usize;

        let mut g = Game {
            pw,
            ph,
            bird_y: 0.0,
            bird_vy: 0.0,
            pipes: Vec::new(),
            score: 0,
            best: 0,
            state: State::Ready,
            frame: 0,
            ground_x: 0.0,
            dead_timer: 0,
            show_hud: false,
            scale,
            ground_h,
            pipe_w,
            pipe_gap,
            bird_x: (pw as f64 * 0.22).max(10.0),
            gravity: 0.20 * scale,
            flap_vel: -2.0 * scale,
            pipe_speed: 1.1 * (pw as f64 / 80.0).max(0.8),
            pipe_spacing: (pw as f64 * 0.42).max(28.0),
        };
        g.bird_y = (ph - ground_h) as f64 * 0.4;
        g
    }

    fn resize(&mut self, pw: usize, ph: usize) {
        *self = Game {
            best: self.best,
            ..Game::new(pw, ph)
        };
    }

    fn sky_h(&self) -> usize {
        self.ph - self.ground_h
    }

    fn flap(&mut self) {
        match self.state {
            State::Ready => {
                self.state = State::Playing;
                self.bird_vy = self.flap_vel;
            }
            State::Playing => {
                self.bird_vy = self.flap_vel;
            }
            State::Dead => {
                let best = self.best;
                self.resize(self.pw, self.ph);
                self.best = best;
            }
            State::Dying => {}
        }
    }

    fn update(&mut self) {
        self.frame += 1;

        match self.state {
            State::Ready => {
                self.bird_y = (self.ph - self.ground_h) as f64 * 0.4
                    + (self.frame as f64 * 0.08).sin() * 3.0 * self.scale;
                self.ground_x += 0.5;
            }
            State::Playing => {
                self.bird_vy += self.gravity;
                self.bird_y += self.bird_vy;
                self.ground_x += self.pipe_speed;

                // Spawn pipes
                let should_spawn = self.pipes.is_empty()
                    || self.pipes.last().unwrap().x < self.pw as f64 - self.pipe_spacing;
                if should_spawn {
                    let sky = self.sky_h() as f64;
                    let margin = self.pipe_gap as f64 * 0.7;
                    let range = sky - margin * 2.0;
                    let center = margin + pseudo_rand(self.frame) * range;
                    self.pipes.push(Pipe {
                        x: self.pw as f64 + 2.0,
                        gap_center: center,
                        scored: false,
                    });
                }

                // Move pipes
                for p in &mut self.pipes {
                    p.x -= self.pipe_speed;
                    if !p.scored && p.x + (self.pipe_w as f64) < self.bird_x {
                        p.scored = true;
                        self.score += 1;
                    }
                }
                self.pipes.retain(|p| p.x + self.pipe_w as f64 + 5.0 > 0.0);

                // Collision
                if self.check_collision() {
                    self.state = State::Dying;
                    self.bird_vy = self.flap_vel * 0.6;
                    if self.score > self.best {
                        self.best = self.score;
                    }
                }
            }
            State::Dying => {
                self.bird_vy += self.gravity;
                self.bird_y += self.bird_vy;
                if self.bird_y >= self.sky_h() as f64 - 3.0 {
                    self.bird_y = self.sky_h() as f64 - 3.0;
                    self.state = State::Dead;
                    self.dead_timer = 0;
                }
            }
            State::Dead => {
                self.dead_timer += 1;
            }
        }
    }

    fn check_collision(&self) -> bool {
        let bx = self.bird_x;
        let by = self.bird_y;
        let half_w = 2.0 * self.scale;
        let half_h = 1.5 * self.scale;

        // Ground / ceiling
        if by + half_h >= self.sky_h() as f64 || by - half_h < 0.0 {
            return true;
        }

        for p in &self.pipes {
            let px = p.x;
            let pw = self.pipe_w as f64;
            let gap_top = p.gap_center - self.pipe_gap as f64 / 2.0;
            let gap_bot = p.gap_center + self.pipe_gap as f64 / 2.0;

            if bx + half_w > px && bx - half_w < px + pw {
                if by - half_h < gap_top || by + half_h > gap_bot {
                    return true;
                }
            }
        }
        false
    }

    fn draw(&self, buf: &mut PixelBuf) {
        self.draw_sky(buf);
        self.draw_hills(buf);
        self.draw_pipes(buf);
        self.draw_ground(buf);
        self.draw_bird(buf);
        self.draw_score(buf);

        if self.state == State::Ready {
            self.draw_title(buf);
        }
        if self.state == State::Dead && self.dead_timer > 15 {
            self.draw_game_over(buf);
        }
    }

    fn draw_sky(&self, buf: &mut PixelBuf) {
        let sky_h = self.sky_h();
        for y in 0..sky_h {
            let t = (y as u16 * 256) / sky_h.max(1) as u16;
            let c = Rgb::lerp(SKY_TOP, SKY_BOT, t);
            for x in 0..self.pw {
                buf.set(x as i32, y as i32, c);
            }
        }
    }

    fn draw_hills(&self, buf: &mut PixelBuf) {
        let base = self.sky_h() as i32;
        // Far hills
        for x in 0..self.pw as i32 {
            let fx = (x as f64 + self.ground_x * 0.2) * 0.04;
            let h = (fx.sin() * 6.0 + (fx * 1.7).sin() * 3.0) * self.scale;
            let top = base - h as i32 - (4.0 * self.scale) as i32;
            for y in top..base {
                buf.set(x, y, HILL_FAR);
            }
        }
        // Near hills
        for x in 0..self.pw as i32 {
            let fx = (x as f64 + self.ground_x * 0.4) * 0.06;
            let h = (fx.sin() * 4.0 + (fx * 2.3).sin() * 2.0) * self.scale;
            let top = base - h as i32 - (2.0 * self.scale) as i32;
            for y in top..base {
                buf.set(x, y, HILL_NEAR);
            }
        }
    }

    fn draw_ground(&self, buf: &mut PixelBuf) {
        let gy = self.sky_h() as i32;
        // Grass strip
        for x in 0..self.pw as i32 {
            let alt = ((x as f64 + self.ground_x) as i32 / 3) % 2 == 0;
            buf.set(x, gy, if alt { GRASS } else { GRASS_LIGHT });
            buf.set(x, gy + 1, GRASS);
        }
        // Dirt
        for y in (gy + 2)..self.ph as i32 {
            for x in 0..self.pw as i32 {
                let stripe = ((x as f64 + self.ground_x * 0.8) as i32 + (y - gy) * 2) % 12 < 6;
                buf.set(x, y, if stripe { DIRT } else { DIRT_DARK });
            }
        }
    }

    fn draw_pipes(&self, buf: &mut PixelBuf) {
        let cap_extra = (2.0 * self.scale).max(1.0) as i32;
        let cap_h = (3.0 * self.scale).max(2.0) as i32;
        let pw = self.pipe_w as i32;

        for pipe in &self.pipes {
            let px = pipe.x as i32;
            let gap_top = (pipe.gap_center - self.pipe_gap as f64 / 2.0) as i32;
            let gap_bot = (pipe.gap_center + self.pipe_gap as f64 / 2.0) as i32;

            // Top pipe body
            for x in 0..pw {
                let c = pipe_shade(x, pw);
                for y in 0..gap_top - cap_h {
                    buf.set(px + x, y, c);
                }
            }
            // Top pipe cap
            for x in -cap_extra..(pw + cap_extra) {
                let c = pipe_shade(x + cap_extra, pw + cap_extra * 2);
                for y in (gap_top - cap_h)..gap_top {
                    buf.set(px + x, y, c);
                }
                // Cap edge darkening
                buf.set(px + x, gap_top - cap_h, CAP_DARK);
                buf.set(px + x, gap_top - 1, CAP_DARK);
            }

            // Bottom pipe cap
            for x in -cap_extra..(pw + cap_extra) {
                let c = pipe_shade(x + cap_extra, pw + cap_extra * 2);
                for y in gap_bot..(gap_bot + cap_h) {
                    buf.set(px + x, y, c);
                }
                buf.set(px + x, gap_bot, CAP_DARK);
                buf.set(px + x, gap_bot + cap_h - 1, CAP_DARK);
            }
            // Bottom pipe body
            for x in 0..pw {
                let c = pipe_shade(x, pw);
                for y in (gap_bot + cap_h)..self.sky_h() as i32 {
                    buf.set(px + x, y, c);
                }
            }
        }
    }

    fn draw_bird(&self, buf: &mut PixelBuf) {
        let cx = self.bird_x as i32;
        let cy = self.bird_y as i32;
        let s = self.scale;

        // Determine tilt for visual (shift pixels up/down)
        let tilt = (self.bird_vy / (3.0 * s)).clamp(-1.0, 1.0) as i32;

        // Body core
        let bw = (3.0 * s).max(2.0) as i32;
        let bh = (2.0 * s).max(2.0) as i32;
        buf.fill_rect(cx - bw, cy - bh, bw * 2 + 1, bh * 2, BIRD_Y);

        // Highlight (top of body)
        buf.fill_rect(
            cx - bw + 1,
            cy - bh,
            bw * 2 - 2,
            1.max((s * 0.8) as i32),
            BIRD_HI,
        );

        // Wing
        let wing_y_off = if self.frame % 8 < 4 { -1 } else { 1 };
        let wing_h = (1.5 * s).max(1.0) as i32;
        let wing_w = (2.0 * s).max(1.0) as i32;
        buf.fill_rect(
            cx - bw + 1,
            cy + wing_y_off + tilt,
            wing_w,
            wing_h,
            BIRD_WING,
        );

        // Eye
        let ex = cx + bw - (1.5 * s) as i32;
        let ey = cy - bh + (1.0 * s).max(1.0) as i32;
        let eye_r = (0.8 * s).max(1.0) as i32;
        buf.fill_rect(ex, ey, eye_r + 1, eye_r + 1, BIRD_EYE);
        buf.set(ex + eye_r, ey + eye_r, BIRD_PUPIL);
        if s >= 1.5 {
            buf.set(ex + eye_r - 1, ey + eye_r, BIRD_PUPIL);
        }

        // Beak
        let beak_x = cx + bw;
        let beak_y = cy - (0.5 * s) as i32 + tilt;
        let beak_w = (2.5 * s).max(2.0) as i32;
        let beak_h = (1.5 * s).max(1.0) as i32;
        buf.fill_rect(beak_x, beak_y, beak_w, beak_h / 2 + 1, BIRD_BEAK_HI);
        buf.fill_rect(
            beak_x,
            beak_y + beak_h / 2 + 1,
            beak_w,
            beak_h / 2,
            BIRD_BEAK,
        );

        // Tail
        let tail_w = (1.5 * s).max(1.0) as i32;
        buf.fill_rect(cx - bw - tail_w, cy - 1 + tilt, tail_w, 2, BIRD_WING);
    }

    fn draw_score(&self, buf: &mut PixelBuf) {
        draw_number(buf, self.pw as i32 / 2, 4, self.score, WHITE);
        if self.show_hud {
            self.draw_tuning_hud(buf);
        }
    }

    fn draw_tuning_hud(&self, buf: &mut PixelBuf) {
        // Show tuning values at bottom-right using pixel digits
        // G=gravity  F=flap  S=speed
        // Display as integers (value * 100) for readability
        let g_val = (self.gravity / self.scale * 100.0) as u32;
        let f_val = (-self.flap_vel / self.scale * 100.0) as u32;
        let s_val = (self.pipe_speed / (self.pw as f64 / 80.0).max(0.8) * 100.0) as u32;

        let y = self.ph as i32 - self.ground_h as i32 - 8;
        let x_base = self.pw as i32 - 30;

        // G:value
        draw_number(buf, x_base + 6, y, g_val, Rgb(180, 180, 255));
        // F:value
        draw_number(buf, x_base + 6, y - 7, f_val, Rgb(255, 180, 180));
        // S:value
        draw_number(buf, x_base + 6, y - 14, s_val, Rgb(180, 255, 180));
    }

    fn tune_gravity(&mut self, delta: f64) {
        self.show_hud = true;
        self.gravity = (self.gravity + delta * self.scale).max(0.05 * self.scale);
    }

    fn tune_flap(&mut self, delta: f64) {
        self.show_hud = true;
        self.flap_vel = (self.flap_vel + delta * self.scale).min(-0.5 * self.scale);
    }

    fn tune_speed(&mut self, delta: f64) {
        self.show_hud = true;
        let base = (self.pw as f64 / 80.0).max(0.8);
        self.pipe_speed = (self.pipe_speed + delta * base).max(0.2 * base);
    }

    fn draw_title(&self, buf: &mut PixelBuf) {
        let cx = self.pw as i32 / 2;
        let cy = self.ph as i32 / 4;
        // "FLAP" in big blocky letters
        let text = "FLAPPY";
        let char_w = (4.0 * self.scale) as i32;
        let char_h = (6.0 * self.scale) as i32;
        let total_w = text.len() as i32 * char_w;
        let sx = cx - total_w / 2;

        for (i, _) in text.chars().enumerate() {
            let bx = sx + i as i32 * char_w;
            buf.fill_rect(bx, cy, char_w - 1, char_h, BIRD_Y);
            buf.fill_rect(bx, cy, char_w - 1, 1, BIRD_HI);
        }

        // Subtitle
        let sub_y = cy + char_h + 4;
        let msg = "SPACE TO FLAP";
        let msg_w = msg.len() as i32 * 4;
        let msg_x = cx - msg_w / 2;
        for (i, ch) in msg.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let bx = msg_x + i as i32 * 4;
            // Simple 3-pixel-tall block for each char
            buf.fill_rect(bx, sub_y, 3, 3, WHITE);
        }
    }

    fn draw_game_over(&self, buf: &mut PixelBuf) {
        let cx = self.pw as i32 / 2;
        let cy = self.ph as i32 / 2;
        let panel_w = (40.0 * self.scale).max(30.0) as i32;
        let panel_h = (20.0 * self.scale).max(16.0) as i32;

        // Dark overlay
        for y in 0..self.ph {
            for x in 0..self.pw {
                let c = buf.get(x, y);
                buf.set(x as i32, y as i32, Rgb(c.0 / 2, c.1 / 2, c.2 / 2));
            }
        }

        // Panel background
        let px = cx - panel_w / 2;
        let py = cy - panel_h / 2;
        buf.fill_rect(px - 1, py - 1, panel_w + 2, panel_h + 2, SHADOW);
        buf.fill_rect(px, py, panel_w, panel_h, DIRT);
        buf.fill_rect(px + 1, py + 1, panel_w - 2, panel_h - 2, Rgb(220, 195, 120));

        // Score
        draw_number(buf, cx, py + 4, self.score, WHITE);

        // Best
        draw_number(buf, cx, py + 12, self.best, BIRD_Y);
    }
}

fn pipe_shade(x: i32, total_w: i32) -> Rgb {
    if total_w <= 1 {
        return PIPE_M;
    }
    let t = (x as f64 / (total_w - 1) as f64 * 256.0) as u16;
    if t < 64 {
        Rgb::lerp(PIPE_L, PIPE_M, (t * 4).min(256))
    } else if t < 100 {
        Rgb::lerp(PIPE_M, PIPE_HI, ((t - 64) * 7).min(256))
    } else if t < 160 {
        Rgb::lerp(PIPE_HI, PIPE_R, ((t - 100) * 4).min(256))
    } else {
        Rgb::lerp(PIPE_R, PIPE_L, ((t - 160) * 3).min(256))
    }
}

fn pseudo_rand(seed: u64) -> f64 {
    let x = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bits = (x >> 33) ^ x;
    (bits % 1000) as f64 / 1000.0
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut out = stdout();
    execute!(
        out,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::DisableLineWrap,
    )?;

    let cleanup = |out: &mut io::Stdout| -> io::Result<()> {
        execute!(
            out,
            terminal::LeaveAlternateScreen,
            cursor::Show,
            terminal::EnableLineWrap,
        )?;
        terminal::disable_raw_mode()
    };

    let (cols, rows) = terminal::size()?;
    let pw = cols as usize;
    let ph = rows as usize * 2;

    let mut buf = PixelBuf::new(pw, ph);
    let mut game = Game::new(pw, ph);

    let frame_dur = Duration::from_millis(33); // ~30 fps

    loop {
        let frame_start = Instant::now();

        // Input
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        cleanup(&mut out)?;
                        return Ok(());
                    }
                    KeyCode::Char(' ') | KeyCode::Up | KeyCode::Enter => {
                        game.flap();
                    }
                    // Tuning: a/z = gravity, s/x = flap, d/c = speed
                    KeyCode::Char('a') => game.tune_gravity(0.02),
                    KeyCode::Char('z') => game.tune_gravity(-0.02),
                    KeyCode::Char('s') => game.tune_flap(0.2), // more negative = stronger
                    KeyCode::Char('x') => game.tune_flap(-0.2),
                    KeyCode::Char('d') => game.tune_speed(0.1),
                    KeyCode::Char('c') => game.tune_speed(-0.1),
                    _ => {}
                },
                Event::Resize(c, r) => {
                    let npw = c as usize;
                    let nph = r as usize * 2;
                    buf.resize(npw, nph);
                    game.resize(npw, nph);
                }
                _ => {}
            }
        }

        // Update
        game.update();

        // Render
        game.draw(&mut buf);
        buf.render(&mut out)?;

        // Frame pacing
        let elapsed = frame_start.elapsed();
        if elapsed < frame_dur {
            std::thread::sleep(frame_dur - elapsed);
        }
    }
}
