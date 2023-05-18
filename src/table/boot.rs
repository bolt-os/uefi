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

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{ffi::c_void, mem::size_of, ptr};

use super::TableHeader;
use crate::{
    proto::{DevicePath, Proto, Protocol},
    Event, Guid, Handle, PhysicalAddr, Result, Status, Tpl, VirtualAddr,
};

pub type CreateEventFn = extern "efiapi" fn(
    kind: u32,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: *mut c_void,
    event: *mut Event,
) -> Status;

pub type EventNotifyFn = extern "efiapi" fn(event: Event, ctx: *mut c_void) -> Status;

pub type CreateEventExFn = extern "efiapi" fn(
    kind: u32,
    notify_tpl: Tpl,
    notify_fn: Option<EventNotifyFn>,
    notify_ctx: *mut c_void,
    event_group: *mut Guid,
) -> Status;

pub type CloseEventFn = extern "efiapi" fn(event: Event) -> Status;

pub type SignalEventFn = extern "efiapi" fn(event: Event) -> Status;

pub type WaitForEventFn =
    extern "efiapi" fn(num_events: usize, events: *mut Event, index: *mut usize) -> Status;

pub type CheckEventFn = extern "efiapi" fn(event: Event) -> Status;

pub type SetTimerFn =
    extern "efiapi" fn(event: Event, kind: TimerDelay, trigger_time: u64) -> Status;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TimerDelay {
    Cancel,
    Periodic,
    Relative,
}

pub type RaiseTplFn = extern "efiapi" fn(new: Tpl) -> Tpl;

pub type RestoreTplFn = extern "efiapi" fn(old: Tpl);

pub type AllocatePagesFn = extern "efiapi" fn(
    alloc_type: AllocType,
    memory_type: MemoryType,
    pages: usize,
    memory: *mut PhysicalAddr,
) -> Status;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AllocType {
    AnyPages,
    MaxAddress,
    Address,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemoryType(pub u32);

macro_rules! memory_types {
    ($($name:ident = $value:expr),*$(,)?) => {
        impl MemoryType {
            $(pub const $name: Self = Self($value);)*
        }
    }
}

memory_types! {
    RESERVED                = 0,
    LOADER_CODE             = 1,
    LOADER_DATA             = 2,
    BOOT_SERVICES_CODE      = 3,
    BOOT_SERVICES_DATA      = 4,
    RUNTIME_SERVICES_CODE   = 5,
    RUNTIME_SERVICES_DATA   = 6,
    CONVENTIONAL_MEMORY     = 7,
    UNUSABLE                = 8,
    ACPI_RECLAIM            = 9,
    ACPI_NVS                = 10,
    MMIO                    = 11,
    MMIO_PORT_SPACE         = 12,
    PAL_CODE                = 13,
    PERSISTENT              = 14,
    UNACCEPTED              = 15,
}

#[cfg(feature = "limine")]
impl From<MemoryType> for limine::MemoryKind {
    fn from(value: MemoryType) -> limine::MemoryKind {
        use limine::MemoryKind::*;
        match value {
            MemoryType::LOADER_CODE
            | MemoryType::LOADER_DATA
            | MemoryType::BOOT_SERVICES_CODE
            | MemoryType::BOOT_SERVICES_DATA => BootloaderReclaimable,
            MemoryType::RUNTIME_SERVICES_CODE => EfiRuntimeCode,
            MemoryType::RUNTIME_SERVICES_DATA => EfiRuntimeData,
            MemoryType::CONVENTIONAL_MEMORY => Usable,
            MemoryType::ACPI_RECLAIM => AcpiReclaimable,
            MemoryType::ACPI_NVS => AcpiNvs,
            MemoryType::UNACCEPTED => BadMemory,
            _ => Reserved,
        }
    }
}

pub type FreePagesFn = extern "efiapi" fn(memory: PhysicalAddr, pages: usize) -> Status;

pub type GetMemoryMapFn = extern "efiapi" fn(
    memory_map_size: *mut usize,
    memory_map: *mut MemoryDescriptor,
    map_key: *mut usize,
    descriptor_size: *mut usize,
    descriptor_version: *mut u32,
) -> Status;

#[repr(C)]
#[repr(C)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryDescriptor {
    pub kind:      MemoryType,
    pub phys:      PhysicalAddr,
    pub virt:      VirtualAddr,
    pub num_pages: u64,
    pub attribute: MemoryAttribute,
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct MemoryAttribute : u64 {
        const UC            = 0x0000000000000001;
        const WC            = 0x0000000000000002;
        const WT            = 0x0000000000000004;
        const WB            = 0x0000000000000008;
        const UCE           = 0x0000000000000010;
        const WP            = 0x0000000000001000;
        const RP            = 0x0000000000002000;
        const XP            = 0x0000000000004000;
        const NV            = 0x0000000000008000;
        const MORE_RELIABLE = 0x0000000000010000;
        const RO            = 0x0000000000020000;
        const SP            = 0x0000000000040000;
        const CPU_CRYPTO    = 0x0000000000080000;
        const RUNTIME       = 0x8000000000000000;
        const ISA_VALID     = 0x4000000000000000;
    }
}

