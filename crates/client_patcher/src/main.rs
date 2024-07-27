use anyhow::{anyhow, Result};
use object::{
    coff::CoffHeader,
    pe::{self, ImageSectionHeader, IMAGE_SCN_CNT_INITIALIZED_DATA, IMAGE_SCN_MEM_READ},
    read::pe::{ImageNtHeaders, ImageOptionalHeader},
    LittleEndian as LE,
};
use std::{ffi::CStr, io::Write};
use tracing::trace;

fn main() {
    println!("Hello, world!");
}

fn replace_client_string(
    osclient: &mut Vec<u8>,
    input_string_bytes: &[u8],
    replace_string_bytes: &[u8],
) -> Result<()> {
    let input_string_addr = find_string_in_client(osclient, ".rdata", input_string_bytes)
        .expect("failed getting input string address");
    let replace_string_addr = find_string_in_client(osclient, ".rsps", replace_string_bytes)
        .expect("failed getting replace string address");

    trace!(
        "Input str addr: {:#02X}, replace string addr: {:#02X}",
        input_string_addr,
        replace_string_addr
    );

    let section = get_section_by_name(osclient, ".text")?;

    let section_raw_data = section.pointer_to_raw_data.get(LE) as usize;
    let section_raw_data_size = section.size_of_raw_data.get(LE) as usize;
    let section_virtual_address = section.virtual_address.get(LE) as usize;

    // Address for jav_config string in osclient.exe: 0x59099

    for i in section_raw_data..section_raw_data + section_raw_data_size - 3 {
        let mut offset = u32::from_le_bytes([
            osclient[i],
            osclient[i + 1],
            osclient[i + 2],
            osclient[i + 3],
        ]) as usize;

        // Is actually 0x59CA0 but minus -0xC00
        /*if i == 0x590A0 {
            let offset_in_memory = section_virtual_address - section_raw_data;
            let next_instruction = i + 4;

            println!(
                "Index: {:#02X}, virtual address: {:#02X}, offset: {:#02X}, calc: {:#02X}",
                i,
                section_virtual_address,
                offset,
                next_instruction + offset + (section_virtual_address - section_raw_data)
            );
        }*/

        let next_instruction = i + 4;
        if (next_instruction + offset + (section_virtual_address - section_raw_data))
            == input_string_addr
        {
            trace!(
                "1: {:#02X}, 2: {:#02X}, 3: {:#02X}",
                i + 4,
                section_virtual_address,
                offset
            );

            /*println!("Offset in memory: {:#02X}", offset_in_memory);

            println!("Debug: sec vir addr {:#02X} ", section_virtual_address);

            println!("Input string bytes len: {}", input_string_bytes.len());

            println!("Replace string addr: {:#02X} ", replace_string_addr);*/

            let new_string_offset =
                replace_string_addr - next_instruction - section_virtual_address + section_raw_data;
            trace!("New string offset: {:#02X}", new_string_offset);
            osclient[i..i + 4].copy_from_slice(&u32::to_le_bytes(new_string_offset as u32));

            // Replace the string length if needed
            // TODO: Instead simply scan back and try to find the original string location end, if it matches, we know to replace the string here. That is the most solid solution

            // Iterate 30 (arbritrary number) bytes back and try to find the original string length location
            for j in (i - 30..i).rev() {
                offset = u32::from_le_bytes([
                    osclient[j],
                    osclient[j + 1],
                    osclient[j + 2],
                    osclient[j + 3],
                ]) as usize;

                let p1 = j + 4 + offset + (section_virtual_address - section_raw_data);
                let p2 = input_string_addr + input_string_bytes.len() - 1;

                if p1 == p2 {
                    trace!("Found string length offset addr: {:#02X}", j);

                    osclient[j..j + 4].copy_from_slice(&u32::to_le_bytes(
                        (new_string_offset + replace_string_bytes.len() + ((i - j) - 1)) as u32,
                    ));
                }
            }

            // place in file = index + 0x1000 (virtual addr) - 0xc00 (virtual addr - raw addr)

            /*println!(
                "Pointer to offset place 1, 2, 3: {:#02X} {:#02X} {:#02X}",
                virtual_addr, offset, i
            );
            println!("Call: {:#02X}", ptr);*/

            //if i + 5 + offset as usize == input_string_addr {
            /*println!(
                "Index: {:#02X}, Virtual addr: {:#02X}, raw addr: {:#02X} Offset: {:#02X}, Input string addr: {:#02X}",
                i, testy.virtual_address.0, testy.pointer_to_raw_data.0, offset, input_string_addr
            );*/

            //}
            break;
        }
    }

    Ok(())
}

