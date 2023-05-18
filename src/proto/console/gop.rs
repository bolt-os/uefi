/*
 * Copyright (c) 2023 xvanc and contributors
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its contributors
 *    may be used to endorse or promote products derived from this software without
 *    specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

use core::{ffi::c_int, mem::size_of, ptr};

use crate::{guid, proto::Protocol, Handle, PhysicalAddr, Result, Status};

pub type QueryModeFn = extern "efiapi" fn(
    this: *mut GraphicsOutput,
    mode: u32,
    info_size: *mut usize,
    info: *mut *const ModeInfo,
) -> Status;

pub type SetModeFn = extern "efiapi" fn(this: *mut GraphicsOutput, mode: u32) -> Status;

pub type BltFn = extern "efiapi" fn(
    this: *mut GraphicsOutput,
    buffer: *mut BltPixel,
    operation: BltOperation,
    source_x: usize,
    source_y: usize,
    destination_x: usize,
    destination_y: usize,
    width: usize,
    height: usize,
    delta: usize,
) -> Status;

/// Graphics Output Protocol
///
/// This protocol provides the limited graphics functionality required in the
/// pre-boot environment.
#[repr(C)]
pub struct GraphicsOutput {
    query_mode: QueryModeFn,
    set_mode:   SetModeFn,
    blt:        BltFn,
    mode:       *mut Mode,
}

impl Protocol for GraphicsOutput {
    const GUID: crate::Guid = guid!(
        0x9042a9de,0x23dc,0x4a38,
        {0x96,0xfb,0x7a,0xde,0xd0,0x80,0x51,0x6a}
    );
}

impl GraphicsOutput {
    /// Returns the information structure for the current mode.
    pub fn mode(&self) -> &'static Mode {
        unsafe { &*self.mode }
    }

    /// Requests the information structure for a specific mode.
    pub fn query_mode(&mut self, mode: u32) -> Result<&'static ModeInfo> {
        let mut ptr = ptr::null();
        let mut size = 0;
        (self.query_mode)(self, mode, &mut size, &mut ptr).to_result(())?;
        assert!(size >= size_of::<ModeInfo>());
        Ok(unsafe { &*ptr })
    }

    pub fn set_mode(&mut self, mode: u32) -> Result<()> {
        (self.set_mode)(self, mode).to_result(())
    }

    pub fn all_modes(&mut self) -> impl Iterator<Item = (u32, Result<&'static ModeInfo>)> + '_ {
        let mut current_mode = 0;
        let max_mode = self.mode().max_mode - 1;

        core::iter::from_fn(move || {
            let mode = current_mode;
            current_mode += 1;

            if mode <= max_mode {
                Some((mode, self.query_mode(mode)))
            } else {
                None
            }
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PixelBitmask {
    pub red:      u32,
    pub green:    u32,
    pub blue:     u32,
    pub reserved: u32,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PixelFormat(pub c_int);

impl PixelFormat {
    pub const RGBA8: Self = Self(0);
    pub const BGRA8: Self = Self(1);
    pub const BITMASK: Self = Self(2);
    pub const BLT_ONLY: Self = Self(3);
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ModeInfo {
    pub version:               u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution:   u32,
    pub pixel_format:          PixelFormat,
    pub pixel_info:            PixelBitmask,
    pub pixels_per_scanline:   u32,
}

#[repr(C)]
pub struct Mode {
    /// Number of modes supported by [`QueryModeFn`] and [`SetModeFn`]
    pub max_mode:         u32,
    /// Current mode
    pub mode:             u32,
    info:             *const ModeInfo,
    pub info_size:        usize,
    pub framebuffer_addr: PhysicalAddr,
    pub framebuffer_size: usize,
}

impl Mode {
    pub const fn info(&self) -> &'static ModeInfo {
        unsafe { &*self.info }
    }
}

#[repr(transparent)]
pub struct BltOperation(pub c_int);

impl BltOperation {
    pub const VIDEO_FILL: Self = Self(0);
    pub const VIDEO_TO_BLT_BUFFER: Self = Self(1);
    pub const BUFFER_TO_VIDEO: Self = Self(2);
    pub const VIDEO_TO_VIDEO: Self = Self(3);
}

#[repr(C)]
pub struct BltPixel {
    pub blue:     u8,
    pub green:    u8,
    pub red:      u8,
    pub reserved: u8,
}

#[repr(C)]
pub struct EdidDiscovered {
    edid_size: u32,
    edid:      *const u8,
}

impl Protocol for EdidDiscovered {
    const GUID: crate::Guid = guid!(
        0x1c0c34f6,0xd380,0x41fa,
        {0xa0,0x49,0x8a,0xd0,0x6c,0x1a,0x66,0xaa}
    );
}

impl EdidDiscovered {
    pub const fn as_ptr(&self) -> *const u8 {
        self.edid
    }

    pub const fn size(&self) -> usize {
        self.edid_size as usize
    }

    pub fn as_slice<'a>(&self) -> Option<&'a [u8]> {
        // SAFETY: The firmware would never lie, would it? :^)
        if !self.edid.is_null() {
            unsafe { Some(core::slice::from_raw_parts(self.edid, self.size())) }
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct EdidActive {
    edid_size: u32,
    edid:      *const u8,
}

impl Protocol for EdidActive {
    const GUID: crate::Guid = guid!(
        0xbd8c1056,0x9f36,0x44ec,
        {0x92,0xa8,0xa6,0x33,0x7f,0x81,0x79,0x86}
    );
}

impl EdidActive {
    pub const fn as_ptr(&self) -> *const u8 {
        self.edid
    }

    pub const fn size(&self) -> usize {
        self.edid_size as usize
    }

    pub fn as_slice<'a>(&self) -> Option<&'a [u8]> {
        // SAFETY: The firmware would never lie, would it? :^)
        if !self.edid.is_null() {
            unsafe { Some(core::slice::from_raw_parts(self.edid, self.size())) }
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct EdidOverride {
    pub get_edid: GetEdidFn,
}

impl Protocol for EdidOverride {
    const GUID: crate::Guid = guid!(
        0x48ecb431,0xfb72,0x45c0,
        {0xa9,0x22,0xf4,0x58,0xfe,0x04,0x0b,0xd5}
    );
}

pub type GetEdidFn = extern "efiapi" fn(
    this: *mut EdidOverride,
    child: *mut Handle,
    attributes: *mut u32,
    edid_size: *mut usize,
    edid: *mut *mut u8,
) -> Status;

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct EdidOverrideAttrs : u32 {
        const DONT_OVERRIDE = 1 << 0;
        const ENABLE_HOT_PLUG = 1 << 1;
    }
}
