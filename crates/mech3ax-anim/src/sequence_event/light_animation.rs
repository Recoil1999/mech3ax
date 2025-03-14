use super::ScriptObject;
use crate::AnimDef;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::size::ReprSize;
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::types::{Vec2, Vec3};
use mech3ax_common::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct LightAnimationC {
    name: [u8; 32],
    light_index: u32, // 32
    range: Vec2,      // 36
    zero44: u32,
    zero48: u32,
    zero52: u32,
    zero56: u32,
    color: Vec3,
    zero72: f32,
    zero76: f32,
    zero80: f32,
    zero84: f32,
    zero88: f32,
    zero92: f32,
    runtime: f32,
}
static_assert_size!(LightAnimationC, 100);

#[derive(Debug, Serialize, Deserialize)]
pub struct LightAnimation {
    pub name: String,
    pub range: Vec2,
    pub color: Vec3,
    pub runtime: f32,
}

impl ScriptObject for LightAnimation {
    const INDEX: u8 = 5;
    const SIZE: u32 = LightAnimationC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("light animation size", size == Self::SIZE, read.offset)?;
        let light_anim: LightAnimationC = read.read_struct()?;

        // not sure why this information is duplicated?
        let actual_name = assert_utf8("light anim name", read.prev + 0, || {
            str_from_c_padded(&light_anim.name)
        })?;
        let expected_name =
            anim_def.light_from_index(light_anim.light_index as usize, read.prev + 32)?;
        assert_that!(
            "light anim name",
            &actual_name == &expected_name,
            read.prev + 32
        )?;

        let range_near = light_anim.range.0;
        let range_far = light_anim.range.1;
        if range_near >= 0.0 {
            assert_that!(
                "light anim range far",
                range_far >= range_near,
                read.prev + 40
            )?;
        } else {
            assert_that!(
                "light anim range far",
                range_far <= range_near,
                read.prev + 40
            )?;
        }

        assert_that!(
            "light anim field 44",
            light_anim.zero44 == 0,
            read.prev + 44
        )?;
        assert_that!(
            "light anim field 48",
            light_anim.zero48 == 0,
            read.prev + 48
        )?;
        assert_that!(
            "light anim field 52",
            light_anim.zero52 == 0,
            read.prev + 52
        )?;
        assert_that!(
            "light anim field 56",
            light_anim.zero56 == 0,
            read.prev + 56
        )?;

        let red = light_anim.color.0;
        let green = light_anim.color.1;
        let blue = light_anim.color.2;
        assert_that!("light anim color red", -5.0 <= red <= 5.0, read.prev + 60)?;
        assert_that!("light anim color green", -5.0 <= green <= 5.0, read.prev + 64)?;
        assert_that!("light anim color blue", -5.0 <= blue <= 5.0, read.prev + 68)?;

        assert_that!(
            "light anim field 72",
            light_anim.zero72 == 0.0,
            read.prev + 72
        )?;
        assert_that!(
            "light anim field 76",
            light_anim.zero76 == 0.0,
            read.prev + 76
        )?;
        assert_that!(
            "light anim field 80",
            light_anim.zero80 == 0.0,
            read.prev + 80
        )?;
        assert_that!(
            "light anim field 84",
            light_anim.zero84 == 0.0,
            read.prev + 84
        )?;
        assert_that!(
            "light anim field 88",
            light_anim.zero88 == 0.0,
            read.prev + 88
        )?;
        assert_that!(
            "light anim field 92",
            light_anim.zero92 == 0.0,
            read.prev + 92
        )?;

        assert_that!(
            "light anim runtime",
            light_anim.runtime > 0.0,
            read.prev + 96
        )?;

        Ok(Self {
            name: actual_name,
            range: light_anim.range,
            color: light_anim.color,
            runtime: light_anim.runtime,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let mut name = [0; 32];
        str_to_c_padded(&self.name, &mut name);
        let light_index = anim_def.light_to_index(&self.name)? as u32;

        write.write_struct(&LightAnimationC {
            name,
            light_index,
            range: self.range,
            zero44: 0,
            zero48: 0,
            zero52: 0,
            zero56: 0,
            color: self.color,
            zero72: 0.0,
            zero76: 0.0,
            zero80: 0.0,
            zero84: 0.0,
            zero88: 0.0,
            zero92: 0.0,
            runtime: self.runtime,
        })?;
        Ok(())
    }
}
