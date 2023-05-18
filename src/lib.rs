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

#![no_std]
#![feature(
    decl_macro,                                 // https://github.com/rust-lang/rust/issues/39412
    extended_varargs_abi_support,               // https://github.com/rust-lang/rust/issues/100189
    negative_impls,                             // https://github.com/rust-lang/rust/issues/68318
    new_uninit,                                 // https://github.com/rust-lang/rust/issues/63291
)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "limine")]
extern crate limine;

pub mod proto;
pub mod table;

use core::{ffi::c_void, ptr::{NonNull, self}, sync::atomic::{AtomicPtr, Ordering}};

use table::{SystemTable, BootServices};

pub type Result<T> = core::result::Result<T, Status>;

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Guid {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
}

pub macro guid(
    $a:expr,
    $b:expr,
    $c:expr, { $d:expr, $e:expr, $f:expr, $g:expr, $h:expr, $i:expr, $j:expr, $k:expr }
) {
    Guid {
        a: $a,
        b: $b,
        c: $c,
        d: [$d, $e, $f, $g, $h, $i, $j, $k],
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub struct Status(usize);

macro_rules! status_codes {
    (
        error_codes:
            $(const $e_name:ident = $e_value:expr;)*
        warning_codes:
            $(const $w_name:ident = $w_value:expr;)*

    ) => {
        impl Status {
            $(pub const $e_name: Self = Self::new_error($e_value);)*
            $(pub const $w_name: Self = Self::new_warn($w_value);)*
        }

        impl core::fmt::Debug for Status {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match *self {
                    $(Self::$e_name => write!(f, stringify!($e_name))?,)*
                    $(Self::$w_name => write!(f, stringify!($w_name))?,)*
                    _ => write!(f, "Status({:#x})", self.0)?,
                }
                Ok(())
            }
        }
    };
}

status_codes! {
error_codes:
    const LOAD_ERROR            = 1;
    const INVALID_PARAMETER     = 2;
    const UNSUPPORTED           = 3;
    const BAD_BUFFER_SIZE       = 4;
    const BUFFER_TOO_SMALL      = 5;
    const NOT_READY             = 6;
    const DEVICE_ERROR          = 7;
    const WRITE_PROTECTED       = 8;
    const OUT_OF_RESOURCES      = 9;
    const VOLUME_CORRUPTED      = 10;
    const VOLUME_FULL           = 11;
    const NO_MEDIA              = 12;
    const MEDIA_CHANGED         = 13;
    const NOT_FOUND             = 14;
    const ACCESS_DENIED         = 15;
    const NO_RESPONSE           = 16;
    const NO_MAPPING            = 17;
    const TIMEOUT               = 18;
    const NOT_STARTED           = 19;
    const ALREADY_STARTED       = 20;
    const ABORTED               = 21;
    const ICMP_ERROR            = 22;
    const TFTP_ERROR            = 23;
    const PROTOCOL_ERROR        = 24;
    const INCOMPATIBLE_ERROR    = 25;
    const SECURITY_VIOLATION    = 26;
    const CRC_ERROR             = 27;
    const END_OF_MEDIA          = 28;
    const END_OF_FILE           = 31;
    const INVALID_LANGUAGE      = 32;
    const COMPROMISED_DATA      = 33;
    const IP_ADDRESS_CONFLICT   = 34;
    const HTTP_ERROR            = 35;

warning_codes:
    const WARN_UNKNOWN_GLYPH         = 1;
    const WARN_DELETE_FAILURE        = 2;
    const WARN_WRITE_FAILURE         = 3;
    const WARN_BUFFER_TOO_SMALL      = 4;
    const WARN_STALE_DATA            = 5;
    const WARN_FILE_SYSTEM           = 6;
    const WARN_RESET_REQUESTED       = 7;
}

impl Status {
    pub const SUCCESS: Self = Self(0);

    const HIGH_BIT: usize = 1 << (usize::BITS - 1);

    pub const fn new_error(value: usize) -> Self {
        Self(Self::HIGH_BIT | value)
    }

    pub const fn new_warn(value: usize) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub fn to_result<T>(self, ok: T) -> Result<T> {
        if self == Self::SUCCESS {
            Ok(ok)
        } else {
            Err(self)
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Handle(NonNull<c_void>);

/// Handle to an event structure
#[repr(transparent)]
#[derive(Debug)]
pub struct Event(*mut c_void);

/// Logical Block Address
pub type Lba = u64;

/// Task Priority Level
#[repr(transparent)]
pub struct Tpl(usize);

impl Tpl {
    pub const APPLICATION: Self = Self(4);
    pub const CALLBACK: Self = Self(8);
    pub const NOTIFY: Self = Self(16);
    pub const HIGH_LEVEL: Self = Self(31);
}

pub type PhysicalAddr = u64;
pub type VirtualAddr = u64;

static SYSTEM_TABLE: AtomicPtr<SystemTable> = AtomicPtr::new(ptr::null_mut());
static IMAGE_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

pub unsafe fn bootstrap(image: Handle, system_table: &'static SystemTable) {
    IMAGE_HANDLE.store(image.0.as_ptr(), Ordering::Release);
    SYSTEM_TABLE.store(system_table as *const _ as *mut _, Ordering::Release);
}

pub fn system_table() -> &'static SystemTable {
    let ptr = SYSTEM_TABLE.load(Ordering::Acquire);
    if ptr.is_null() {
        panic!("`uefi::bootstrap()` has not been called");
    }
    unsafe { &*ptr }
}

pub fn image_handle() -> Handle {
    let ptr = IMAGE_HANDLE.load(Ordering::Acquire);
    if ptr.is_null() {
        panic!("`uefi::bootstrap()` has not been called");
    }
    Handle(NonNull::new(ptr).unwrap())
}

pub fn boot_services() -> &'static BootServices {
    system_table().boot_services()
}
