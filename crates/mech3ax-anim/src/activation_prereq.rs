use super::types::*;
use mech3ax_common::assert::{assert_utf8, AssertionError};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, bool_c, static_assert_size, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Read, Write};

#[derive(Debug, FromPrimitive, PartialEq)]
#[repr(u32)]
enum ActivPrereqType {
    Animation = 1,
    Object = 2,
    Parent = 3,
}

#[repr(C)]
struct ActivPrereqAnimC {
    name: [u8; 32], // 00
    zero32: u32,    // 32
    zero36: u32,    // 36
}
static_assert_size!(ActivPrereqAnimC, 40);

#[repr(C)]
struct ActivPrereqObjC {
    active: u32,    // 00
    name: [u8; 32], // 32
    pointer: u32,   // 36
}
static_assert_size!(ActivPrereqObjC, 40);

fn read_activ_prereq_anim<R: Read>(read: &mut CountingReader<R>) -> Result<ActivationPrereq> {
    let prereq: ActivPrereqAnimC = read.read_struct()?;
    let name = assert_utf8("anim def activ prereq a name", read.prev + 0, || {
        str_from_c_padded(&prereq.name)
    })?;
    assert_that!(
        "anim def activ prereq a field 32",
        prereq.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def activ prereq a field 36",
        prereq.zero36 == 0,
        read.prev + 36
    )?;
    Ok(ActivationPrereq::Animation(name))
}

fn read_activ_prereq_parent<R: Read>(
    read: &mut CountingReader<R>,
    required: bool,
) -> Result<ActivationPrereq> {
    let prereq: ActivPrereqObjC = read.read_struct()?;
    assert_that!(
        "anim def activ prereq p active",
        prereq.active == 0,
        read.prev + 0
    )?;
    let name = assert_utf8("anim def activ prereq p name", read.prev + 4, || {
        str_from_c_padded(&prereq.name)
    })?;
    assert_that!(
        "anim def activ prereq p pointer",
        prereq.pointer != 0,
        read.prev + 36
    )?;
    Ok(ActivationPrereq::Parent(PrereqObject {
        name,
        required,
        active: false,
        pointer: prereq.pointer,
    }))
}

fn read_activ_prereq_object<R: Read>(
    read: &mut CountingReader<R>,
    required: bool,
) -> Result<ActivationPrereq> {
    let prereq: ActivPrereqObjC = read.read_struct()?;
    let active = assert_that!("anim def activ prereq o active", bool prereq.active, read.prev + 0)?;
    let name = assert_utf8("anim def activ prereq o name", read.prev + 4, || {
        str_from_c_padded(&prereq.name)
    })?;
    assert_that!(
        "anim def activ prereq o pointer",
        prereq.pointer != 0,
        read.prev + 36
    )?;
    Ok(ActivationPrereq::Object(PrereqObject {
        name,
        required,
        active,
        pointer: prereq.pointer,
    }))
}

fn read_activ_prereq<R: Read>(read: &mut CountingReader<R>) -> Result<ActivationPrereq> {
    let optional = read.read_u32()?;
    let required = !assert_that!("anim def activ prereq optional", bool optional, read.prev)?;
    let prereq_type_raw = read.read_u32()?;
    match FromPrimitive::from_u32(prereq_type_raw) {
        Some(ActivPrereqType::Animation) => {
            assert_that!(
                "anim def activ prereq required",
                required == true,
                read.prev - 4
            )?;
            read_activ_prereq_anim(read)
        }
        Some(ActivPrereqType::Parent) => read_activ_prereq_parent(read, required),
        Some(ActivPrereqType::Object) => read_activ_prereq_object(read, required),
        None => {
            let msg = format!(
                "Expected valid activ prereq type, but was {} (at {})",
                prereq_type_raw, read.prev
            );
            Err(AssertionError(msg).into())
        }
    }
}

pub fn read_activ_prereqs<R: Read>(
    read: &mut CountingReader<R>,
    count: u8,
) -> Result<Vec<ActivationPrereq>> {
    (0..count).map(|_| read_activ_prereq(read)).collect()
}

fn write_activ_prereq_anim<W: Write>(write: &mut W, name: &str) -> Result<()> {
    let mut fill = [0; 32];
    str_to_c_padded(name, &mut fill);
    // always required (not optional)
    write.write_u32(bool_c!(false))?;
    write.write_u32(ActivPrereqType::Animation as u32)?;
    write.write_struct(&ActivPrereqAnimC {
        name: fill,
        zero32: 0,
        zero36: 0,
    })?;
    Ok(())
}

fn write_activ_prereq_object<W: Write>(
    write: &mut W,
    object: &PrereqObject,
    prereq_type: ActivPrereqType,
) -> Result<()> {
    let mut name = [0; 32];
    str_to_c_padded(&object.name, &mut name);
    write.write_u32(bool_c!(!object.required))?;
    write.write_u32(prereq_type as u32)?;
    write.write_struct(&ActivPrereqObjC {
        active: bool_c!(object.active),
        name,
        pointer: object.pointer,
    })?;
    Ok(())
}

pub fn write_activ_prereqs<W: Write>(
    write: &mut W,
    activ_prereqs: &[ActivationPrereq],
) -> Result<()> {
    for activ_prereq in activ_prereqs {
        match activ_prereq {
            ActivationPrereq::Animation(name) => write_activ_prereq_anim(write, name)?,
            ActivationPrereq::Object(object) => {
                write_activ_prereq_object(write, object, ActivPrereqType::Object)?
            }
            ActivationPrereq::Parent(object) => {
                write_activ_prereq_object(write, object, ActivPrereqType::Parent)?
            }
        }
    }
    Ok(())
}
