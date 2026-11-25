//! nsh-skia's addition to this crate (not in rust-skia).
//!
//! rust-skia's Linux archives hold Skia's own HarfBuzz and libwebp under
//! their public names. In a GTK app those names are also the system
//! libraries': NSH's text measuring calls `hb_ot_metrics_get_position`, which
//! then is Skia's, with a font that is the system HarfBuzz's, and crashes.
//! This renames them in every archive, in place: each name keeps its length,
//! so only bytes of the string tables change.

use std::{fs, io, path::Path};

/// What names hold, and what they hold instead.
const RENAMES: &[(&[u8], &[u8])] = &[(b"hb_", b"hk_"), (b"WebP", b"SkWP"), (b"VP8", b"SV8")];

/// Renames the symbols of every archive in `directory`.
pub fn rename(directory: &Path) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "a") {
            let mut archive = fs::read(&path)?;
            if rename_archive(&mut archive).is_none() {
                return Err(io::Error::other(format!("{}: not an ELF archive", path.display())));
            }
            fs::write(&path, archive)?;
        }
    }
    Ok(())
}

fn rename_bytes(names: &mut [u8]) {
    for (from, to) in RENAMES {
        let mut at = 0;
        while at + from.len() <= names.len() {
            if names[at..].starts_with(from) {
                names[at..at + to.len()].copy_from_slice(to);
                at += from.len();
            } else {
                at += 1;
            }
        }
    }
}

fn rename_archive(archive: &mut [u8]) -> Option<()> {
    const MAGIC: &[u8] = b"!<arch>\n";
    const HEADER: usize = 60;
    if !archive.starts_with(MAGIC) {
        return None;
    }
    let mut at = MAGIC.len();
    while at + HEADER <= archive.len() {
        let name = archive[at..at + 16].to_vec();
        let size: usize = std::str::from_utf8(&archive[at + 48..at + 58]).ok()?.trim().parse().ok()?;
        at += HEADER;
        let member = archive.get_mut(at..at + size)?;
        if name.starts_with(b"/ ") {
            // The index: a count, that many offsets, then names.
            let count = u32::from_be_bytes(member.get(..4)?.try_into().ok()?) as usize;
            rename_bytes(member.get_mut(4 + count * 4..)?);
        } else if name.starts_with(b"/SYM64/") {
            let count = u64::from_be_bytes(member.get(..8)?.try_into().ok()?) as usize;
            rename_bytes(member.get_mut(8 + count * 8..)?);
        } else if member.starts_with(b"\x7fELF") {
            rename_elf(member)?;
        }
        at += size + size % 2;
    }
    Some(())
}

/// Renames the symbols of a 64-bit little-endian object.
fn rename_elf(object: &mut [u8]) -> Option<()> {
    const SHT_SYMTAB: u32 = 2;
    let u16_at = |o: &[u8], at: usize| Some(u16::from_le_bytes(o.get(at..at + 2)?.try_into().ok()?));
    let u32_at = |o: &[u8], at: usize| Some(u32::from_le_bytes(o.get(at..at + 4)?.try_into().ok()?));
    let u64_at = |o: &[u8], at: usize| Some(u64::from_le_bytes(o.get(at..at + 8)?.try_into().ok()?) as usize);
    if object.get(4..6)? != [2, 1] {
        return None;
    }
    let table = u64_at(object, 0x28)?;
    let entry = usize::from(u16_at(object, 0x3a)?);
    let mut count = usize::from(u16_at(object, 0x3c)?);
    if count == 0 && table != 0 {
        // Too many for the header: the first section's size.
        count = u64_at(object, table + 0x20)?;
    }
    for section in (0..count).map(|i| table + i * entry) {
        if u32_at(object, section + 4)? == SHT_SYMTAB {
            let names = table + u32_at(object, section + 0x28)? as usize * entry;
            let (offset, size) = (u64_at(object, names + 0x18)?, u64_at(object, names + 0x20)?);
            rename_bytes(object.get_mut(offset..offset + size)?);
        }
    }
    Some(())
}
