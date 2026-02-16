use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{self, Color as CColor},
    terminal,
};
use fundsp::prelude32 as dsp;
use rodio::{OutputStream, OutputStreamHandle, Sink, buffer::SamplesBuffer};
use std::io::{self, Write, stdout};
use std::time::{Duration, Instant};

// ── Sounds ──────────────────────────────────────────────────────────────────
const SAMPLE_RATE: u32 = 44_100;
const DEATH_DURATION: f32 = 0.5;

struct Audio {
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl Audio {
    fn new() -> Result<Self, rodio::StreamError> {
        let (stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            handle,
        })
    }
}

fn play_death(audio: &Audio) {
    let samples = generate_death_samples(SAMPLE_RATE, DEATH_DURATION);
    play_samples(audio, samples);
}

fn play_flap(audio: &Audio) {
    let samples = generate_flap_samples(SAMPLE_RATE);
    play_samples(audio, samples);
}

fn play_score(audio: &Audio) {
    let samples = generate_score_samples(SAMPLE_RATE);
    play_samples(audio, samples);
}

fn play_whoosh(audio: &Audio) {
    let samples = generate_whoosh_samples(SAMPLE_RATE);
    play_samples(audio, samples);
}

fn play_samples(audio: &Audio, samples: Vec<f32>) {
    if let Ok(sink) = Sink::try_new(&audio.handle) {
        let source = SamplesBuffer::new(1, SAMPLE_RATE, samples);
        sink.append(source);
        sink.detach();
    }
}

fn generate_death_samples(sample_rate: u32, duration: f32) -> Vec<f32> {
    let mut node = (dsp::lfo(|t: f32| dsp::lerp(400.0, 80.0, (t / 0.4).min(1.0))) >> dsp::saw())
        * dsp::lfo(|t: f32| dsp::lerp(0.15, 0.0, (t / duration).min(1.0)));
    render_mono(&mut node, sample_rate, duration)
}

fn generate_flap_samples(sample_rate: u32) -> Vec<f32> {
    let duration = 0.12;
    let mut node = (dsp::lfo(|t: f32| {
        if t < 0.08 {
            dsp::xerp(400.0, 800.0, (t / 0.08).min(1.0))
        } else {
            800.0
        }
    }) >> dsp::sine())
        * dsp::lfo(|t: f32| dsp::xerp(0.15, 0.001, (t / duration).min(1.0)));
    render_mono(&mut node, sample_rate, duration)
}

fn generate_score_samples(sample_rate: u32) -> Vec<f32> {
    const NOTES: [f32; 2] = [520.0, 680.0];
    let note_gap = 0.1f32;
    let note_len = 0.15f32;
    let total_duration = note_gap * (NOTES.len() as f32 - 1.0) + note_len;
    let total_samples = (sample_rate as f32 * total_duration) as usize;
    let mut samples = vec![0.0f32; total_samples];

    for (idx, freq) in NOTES.iter().enumerate() {
        let start = (note_gap * idx as f32 * sample_rate as f32) as usize;
        let mut node = dsp::sine_hz(*freq)
            * dsp::lfo(|t: f32| dsp::xerp(0.12, 0.001, (t / note_len).min(1.0)));
        let tone = render_mono(&mut node, sample_rate, note_len);
        for (i, s) in tone.into_iter().enumerate() {
            let target = start + i;
            if target < total_samples {
                samples[target] += s;
            }
        }
    }

    samples
}

fn generate_whoosh_samples(sample_rate: u32) -> Vec<f32> {
    let duration = 0.08;
    let mut node = (dsp::noise() >> dsp::bandpass_hz(1200.0, 0.5) >> dsp::mul(0.1))
        * dsp::lfo(|t: f32| dsp::xerp(0.3, 0.001, (t / duration).min(1.0)));
    render_mono(&mut node, sample_rate, duration)
}

fn render_mono(node: &mut dyn dsp::AudioUnit, sample_rate: u32, duration: f32) -> Vec<f32> {
    node.set_sample_rate(sample_rate as f64);
    node.reset();

    let sample_count = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        samples.push(node.get_mono());
    }
    samples
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

const FLAPPY_LOGO: [&str; 7] = [
    " XXXXXXXXX  XXXX         XXXXXXXXX   XXXXXXXXX   XXXXXXXXX  XXX      XXX",
    "XXXXXXXXXXX XXXX        XXXXXXXXXXX XXXXXXXXXXX XXXXXXXXXXX XXXX    XXXX",
    "XXXX        XXXX        XXXX   XXXX XXXX   XXXX XXXX   XXXX  XXXX  XXXX",
    "XXXXXXXX    XXXX        XXXXXXXXXXX XXXXXXXXXXX XXXXXXXXXXX   XXXXXXXX",
    "XXXXXXXX    XXXX        XXXXXXXXXXX XXXXXXXXXX  XXXXXXXXXX      XXXX",
    "XXXX        XXXXXXXXXXX XXXX   XXXX XXXX        XXXX            XXXX",
    "XXXX         XXXXXXXXXX XXXX   XXXX XXXX        XXXX            XXXX",
];

