#![feature(specialization)]

#[macro_use]
extern crate pyo3;

use pyo3::prelude::*;
use subclassing::Subclassable;

pub mod othermod;
pub mod subclassing;
pub mod datetime;
