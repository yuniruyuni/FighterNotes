//! 正解データ付き合成シーンの描画。
//!
//! SF6 のリプレイ画面を模した簡易世界を world 単位で持ち、カメラ投影後の
//! 480x270 RGBA を生成する。実映像で精度を制限している現象——カメラの
//! パン・ズーム、静止キャラの無モーション、hitstop 中の VFX、接地影——を
//! 決定的に再現し、抽出器の出力を world の正解と比較できるようにする。
//!
//! これは実映像の代替ではなく、アルゴリズムの相対比較用の proxy である。

pub const WIDTH: u32 = 480;
pub const HEIGHT: u32 = 270;

/// 接地線の screen y。フレームメーター除外帯 (0.70..0.86) の下に足と
/// すねが十数 px 残る実映像のレイアウトに合わせる。
pub const GROUND_SCREEN_Y: f32 = 243.0;
/// ステージ半幅(world 単位)。壁とカメラクランプの基準。
pub const STAGE_HALF_WIDTH: f32 = 7.6;
pub const CHAR_HALF_WIDTH: f32 = 0.45;
pub const CHAR_HEIGHT: f32 = 1.8;

/// キャラクターの見た目の動き方。フレーム間差分に映る量を決める。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Animation {
    /// ガード硬直・hitstop。テクスチャ位相が進まず差分に映らない。
    Frozen,
    /// 立ち・しゃがみの待機。上半身だけ緩く位相が進む。
    Subtle,
    /// 歩行・攻撃動作。全身の位相が毎フレーム進む。
    Full,
}

#[derive(Clone, Copy)]
pub struct ActorState {
    /// 足元中心の world x。
    pub x: f32,
    /// 接地からの高さ(world 単位)。0 で接地。
    pub air_y: f32,
    pub anim: Animation,
    /// テクスチャ位相。Frozen では進めない。
    pub phase: u32,
    pub base_color: [f32; 3],
}

impl ActorState {
    pub fn grounded(x: f32, base_color: [f32; 3]) -> Self {
        Self {
            x,
            air_y: 0.0,
            anim: Animation::Full,
            phase: 0,
            base_color,
        }
    }