pub type AllocatePoolFn =
    extern "efiapi" fn(pool_type: MemoryType, size: usize, buffer: *mut *mut c_void) -> Status;

pub type FreePoolFn = extern "efiapi" fn(buffer: *mut c_void) -> Status;

pub type InstallProtocolInterfaceFn = extern "efiapi" fn(
    handle: *mut Handle,
    protocol: *mut Guid,
    interface_type: InterfaceType,
    interface: *mut c_void,
) -> Status;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InterfaceType {
    Native,
}

pub type UninstallProtocolInterfaceFn =
    extern "efiapi" fn(handle: Handle, protocol: *mut Guid, interface: *mut c_void) -> Status;

pub type ReinstallProtocolInterfaceFn = extern "efiapi" fn(
    handle: Handle,
    protocol: *mut Guid,
    old_interface: *mut c_void,
    new_interface: *mut c_void,
) -> Status;

pub type RegisterProtocolNotifyFn =
    extern "efiapi" fn(protocol: *mut Guid, event: Event, registration: *mut *mut c_void) -> Status;

pub type LocateHandleFn = extern "efiapi" fn(
    search_type: LocateSearchType,
    protocol: *mut Guid,
    search_key: *mut c_void,
    buffer_size: *mut usize,
    buffer: *mut Handle,
) -> Status;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LocateSearchType {
    AllHandles       = 0,
    ByRegisterNotify = 1,
    ByProtocol       = 2,
}

pub type HandleProtocolFn =
    extern "efiapi" fn(handle: Handle, protocol: *mut Guid, interface: *mut *mut c_void) -> Status;

pub type LocateDevicePathFn = extern "efiapi" fn(
    protocol: *mut Guid,
    device_path: *mut Proto<DevicePath>,
    device: *mut Handle,
) -> Status;

pub type OpenProtocolFn = extern "efiapi" fn(
    handle: Handle,
    protocol: *mut Guid,
    interface: *mut *mut c_void,
    agent_handle: Handle,
    controller_handle: Handle,
    attributes: OpenProtocolAttributes,
) -> Status;

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct OpenProtocolAttributes : u32 {
        const BY_HANDLE_PROTOCOL  = 0x00000001;
        const GET_PROTOCOL        = 0x00000002;
        const TEST_PROTOCOL       = 0x00000004;
        const BY_CHILD_CONTROLLER = 0x00000008;
        const BY_DRIVER           = 0x00000010;
        const EXCLUSIVE           = 0x00000020;
    }
}

pub type CloseProtocolFn = extern "efiapi" fn(
    handle: Handle,
    protocol: *mut Guid,
    agent_handle: Handle,
    controller_handle: Handle,
) -> Status;

pub type OpenProtocolInformationFn = extern "efiapi" fn(
    handle: Handle,
    protocol: *mut Guid,
    entry_buffer: *mut *mut OpenProtocolInformationEntry,
    entry_count: usize,
) -> Status;