fn get_section_by_name(osclient: &Vec<u8>, section: &str) -> Result<ImageSectionHeader> {
    let in_dos_header = pe::ImageDosHeader::parse(&**osclient)?;
    let mut offset = in_dos_header.nt_headers_offset().into();
    let (in_nt_headers, _) = pe::ImageNtHeaders64::parse(&**osclient, &mut offset)?;
    let in_file_header = in_nt_headers.file_header();
    let in_sections = in_file_header.sections(&**osclient, offset)?;

    let mut in_sections_index = Vec::new();
    for (index, _) in in_sections.iter().enumerate() {
        in_sections_index.push(index + 1);
    }

    for index in &in_sections_index {
        let in_section = in_sections.section(*index)?;
        let section_name = CStr::from_bytes_until_nul(&in_section.name)?.to_str()?;

        if section == section_name {
            trace!(
                "Found section with name: {}, len: {}",
                section_name,
                section_name.len()
            );

            return Ok(*in_section);
        }

        /*debug_assert_eq!(range.virtual_address, in_section.virtual_address.get(LE));
        debug_assert_eq!(range.file_offset, in_section.pointer_to_raw_data.get(LE));
        debug_assert_eq!(range.file_size, in_section.size_of_raw_data.get(LE));*/
    }

    Err(anyhow!("Failed finding secction"))
}

fn find_string_in_client(
    osclient: &Vec<u8>,
    section: &str,
    input_string_bytes: &[u8],
) -> Result<usize> {
    let section = get_section_by_name(osclient, section)?;

    let mut found = false;

    let section_raw_data = section.pointer_to_raw_data.get(LE) as usize;
    let section_raw_data_size = section.size_of_raw_data.get(LE) as usize;
    let section_virtual_address = section.virtual_address.get(LE) as usize;

    for mut i in section_raw_data..section_raw_data + section_raw_data_size {
        for j in input_string_bytes {
            found = true;

            if *j != osclient[i] {
                found = false;
                break;
            }

            i += 1;
        }

        if found {
            /*println!(
                "Raw pointer, {:#02X}, i: {:#02X}, returned as raw: {:#02X}, returned as virtual: {:#02X}",
                testy.pointer_to_raw_data.0,
                i,
                testy.pointer_to_raw_data.0 as usize + i - input_string_bytes.len(), i - input_string_bytes.len() + testy.virtual_address.0 as usize
            );*/
            return Ok(i - section_raw_data + section_virtual_address - input_string_bytes.len());
        }
    }

    Err(anyhow!("Failed finding string in client"))
}

pub fn patch_rsps(osclient: &mut Vec<u8>, base: usize) -> Result<()> {
    // Overwrite the osclient data with the modified file
    *osclient = create_rsps_section::<pe::ImageNtHeaders64>(osclient)?;

    replace_client_string(
        osclient,
        b"http://oldschool.runescape.com/jav_config.ws?m=0\0",
        b"https://raw.githubusercontent.com/AlterRSPS/Resources/main/jav_configs/jav_config.ws\0",
    )
    .expect("failed replacing jav_config string");
    replace_client_string(
        osclient,
        b"https://oldschool.runescape.com/slr.ws?order=LPWM\0",
        b"https://127.0.0.1/slr.ws?order=LPWM\0",
    )
    .expect("failed replacing world list string");
    replace_client_string(osclient, b"192.168.1.\0", b"raw.githubusercontent.com\0")
        .expect("failed replacing whitelist string");

    Ok(())
}

