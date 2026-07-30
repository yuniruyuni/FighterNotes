//! 実ゲーム動画の複数サンプルから集計した攻撃属性グリフの多数決マスク。
//!
//! 32x20 の各行を bit0=左端として保持する。元フレームやcropは保持せず、
//! 「上段・中段・下段・投げ」の分類に必要な二値統計だけを収録する。

use super::model::AttackAttribute;

pub(super) const ATTRIBUTE_TEMPLATES: [(AttackAttribute, [u32; 20]); 4] = [
    (
        AttackAttribute::Upper,
        [
            0x00000000, 0x1f7c0080, 0x1b1c00c0, 0x180400c0, 0x5b0400c0, 0x593c00c0, 0x79bc00c0,
            0x31841fc0, 0x00041dc0, 0x3fbc00c0, 0x333c00c0, 0x310400c0, 0x130400c0, 0x1a6400c0,
            0x0e7e00c0, 0x0e0f00c0, 0x3f067ffe, 0x61c47ffe, 0x00040000, 0x00000000,
        ],
    ),
    (
        AttackAttribute::Middle,
        [
            0x00000080, 0x1e7c0080, 0x1b0c0000, 0x110425cc, 0x5b0437fe, 0x593c3086, 0x79bc2086,
            0x31842086, 0x00042086, 0x3f3c2086, 0x333c31c6, 0x31063ffe, 0x130431c6, 0x1a642086,
            0x0e7e0180, 0x0e0e0080, 0x3f040180, 0x71840080, 0x00040080, 0x00000000,
        ],
    ),
    (
        AttackAttribute::Lower,
        [
            0x00000000, 0x1e7c7ffe, 0x1b1c7ffe, 0x1b040080, 0x5b0400c0, 0x593c00c0, 0x79bc00c0,
            0x308400c0, 0x000407c0, 0x3fbc0cc0, 0x333c18c0, 0x310430c0, 0x130400c0, 0x1a6600c0,
            0x0e7f00c0, 0x0e0f00c0, 0x3f0600c0, 0x71c400c0, 0x00040080, 0x00000000,
        ],
    ),
    (
        AttackAttribute::Throw,
        [
            0x40000008, 0x54040f08, 0x54040f08, 0x0404098c, 0x0c0649be, 0x3fe6599e, 0x1c0678cc,
            0x0c0638cc, 0x0c06000c, 0x0c063ffc, 0x0c06359e, 0x0c06308f, 0x0c06118c, 0x04061b08,
            0x06060e0c, 0x07060e08, 0x03861f8c, 0x00c071ce, 0x00000000, 0x00000000,
        ],
    ),
];
