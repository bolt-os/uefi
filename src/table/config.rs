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

use crate::{guid, Guid};

#[derive(Debug)]
pub struct ConfigTable {
    entries: &'static [ConfigurationEntry],
}

impl ConfigTable {
    pub(super) const unsafe fn new(data: *mut c_void, len: usize) -> ConfigTable {
        Self {
            entries: core::slice::from_raw_parts(data.cast(), len),
        }
    }

    pub fn get_table(&self, guid: TableGuid) -> Option<*mut c_void> {
        for entry in self.entries {
            if entry.vendor_guid == guid {
                return Some(entry.vendor_table);
            }
        }
        None
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ConfigurationEntry {
    pub vendor_guid:  TableGuid,
    pub vendor_table: *mut c_void,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableGuid(pub Guid);

macro_rules! table_guids {
    ($($name:ident = $guid:expr;)*) => {
        impl TableGuid {
            $(pub const $name: Self = Self($guid);)*
        }
    }
}

table_guids! {
    ACPI = guid!(0xeb9d2d30,0x2d88,0x11d3,{0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d});
    // ACPI 2.0 + should use `ACPI_20`
    ACPI_20 = guid!(0x8868e871,0xe4f1,0x11d3,{0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81});
    SAL_SYSTEM = guid!(0xeb9d2d32,0x2d88,0x11d3,{0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d});
    SMBIOS = guid!(0xeb9d2d31,0x2d88,0x11d3,{0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d});
    SMBIOS3 = guid!(0xf2fd1544,0x9794,0x4a2c,{0x99,0x2e,0xe5,0xbb,0xcf,0x20,0xe3,0x94});
    MPS = guid!(0xeb9d2d2f,0x2d88,0x11d3,{0x9a,0x16,0x00,0x90,0x27,0x3f,0xc1,0x4d});

    JSON_CONFIG_DATA = guid!(0x87367f87,0x1119,0x41ce,{0xaa,0xec,0x8b,0xe0,0x11,0x1f,0x55,0x8a});
    JSON_CAPSULE_DATA = guid!(0x35e7a725,0x8dd2,0x4cac,{0x80,0x11,0x33,0xcd,0xa8,0x10,0x90,0x56});
    JSON_CAPSULE_RESULT = guid!(0xdbc461c3,0xb3de,0x422a,{0xb9,0xb4,0x98,0x86,0xfd,0x49,0xa1,0xe5});

    DEVICE_TREE = guid!(0xb1b621d5,0xf19c,0x41a5,{0x83,0x0b,0xd9,0x15,0x2c,0x69,0xaa,0xe0});

    RT_PROPERTIES = guid!(0xeb66918a,0x7eef,0x402a,{0x84,0x2e,0x93,0x1d,0x21,0xc3,0x8a,0xe9});

    MEMORY_ATTRIBUTES = guid!(0xdcfa911d,0x26eb,0x469f,{0xa2,0x20,0x38,0xb7,0xdc,0x46,0x12,0x20});
}

#[repr(C)]
pub struct RuntimeProperties {
    pub version:                    u16,
    pub length:                     u16,
    pub runtime_services_supported: RtSupport,
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct RtSupport : u32 {
        const GET_TIME = 0x0001;
        const SET_TIME = 0x0002;
        const GET_WAKEUP_TIME = 0x0004;
        const SET_WAKEUP_TIME = 0x0008;
        const GET_VARIABLE = 0x0010;
        const GET_NEXT_VARIABLE_NAME = 0x0020;
        const SET_VARIABLE = 0x0040;
        const SET_VIRTUAL_ADDRESS_MAP = 0x0080;
        const CONVERT_POINTER = 0x0100;
        const GET_NEXT_HIGH_MONOTONIC_COUNT = 0x0200;
        const RESET_SYSTEM = 0x0400;
        const UPDATE_CAPSULE = 0x0800;
        const QUERY_CAPSULE_CAPABILITIES = 0x1000;
        const QUERY_VARIABLE_INFO = 0x2000;
    }
}
