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

use crate::{guid, proto::Protocol, Event, Guid, Result, Status};

pub type InputResetFn =
    extern "efiapi" fn(this: *mut SimpleTextInput, extended_verification: bool) -> Status;

pub type InputReadKeystrokeFn =
    extern "efiapi" fn(this: *mut SimpleTextInput, key: *mut InputKey) -> Status;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub struct InputKey {
    pub scancode:  u16,
    pub codepoint: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct SimpleTextInput {
    reset:          InputResetFn,
    read_keystroke: InputReadKeystrokeFn,
    wait_for_key:   Event,
}

impl Protocol for SimpleTextInput {
    const GUID: Guid = guid!(
        0x387477c1, 0x69c7, 0x11d2,
        {0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b}
    );
}

impl SimpleTextInput {
    /// Reset the input device
    pub fn reset(&mut self, extended_verification: bool) -> Result<()> {
        (self.reset)(self, extended_verification).to_result(())
    }

    /// Read the next keystroke from the input device
    pub fn read_keystroke(&mut self) -> Result<InputKey> {
        let mut key = InputKey::default();
        (self.read_keystroke)(self, &mut key).to_result(key)
    }
}