    pub fn advance_phase(&mut self) {
        match self.anim {
            Animation::Frozen => {}
            Animation::Subtle | Animation::Full => self.phase = self.phase.wrapping_add(1),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Camera {
    pub center_x: f32,
    /// 1 world 単位あたりの pixel 数。ズームアウトで小さくなる。
    pub scale: f32,
}

impl Camera {
    /// SF6 を模した決定的カメラ。中点を追い、距離でズームし、壁でパンを
    /// クランプする。
    pub fn follow(p1_x: f32, p2_x: f32) -> Self {
        let separation = (p2_x - p1_x).abs();
        let scale = (66.0 - 3.3 * separation).clamp(44.0, 64.0);
        let view_half = WIDTH as f32 / 2.0 / scale;
        let limit = (STAGE_HALF_WIDTH - view_half).max(0.0);
        let center_x = ((p1_x + p2_x) / 2.0).clamp(-limit, limit);
        Self { center_x, scale }
    }

    pub fn screen_x(&self, world_x: f32) -> f32 {
        WIDTH as f32 / 2.0 + (world_x - self.center_x) * self.scale
    }

    pub fn world_x(&self, screen_x: f32) -> f32 {
        (screen_x - WIDTH as f32 / 2.0) / self.scale + self.center_x
    }
}

/// hitstop 中に描く打撃エフェクト。
#[derive(Clone, Copy)]
pub struct Spark {
    /// 衝突位置(world x と接地からの高さ)。
    pub world_x: f32,
    pub world_y: f32,
    /// 出現からの経過フレーム。大きさと明るさの脈動に使う。
    pub age: u32,
}

/// 1 フレーム分の正解データ。抽出器の出力と突き合わせる。
#[derive(Clone, Copy)]
pub struct GtFrame {
    /// 足元中心の正規化 screen 座標。
    pub p1_anchor: (f32, f32),
    pub p2_anchor: (f32, f32),
    pub p1_air: bool,
    pub p2_air: bool,
    /// 本体間の world 距離。
    pub separation: f32,
    /// このフレームを精度集計に含めるか。追跡初期化用の助走は除外する。
    pub measure: bool,
    /// 衝突位置の正規化 screen 座標。スパーク表示フレームだけ持つ。
    pub contact: Option<(f32, f32)>,
    /// カメラの正解(中心の world x と scale)。
    pub cam_center_x: f32,
    pub cam_scale: f32,
}

pub struct Scene {
    pub p1: ActorState,
    pub p2: ActorState,
    pub camera: Camera,
    pub sparks: Vec<Spark>,
    pub frame_index: u32,
}

impl Scene {
    pub fn new(p1_x: f32, p2_x: f32) -> Self {
        Self {
            p1: ActorState::grounded(p1_x, [70.0, 130.0, 205.0]),
            p2: ActorState::grounded(p2_x, [205.0, 110.0, 60.0]),
            camera: Camera::follow(p1_x, p2_x),
            sparks: Vec::new(),
            frame_index: 0,
        }
    }

    pub fn ground_truth(&self, measure: bool) -> GtFrame {
        let anchor = |actor: &ActorState| {
            let sx = self.camera.screen_x(actor.x) / WIDTH as f32;
            let sy = (GROUND_SCREEN_Y - actor.air_y * self.camera.scale) / HEIGHT as f32;
            (sx, sy)
        };
        let contact = self.sparks.first().map(|spark| {
            (
                self.camera.screen_x(spark.world_x) / WIDTH as f32,
                (GROUND_SCREEN_Y - spark.world_y * self.camera.scale) / HEIGHT as f32,
            )
        });
        GtFrame {
            p1_anchor: anchor(&self.p1),
            p2_anchor: anchor(&self.p2),
            p1_air: self.p1.air_y > 0.08,
            p2_air: self.p2.air_y > 0.08,
            separation: (self.p2.x - self.p1.x).abs(),
            measure,
            contact,
            cam_center_x: self.camera.center_x,
            cam_scale: self.camera.scale,
        }
    }

    pub fn render(&self) -> Vec<u8> {
        let mut rgba = vec![255u8; WIDTH as usize * HEIGHT as usize * 4];
        let camera = self.camera;
        for py in 0..HEIGHT {
            for px in 0..WIDTH {
                let world_x = camera.world_x(px as f32 + 0.5);
                let mut color = if (py as f32) < GROUND_SCREEN_Y {
                    backdrop_color(camera, world_x, py)
                } else {
                    floor_color(world_x, py)
                };
                shade_shadow(&mut color, &self.p1, camera, px, py);
                shade_shadow(&mut color, &self.p2, camera, px, py);
                paint_actor(&mut color, &self.p1, camera, px, py);
                paint_actor(&mut color, &self.p2, camera, px, py);
                for spark in &self.sparks {
                    paint_spark(&mut color, spark, camera, px, py);
                }
                add_sensor_noise(&mut color, px, py, self.frame_index);
                let index = (py as usize * WIDTH as usize + px as usize) * 4;
                rgba[index] = color[0].clamp(0.0, 255.0) as u8;
                rgba[index + 1] = color[1].clamp(0.0, 255.0) as u8;
                rgba[index + 2] = color[2].clamp(0.0, 255.0) as u8;
            }
        }
        rgba
    }
}

/// 背景。world 座標に固定したテクスチャを持たせ、カメラの動きが
/// 差分として観測される実映像の性質を再現する。
fn backdrop_color(camera: Camera, world_x: f32, py: u32) -> [f32; 3] {
    let depth = py as f32 / GROUND_SCREEN_Y;
    let base = 96.0 - 30.0 * depth;
    let world_y = (GROUND_SCREEN_Y - py as f32) / camera.scale;
    let tone = 20.0 * grid_noise(world_x * 3.0, world_y * 3.0, 11);
    [base + tone, base + 4.0 + tone, base + 10.0 + tone]
}

/// 床。トレーニングステージ風の 1 unit 間隔のグリッド線を持つ。
fn floor_color(world_x: f32, py: u32) -> [f32; 3] {
    let mut base = 66.0 - (py as f32 - GROUND_SCREEN_Y) * 0.5;
    let fractional = world_x - world_x.floor();
    if fractional < 0.06 {
        base += 22.0;
    }
    let tone = 12.0 * grid_noise(world_x * 4.0, py as f32, 29);
    [base + tone, base - 3.0 + tone, base - 8.0 + tone]
}

/// 接地影。空中では小さくなるが位置は足元直下の床に残る。実物に合わせて
/// 中心部は濃くはっきり描く。
fn shade_shadow(color: &mut [f32; 3], actor: &ActorState, camera: Camera, px: u32, py: u32) {
    let band = 0.18 * camera.scale;
    let dy = py as f32 - GROUND_SCREEN_Y;
    if !(0.0..band).contains(&dy) {
        return;
    }
    let shrink = 1.0 / (1.0 + 0.6 * actor.air_y);
    let half_width = 0.55 * camera.scale * shrink * (1.0 - 0.6 * dy / band);
    let dx = (px as f32 - camera.screen_x(actor.x)).abs();
    if dx < half_width {
        let ratio = dx / half_width;
        let strength = 0.40 + 0.45 * ratio * ratio;
        color[0] *= strength;
        color[1] *= strength;
        color[2] *= strength;
    }
}

fn paint_actor(color: &mut [f32; 3], actor: &ActorState, camera: Camera, px: u32, py: u32) {
    let feet_y = GROUND_SCREEN_Y - actor.air_y * camera.scale;
    let top_y = feet_y - CHAR_HEIGHT * camera.scale;
    let left = camera.screen_x(actor.x - CHAR_HALF_WIDTH);
    let right = camera.screen_x(actor.x + CHAR_HALF_WIDTH);
    let fx = px as f32;
    let fy = py as f32;
    if fx < left || fx >= right || fy < top_y || fy >= feet_y {
        return;
    }
    // Subtle は上半身だけ 6 フレームごとに位相を進め、待機動作の小さな
    // モーションを再現する。Frozen は位相が完全に止まる。
    let local_phase = match actor.anim {
        Animation::Frozen => actor.phase,
        Animation::Subtle => {
            if fy < top_y + CHAR_HEIGHT * camera.scale * 0.5 {
                actor.phase / 6
            } else {
                0
            }
        }
        Animation::Full => actor.phase,
    };
    let cell_x = ((fx - left) / 3.0) as i32;
    let cell_y = ((fy - top_y) / 3.0) as i32;
    let tone = 0.72 + 0.55 * (grid_noise_seeded(cell_x, cell_y, local_phase) + 0.5);
    color[0] = actor.base_color[0] * tone;
    color[1] = actor.base_color[1] * tone;
    color[2] = actor.base_color[2] * tone;
}

/// 打撃スパーク。明るく彩度の高い放射状の星形で、既存の effect セル判定
/// (max>=145 かつ max-min>=65)に確実に載る色を使う。実物のヒット
/// エフェクトに合わせて、回転・拡大・明滅を毎フレーム大きく変化させる。
fn paint_spark(color: &mut [f32; 3], spark: &Spark, camera: Camera, px: u32, py: u32) {
    let center_x = camera.screen_x(spark.world_x);
    let center_y = GROUND_SCREEN_Y - spark.world_y * camera.scale;
    let growth = 0.70 + 0.08 * spark.age as f32;
    let radius = 0.38 * camera.scale * growth;
    let dx = px as f32 - center_x;
    let dy = py as f32 - center_y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance >= radius {
        return;
    }
    let angle = dy.atan2(dx) + spark.age as f32 * 0.45;
    let spike = 0.60 + 0.40 * (angle * 6.0).cos().abs();
    let v = distance / (radius * spike);
    if v < 1.0 {
        let flicker = if spark.age.is_multiple_of(2) {
            1.0
        } else {
            0.82
        };
        color[0] = 255.0 * flicker;
        color[1] = (225.0 - 120.0 * v) * flicker;
        color[2] = 60.0 * (1.0 - v);
    }
}

/// 圧縮・センサー由来の微小ノイズ。セル平均後に motion 閾値を越えない
/// 振幅で、完全静止画同士でも差分が 0 にならない実映像の性質を残す。
fn add_sensor_noise(color: &mut [f32; 3], px: u32, py: u32, frame: u32) {
    let h = hash(px.wrapping_mul(374_761_393) ^ py.wrapping_mul(668_265_263) ^ frame);
    let delta = ((h & 0x7) as f32) - 3.5;
    color[0] += delta;
    color[1] += delta;
    color[2] += delta;
}

fn grid_noise(x: f32, y: f32, seed: u32) -> f32 {
    grid_noise_seeded(x.floor() as i32, y.floor() as i32, seed)
}

/// [-0.5, 0.5] の決定的ノイズ。
fn grid_noise_seeded(ix: i32, iy: i32, seed: u32) -> f32 {
    let h = hash(
        (ix as u32)
            .wrapping_mul(374_761_393)
            .wrapping_add((iy as u32).wrapping_mul(668_265_263))
            .wrapping_add(seed.wrapping_mul(2_246_822_519)),
    );
    (h & 0xffff) as f32 / 65_535.0 - 0.5
}

fn hash(mut v: u32) -> u32 {
    v ^= v >> 16;
    v = v.wrapping_mul(0x7feb_352d);
    v ^= v >> 15;
    v = v.wrapping_mul(0x846c_a68b);
    v ^= v >> 16;
    v
}
