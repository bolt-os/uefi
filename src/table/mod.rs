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

use super::{
    proto::{
        console::{text_input::SimpleTextInput, text_output::SimpleTextOutput},
        Proto,
    },
    Handle,
};

pub mod boot;
pub use boot::*;

pub mod config;
pub use config::*;

#[repr(C)]
#[derive(Debug)]
pub struct TableHeader {
    pub signature:   u64,
    pub revision:    u32,
    pub header_size: u32,
    pub checksum:    u32,
    pub reserved:    u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct SystemTable {
    pub header:               TableHeader,
    pub firmware_vendor:      *mut u16,
    pub firmware_revision:    u32,
    pub stdin_handle:         Handle,
    pub stdin:                Proto<SimpleTextInput>,
    pub stdout_handle:        Handle,
    pub stdout:               Proto<SimpleTextOutput>,
    pub stderr_handle:        Handle,
    pub stderr:               Proto<SimpleTextOutput>,
    pub runtime_services:     *mut (),
    pub boot_services:        *mut BootServices,
    pub config_table_entries: usize,
    pub config_table:         *mut c_void,
}

impl SystemTable {
    pub fn boot_services(&self) -> &'static BootServices {
        unsafe { &*self.boot_services }
    }

    pub fn config_table(&self) -> ConfigTable {
        unsafe { ConfigTable::new(self.config_table, self.config_table_entries) }
    }
}

//
// pub trait Services {}
//
// mod private {
//     pub trait Sealed {}
// }
//
// pub struct Boot;
//
// impl private::Sealed for Boot {}
// impl Services for Boot {}
//
// #[repr(transparent)]
// pub struct SystemTable<S: Services> {
//     ptr: *mut SystemTableInner,
//     view: PhantomData<S>,
// }
//
// impl<S: Services> SystemTable<S> {
//     pub unsafe fn unsafe_clone(&self) -> Self {
//         Self {
//             ptr: self.ptr,
//             view: self.view,
//         }
//     }
//
//     pub fn config_table(&self) {}
// }
//
// impl SystemTable<Boot> {
//     pub fn boot_services(&self) {}
// }
