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

use core::fmt;

use crate::{guid, proto::Protocol, Result, Status};

pub type ResetFn =
    extern "efiapi" fn(this: *mut SimpleTextOutput, extended_verification: bool) -> Status;

pub type StringFn = extern "efiapi" fn(this: *mut SimpleTextOutput, string: *mut u16) -> Status;

pub type QueryModeFn = extern "efiapi" fn(
    this: *mut SimpleTextOutput,
    mode: usize,
    cols: *mut usize,
    rows: *mut usize,
) -> Status;

pub type SetModeFn = extern "efiapi" fn(this: *mut SimpleTextOutput, mode: usize) -> Status;

pub type SetAttributeFn =
    extern "efiapi" fn(this: *mut SimpleTextOutput, attribute: usize) -> Status;

pub type ClearScreenFn = extern "efiapi" fn(this: *mut SimpleTextOutput) -> Status;

pub type SetCursorPositionFn =
    extern "efiapi" fn(this: *mut SimpleTextOutput, column: usize, row: usize) -> Status;

pub type EnableCursorFn = extern "efiapi" fn(this: *mut SimpleTextOutput, visible: bool) -> Status;

#[repr(C)]
pub struct SimpleTextOutputMode {
    pub max_mode:       i32,
    pub mode:           i32,
    pub attribute:      i32,
    pub cursor_column:  i32,
    pub cursor_row:     i32,
    pub cursor_visible: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WindowSize {
    pub rows: usize,
    pub cols: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct SimpleTextOutput {
    reset:               ResetFn,
    output_string:       StringFn,
    test_string:         StringFn,
    query_mode:          QueryModeFn,
    set_mode:            SetModeFn,
    set_attribute:       SetAttributeFn,
    clear_screen:        ClearScreenFn,
    set_cursor_position: SetCursorPositionFn,
    enable_cursor:       EnableCursorFn,
    mode:                *mut SimpleTextOutputMode,
}

impl Protocol for SimpleTextOutput {
    const GUID: crate::Guid = guid!(
        0x387477c2, 0x69c7, 0x11d2,
        {0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b}
    );
}

fn check_null_terminated(s: &[u16]) -> bool {
    for c in s {
        if *c == 0 {
            return true;
        }
    }
    false
}

impl SimpleTextOutput {
    pub fn reset(&mut self, extended_verification: bool) -> Result<()> {
        let status = (self.reset)(self, extended_verification);
        status.to_result(())
    }

    pub fn output_string(&mut self, s: &[u16]) -> Result<()> {
        if !check_null_terminated(s) {
            panic!("output_string: string must be null terminated");
        }
        let status = (self.output_string)(self, s.as_ptr().cast_mut());
        status.to_result(())
    }

    pub fn test_string(&mut self, s: &[u16]) -> Result<()> {
        if !check_null_terminated(s) {
            panic!("test_string: string must be null terminated");
        }
        let status = (self.test_string)(self, s.as_ptr().cast_mut());
        status.to_result(())
    }

    pub fn query_mode(&mut self, mode: usize) -> Result<WindowSize> {
        let mut size = WindowSize::default();
        (self.query_mode)(self, mode, &mut size.cols, &mut size.rows).to_result(size)
    }

    pub fn set_mode(&mut self, mode: usize) -> Result<()> {
        (self.set_mode)(self, mode).to_result(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        (self.clear_screen)(self).to_result(())
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) -> Result<()> {
        (self.set_cursor_position)(self, col, row).to_result(())
    }

    pub fn enable_cursor(&mut self, visible: bool) -> Result<()> {
        (self.enable_cursor)(self, visible).to_result(())
    }
}

#[cfg(feature = "alloc")]
impl fmt::Write for SimpleTextOutput {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use alloc::vec::Vec;

        let s = s
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>();
        self.output_string(&s).map_err(|_| fmt::Error)
    }
}

#[cfg(not(feature = "alloc"))]
impl fmt::Write for SimpleTextOutput {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for char in s.encode_utf16() {
            self.output_string(&[char, 0]).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}
