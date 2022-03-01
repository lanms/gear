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

//! Program creation API for Gear programs.

use crate::{ActorId, CodeHash};

mod sys {
    extern "C" {
        // Instead of providing one pointer to multiple fix len params
        // (`fix_len_params_ptr`), we could provide each param as a separate argument to
        // the function and in that case we would have 8 function arguments
        // (five u8 raw pointers, two u32 params, one u64 param). But such amount (size)
        // of function arguments fails proper work of the host function on Apple
        // M1 CPUs when using wasmtime version 0.27. So as a workaround, we provide a
        // pointer to one buffer in which multiple params are encoded: code hash,
        // gas limit and message value.
        pub fn gr_create_program_wgas(
            fix_len_params_ptr: *const u8,
            salt_ptr: *const u8,
            salt_len: u32,
            data_ptr: *const u8,
            data_len: u32,
            program_id_ptr: *mut u8,
        );
    }
}

/// Creates a new program and returns its address.
///
/// The function creates a program initialization message and, as
/// any message send function in the crate, this one requires common additional
/// data for message execution, such as:
/// 1. `payload` that can be used in `init` function of the newly deployed
/// "child" program; 2. `gas_limit`, provided for the program initialization;
/// 3. `value`, sent with the message.
/// Code of newly creating program must be represented as blake2b hash
/// (`code_hash` parameter).
///
/// # Examples
///
/// In order to generate an address for a new program `salt` must be provided.
/// Control of salt uniqueness is fully on a program developer side.
///
/// Basically we can use "automatic" salt generation ("nonce"):
/// ```
/// use gcore::prog;
/// use gcore::CodeHash;
///
/// static mut NONCE: i32 = 0;
///
/// fn increase() {
///     unsafe {
///         NONCE += 1;
///     }
/// }
///
/// fn get() -> i32 {
///     unsafe { NONCE }
/// }
///
/// pub unsafe extern "C" fn handle() {
///     let submitted_code: CodeHash =
///         hex_literal::hex!("abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a")
///             .into();
///     let new_program_id =
///         prog::create_program_with_gas(submitted_code, &get().to_le_bytes(), b"", 10_000, 0);
/// }
/// ```
/// Another case for salt is to receive it as an input:
/// ```
/// use gcore::{msg, prog};
/// # use gcore::CodeHash;
///
/// pub unsafe extern "C" fn handle() {
///     # let submitted_code: CodeHash = hex_literal::hex!("abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a").into();
///     let mut salt = vec![0u8; msg::size()];
///     msg::load(&mut salt[..]);
///     let new_program_id = prog::create_program_with_gas(submitted_code, &salt, b"", 10_000, 0);
/// }
/// ```
///
/// What's more, messages can be sent to a new program:
/// ```
/// use gcore::{msg, prog};
/// # use gcore::CodeHash;
///
/// pub unsafe extern "C" fn handle() {
///     # let submitted_code: CodeHash = hex_literal::hex!("abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a").into();
///     # let mut salt = vec![0u8; msg::size()];
///     # msg::load(&mut salt[..]);
///     let new_program_id = prog::create_program_with_gas(submitted_code, &salt, b"", 10_000, 0);
///     msg::send_with_gas(new_program_id, b"payload for a new program", 10_000, 0);
/// }
/// ```
pub fn create_program_with_gas(
    code_hash: CodeHash,
    salt: &[u8],
    payload: &[u8],
    gas_limit: u64,
    value: u128,
) -> ActorId {
    let mut inner = [0u8; 56];
    let fix_len_data = code_hash
        .iter()
        .copied()
        .chain(gas_limit.to_le_bytes())
        .chain(value.to_le_bytes());
    populate_from_iterator(&mut inner, fix_len_data);

    unsafe {
        let mut program_id = ActorId::default();
        sys::gr_create_program_wgas(
            inner.as_slice().as_ptr(),
            salt.as_ptr(),
            salt.len() as _,
            payload.as_ptr(),
            payload.len() as _,
            program_id.as_mut_slice().as_mut_ptr(),
        );
        program_id
    }
}

fn populate_from_iterator(src: &mut [u8], data: impl Iterator<Item = u8>) {
    for (s, d) in src.iter_mut().zip(data) {
        *s = d;
    }
}
