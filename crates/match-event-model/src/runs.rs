//! 同じ meter 状態が同じ epoch で続く区間の列挙。
//!
//! 手で添字を進める走査は、進める式そのものが壊れると（増分が消える、
//! 逆向きになる）その場で停止しなくなる。実際に epoch を読めない区間で
//! 解析が固まる不具合を起こしたため、区間の切り出しは添字演算を持たない
//! この一箇所へ寄せる。

use super::MeterState;

/// 連続した同一状態・同一 epoch の区間。両端とも含む添字。
pub struct MeterRun {
    pub start: usize,
    pub end: usize,
    pub epoch: i32,
}

/// `state` が `wanted` であり、かつ epoch を読めて負でない区間を列挙する。
///
/// epoch を読めない位置は区間に入れない。値が変わった位置で区間を切るので、
/// meter のリセットをまたいだ区間は生まれない。
pub fn runs_of(state: &[MeterState], epoch: &[i32], wanted: MeterState) -> Vec<MeterRun> {
    let keyed: Vec<Option<i32>> = state
        .iter()
        .enumerate()
        .map(|(index, value)| {
            (*value == wanted)
                .then(|| epoch.get(index).copied())
                .flatten()
                .filter(|epoch| *epoch >= 0)
        })
        .collect();

    keyed
        .chunk_by(|left, right| left == right && left.is_some())
        .scan(0usize, |start, chunk| {
            let begin = *start;
            *start += chunk.len();
            Some((begin, chunk))
        })
        .filter_map(|(begin, chunk)| {
            let epoch = (*chunk.first()?)?;
            Some(MeterRun {
                start: begin,
                end: begin + chunk.len() - 1,
                epoch,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn epochs(len: usize, value: i32) -> Vec<i32> {
        vec![value; len]
    }

    #[test]
    fn a_run_covers_consecutive_matching_frames() {
        let state = vec![
            MeterState::Free,
            MeterState::Active,
            MeterState::Active,
            MeterState::Free,
        ];

        let runs = runs_of(&state, &epochs(4, 0), MeterState::Active);

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].start, 1);
        assert_eq!(runs[0].end, 2);
        assert_eq!(runs[0].epoch, 0);
    }

    /// epoch が変われば別の区間。meter のリセットをまたいで1つの行動として
    /// 結んではならない。
    #[test]
    fn a_changed_epoch_splits_the_run() {
        let state = vec![MeterState::Active; 4];
        let epoch = vec![0, 0, 1, 1];

        let runs = runs_of(&state, &epoch, MeterState::Active);

        assert_eq!(runs.len(), 2);
        assert_eq!((runs[0].start, runs[0].end, runs[0].epoch), (0, 1, 0));
        assert_eq!((runs[1].start, runs[1].end, runs[1].epoch), (2, 3, 1));
    }

    /// epoch を読めない位置は区間に入れない。ここで停止しなくなる走査を
    /// 書いたことがあるので、読めない区間だけの入力も必ず終わる。
    #[test]
    fn unreadable_epochs_are_excluded_without_stalling() {
        let state = vec![MeterState::Active; 4];

        assert!(runs_of(&state, &[], MeterState::Active).is_empty());
        assert!(runs_of(&state, &epochs(4, -1), MeterState::Active).is_empty());

        // 読める位置だけが区間になる。
        let partial = runs_of(&state, &[0, 0, -1, 0], MeterState::Active);
        assert_eq!(partial.len(), 2);
        assert_eq!((partial[0].start, partial[0].end), (0, 1));
        assert_eq!((partial[1].start, partial[1].end), (3, 3));
    }

    #[test]
    fn a_run_reaching_the_last_frame_is_closed() {
        let state = vec![MeterState::Free, MeterState::Active, MeterState::Active];

        let runs = runs_of(&state, &epochs(3, 2), MeterState::Active);

        assert_eq!(runs.len(), 1);
        assert_eq!((runs[0].start, runs[0].end), (1, 2));
    }

    #[test]
    fn no_matching_state_yields_no_runs() {
        let state = vec![MeterState::Free; 3];

        assert!(runs_of(&state, &epochs(3, 0), MeterState::Active).is_empty());
    }
}
