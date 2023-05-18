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

use core::ffi::c_void;

use crate::{
    guid,
    proto::{Proto, Protocol},
    Guid, Lba, Result, Status,
};

pub type ResetFn = extern "efiapi" fn(this: *mut BlockIo, extended_verification: bool) -> Status;

pub type ReadBlocksFn = extern "efiapi" fn(
    this: *mut BlockIo,
    media_id: u32,
    lba: Lba,
    buffer_size: usize,
    buffer: *mut c_void,
) -> Status;

pub type WriteBlocksFn = extern "efiapi" fn(
    this: *mut BlockIo,
    media_id: u32,
    lba: Lba,
    buffer_size: usize,
    buffer: *mut c_void,
) -> Status;

pub type FlushBlocksFn = extern "efiapi" fn(this: *mut BlockIo) -> Status;

#[repr(C)]
pub struct BlockIo {
    pub revision: u64,
    media:        *mut BlockIoMedia,
    reset:        ResetFn,
    read_blocks:  ReadBlocksFn,
    write_blocks: WriteBlocksFn,
    flush_blocks: FlushBlocksFn,
}

impl Protocol for BlockIo {
    const GUID: Guid = guid!(
        0x964e5b21,0x6459,0x11d2,
        {0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b}
    );
}

impl Proto<BlockIo> {
    pub fn media(&self) -> &BlockIoMedia {
        unsafe { &*self.media }
    }

    pub fn reset(&mut self, extended_verification: bool) -> Result<()> {
        (self.reset)(self.as_ptr(), extended_verification).to_result(())
    }

    pub fn read_blocks(&mut self, media_id: u32, lba: Lba, buf: &mut [u8]) -> Result<()> {
        (self.read_blocks)(
            self.as_ptr(),
            media_id,
            lba,
            buf.len(),
            buf.as_mut_ptr().cast(),
        )
        .to_result(())
    }

    pub fn write_blocks(&mut self, media_id: u32, lba: Lba, buf: &mut [u8]) -> Result<()> {
        (self.write_blocks)(
            self.as_ptr(),
            media_id,
            lba,
            buf.len(),
            buf.as_mut_ptr().cast(),
        )
        .to_result(())
    }

    pub fn flush_blocks(&mut self) -> Result<()> {
        (self.flush_blocks)(self.as_ptr()).to_result(())
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct BlockIoMedia {
    /// Current media ID
    ///
    /// This value is updated if the media changes.
    pub media_id:          u32,
    pub removable_media:   bool,
    pub media_present:     bool,
    pub logical_partition: bool,
    pub read_only:         bool,
    pub write_caching:     bool,
    /// Block size of the device, in bytes
    pub block_size:        u32,
    /// Minimum alignment required for transfer buffers
    ///
    /// The value must be a power-of-two. A value of 0 or 1 indicates no alignment restrictions.
    pub io_align:          u32,
    /// Last addressable LBA on the device
    pub last_block:        Lba,

    // Revision 2+
    pub lowest_aligned_lba:                Lba,
    pub logical_blocks_per_physical_block: u32,

    // Revision 3+
    pub optimal_transfer_length_granularity: u32,
}
