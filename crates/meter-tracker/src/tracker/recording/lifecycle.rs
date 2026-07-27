use crate::model::TimelineSegment;

use super::super::MeterTracker;

impl MeterTracker {
    pub(crate) fn open_segment(&mut self, edge_cell: i64) {
        self.segment_id += 1;
        self.left.segments.push(TimelineSegment {
            segment_id: self.segment_id,
            entries: vec![],
        });
        self.right.segments.push(TimelineSegment {
            segment_id: self.segment_id,
            entries: vec![],
        });
        self.absolute_frame = Some(edge_cell);
        self.reads.get_mut("left").expect("left reads").clear();
        self.reads.get_mut("right").expect("right reads").clear();
        self.dwell.clear();
        self.emitted
            .get_mut("left")
            .expect("left emitted reads")
            .clear();
        self.emitted
            .get_mut("right")
            .expect("right emitted reads")
            .clear();
        self.divergence = 0;
        self.divergent_edge = None;
        self.still_frames = 0;
    }

    pub(crate) fn close_segment(&mut self) {
        let Some(absolute) = self.absolute_frame else {
            return;
        };
        self.finalize_until(absolute);
        self.absolute_frame = None;
        self.previous = None;
    }
}
