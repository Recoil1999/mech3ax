use crate::bin::FromU8;
use crate::size::static_assert_size;
use mech3ax_common::PeError as Error;

type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: u16,
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: i32,
}
unsafe impl FromU8 for IMAGE_DOS_HEADER {}
static_assert_size!(IMAGE_DOS_HEADER, 64);

impl IMAGE_DOS_HEADER {
    pub const SIGNATURE: u16 = u16::from_le_bytes([b'M', b'Z']);
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_FILE_HEADER {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}
unsafe impl FromU8 for IMAGE_FILE_HEADER {}
static_assert_size!(IMAGE_FILE_HEADER, 20);
impl IMAGE_FILE_HEADER {
    pub const MACHINE_I386: u16 = 0x014c;
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_DATA_DIRECTORY {
    pub virtual_address: u32,
    pub size: u32,
}
unsafe impl FromU8 for IMAGE_DATA_DIRECTORY {}
static_assert_size!(IMAGE_DATA_DIRECTORY, 8);

pub const IMAGE_NUMBEROF_DIRECTORY_ENTRIES: usize = 16;
pub type ImageDataDirectories = [IMAGE_DATA_DIRECTORY; IMAGE_NUMBEROF_DIRECTORY_ENTRIES];

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_OPTIONAL_HEADER32 {
    pub magic: u16,
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_operating_system_version: u16,
    pub minor_operating_system_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub check_sum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    pub data_directory: ImageDataDirectories,
}
unsafe impl FromU8 for IMAGE_OPTIONAL_HEADER32 {}
static_assert_size!(IMAGE_OPTIONAL_HEADER32, 224);

impl IMAGE_OPTIONAL_HEADER32 {
    pub const MAGIC: u16 = 0x010b;
    pub const SUBSYSTEM_WINDOWS_GUI: u16 = 2;
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_NT_HEADERS {
    pub signature: u32,
    pub file_header: IMAGE_FILE_HEADER,
    pub optional_header: IMAGE_OPTIONAL_HEADER32,
}
unsafe impl FromU8 for IMAGE_NT_HEADERS {}
static_assert_size!(
    IMAGE_NT_HEADERS,
    4 + IMAGE_FILE_HEADER::SIZE + IMAGE_OPTIONAL_HEADER32::SIZE
);

impl IMAGE_NT_HEADERS {
    pub const SIGNATURE: u32 = u32::from_le_bytes([b'P', b'E', 0, 0]);
}

const IMAGE_SIZEOF_SHORT_NAME: usize = 8;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_SECTION_HEADER {
    pub name: [u8; IMAGE_SIZEOF_SHORT_NAME],
    pub virtual_size: u32, // also PhysicalAddress for some
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}
unsafe impl FromU8 for IMAGE_SECTION_HEADER {}
static_assert_size!(IMAGE_SECTION_HEADER, 40);

impl IMAGE_SECTION_HEADER {
    pub fn name(&self) -> &str {
        if let Ok(name) = std::str::from_utf8(&self.name) {
            name.trim_end_matches('\0')
        } else {
            "<unk>"
        }
    }

    pub fn virt_to_real(&self, virt: u32) -> Result<Option<usize>> {
        // prevent out-of-section read (underflow)
        let delta = virt
            .checked_sub(self.virtual_address)
            .ok_or_else(|| Error::Underflow {
                section: self.name().to_owned(),
                value: virt,
                bound: self.virtual_address,
            })?;
        // prevent out-of-section read (overflow)
        if delta > self.virtual_size {
            return Err(Error::Overflow {
                section: self.name().to_owned(),
                value: virt,
                bound: self.virtual_address + self.virtual_size,
            });
        }
        // the virtual size can be bigger than the size of raw data (uninitialized)
        if delta > self.size_of_raw_data {
            Ok(None)
        } else {
            let real = self.pointer_to_raw_data + delta;
            Ok(Some(real.try_into()?))
        }
    }

    #[allow(dead_code)]
    pub fn real_to_virt(&self, real: usize) -> Result<Option<u32>> {
        let real: u32 = real.try_into()?;
        // prevent out-of-section read (underflow)
        let delta = real
            .checked_sub(self.pointer_to_raw_data)
            .ok_or_else(|| Error::Underflow {
                section: self.name().to_owned(),
                value: real,
                bound: self.pointer_to_raw_data,
            })?;
        // prevent out-of-section read (overflow)
        if delta > self.size_of_raw_data {
            return Err(Error::Overflow {
                section: self.name().to_owned(),
                value: real,
                bound: self.pointer_to_raw_data + self.size_of_raw_data,
            });
        }
        // the size of raw data can be bigger than the virtual size (alignment padding)
        if delta > self.virtual_size {
            Ok(None)
        } else {
            let virt = self.virtual_address + delta;
            Ok(Some(virt))
        }
    }

    pub fn get_section_bytes<'a, 'b>(&'a self, buf: &'b [u8]) -> &'b [u8] {
        &buf[self.pointer_to_raw_data as _..][..self.size_of_raw_data as _]
    }
}
