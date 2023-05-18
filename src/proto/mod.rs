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

use core::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::{guid, Guid};

pub mod console;
pub mod media;
pub mod riscv;

pub trait Protocol {
    const GUID: Guid;
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Proto<P: Protocol> {
    ptr: NonNull<P>,
}

impl<P: Protocol> Proto<P> {
    pub const fn as_ptr(&self) -> *mut P {
        self.ptr.as_ptr()
    }
}

// impl<P: Protocol> Clone for Proto<P> {
//     fn clone(&self) -> Self {
//         Self { ptr: self.ptr }
//     }
// }
//
// impl<P: Protocol> Copy for Proto<P> {}

impl<P: Protocol> Deref for Proto<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<P: Protocol> DerefMut for Proto<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}


pub struct DevicePath {}

impl Protocol for DevicePath {
    const GUID: Guid = guid!(0, 0, 0, {0,0,0,0,0,0,0,0});
}
