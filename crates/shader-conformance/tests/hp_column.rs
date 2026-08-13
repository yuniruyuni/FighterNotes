//! HP バーの列分類が、shader と参照実装で同じ答えになることを確かめる。
//!
//! アプリはブラウザの WebGPU でこの shader を走らせる。ここでは同じ WGSL を
//! ソフトウェア実装 (lavapipe) で走らせ、`classify_columns` の答えと 1 列ずつ
//! 突き合わせる。
//!
//! ソフトウェア実装が一致することは、実機の GPU が一致することを保証しない。
//! 除算の丸めは処理系ごとに違いうる。この試験が守るのは shader の筋であって、
//! 数値の一致は実機での突き合わせで別途確かめている。

use hud_vision::frame_features::{hp_column_scan, hp_columns_from_strip};
use wgpu::util::DeviceExt as _;

const WIDTH: u32 = 1920;
const STRIP_HEIGHT: u32 = 70;
const SHADER: &str = include_str!("../../hud-vision/shaders/hp_column.wgsl");

/// 一面の色、白い枠、斜めの縞。斜めの縞は走査の傾きを踏む。
fn strips() -> Vec<(&'static str, Vec<u8>)> {
    let mut cases = Vec::new();
    for (name, rgb) in [
        ("赤一面", [220u8, 40, 40]),
        ("青一面", [40, 90, 220]),
        ("白一面", [250, 250, 250]),
        ("橙一面", [230, 130, 30]),
        ("暗い一面", [12, 10, 14]),
    ] {
        cases.push((name, painted(|_, _| rgb)));
    }
    cases.push((
        "縦縞",
        painted(|x, _| {
            if x % 7 < 3 {
                [250, 250, 250]
            } else {
                [200, 30, 30]
            }
        }),
    ));
    cases.push((
        "斜め縞",
        painted(|x, y| {
            if (x + y * 4 / 3) % 11 < 4 {
                [240, 240, 240]
            } else {
                [30, 60, 200]
            }
        }),
    ));
    // 閾値の周りを 1 ずつ跨がせる。列の判定は 22 行の多数決なので、走査が
    // 斜めに動く幅より広い帯にして、1 列が 1 色だけを見るようにする。
    // 細かい絵だと 1 画素の違いが票に埋もれ、閾値の書き換えを見逃す。
    cases.push((
        "白の境目",
        banded(|band| {
            let value = (176 + band % 12) as u8;
            [value, value, value]
        }),
    ));
    cases.push((
        "黄白の境目",
        banded(|band| {
            [
                (160 + band % 12) as u8,
                (145 + band % 12) as u8,
                (95 + band % 12) as u8,
            ]
        }),
    ));
    cases.push((
        "彩度の境目",
        banded(|band| {
            // 最大値を固定して最小値を動かすと、彩度だけが刻まれる。
            [200, (116 + band % 16) as u8, (116 + band % 16) as u8]
        }),
    ));
    cases.push(("明度の境目", banded(|band| [(54 + band % 14) as u8, 0, 0])));
    cases.push((
        "色相の境目",
        banded(|band| {
            // 中間チャンネルを動かすと色相だけが刻まれる。GPU の除算が
            // 効くのはここ。
            [200, (40 + band % 60) as u8, 40]
        }),
    ));
    cases.push((
        "色の階調",
        painted(|x, y| {
            [
                (x % 256) as u8,
                ((x / 3 + y * 7) % 256) as u8,
                ((x / 5 + y * 3) % 256) as u8,
            ]
        }),
    ));
    cases
}

/// 走査の斜めより広い帯で塗る。1 列が 1 色だけを見る絵になる。
fn banded(color: impl Fn(u32) -> [u8; 3]) -> Vec<u8> {
    painted(|x, _| color(x / 24))
}

fn painted(color: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
    let mut rgba = vec![0u8; (WIDTH * STRIP_HEIGHT * 4) as usize];
    for y in 0..STRIP_HEIGHT {
        for x in 0..WIDTH {
            let at = ((y * WIDTH + x) * 4) as usize;
            rgba[at..at + 3].copy_from_slice(&color(x, y));
            rgba[at + 3] = 255;
        }
    }
    rgba
}

#[test]
fn the_shader_classifies_columns_exactly_as_the_reference_does() {
    let Some(gpu) = Gpu::open() else {
        panic!(
            "ソフトウェアの Vulkan 実装が見つからない。lavapipe (mesa-vulkan-drivers / vulkan-swrast) を入れること"
        );
    };

    for (name, strip) in strips() {
        let from_shader = gpu.classify(&strip);
        for (side_index, side) in ["p1", "p2"].into_iter().enumerate() {
            let expected = hp_columns_from_strip(&strip, side);
            let width = expected.len();
            let actual = &from_shader[side_index * width..(side_index + 1) * width];

            let differing: Vec<usize> = (0..width)
                .filter(|&column| u32::from(expected[column]) != actual[column])
                .collect();
            assert!(
                differing.is_empty(),
                "{name} の {side} で {} 列ずれた。最初は列 {} (参照 {} / shader {})",
                differing.len(),
                differing[0],
                expected[differing[0]],
                actual[differing[0]],
            );
        }
    }
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    columns_per_frame: u32,
}

impl Gpu {
    fn open() -> Option<Self> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hp_column"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("hp_column"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let columns_per_frame = hp_column_scan("p1")[1];
        Some(Self {
            device,
            queue,
            pipeline,
            columns_per_frame,
        })
    }

    /// strip 1 枚を分類し、p1・p2 の順に並んだ列の色を返す。
    fn classify(&self, strip: &[u8]) -> Vec<u32> {
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("strip"),
                size: wgpu::Extent3d {
                    width: WIDTH,
                    height: STRIP_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            strip,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // uniform は 16 バイト単位。走査の形も 4 つずつに詰める。
        let mut scans = [0u32; 16];
        for (side_index, side) in ["p1", "p2"].into_iter().enumerate() {
            let scan = hp_column_scan(side);
            scans[side_index * 8..side_index * 8 + 4].copy_from_slice(&scan[..4]);
            scans[side_index * 8 + 4..side_index * 8 + 6].copy_from_slice(&scan[4..6]);
        }
        let scans_buffer = self.buffer(
            "scans",
            bytemuck::cast_slice(&scans),
            wgpu::BufferUsages::UNIFORM,
        );
        let sv = hud_vision::frame_features::hsv_sv_table();
        let sv_buffer = self.buffer("sv", bytemuck::cast_slice(&sv), wgpu::BufferUsages::STORAGE);
        let norm = hud_vision::frame_features::channel_norm_table();
        let norm_buffer = self.buffer(
            "norm",
            bytemuck::cast_slice(&norm),
            wgpu::BufferUsages::STORAGE,
        );

        let bytes = (self.columns_per_frame * 2 * 4) as u64;
        let columns = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("columns"),
            size: bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging"),
            size: bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scans_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: columns.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: sv_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: norm_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(self.columns_per_frame.div_ceil(64), 1, 2);
        }
        encoder.copy_buffer_to_buffer(&columns, 0, &staging, 0, bytes);
        self.queue.submit([encoder.finish()]);

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("読み戻し");
        let values = bytemuck::cast_slice::<u8, u32>(&slice.get_mapped_range()).to_vec();
        staging.unmap();
        values
    }

    fn buffer(&self, label: &str, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
    }
}
