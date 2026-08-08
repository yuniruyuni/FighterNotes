pub(super) struct LowReturn {
    pub(super) frame: u32,
    pub(super) round_no: u32,
    pub(super) drop: f32,
    pub(super) input: String,
    pub(super) exact_damage: Option<u32>,
    pub(super) final_scaling_percent: Option<u32>,
}
