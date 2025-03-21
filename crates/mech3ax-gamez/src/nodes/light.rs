use super::flags::NodeBitFlags;
use super::types::{Block, Light, NodeVariant, NodeVariants, BLOCK_EMPTY, ZONE_DEFAULT};
use mech3ax_common::assert::{assert_all_zero, AssertionError};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::light::LightFlags;
use mech3ax_common::size::ReprSize;
use mech3ax_common::types::{Vec2, Vec3};
use mech3ax_common::{assert_that, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct LightC {
    direction: Vec3,    // 000
    translation: Vec3,  // 012
    zero024: [u8; 112], // 024
    one136: f32,        // 136
    zero140: f32,       // 140
    zero144: f32,       // 144
    zero148: f32,       // 148
    zero152: f32,       // 152
    diffuse: f32,       // 156
    ambient: f32,       // 160
    color: Vec3,        // 164
    flags: u32,         // 176
    range: Vec2,        // 180
    range_near_sq: f32, // 188
    range_far_sq: f32,  // 192
    range_inv: f32,     // 196
    parent_count: u32,  // 200
    parent_ptr: u32,    // 204
}
static_assert_size!(LightC, 208);

const BLOCK_LIGHT: Block = (1.0, 1.0, -2.0, 2.0, 2.0, -1.0);
const LIGHT_NAME: &str = "sunlight";

pub fn assert_variants(node: NodeVariants, offset: u32) -> Result<NodeVariant> {
    let name = &node.name;
    assert_that!("light name", name == LIGHT_NAME, offset + 0)?;
    assert_that!(
        "light flags",
        node.flags == NodeBitFlags::DEFAULT | NodeBitFlags::UNK08,
        offset + 36
    )?;
    assert_that!("light field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("light zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    assert_that!("light data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("light mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!(
        "light area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("light has parent", node.has_parent == false, offset + 84)?;
    // parent array ptr is already asserted
    assert_that!(
        "light children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children array ptr is already asserted
    assert_that!("light block 1", node.unk116 == BLOCK_LIGHT, offset + 116)?;
    assert_that!("light block 2", node.unk140 == BLOCK_EMPTY, offset + 140)?;
    assert_that!("light block 3", node.unk164 == BLOCK_EMPTY, offset + 164)?;
    assert_that!("light field 196", node.unk196 == 0, offset + 196)?;
    Ok(NodeVariant::Light(node.data_ptr))
}

fn assert_light(light: &LightC, offset: u32) -> Result<()> {
    assert_that!("translation", light.translation == Vec3::EMPTY, offset + 12)?;
    assert_all_zero("field 024", offset + 24, &light.zero024)?;

    assert_that!("field 136", light.one136 == 1.0, offset + 136)?;
    assert_that!("field 140", light.zero140 == 0.0, offset + 140)?;
    assert_that!("field 144", light.zero144 == 0.0, offset + 144)?;
    assert_that!("field 148", light.zero148 == 0.0, offset + 148)?;
    assert_that!("field 152", light.zero152 == 0.0, offset + 152)?;

    assert_that!("diffuse", 0.0 <= light.diffuse <= 1.0, offset + 156)?;
    assert_that!("ambient", 0.0 <= light.ambient <= 1.0, offset + 160)?;

    assert_that!("color", light.color == Vec3(1.0, 1.0, 1.0), offset + 164)?;

    let flags = LightFlags::from_bits(light.flags).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid light flags, but was 0x{:08X} (at {})",
            light.flags,
            offset + 176
        ))
    })?;
    assert_that!("flag", flags == LightFlags::DEFAULT, offset + 176)?;

    let range_near = light.range.0;
    let range_far = light.range.1;
    assert_that!("range near", range_near > 0.0, offset + 180)?;
    assert_that!("range far", range_far > range_near, offset + 184)?;
    let expected = range_near * range_near;
    assert_that!(
        "range near sq",
        light.range_near_sq == expected,
        offset + 188
    )?;
    let expected = range_far * range_far;
    assert_that!("range far sq", light.range_far_sq == expected, offset + 192)?;
    let expected = 1.0 / (range_far - range_near);
    assert_that!("range inv", light.range_inv == expected, offset + 196)?;

    assert_that!("parent count", light.parent_count == 1, offset + 200)?;
    assert_that!("parent ptr", light.parent_ptr != 0, offset + 204)?;
    Ok(())
}

pub fn read<R>(read: &mut CountingReader<R>, data_ptr: u32) -> Result<Light>
where
    R: Read,
{
    let light: LightC = read.read_struct()?;
    assert_light(&light, read.prev)?;

    // read as a result of parent_count, but is always 0
    let zero = read.read_u32()?;
    assert_that!("parent value", zero == 0, read.prev)?;

    Ok(Light {
        name: LIGHT_NAME.to_owned(),
        direction: light.direction,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        range: light.range,
        parent_ptr: light.parent_ptr,
        data_ptr,
    })
}

pub fn make_variants(light: &Light) -> NodeVariants {
    NodeVariants {
        name: LIGHT_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT | NodeBitFlags::UNK08,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: light.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: BLOCK_LIGHT,
        unk140: BLOCK_EMPTY,
        unk164: BLOCK_EMPTY,
        unk196: 0,
    }
}

pub fn write<W>(write: &mut W, light: &Light) -> Result<()>
where
    W: Write,
{
    write.write_struct(&LightC {
        direction: light.direction,
        translation: Vec3::EMPTY,
        zero024: [0; 112],
        one136: 1.0,
        zero140: 0.0,
        zero144: 0.0,
        zero148: 0.0,
        zero152: 0.0,
        diffuse: light.diffuse,
        ambient: light.ambient,
        color: light.color,
        flags: LightFlags::DEFAULT.bits(),
        range: light.range,
        range_near_sq: light.range.0 * light.range.0,
        range_far_sq: light.range.1 * light.range.1,
        range_inv: 1.0 / (light.range.1 - light.range.0),
        parent_count: 1,
        parent_ptr: light.parent_ptr,
    })?;
    // written as a result of parent_count, but is always 0
    write.write_u32(0)?;
    Ok(())
}

pub fn size() -> u32 {
    LightC::SIZE + 4
}
