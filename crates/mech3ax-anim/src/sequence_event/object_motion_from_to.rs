use super::ScriptObject;
use crate::AnimDef;
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::size::ReprSize;
use mech3ax_common::types::Vec3;
use mech3ax_common::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

bitflags::bitflags! {
    struct ObjectMotionFromToFlags: u32 {
        const TRANSLATE = 1 << 0;
        const ROTATE = 1 << 1;
        const SCALE = 1 << 2;
        const MORPH = 1 << 3;
    }
}

#[repr(C)]
struct ObjectMotionFromToC {
    flags: u32,            // 000
    node_index: u32,       // 004
    morph_from: f32,       // 008
    morph_to: f32,         // 012
    morph_delta: f32,      // 016
    translate_from: Vec3,  // 020
    translate_to: Vec3,    // 032
    translate_delta: Vec3, // 044
    rotate_from: Vec3,     // 056
    rotate_to: Vec3,       // 068
    rotate_delta: Vec3,    // 080
    scale_from: Vec3,      // 092
    scale_to: Vec3,        // 104
    scale_delta: Vec3,     // 116
    run_time: f32,         // 128
}
static_assert_size!(ObjectMotionFromToC, 132);

#[derive(Debug, Serialize, Deserialize)]
pub struct FloatFromTo {
    pub from: f32,
    pub to: f32,
    pub delta: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vec3FromTo {
    pub from: Vec3,
    pub to: Vec3,
    pub delta: Vec3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectMotionFromTo {
    pub node: String,
    pub run_time: f32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub morph: Option<FloatFromTo>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub translate: Option<Vec3FromTo>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rotate: Option<Vec3FromTo>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scale: Option<Vec3FromTo>,
}

impl ScriptObject for ObjectMotionFromTo {
    const INDEX: u8 = 11;
    const SIZE: u32 = ObjectMotionFromToC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object motion from to size",
            size == Self::SIZE,
            read.offset
        )?;
        let motion: ObjectMotionFromToC = read.read_struct()?;
        let flags = ObjectMotionFromToFlags::from_bits(motion.flags).ok_or_else(|| {
            AssertionError(format!(
                "Expected valid object motion from flags, but was {:08X} (at {})",
                motion.flags,
                read.prev + 0
            ))
        })?;
        let node = anim_def.node_from_index(motion.node_index as usize, read.prev + 4)?;

        let morph = if flags.contains(ObjectMotionFromToFlags::MORPH) {
            Some(FloatFromTo {
                from: motion.morph_from,
                to: motion.morph_to,
                delta: motion.morph_delta,
            })
        } else {
            assert_that!(
                "object motion morph from",
                motion.morph_from == 0.0,
                read.prev + 8
            )?;
            assert_that!(
                "object motion morph to",
                motion.morph_to == 0.0,
                read.prev + 12
            )?;
            assert_that!(
                "object motion morph delta",
                motion.morph_delta == 0.0,
                read.prev + 16
            )?;
            None
        };

        let translate = if flags.contains(ObjectMotionFromToFlags::TRANSLATE) {
            Some(Vec3FromTo {
                from: motion.translate_from,
                to: motion.translate_to,
                delta: motion.translate_delta,
            })
        } else {
            assert_that!(
                "object motion translate from",
                motion.translate_from == Vec3::EMPTY,
                read.prev + 20
            )?;
            assert_that!(
                "object motion translate to",
                motion.translate_to == Vec3::EMPTY,
                read.prev + 32
            )?;
            assert_that!(
                "object motion translate delta",
                motion.translate_delta == Vec3::EMPTY,
                read.prev + 44
            )?;
            None
        };

        let rotate = if flags.contains(ObjectMotionFromToFlags::ROTATE) {
            Some(Vec3FromTo {
                from: motion.rotate_from,
                to: motion.rotate_to,
                delta: motion.rotate_delta,
            })
        } else {
            assert_that!(
                "object motion rotate from",
                motion.rotate_from == Vec3::EMPTY,
                read.prev + 56
            )?;
            assert_that!(
                "object motion rotate to",
                motion.rotate_to == Vec3::EMPTY,
                read.prev + 68
            )?;
            assert_that!(
                "object motion rotate delta",
                motion.rotate_delta == Vec3::EMPTY,
                read.prev + 80
            )?;
            None
        };

        let scale = if flags.contains(ObjectMotionFromToFlags::SCALE) {
            Some(Vec3FromTo {
                from: motion.scale_from,
                to: motion.scale_to,
                delta: motion.scale_delta,
            })
        } else {
            assert_that!(
                "object motion scale from",
                motion.scale_from == Vec3::EMPTY,
                read.prev + 92
            )?;
            assert_that!(
                "object motion scale to",
                motion.scale_to == Vec3::EMPTY,
                read.prev + 104
            )?;
            assert_that!(
                "object motion scale delta",
                motion.scale_delta == Vec3::EMPTY,
                read.prev + 116
            )?;
            None
        };

        assert_that!(
            "object motion from to runtime",
            motion.run_time > 0.0,
            read.prev + 128
        )?;

        Ok(Self {
            node,
            run_time: motion.run_time,
            morph,
            translate,
            rotate,
            scale,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let mut flags = ObjectMotionFromToFlags::empty();
        if self.translate.is_some() {
            flags |= ObjectMotionFromToFlags::TRANSLATE;
        }
        if self.rotate.is_some() {
            flags |= ObjectMotionFromToFlags::ROTATE;
        }
        if self.scale.is_some() {
            flags |= ObjectMotionFromToFlags::SCALE;
        }
        if self.morph.is_some() {
            flags |= ObjectMotionFromToFlags::MORPH;
        }

        let node_index = anim_def.node_to_index(&self.node)? as u32;

        let (morph_from, morph_to, morph_delta) = if let Some(morph) = &self.morph {
            (morph.from, morph.to, morph.delta)
        } else {
            (0.0, 0.0, 0.0)
        };

        let (translate_from, translate_to, translate_delta) =
            if let Some(translate) = &self.translate {
                (translate.from, translate.to, translate.delta)
            } else {
                (Vec3::EMPTY, Vec3::EMPTY, Vec3::EMPTY)
            };

        let (rotate_from, rotate_to, rotate_delta) = if let Some(rotate) = &self.rotate {
            (rotate.from, rotate.to, rotate.delta)
        } else {
            (Vec3::EMPTY, Vec3::EMPTY, Vec3::EMPTY)
        };

        let (scale_from, scale_to, scale_delta) = if let Some(scale) = &self.scale {
            (scale.from, scale.to, scale.delta)
        } else {
            (Vec3::EMPTY, Vec3::EMPTY, Vec3::EMPTY)
        };

        write.write_struct(&ObjectMotionFromToC {
            flags: flags.bits(),
            node_index,
            morph_from,
            morph_to,
            morph_delta,
            translate_from,
            translate_to,
            translate_delta,
            rotate_from,
            rotate_to,
            rotate_delta,
            scale_from,
            scale_to,
            scale_delta,
            run_time: self.run_time,
        })?;
        Ok(())
    }
}
