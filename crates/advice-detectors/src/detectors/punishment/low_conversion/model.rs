pub struct LowReturn {
    pub frame: u32,
    pub round_no: u32,
    pub drop: f32,
    pub input: String,
    pub exact_damage: Option<u32>,
    pub final_scaling_percent: Option<u32>,
}
