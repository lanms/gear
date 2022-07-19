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

//! Module for scheduler implementation.
//!
//! Scheduler provides API for all available regular or time-dependent actions.

mod scope;
mod task;

pub use scope::*;
pub use task::*;

use crate::storage::{CountedByKey, KeyIterableByKeyMap, ValueStorage};
use core::fmt::Debug;

/// Represents scheduler's logic of centralized delayed tasks management logic.
pub trait Scheduler {
    /// Block number type of the messenger.
    type BlockNumber;
    /// Task type.
    type Task;
    /// Cost type.
    type Cost;
    /// Missed blocks collection representation.
    type MissedBlocksCollection;
    /// Inner error type generated by gear's storage types.
    type Error: TaskPoolError;
    /// Output error of each storage algorithm.
    ///
    /// Implements `From<Self::Error>` to be able to return
    /// any required error type.
    type OutputError: From<Self::Error> + Debug;

    /// Storing costs per block.
    type CostsPerBlock: SchedulingCostsPerBlock<BlockNumber = Self::BlockNumber, Cost = Self::Cost>;

    /// Block numbers, which have already passed,
    /// but still contain tasks to deal with.
    ///
    /// Used for checking if scheduler is able to process
    /// current block aimed tasks, or there are some
    /// incomplete job from previous blocks.
    type MissedBlocks: ValueStorage<Value = Self::MissedBlocksCollection>;

    /// Gear task pool.
    ///
    /// Task pool contains tasks with block number when they should be done.
    type TaskPool: TaskPool<
            BlockNumber = Self::BlockNumber,
            Task = Self::Task,
            Error = Self::Error,
            OutputError = Self::OutputError,
        > + CountedByKey<Key = Self::BlockNumber, Length = usize>
        + KeyIterableByKeyMap<Key1 = Self::BlockNumber, Key2 = Self::Task>;

    /// Resets all related to messenger storages.
    ///
    /// It's temporary production solution to avoid DB migrations,
    /// would be available for tests purposes only in future.
    fn reset() {
        Self::MissedBlocks::kill();
        Self::TaskPool::clear();
    }
}

/// Storing costs getter trait.
pub trait SchedulingCostsPerBlock {
    /// Block number type.
    type BlockNumber;
    /// Cost type.
    type Cost;

    /// Extra reserve for being able to pay for missed blocks.
    fn reserve_for() -> Self::BlockNumber;

    /// Cost for storing code per block.
    fn code() -> Self::Cost;
    /// Cost for storing message in mailbox per block.
    fn mailbox() -> Self::Cost;
    /// Cost for storing program per block.
    fn program() -> Self::Cost;
    /// Cost for storing message in waitlist per block.
    fn waitlist() -> Self::Cost;
}