#[repr(C)]
pub struct OpenProtocolInformationEntry {
    pub agent_handle:      Handle,
    pub controller_handle: Handle,
    pub attributes:        OpenProtocolAttributes,
    pub open_count:        u32,
}

pub type ConnectControllerFn = extern "efiapi" fn(
    controller_handle: Handle,
    driver_image_handle: *mut Handle,
    remaining_device_path: Proto<DevicePath>,
    recursive: bool,
) -> Status;

pub type DisconnectControllerFn = extern "efiapi" fn(
    controller_handle: Handle,
    driver_image_handle: Handle,
    child_handle: Handle,
) -> Status;

pub type ProtocolsPerHandleFn = extern "efiapi" fn(
    handle: Handle,
    protocol_buffer: *mut *mut *mut Guid,
    protocol_buffer_count: *mut usize,
) -> Status;

pub type LocateHandleBufferFn = extern "efiapi" fn(
    search_type: LocateSearchType,
    protocol: *mut Guid,
    search_key: *mut c_void,
    num_handles: *mut usize,
    buffer: *mut *mut Handle,
) -> Status;

pub type LocateProtocolFn = extern "efiapi" fn(
    protocol: *mut Guid,
    registration: *mut c_void,
    interface: *mut *mut c_void,
) -> Status;

pub type InstallMultipleProtocolInterfacesFn =
    extern "efiapi" fn(handle: *mut Handle, ...) -> Status;

pub type UninstallMultipleProtocolInterfacesFn =
    extern "efiapi" fn(handle: *mut Handle, ...) -> Status;

/*
 * Image Services
 */

pub type LoadImageFn = extern "efiapi" fn(
    boot_policy: bool,
    parent_image_handle: Handle,
    device_path: Option<Proto<DevicePath>>,
    source_buffer: *mut c_void,
    source_size: usize,
    image_handle: *mut Handle,
) -> Status;

pub type StartImageFn = extern "efiapi" fn(
    image_handle: Handle,
    exit_data_size: *mut usize,
    exit_data: *mut *mut u16,
) -> Status;

pub type UnloadImageFn = extern "efiapi" fn(image_handle: Handle) -> Status;

pub type ExitFn = extern "efiapi" fn(
    image_handle: Handle,
    exit_status: Status,
    exit_data_size: usize,
    exit_data: *mut u16,
) -> Status;

pub type ExitBootServicesFn = extern "efiapi" fn(image_handle: Handle, map_key: usize) -> Status;

/*
 * Misc. Boot Services
 */

pub type SetWatchdogTimerFn = extern "efiapi" fn(
    timeout: usize,
    watchdog_code: u64,
    data_size: usize,
    watchdog_data: *mut u16,
) -> Status;

pub type StallFn = extern "efiapi" fn(microseconds: usize) -> Status;

pub type CopyMemFn = extern "efiapi" fn(dest: *mut c_void, src: *mut c_void, length: usize);

pub type SetMemFn = extern "efiapi" fn(buffer: *mut c_void, size: usize, value: u8);

pub type GetNextMonotonicCountFn = extern "efiapi" fn(count: *mut u64) -> Status;

pub type InstallConfigurationTableFn =
    extern "efiapi" fn(guid: *mut Guid, table: *mut c_void) -> Status;

pub type CalculateCrc32Fn =
    extern "efiapi" fn(data: *mut c_void, data_size: usize, crc32: *mut u32) -> Status;

#[repr(C)]
#[derive(Debug)]
pub struct BootServices {
    pub header: TableHeader,

    // Task Priority Services
    raise_tpl:   RaiseTplFn,
    restore_tpl: RestoreTplFn,

    // Memory Services
    allocate_pages: AllocatePagesFn,
    free_pages:     FreePagesFn,
    get_memory_map: GetMemoryMapFn,
    allocate_pool:  AllocatePoolFn,
    free_pool:      FreePoolFn,

