pub(super) struct UncertaintyWindow {
    original: Vec<bool>,
    expanded: Vec<bool>,
}

impl UncertaintyWindow {
    pub(super) fn new(values: &[f32], gap_fill: usize) -> Self {
        let original: Vec<_> = values.iter().map(|value| *value < 0.0).collect();
        let expanded = expand_uncertain(&original, gap_fill);
        Self { original, expanded }
    }

    pub(super) fn obscure_neighbors(&self, values: &mut [f32]) {
        for (value, &uncertain) in values.iter_mut().zip(&self.expanded) {
            if uncertain && *value >= 0.0 {
                *value = -1.0;
            }
        }
    }

    /// K.O. 後の確定低値を、元から不確実だったフレームだけへ逆向きに伝える。
    pub(super) fn backward_fill(&self, values: &mut [f32]) {
        let mut next = None;
        for index in (0..values.len()).rev() {
            if self.original[index] {
                if let Some(next) = next {
                    if values[index] >= 0.0 {
                        values[index] = values[index].min(next);
                    }
                }
            } else if !self.expanded[index] {
                next = Some(values[index]);
            }
        }
    }
}

pub(super) fn forward_fill(values: &mut [f32]) {
    let mut previous = -1.0;
    for value in values {
        if *value >= 0.0 {
            previous = *value;
        } else if previous >= 0.0 {
            *value = previous;
        }
    }
}

pub(super) fn expand_uncertain(uncertain: &[bool], gap_fill: usize) -> Vec<bool> {
    let mut expanded = uncertain.to_vec();
    for (index, &is_uncertain) in uncertain.iter().enumerate() {
        if is_uncertain {
            let start = index.saturating_sub(gap_fill);
            let end = (index + gap_fill + 1).min(uncertain.len());
            expanded[start..end].fill(true);
        }
    }
    expanded
}
