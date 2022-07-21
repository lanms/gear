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

use crate::{Error, LAZY_PAGES_CONTEXT};
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
    let native_ps = region::page::size();
    let gear_ps = PageNumber::size();

    log::debug!("Interrupted, exception info = {:?}", info);

    let mem = info.fault_addr;
    let is_write = info.is_write.unwrap_or(false);

    let native_page_addr = region::page::floor(mem) as usize;
    let wasm_mem_addr = LAZY_PAGES_CONTEXT
        .with(|ctx| ctx.borrow().wasm_mem_addr)
        .ok_or(Error::WasmMemAddrIsNotSet)? as usize;

    if wasm_mem_addr > native_page_addr {
        return Err(Error::SignalFromUnknownMemory {
            wasm_mem_begin: wasm_mem_addr,
            native_page: native_page_addr,
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
        gear_page.0..gear_page.0 + gear_pages_num as u32,
        unprot_addr
    );

    let unprot_size = gear_pages_num as usize * gear_ps;
    region::protect(unprot_addr as *mut (), unprot_size, Protection::READ_WRITE)?;

    let is_first_access = LAZY_PAGES_CONTEXT.with(|ctx| {
            !ctx.borrow()
                .accessed_native_pages
                .contains(&native_page_addr)
        });

    // if is_first_access && !is_write {
    //     log::trace!("First access - allow to read page");
    //     region::protect(unprot_addr as *mut (), unprot_size, Protection::READ)?;
    // } else {
    //     log::trace!(
    //         "It's {} - allow to read/write page",
    //         if is_write {
    //             "write access"
    //         } else {
    //             "second access"
    //         }
    //     );
    //     region::protect(unprot_addr as *mut (), unprot_size, Protection::READ_WRITE)?;
    // }

    LAZY_PAGES_CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();

        if is_first_access {
            ctx.accessed_native_pages.insert(native_page_addr);
        } else {
            log::trace!("Second access (write) - no need to store data from storage, keep r/w prot");
            ctx.released_lazy_pages.extend((0..gear_pages_num as u32).map(|p| (gear_page + p.into(), None)));
            return Ok(());
        }

        for idx in 0..gear_pages_num as u32 {
            let page = gear_page + idx.into();

            let ptr = (unprot_addr as *mut u8).add(idx as usize * gear_ps);
            let buffer_as_slice = std::slice::from_raw_parts_mut(ptr, gear_ps);

            // TODO: simplify before release (issue #1147). Currently we must support here all old runtimes.
            // For new runtimes we have to calc page key from program pages prefix.
            let page_key = if let Some(prefix) = &ctx.program_storage_prefix {
                page_key_in_storage(prefix, page)
            } else {
                // This case is for old runtimes support
                ctx.lazy_pages_info
                    .remove(&page)
                    .ok_or(Error::LazyPageNotExistForSignalAddr(mem, page))?
            };
            let res = sp_io::storage::read(&page_key, buffer_as_slice, 0);

            if res.is_none() {
                log::trace!("{:?} has no data in storage", page);
            } else {
                log::trace!("{:?} has data in storage", page);
            }

            if let Some(size) = res.filter(|&size| size as usize != PageNumber::size()) {
                return Err(Error::InvalidPageSize {
                    expected: PageNumber::size(),
                    actual: size,
                });
            }

            if is_write {
                let _ = ctx.released_lazy_pages.insert(page, None);
            }

            // let page_buf = PageBuf::new_from_vec(buffer_as_slice.to_vec())
            //     .expect("Cannot panic here, because we create slice with PageBuf size");

            // if ctx
            //     .released_lazy_pages
            //     .insert(page, Some(page_buf))
            //     .is_some()
            // {
            //     return Err(Error::DoubleRelease(page));
            // }
        }

        if !is_write {
            log::trace!("First access - set read prot");
            region::protect(unprot_addr as *mut (), unprot_size, Protection::READ)?;
        } else {
            log::trace!("First write access - keep r/w prot");
        }

        Ok(())
    })
}