    // Event and Timer Services
    create_event:   CreateEventFn,
    set_timer:      SetTimerFn,
    wait_for_event: WaitForEventFn,
    signal_event:   SignalEventFn,
    close_event:    CloseEventFn,
    check_event:    CheckEventFn,

    // Protocol Handler Services
    install_protocol_interface:   InstallProtocolInterfaceFn,
    reinstall_protocol_interface: ReinstallProtocolInterfaceFn,
    uninstall_protocol_interface: UninstallProtocolInterfaceFn,
    handle_protocol:              HandleProtocolFn,
    reserved:                     *mut c_void,
    register_protocol_notify:     RegisterProtocolNotifyFn,
    locate_handle:                LocateHandleFn,
    locate_device_path:           LocateDevicePathFn,
    install_configuration_table:  InstallConfigurationTableFn,

    // Image Services
    load_image:         LoadImageFn,
    start_image:        StartImageFn,
    exit:               ExitFn,
    unload_image:       UnloadImageFn,
    exit_boot_services: ExitBootServicesFn,

    // Misc. Boot Services
    get_next_monotonic_count: GetNextMonotonicCountFn,
    stall:                    StallFn,
    set_watchdog_timer:       SetWatchdogTimerFn,

    // EFI 1.1+

    // DriverSupport Services
    connect_controller:    ConnectControllerFn,
    disconnect_controller: DisconnectControllerFn,

    // Open and Close Protocol Services
    open_protocol:             OpenProtocolFn,
    close_protocol:            CloseProtocolFn,
    open_protocol_information: OpenProtocolInformationFn,

    // Library Services
    protocols_per_handle:                   ProtocolsPerHandleFn,
    locate_handle_buffer:                   LocateHandleBufferFn,
    locate_protocol:                        LocateProtocolFn,
    install_multiple_protocol_interfaces:   InstallMultipleProtocolInterfacesFn,
    uninstall_multiple_protocol_interfaces: UninstallMultipleProtocolInterfacesFn,

    // 32-bit CRC Services
    calculate_crc32: CalculateCrc32Fn,

    // Misc. Services
    copy_mem: CopyMemFn,
    set_mem:  SetMemFn,

    // EFI 2.0+
    create_event_ex: CreateEventExFn,
}

impl !Sync for BootServices {}

/// Task Priority Services
impl BootServices {
    /// Raises the task's priority level, returning the previous one
    ///
    /// The new priority level must be
    pub fn raise_tpl(&self, tpl: Tpl) -> Tpl {
        (self.raise_tpl)(tpl)
    }

    pub fn restore_tpl(&self, old: Tpl) {
        (self.restore_tpl)(old);
    }
}

pub enum AllocPagesType {
    Any,
    Max(PhysicalAddr),
    Addr(PhysicalAddr),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryMapInfo {
    pub buffer_size:        usize,
    pub descriptor_version: u32,
    pub descriptor_size:    usize,
    pub map_key:            usize,
}

/// Memory Services
impl BootServices {
    pub fn allocate_pages(
        &self,
        alloc_type: AllocPagesType,
        memory_type: MemoryType,
        num_pages: usize,
    ) -> Result<PhysicalAddr> {
        let (alloc_type, mut memory) = match alloc_type {
            AllocPagesType::Any => (AllocType::AnyPages, 0),
            AllocPagesType::Max(addr) => (AllocType::MaxAddress, addr),
            AllocPagesType::Addr(addr) => (AllocType::Address, addr),
        };
        let status = (self.allocate_pages)(alloc_type, memory_type, num_pages, &mut memory);
        status.to_result(memory)
    }

    pub unsafe fn free_pages(&self, memory: PhysicalAddr, num_pages: usize) -> Result<()> {
        (self.free_pages)(memory, num_pages).to_result(())
    }

    pub fn allocate_pool(&self, pool_type: MemoryType, size: usize) -> Result<*mut u8> {
        let mut buffer = ptr::null_mut();
        let status = (self.allocate_pool)(pool_type, size, &mut buffer);
        status.to_result(buffer.cast())
    }