// Taken from "pecopy" at https://github.com/gimli-rs/object/blob/master/crates/examples/src/bin/pecopy.rs and implemented extra details
// All RSPS features are commented in this function, and will have a "RSPS" part in their comment
fn create_rsps_section<Pe: ImageNtHeaders>(in_data: &[u8]) -> Result<Vec<u8>> {
    let in_dos_header = pe::ImageDosHeader::parse(in_data)?;
    let mut offset = in_dos_header.nt_headers_offset().into();
    let in_rich_header = object::read::pe::RichHeaderInfo::parse(in_data, offset);
    let (in_nt_headers, in_data_directories) = Pe::parse(in_data, &mut offset)?;
    let in_file_header = in_nt_headers.file_header();
    let in_optional_header = in_nt_headers.optional_header();
    let in_sections = in_file_header.sections(in_data, offset)?;

    let mut out_data = Vec::new();
    let mut writer = object::write::pe::Writer::new(
        in_nt_headers.is_type_64(),
        in_optional_header.section_alignment(),
        in_optional_header.file_alignment(),
        &mut out_data,
    );

    // Reserve file ranges and virtual addresses.
    writer.reserve_dos_header_and_stub();
    if let Some(in_rich_header) = in_rich_header.as_ref() {
        writer.reserve(in_rich_header.length as u32 + 8, 4);
    }
    writer.reserve_nt_headers(in_data_directories.len());

    // Copy data directories that don't have special handling.
    let cert_dir = in_data_directories
        .get(pe::IMAGE_DIRECTORY_ENTRY_SECURITY)
        .map(pe::ImageDataDirectory::address_range);
    let reloc_dir = in_data_directories
        .get(pe::IMAGE_DIRECTORY_ENTRY_BASERELOC)
        .map(pe::ImageDataDirectory::address_range);
    for (i, dir) in in_data_directories.iter().enumerate() {
        if dir.virtual_address.get(LE) == 0
            || i == pe::IMAGE_DIRECTORY_ENTRY_SECURITY
            || i == pe::IMAGE_DIRECTORY_ENTRY_BASERELOC
        {
            continue;
        }
        writer.set_data_directory(i, dir.virtual_address.get(LE), dir.size.get(LE));
    }

    // Determine which sections to copy.
    // We ignore any existing ".reloc" section since we recreate it ourselves.
    let mut in_sections_index = Vec::new();
    for (index, in_section) in in_sections.iter().enumerate() {
        if reloc_dir == Some(in_section.pe_address_range()) {
            continue;
        }
        in_sections_index.push(index + 1);
    }

    let mut out_sections_len = in_sections_index.len();
    if reloc_dir.is_some() {
        out_sections_len += 1;
    }

    // +1 for RSPS section
    out_sections_len += 1;
    trace!("Added +1 to out_sections for RSPS section");

    writer.reserve_section_headers(out_sections_len as u16);

    let mut in_sections_data = Vec::new();
    for index in &in_sections_index {
        let in_section = in_sections.section(*index)?;
        let range = writer.reserve_section(
            in_section.name,
            in_section.characteristics.get(LE),
            in_section.virtual_size.get(LE),
            in_section.size_of_raw_data.get(LE),
        );
        debug_assert_eq!(range.virtual_address, in_section.virtual_address.get(LE));
        debug_assert_eq!(range.file_offset, in_section.pointer_to_raw_data.get(LE));
        debug_assert_eq!(range.file_size, in_section.size_of_raw_data.get(LE));
        in_sections_data.push((range.file_offset, in_section.pe_data(in_data)?));
    }

    trace!("Starting on .rsps section");
    // Write RSPS section
    // Create the .rsps section
    let rsps_str = ".rsps";
    let mut rsps_str_arr = [0u8; 8];
    rsps_str_arr[..rsps_str.len()].copy_from_slice(rsps_str.as_bytes());
    let range = writer.reserve_section(
        rsps_str_arr,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
        0x1000,
        0x200,
    );
    // Data to be written to the rsps section itself
    let mut rsps_data = Vec::new();
    rsps_data.write_all(
        b"https://raw.githubusercontent.com/AlterRSPS/Resources/main/jav_configs/jav_config.ws\0",
    )?; //jav_config
    rsps_data.write_all(b"raw.githubusercontent.com\0")?; // domain whitelist (jagex.com)
    rsps_data.write_all(b"https://127.0.0.1/slr.ws?order=LPWM\0")?; // world list

    in_sections_data.push((range.file_offset, &rsps_data));
    trace!("Finished .rsps section");

    if reloc_dir.is_some() {
        let mut blocks = in_data_directories
            .relocation_blocks(in_data, &in_sections)?
            .unwrap();
        while let Some(block) = blocks.next()? {
            for reloc in block {
                writer.add_reloc(reloc.virtual_address, reloc.typ);
            }
        }
        writer.reserve_reloc_section();
    }

    if let Some((_, size)) = cert_dir {
        // TODO: reserve individual certificates
        writer.reserve_certificate_table(size);
    }

    // Start writing.
    writer.write_dos_header_and_stub()?;
    if let Some(in_rich_header) = in_rich_header.as_ref() {
        // TODO: recalculate xor key
        writer.write_align(4);
        writer.write(&in_data[in_rich_header.offset..][..in_rich_header.length + 8]);
    }
    writer.write_nt_headers(object::write::pe::NtHeaders {
        machine: in_file_header.machine.get(LE),
        time_date_stamp: in_file_header.time_date_stamp.get(LE),
        characteristics: in_file_header.characteristics.get(LE),
        major_linker_version: in_optional_header.major_linker_version(),
        minor_linker_version: in_optional_header.minor_linker_version(),
        address_of_entry_point: in_optional_header.address_of_entry_point(),
        image_base: in_optional_header.image_base(),
        major_operating_system_version: in_optional_header.major_operating_system_version(),
        minor_operating_system_version: in_optional_header.minor_operating_system_version(),
        major_image_version: in_optional_header.major_image_version(),
        minor_image_version: in_optional_header.minor_image_version(),
        major_subsystem_version: in_optional_header.major_subsystem_version(),
        minor_subsystem_version: in_optional_header.minor_subsystem_version(),
        subsystem: in_optional_header.subsystem(),
        dll_characteristics: in_optional_header.dll_characteristics(),
        size_of_stack_reserve: in_optional_header.size_of_stack_reserve(),
        size_of_stack_commit: in_optional_header.size_of_stack_commit(),
        size_of_heap_reserve: in_optional_header.size_of_heap_reserve(),
        size_of_heap_commit: in_optional_header.size_of_heap_commit(),
    });
    writer.write_section_headers();
    for (offset, data) in in_sections_data {
        writer.write_section(offset, data);
    }
    writer.write_reloc_section();
    if let Some((address, size)) = cert_dir {
        // TODO: write individual certificates
        writer.write_certificate_table(&in_data[address as usize..][..size as usize]);
    }

    debug_assert_eq!(writer.reserved_len() as usize, writer.len());

    Ok(out_data)
}
