use crate::advice::{AdviceCard, AdviceKind, EvidenceClip, OBSERVATION_REVIEW_CAVEAT};
use crate::match_events::{EventConfidence, MatchEvents, SuperArtEvent};

const LOW_ENTRY_SCALING: u32 = 50;

fn super_label(event: &SuperArtEvent) -> &'static str {
    if event.critical_art {
        "CA"
    } else {
        match event.level {
            1 => "SA1",
            2 => "SA2",
            _ => "SA3",
        }
    }
}

pub(crate) fn detect_low_scaling_super(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let uses: Vec<_> = events
        .super_arts
        .iter()
        .filter(|event| event.side == own && !event.ko)
        .filter_map(|event| {
            let evidence = events.reliable_attack_evidence_for_super(event)?;
            let entry = evidence.entry_scaling_percent?;
            let marginal = evidence.marginal_damage?;
            (entry <= LOW_ENTRY_SCALING && evidence.confidence != EventConfidence::Low)
                .then_some((event, evidence, entry, marginal))
        })
        .collect();
    if uses.is_empty() {
        return None;
    }
    let marginal_total: u32 = uses.iter().map(|(_, _, _, marginal)| *marginal).sum();
    Some(AdviceCard {
        id: "low_scaling_super".to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::Medium,
        title: "低い補正率でSA/CAを組み込んだ場面".to_string(),
        severity: 0.03 * uses.len() as f32,
        // 損失は機会費用であり、この指摘が原因で失った HP ではない。
        hp_lost: None,
        description: format!(
            "ゲーム内表示で、投入時の補正率が {LOW_ENTRY_SCALING}% 以下かつKOに至らなかったSA/CAを {} 回確認しました。SA投入後に増えた表示ダメージは合計 {marginal_total} です。低い補正率だけで使用ミスとは{OBSERVATION_REVIEW_CAVEAT}。残り体力、運び、起き攻め、ゲージ持ち越しを含めて使用目的を確認するための場面一覧です。",
            uses.len()
        ),
        practice: "各クリップで、SAを使わない安定ルートとのダメージ差、相手の残り体力、画面位置を確認します。KO・端到達・有利状況のいずれにも寄与していない場面が繰り返される場合だけ、SAへつなぐ補正率や確認条件を見直しましょう。".to_string(),
        evidence: uses
            .iter()
            .map(|(event, evidence, entry, marginal)| EvidenceClip {
                frame: event.frame,
                end_frame: None,
                label: format!(
                    "R{} {} 投入時{}%補正・コンボ{}・SA以降+{}",
                    event.round_no,
                    super_label(event),
                    entry,
                    evidence.combo_damage,
                    marginal
                ),
            })
            .collect(),
    })
}