const FLAPPY_LETTER_PITCH: i32 = 12;
const FLAPPY_LETTER_GAP: i32 = 2;
const FLAPPY_LETTER_COUNT: i32 = 6;

fn flappy_logo_width(scale: i32) -> i32 {
    let s = scale.max(1);
    let base = FLAPPY_LOGO[0].chars().count() as i32 * s;
    let extra = (FLAPPY_LETTER_COUNT - 1) * FLAPPY_LETTER_GAP * s;
    base + extra
}

fn draw_flappy_logo(buf: &mut PixelBuf, x: i32, y: i32, scale: i32) {
    let s = scale.max(1);

    draw_flappy_logo_flat(buf, x - 1, y - 1, s, SHADOW);
    draw_flappy_logo_flat(buf, x, y - 1, s, SHADOW);
    draw_flappy_logo_flat(buf, x + 2, y, s, SHADOW);
    draw_flappy_logo_flat(buf, x, y + 2, s, SHADOW);
    draw_flappy_logo_flat(buf, x + 2, y + 2, s, SHADOW);

    // First pass: light yellow.
    draw_flappy_logo_flat(buf, x, y, s, BIRD_HI);

    // Second pass: darker yellow offset for a 3D look.
    draw_flappy_logo_flat(buf, x + 1, y + 1, s, BIRD_Y);
}

fn draw_flappy_logo_flat(buf: &mut PixelBuf, x: i32, y: i32, s: i32, color: Rgb) {
    // Draw each source row as two pixel rows (sub-pixel friendly).
    for (row, line) in FLAPPY_LOGO.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == 'X' {
                let col_i32 = col as i32;
                let letter_idx = (col_i32 / FLAPPY_LETTER_PITCH).clamp(0, FLAPPY_LETTER_COUNT - 1);
                let px = x + col_i32 * s + letter_idx * FLAPPY_LETTER_GAP * s;
                let py = y + row as i32 * (2 * s);
                buf.fill_rect(px, py, s, s, color);
                buf.fill_rect(px, py + s, s, s, color);
            }
        }
    }
}

fn glyph_4x6(ch: char) -> [u8; 6] {
    match ch {
        'A' => [
            0b01000000, 0b10100000, 0b10100000, 0b11100000, 0b10100000, 0b00000000,
        ],
        'C' => [
            0b01000000, 0b10100000, 0b10000000, 0b10100000, 0b01000000, 0b00000000,
        ],
        'E' => [
            0b11100000, 0b10000000, 0b11000000, 0b10000000, 0b11100000, 0b00000000,
        ],
        'F' => [
            0b11100000, 0b10000000, 0b11100000, 0b10000000, 0b10000000, 0b00000000,
        ],
        'L' => [
            0b10000000, 0b10000000, 0b10000000, 0b10000000, 0b11100000, 0b00000000,
        ],
        'O' => [
            0b01000000, 0b10100000, 0b10100000, 0b10100000, 0b01000000, 0b00000000,
        ],
        'P' => [
            0b11000000, 0b10100000, 0b11000000, 0b10000000, 0b10000000, 0b00000000,
        ],
        'S' => [
            0b01100000, 0b10000000, 0b01000000, 0b00100000, 0b11000000, 0b00000000,
        ],
        'T' => [
            0b11100000, 0b01000000, 0b01000000, 0b01000000, 0b01000000, 0b00000000,
        ],
        ' ' => [0; 6],
        _ => [0; 6],
    }
}

fn text_width_4x6(text: &str, scale: i32) -> i32 {
    if text.is_empty() {
        0
    } else {
        (text.chars().count() as i32 * 5 - 1) * scale.max(1)
    }
}

