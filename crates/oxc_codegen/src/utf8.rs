use std::char;

#[inline(always)]
pub(crate) const fn utf8_lead_byte_width(byte: u8) -> usize {
    if byte & 0x80 == 0 {
        1
    } else if byte & 0xE0 == 0xC0 {
        2
    } else if byte & 0xF0 == 0xE0 {
        3
    } else {
        4
    }
}

#[inline(always)]
pub(crate) unsafe fn decode_utf8_from_lead(lead: u8, cont: *const u8, width: usize) -> char {
    match width {
        1 => lead as char,
        2 => unsafe { decode_two(lead, cont) },
        3 => unsafe { decode_three(lead, cont) },
        4 => unsafe { decode_four(lead, cont) },
        _ => unsafe { std::hint::unreachable_unchecked() },
    }
}

#[inline(always)]
unsafe fn decode_two(lead: u8, cont: *const u8) -> char {
    let b2 = unsafe { *cont };
    debug_assert_eq!(lead & 0xE0, 0xC0);
    debug_assert_eq!(b2 & 0xC0, 0x80);
    let code = ((lead & 0x1F) as u32) << 6 | (b2 & 0x3F) as u32;
    debug_assert!(char::from_u32(code).is_some());
    unsafe { char::from_u32_unchecked(code) }
}

#[inline(always)]
unsafe fn decode_three(lead: u8, cont: *const u8) -> char {
    let b2 = unsafe { *cont };
    let cont1 = unsafe { cont.add(1) };
    let b3 = unsafe { *cont1 };
    debug_assert_eq!(lead & 0xF0, 0xE0);
    debug_assert_eq!(b2 & 0xC0, 0x80);
    debug_assert_eq!(b3 & 0xC0, 0x80);
    let code = ((lead & 0x0F) as u32) << 12 | ((b2 & 0x3F) as u32) << 6 | (b3 & 0x3F) as u32;
    debug_assert!(char::from_u32(code).is_some());
    unsafe { char::from_u32_unchecked(code) }
}

#[inline(always)]
unsafe fn decode_four(lead: u8, cont: *const u8) -> char {
    let b2 = unsafe { *cont };
    let cont1 = unsafe { cont.add(1) };
    let b3 = unsafe { *cont1 };
    let cont2 = unsafe { cont.add(2) };
    let b4 = unsafe { *cont2 };
    debug_assert_eq!(lead & 0xF8, 0xF0);
    debug_assert_eq!(b2 & 0xC0, 0x80);
    debug_assert_eq!(b3 & 0xC0, 0x80);
    debug_assert_eq!(b4 & 0xC0, 0x80);
    let code = ((lead & 0x07) as u32) << 18
        | ((b2 & 0x3F) as u32) << 12
        | ((b3 & 0x3F) as u32) << 6
        | (b4 & 0x3F) as u32;
    debug_assert!(char::from_u32(code).is_some());
    unsafe { char::from_u32_unchecked(code) }
}