    pub unsafe fn free_pool(&self, buffer: *mut u8) -> Result<()> {
        (self.free_pool)(buffer.cast()).to_result(())
    }

    pub fn get_memory_map_info(&self) -> Result<MemoryMapInfo> {
        let mut info = MemoryMapInfo::default();

        match (self.get_memory_map)(
            &mut info.buffer_size,
            ptr::null_mut(),
            &mut info.map_key,
            &mut info.descriptor_size,
            &mut info.descriptor_version,
        ) {
            Status::BUFFER_TOO_SMALL => Ok(info),
            status => Err(status),
        }
    }

    pub fn get_memory_map(&self, buffer: &mut [u8], key: usize) -> Result<MemoryMapInfo> {
        let mut info = MemoryMapInfo {
            buffer_size: buffer.len(),
            map_key: key,
            ..Default::default()
        };

        match (self.get_memory_map)(
            &mut info.buffer_size,
            buffer.as_mut_ptr().cast(),
            &mut info.map_key,
            &mut info.descriptor_size,
            &mut info.descriptor_version,
        ) {
            Status::SUCCESS => Ok(info),
            status => Err(status),
        }
    }
}

/// Event and Timer Services
impl BootServices {}

/// Protocol Handler Services
impl BootServices {
    #[cfg(feature = "alloc")]
    pub fn handles_by_protocol<P: Protocol>(&self) -> Result<Box<[Handle]>> {
        let mut guid = P::GUID;
        let mut buffer_size = 0;

        match (self.locate_handle)(
            LocateSearchType::ByProtocol,
            &mut guid,
            ptr::null_mut(),
            &mut buffer_size,
            ptr::null_mut(),
        ) {
            Status::BUFFER_TOO_SMALL => {}
            Status::NOT_FOUND => panic!("no block devices"),
            Status::SUCCESS => panic!(),
            status => status.to_result(())?,
        }

        buffer_size = (buffer_size + (size_of::<Handle>() - 1)) & !(size_of::<Handle>() - 1);

        let mut buffer = Box::new_uninit_slice(buffer_size / size_of::<Handle>());

        (self.locate_handle)(
            LocateSearchType::ByProtocol,
            &mut guid,
            ptr::null_mut(),
            &mut buffer_size,
            buffer.as_mut_ptr().cast(),
        )
        .to_result(())?;

        Ok(unsafe { buffer.assume_init() })
    }

    pub fn protocol_for_handle<P: Protocol>(&self, handle: Handle) -> Result<Proto<P>> {
        let mut guid = P::GUID;
        let mut proto = Option::<Proto<P>>::None;
        // if self.header.revision >= (1 << 16) | 10 {
        //     // OpenProtocol
        //     (self.open_protocol)(handle, &mut guid, ptr::addr_of_mut!(proto).cast(), )
        //     todo!()
        // } else {
        (self.handle_protocol)(handle, &mut guid, ptr::addr_of_mut!(proto).cast()).to_result(())?;
        Ok(proto.unwrap())
        // }
    }

    pub fn first_protocol<P: Protocol>(&self) -> Result<Proto<P>> {
        if self.header.revision >= (1 << 16) | 10 {
            let mut guid = P::GUID;
            let mut proto = Option::<Proto<P>>::None;
            (self.locate_protocol)(&mut guid, ptr::null_mut(), ptr::addr_of_mut!(proto).cast())
                .to_result(())?;
            Ok(proto.unwrap())
        } else {
            let handles = self.handles_by_protocol::<P>()?;
            self.protocol_for_handle(handles[0])
        }
    }
}

/// Image Services
impl BootServices {
    pub fn exit_boot_services(&self, image_handle: Handle, map_key: usize) -> Result<()> {
        (self.exit_boot_services)(image_handle, map_key).to_result(())
    }
}

/// Misc. Boot Services
impl BootServices {
    pub fn next_monotonic_count(&self) -> Result<u64> {
        let mut count = 0;
        let status = (self.get_next_monotonic_count)(&mut count);
        status.to_result(count)
    }
}

/// DriverSupport Services
impl BootServices {}
