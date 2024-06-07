// SPDX-License-Identifier: GPL-3.0-or-later
// (C) Copyright 2023-2024 Greg Whiteley

pub mod data;
pub mod graph;
pub mod error;

pub type Error = error::Error;
type Result<T> = std::result::Result<T, error::Error>;