fn draw_text_4x6(buf: &mut PixelBuf, x: i32, y: i32, text: &str, color: Rgb, scale: i32) {
    let s = scale.max(1);
    let mut cursor_x = x;

    for ch in text.chars() {
        let rows = glyph_4x6(ch);
        for (row, bits) in rows.iter().enumerate() {
            for col in 0..4 {
                if ((bits >> (7 - col)) & 1) == 1 {
                    buf.fill_rect(cursor_x + col * s, y + row as i32 * s, s, s, color);
                }
            }
        }
        cursor_x += 5 * s;
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

enum GameEvent {
    Flap,
    Score,
    Whoosh,
    Death,
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

    fn flap(&mut self) -> Option<GameEvent> {
        match self.state {
            State::Ready => {
                self.state = State::Playing;
                self.bird_vy = self.flap_vel;
                Some(GameEvent::Flap)
            }
            State::Playing => {
                self.bird_vy = self.flap_vel;
                Some(GameEvent::Flap)
            }
            State::Dead => {
                let best = self.best;
                self.resize(self.pw, self.ph);
                self.best = best;
                None
            }
            State::Dying => None,
        }
    }

    fn update(&mut self) -> Vec<GameEvent> {
        self.frame += 1;
        let mut events = Vec::new();

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
                    events.push(GameEvent::Whoosh);
                }

                // Move pipes
                for p in &mut self.pipes {
                    p.x -= self.pipe_speed;
                    if !p.scored && p.x + (self.pipe_w as f64) < self.bird_x {
                        p.scored = true;
                        self.score += 1;
                        events.push(GameEvent::Score);
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
                    events.push(GameEvent::Death);
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
        events
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

        // Body: square with cut corners (chamfered rectangle)
        let bw = (3.0 * s).max(2.0) as i32;
        let bh = (2.0 * s).max(2.0) as i32;
        let body_top = cy - bh;
        let total_h = bh * 2;
        let corner = (1.0 * s).max(1.0) as i32; // how many rows to chamfer
        for row in 0..total_h {
            let y = body_top + row;
            // Cut corners: reduce width in the first/last `corner` rows
            let inset = if row < corner {
                corner - row
            } else if row >= total_h - corner {
                row - (total_h - corner) + 1
            } else {
                0
            };
            let half_w = bw - inset;
            if half_w > 0 {
                buf.fill_rect(cx - half_w, y, half_w * 2 + 1, 1, BIRD_Y);
            }
        }

        // Highlight (top rows of body)
        let hi_rows = 1.max((s * 0.8) as i32);
        for row in 1..(1 + hi_rows).min(total_h / 2) {
            let y = body_top + row;
            let inset = if row < corner { corner - row } else { 0 };
            let half_w = bw - inset - 1;
            if half_w > 0 {
                buf.fill_rect(cx - half_w, y, half_w * 2 + 1, 1, BIRD_HI);
            }
        }

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

        // Beak as an isosceles triangle: base on the left, point at center-right
        let beak_x = cx + bw;
        let beak_w = (2.5 * s).max(2.0) as i32;
        let beak_half_h = (0.75 * s).max(1.0) as i32;
        let beak_total_h = beak_half_h * 2 + 1;
        let beak_center_y = cy + tilt;
        let beak_top = beak_center_y - beak_half_h;
        for row in 0..beak_total_h {
            // Distance from center row: 0 at center, beak_half_h at edges
            let dist = (row - beak_half_h).abs();
            // Full width at center (dist=0), narrows to 1 at edges
            let frac = 1.0 - dist as f64 / (beak_half_h + 1) as f64;
            let w = (frac * beak_w as f64).max(1.0) as i32;
            let color = if row <= beak_half_h {
                BIRD_BEAK_HI
            } else {
                BIRD_BEAK
            };
            buf.fill_rect(beak_x, beak_top + row, w, 1, color);
        }

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
        let cy = self.ph as i32 / 3;
        let title_scale = 1;
        let title_w = flappy_logo_width(title_scale);
        let title_h = FLAPPY_LOGO.len() as i32 * title_scale * 2;
        let title_x = cx - title_w / 2;

        draw_flappy_logo(buf, title_x, cy, title_scale);

        // Subtitle in a white box with normal-size dark text.
        let msg = "SPACE TO FLAP";
        let msg_scale = 1;
        let msg_w = text_width_4x6(msg, msg_scale);
        let msg_h = 6 * msg_scale;
        let pad_x = 2;
        let pad_y = 1;
        let box_w = msg_w + pad_x * 2;
        let box_h = msg_h + pad_y * 2;
        let box_x = cx - box_w / 2;
        let box_y = cy + title_h + 4;

        buf.fill_rect(box_x - 1, box_y - 1, box_w + 2, box_h + 1, SHADOW);
        buf.fill_rect(box_x, box_y, box_w, box_h - 1, WHITE);
        draw_text_4x6(
            buf,
            box_x + pad_x,
            box_y + pad_y,
            msg,
            BIRD_PUPIL,
            msg_scale,
        );
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
    let audio = Audio::new().ok();

    let frame_dur = Duration::from_millis(33); // ~30 fps
    let mut event_buf = Vec::new();

    loop {
        let frame_start = Instant::now();
        event_buf.clear();

        // Input
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        cleanup(&mut out)?;
                        return Ok(());
                    }
                    KeyCode::Char(' ') | KeyCode::Up | KeyCode::Enter => {
                        if let Some(event) = game.flap() {
                            event_buf.push(event);
                        }
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
        event_buf.extend(game.update());

        if let Some(audio) = audio.as_ref() {
            for event in event_buf.drain(..) {
                match event {
                    GameEvent::Flap => play_flap(audio),
                    GameEvent::Score => play_score(audio),
                    GameEvent::Whoosh => play_whoosh(audio),
                    GameEvent::Death => play_death(audio),
                }
            }
        } else {
            event_buf.clear();
        }

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
