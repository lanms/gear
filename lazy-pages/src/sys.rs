// This file is part of Gear.

// Copyright (C) 2022 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Lazy pages signal handler functionality.

use std::cell::RefMut;

use crate::{Error, LazyPagesExecutionContext, LAZY_PAGES_CONTEXT};
use cfg_if::cfg_if;
use gear_core::memory::PageNumber;
use region::Protection;

cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(unix)] {
        mod unix;
        pub use unix::*;
    } else {
        compile_error!("lazy pages are not supported on your system. Disable `lazy-pages` feature");
    }
}

#[derive(Debug)]
pub struct ExceptionInfo {
    /// Address where fault is occurred
    pub fault_addr: *const (),
    pub is_write: Option<bool>,
}

/// Returns key which `page` has in storage.
/// `prefix` is current program prefix in storage.
fn page_key_in_storage(prefix: &Vec<u8>, page: PageNumber) -> Vec<u8> {
    let mut key = Vec::with_capacity(prefix.len() + std::mem::size_of::<u32>());
    key.extend(prefix);
    key.extend(page.0.to_le_bytes());
    key
}

// TODO: version1
unsafe fn user_signal_handler_internal(
    mut ctx: RefMut<LazyPagesExecutionContext>,
    info: ExceptionInfo,
) -> Result<(), Error> {
    let native_ps = region::page::size();
    let gear_ps = PageNumber::size();

    let mem = info.fault_addr;
    let is_write = info.is_write.unwrap_or(false);

    let native_page_addr = region::page::floor(mem) as usize;
    let wasm_mem_addr = ctx.wasm_mem_addr.ok_or(Error::WasmMemAddrIsNotSet)? as usize;
    let wasm_mem_size = ctx.wasm_mem_size.ok_or(Error::WasmMemSizeIsNotSet)?;
    let wasm_mem_end_addr = wasm_mem_addr
        .checked_add(wasm_mem_size)
        .ok_or(Error::WasmMemEndOverflow)?;

    if native_page_addr < wasm_mem_addr || native_page_addr >= wasm_mem_end_addr {
        return Err(Error::SignalFromUnknownMemory {
            wasm_mem_addr,
            wasm_mem_end_addr,
            native_page_addr,
        });
    }

    // First gear page, for which we will remove protection
    let gear_page = PageNumber(((native_page_addr - wasm_mem_addr) / gear_ps) as u32);

    let (gear_pages_num, unprot_addr) = if native_ps > gear_ps {
        assert_eq!(native_ps % gear_ps, 0);
        ((native_ps / gear_ps) as u32, native_page_addr)
    } else {
        assert_eq!(gear_ps % native_ps, 0);
        (1, wasm_mem_addr + gear_page.offset())
    };

    let accessed_page = PageNumber(((mem as usize - wasm_mem_addr) / gear_ps) as u32);
    log::debug!(
        "mem={:?} accessed={:?},{:?} pages={:?} page_native_addr={:#x}",
        mem,
        accessed_page,
        accessed_page.to_wasm_page(),
        gear_page.0..=gear_page.0 + gear_pages_num - 1,
        unprot_addr
    );

    // Set r/w protection in order to load data from storage into page buffer,
    // or if it's second access, then it's definitly `write` and we also must set r/w protection.
    let unprot_size = gear_pages_num as usize * gear_ps;
    region::protect(unprot_addr as *mut (), unprot_size, Protection::READ_WRITE)?;

    let is_first_access = !ctx.accessed_native_pages.contains(&native_page_addr);

    if is_first_access {
        ctx.accessed_native_pages.insert(native_page_addr);
    } else {
        log::trace!("Second access - no need to load data from storage, keep r/w prot");
        for page in (0..gear_pages_num).map(|p| gear_page + p.into()) {
            if ctx.released_lazy_pages.insert(page, None).is_some() {
                return Err(Error::DoubleRelease(page));
            }
        }
        return Ok(());
    }

    for idx in 0..gear_pages_num {
        let page = gear_page + idx.into();

        let ptr = (unprot_addr as *mut u8).add(idx as usize * gear_ps);
        let buffer_as_slice = std::slice::from_raw_parts_mut(ptr, gear_ps);

        let page_key = if let Some(prefix) = &ctx.program_storage_prefix {
            page_key_in_storage(prefix, page)
        } else {
            return Err(Error::ProgramPrefixIsNotSet);
        };
        let res = sp_io::storage::read(&page_key, buffer_as_slice, 0);

        log::trace!(
            "{:?} has{} data in storage",
            page,
            if res.is_none() { " no" } else { "" },
        );

        if let Some(size) = res.filter(|&size| size as usize != PageNumber::size()) {
            return Err(Error::InvalidPageSize {
                expected: PageNumber::size(),
                actual: size,
            });
        }

        if is_write && ctx.released_lazy_pages.insert(page, None).is_some() {
            return Err(Error::DoubleRelease(page));
        }
    }

    if !is_write {
        log::trace!("First access - set read prot");
        region::protect(unprot_addr as *mut (), unprot_size, Protection::READ)?;
    } else {
        log::trace!("First is write access - keep r/w prot");
    }

    Ok(())
}

/// Before contract execution some pages from wasm memory buffer are protected,
/// and cannot be accessed anyhow. When wasm executer tries to access one of these pages,
/// OS emits sigsegv or sigbus or EXCEPTION_ACCESS_VIOLATION. We handle the signal in this function.
/// Using OS signal info, we identify memory location and wasm page.
/// We remove read and write protections for page,
/// then we load wasm page data from storage to wasm page memory location.
/// Also we save page data to [RELEASED_LAZY_PAGES] in order to identify later
/// whether page is changed after execution.
/// After signal handler is done, OS returns execution to the same machine
/// instruction, which cause signal. Now memory which this instruction accesses
/// is not protected and with correct data.
pub unsafe fn user_signal_handler(info: ExceptionInfo) -> Result<(), Error> {
    log::debug!("Interrupted, exception info = {:?}", info);
    LAZY_PAGES_CONTEXT.with(|ctx| user_signal_handler_internal(ctx.borrow_mut(), info))
}
