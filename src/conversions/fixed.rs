#![cfg(feature = "fixed")]
//! Conversions to and from [fixed](https://docs.rs/fixed)'s type.
//!
//! This is useful for converting Python's decimal.Decimal into and from a native Rust type (fixed).
//!
//! # Setup
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"fixed\"] }")]
//! fixed = "1.23.1"
//! ```
//!
//! Note that you must use a compatible version of fixed and PyO3.
//! The required fixed version may vary based on the version of PyO3.
//!
//! ```

use crate::exceptions::PyValueError;
use crate::once_cell::GILOnceCell;
use crate::types::PyType;
use crate::{intern, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};
use fixed::types::*;
use std::str::FromStr;

static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_decimal_cls(py: Python<'_>) -> PyResult<&PyType> {
    DECIMAL_CLS
        .get_or_try_init(py, || {
            py.import(intern!(py, "decimal"))?
                .getattr(intern!(py, "Decimal"))?
                .extract()
        })
        .map(|ty| ty.as_ref(py))
}

macro_rules! fixed_conversion {
    ($decimal: ty) => {
        impl FromPyObject<'_> for $decimal {
            fn extract(obj: &PyAny) -> PyResult<Self> {
                // use the string representation to not be lossy
                if let Ok(val) = obj.extract::<f64>() {
                    Ok(<$decimal>::from_num(val))
                } else {
                    <$decimal>::from_str(obj.str()?.to_str()?)
                        .map_err(|e| PyValueError::new_err(e.to_string()))
                }
            }
        }

        impl ToPyObject for $decimal {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                // TODO: handle error gracefully when ToPyObject can error
                // look up the decimal.Decimal
                let dec_cls = get_decimal_cls(py).expect("failed to load decimal.Decimal");

                // now call the constructor
                // lossy with f64
                // let ret = dec_cls
                //     .call1((self.to_num::<f64>(),))
                //     .expect("failed to call decimal.Decimal(value)");

                // if don't want to be lossy, use this instead:
                let ret = dec_cls
                    .call1((self.to_string(),))
                    .expect("failed to call decimal.Decimal(value)");

                ret.to_object(py)
            }
        }

        impl IntoPy<PyObject> for $decimal {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.to_object(py)
            }
        }
    };
}

/// [`FixedI8`] with eight integer bits and no fractional bits.
fixed_conversion!(I8F0);
/// [`FixedI8`] with seven integer bits and one fractional bit.
fixed_conversion!(I7F1);
/// [`FixedI8`] with six integer bits and two fractional bits.
fixed_conversion!(I6F2);
/// [`FixedI8`] with five integer bits and three fractional bits.
fixed_conversion!(I5F3);
/// [`FixedI8`] with four integer bits and four fractional bits.
fixed_conversion!(I4F4);
/// [`FixedI8`] with three integer bits and five fractional bits.
fixed_conversion!(I3F5);
/// [`FixedI8`] with two integer bits and six fractional bits.
fixed_conversion!(I2F6);
/// [`FixedI8`] with one integer bit and seven fractional bits.
fixed_conversion!(I1F7);
/// [`FixedI8`] with no integer bits and eight fractional bits.
fixed_conversion!(I0F8);
/// [`FixedI16`] with 16 integer bits and no fractional bits.
fixed_conversion!(I16F0);
/// [`FixedI16`] with 15 integer bits and one fractional bit.
fixed_conversion!(I15F1);
/// [`FixedI16`] with 14 integer bits and two fractional bits.
fixed_conversion!(I14F2);
/// [`FixedI16`] with 13 integer bits and three fractional bits.
fixed_conversion!(I13F3);
/// [`FixedI16`] with 12 integer bits and four fractional bits.
fixed_conversion!(I12F4);
/// [`FixedI16`] with 11 integer bits and five fractional bits.
fixed_conversion!(I11F5);
/// [`FixedI16`] with 10 integer bits and six fractional bits.
fixed_conversion!(I10F6);
/// [`FixedI16`] with nine integer bits and seven fractional bits.
fixed_conversion!(I9F7);
/// [`FixedI16`] with eight integer bits and eight fractional bits.
fixed_conversion!(I8F8);
/// [`FixedI16`] with seven integer bits and nine fractional bits.
fixed_conversion!(I7F9);
/// [`FixedI16`] with six integer bits and 10 fractional bits.
fixed_conversion!(I6F10);
/// [`FixedI16`] with five integer bits and 11 fractional bits.
fixed_conversion!(I5F11);
/// [`FixedI16`] with four integer bits and 12 fractional bits.
fixed_conversion!(I4F12);
/// [`FixedI16`] with three integer bits and 13 fractional bits.
fixed_conversion!(I3F13);
/// [`FixedI16`] with two integer bits and 14 fractional bits.
fixed_conversion!(I2F14);
/// [`FixedI16`] with one integer bit and 15 fractional bits.
fixed_conversion!(I1F15);
/// [`FixedI16`] with no integer bits and 16 fractional bits.
fixed_conversion!(I0F16);
/// [`FixedI32`] with 32 integer bits and no fractional bits.
fixed_conversion!(I32F0);
/// [`FixedI32`] with 31 integer bits and one fractional bit.
fixed_conversion!(I31F1);
/// [`FixedI32`] with 30 integer bits and two fractional bits.
fixed_conversion!(I30F2);
/// [`FixedI32`] with 29 integer bits and three fractional bits.
fixed_conversion!(I29F3);
/// [`FixedI32`] with 28 integer bits and four fractional bits.
fixed_conversion!(I28F4);
/// [`FixedI32`] with 27 integer bits and five fractional bits.
fixed_conversion!(I27F5);
/// [`FixedI32`] with 26 integer bits and six fractional bits.
fixed_conversion!(I26F6);
/// [`FixedI32`] with 25 integer bits and seven fractional bits.
fixed_conversion!(I25F7);
/// [`FixedI32`] with 24 integer bits and eight fractional bits.
fixed_conversion!(I24F8);
/// [`FixedI32`] with 23 integer bits and nine fractional bits.
fixed_conversion!(I23F9);
/// [`FixedI32`] with 22 integer bits and 10 fractional bits.
fixed_conversion!(I22F10);
/// [`FixedI32`] with 21 integer bits and 11 fractional bits.
fixed_conversion!(I21F11);
/// [`FixedI32`] with 20 integer bits and 12 fractional bits.
fixed_conversion!(I20F12);
/// [`FixedI32`] with 19 integer bits and 13 fractional bits.
fixed_conversion!(I19F13);
/// [`FixedI32`] with 18 integer bits and 14 fractional bits.
fixed_conversion!(I18F14);
/// [`FixedI32`] with 17 integer bits and 15 fractional bits.
fixed_conversion!(I17F15);
/// [`FixedI32`] with 16 integer bits and 16 fractional bits.
fixed_conversion!(I16F16);
/// [`FixedI32`] with 15 integer bits and 17 fractional bits.
fixed_conversion!(I15F17);
/// [`FixedI32`] with 14 integer bits and 18 fractional bits.
fixed_conversion!(I14F18);
/// [`FixedI32`] with 13 integer bits and 19 fractional bits.
fixed_conversion!(I13F19);
/// [`FixedI32`] with 12 integer bits and 20 fractional bits.
fixed_conversion!(I12F20);
/// [`FixedI32`] with 11 integer bits and 21 fractional bits.
fixed_conversion!(I11F21);
/// [`FixedI32`] with 10 integer bits and 22 fractional bits.
fixed_conversion!(I10F22);
/// [`FixedI32`] with nine integer bits and 23 fractional bits.
fixed_conversion!(I9F23);
/// [`FixedI32`] with eight integer bits and 24 fractional bits.
fixed_conversion!(I8F24);
/// [`FixedI32`] with seven integer bits and 25 fractional bits.
fixed_conversion!(I7F25);
/// [`FixedI32`] with six integer bits and 26 fractional bits.
fixed_conversion!(I6F26);
/// [`FixedI32`] with five integer bits and 27 fractional bits.
fixed_conversion!(I5F27);
/// [`FixedI32`] with four integer bits and 28 fractional bits.
fixed_conversion!(I4F28);
/// [`FixedI32`] with three integer bits and 29 fractional bits.
fixed_conversion!(I3F29);
/// [`FixedI32`] with two integer bits and 30 fractional bits.
fixed_conversion!(I2F30);
/// [`FixedI32`] with one integer bit and 31 fractional bits.
fixed_conversion!(I1F31);
/// [`FixedI32`] with no integer bits and 32 fractional bits.
fixed_conversion!(I0F32);
/// [`FixedI64`] with 64 integer bits and no fractional bits.
fixed_conversion!(I64F0);
/// [`FixedI64`] with 63 integer bits and one fractional bit.
fixed_conversion!(I63F1);
/// [`FixedI64`] with 62 integer bits and two fractional bits.
fixed_conversion!(I62F2);
/// [`FixedI64`] with 61 integer bits and three fractional bits.
fixed_conversion!(I61F3);
/// [`FixedI64`] with 60 integer bits and four fractional bits.
fixed_conversion!(I60F4);
/// [`FixedI64`] with 59 integer bits and five fractional bits.
fixed_conversion!(I59F5);
/// [`FixedI64`] with 58 integer bits and six fractional bits.
fixed_conversion!(I58F6);
/// [`FixedI64`] with 57 integer bits and seven fractional bits.
fixed_conversion!(I57F7);
/// [`FixedI64`] with 56 integer bits and eight fractional bits.
fixed_conversion!(I56F8);
/// [`FixedI64`] with 55 integer bits and nine fractional bits.
fixed_conversion!(I55F9);
/// [`FixedI64`] with 54 integer bits and 10 fractional bits.
fixed_conversion!(I54F10);
/// [`FixedI64`] with 53 integer bits and 11 fractional bits.
fixed_conversion!(I53F11);
/// [`FixedI64`] with 52 integer bits and 12 fractional bits.
fixed_conversion!(I52F12);
/// [`FixedI64`] with 51 integer bits and 13 fractional bits.
fixed_conversion!(I51F13);
/// [`FixedI64`] with 50 integer bits and 14 fractional bits.
fixed_conversion!(I50F14);
/// [`FixedI64`] with 49 integer bits and 15 fractional bits.
fixed_conversion!(I49F15);
/// [`FixedI64`] with 48 integer bits and 16 fractional bits.
fixed_conversion!(I48F16);
/// [`FixedI64`] with 47 integer bits and 17 fractional bits.
fixed_conversion!(I47F17);
/// [`FixedI64`] with 46 integer bits and 18 fractional bits.
fixed_conversion!(I46F18);
/// [`FixedI64`] with 45 integer bits and 19 fractional bits.
fixed_conversion!(I45F19);
/// [`FixedI64`] with 44 integer bits and 20 fractional bits.
fixed_conversion!(I44F20);
/// [`FixedI64`] with 43 integer bits and 21 fractional bits.
fixed_conversion!(I43F21);
/// [`FixedI64`] with 42 integer bits and 22 fractional bits.
fixed_conversion!(I42F22);
/// [`FixedI64`] with 41 integer bits and 23 fractional bits.
fixed_conversion!(I41F23);
/// [`FixedI64`] with 40 integer bits and 24 fractional bits.
fixed_conversion!(I40F24);
/// [`FixedI64`] with 39 integer bits and 25 fractional bits.
fixed_conversion!(I39F25);
/// [`FixedI64`] with 38 integer bits and 26 fractional bits.
fixed_conversion!(I38F26);
/// [`FixedI64`] with 37 integer bits and 27 fractional bits.
fixed_conversion!(I37F27);
/// [`FixedI64`] with 36 integer bits and 28 fractional bits.
fixed_conversion!(I36F28);
/// [`FixedI64`] with 35 integer bits and 29 fractional bits.
fixed_conversion!(I35F29);
/// [`FixedI64`] with 34 integer bits and 30 fractional bits.
fixed_conversion!(I34F30);
/// [`FixedI64`] with 33 integer bits and 31 fractional bits.
fixed_conversion!(I33F31);
/// [`FixedI64`] with 32 integer bits and 32 fractional bits.
fixed_conversion!(I32F32);
/// [`FixedI64`] with 31 integer bits and 33 fractional bits.
fixed_conversion!(I31F33);
/// [`FixedI64`] with 30 integer bits and 34 fractional bits.
fixed_conversion!(I30F34);
/// [`FixedI64`] with 29 integer bits and 35 fractional bits.
fixed_conversion!(I29F35);
/// [`FixedI64`] with 28 integer bits and 36 fractional bits.
fixed_conversion!(I28F36);
/// [`FixedI64`] with 27 integer bits and 37 fractional bits.
fixed_conversion!(I27F37);
/// [`FixedI64`] with 26 integer bits and 38 fractional bits.
fixed_conversion!(I26F38);
/// [`FixedI64`] with 25 integer bits and 39 fractional bits.
fixed_conversion!(I25F39);
/// [`FixedI64`] with 24 integer bits and 40 fractional bits.
fixed_conversion!(I24F40);
/// [`FixedI64`] with 23 integer bits and 41 fractional bits.
fixed_conversion!(I23F41);
/// [`FixedI64`] with 22 integer bits and 42 fractional bits.
fixed_conversion!(I22F42);
/// [`FixedI64`] with 21 integer bits and 43 fractional bits.
fixed_conversion!(I21F43);
/// [`FixedI64`] with 20 integer bits and 44 fractional bits.
fixed_conversion!(I20F44);
/// [`FixedI64`] with 19 integer bits and 45 fractional bits.
fixed_conversion!(I19F45);
/// [`FixedI64`] with 18 integer bits and 46 fractional bits.
fixed_conversion!(I18F46);
/// [`FixedI64`] with 17 integer bits and 47 fractional bits.
fixed_conversion!(I17F47);
/// [`FixedI64`] with 16 integer bits and 48 fractional bits.
fixed_conversion!(I16F48);
/// [`FixedI64`] with 15 integer bits and 49 fractional bits.
fixed_conversion!(I15F49);
/// [`FixedI64`] with 14 integer bits and 50 fractional bits.
fixed_conversion!(I14F50);
/// [`FixedI64`] with 13 integer bits and 51 fractional bits.
fixed_conversion!(I13F51);
/// [`FixedI64`] with 12 integer bits and 52 fractional bits.
fixed_conversion!(I12F52);
/// [`FixedI64`] with 11 integer bits and 53 fractional bits.
fixed_conversion!(I11F53);
/// [`FixedI64`] with 10 integer bits and 54 fractional bits.
fixed_conversion!(I10F54);
/// [`FixedI64`] with nine integer bits and 55 fractional bits.
fixed_conversion!(I9F55);
/// [`FixedI64`] with eight integer bits and 56 fractional bits.
fixed_conversion!(I8F56);
/// [`FixedI64`] with seven integer bits and 57 fractional bits.
fixed_conversion!(I7F57);
/// [`FixedI64`] with six integer bits and 58 fractional bits.
fixed_conversion!(I6F58);
/// [`FixedI64`] with five integer bits and 59 fractional bits.
fixed_conversion!(I5F59);
/// [`FixedI64`] with four integer bits and 60 fractional bits.
fixed_conversion!(I4F60);
/// [`FixedI64`] with three integer bits and 61 fractional bits.
fixed_conversion!(I3F61);
/// [`FixedI64`] with two integer bits and 62 fractional bits.
fixed_conversion!(I2F62);
/// [`FixedI64`] with one integer bit and 63 fractional bits.
fixed_conversion!(I1F63);
/// [`FixedI64`] with no integer bits and 64 fractional bits.
fixed_conversion!(I0F64);
/// [`FixedI128`] with 128 integer bits and no fractional bits.
fixed_conversion!(I128F0);
/// [`FixedI128`] with 127 integer bits and one fractional bit.
fixed_conversion!(I127F1);
/// [`FixedI128`] with 126 integer bits and two fractional bits.
fixed_conversion!(I126F2);
/// [`FixedI128`] with 125 integer bits and three fractional bits.
fixed_conversion!(I125F3);
/// [`FixedI128`] with 124 integer bits and four fractional bits.
fixed_conversion!(I124F4);
/// [`FixedI128`] with 123 integer bits and five fractional bits.
fixed_conversion!(I123F5);
/// [`FixedI128`] with 122 integer bits and six fractional bits.
fixed_conversion!(I122F6);
/// [`FixedI128`] with 121 integer bits and seven fractional bits.
fixed_conversion!(I121F7);
/// [`FixedI128`] with 120 integer bits and eight fractional bits.
fixed_conversion!(I120F8);
/// [`FixedI128`] with 119 integer bits and nine fractional bits.
fixed_conversion!(I119F9);
/// [`FixedI128`] with 118 integer bits and 10 fractional bits.
fixed_conversion!(I118F10);
/// [`FixedI128`] with 117 integer bits and 11 fractional bits.
fixed_conversion!(I117F11);
/// [`FixedI128`] with 116 integer bits and 12 fractional bits.
fixed_conversion!(I116F12);
/// [`FixedI128`] with 115 integer bits and 13 fractional bits.
fixed_conversion!(I115F13);
/// [`FixedI128`] with 114 integer bits and 14 fractional bits.
fixed_conversion!(I114F14);
/// [`FixedI128`] with 113 integer bits and 15 fractional bits.
fixed_conversion!(I113F15);
/// [`FixedI128`] with 112 integer bits and 16 fractional bits.
fixed_conversion!(I112F16);
/// [`FixedI128`] with 111 integer bits and 17 fractional bits.
fixed_conversion!(I111F17);
/// [`FixedI128`] with 110 integer bits and 18 fractional bits.
fixed_conversion!(I110F18);
/// [`FixedI128`] with 109 integer bits and 19 fractional bits.
fixed_conversion!(I109F19);
/// [`FixedI128`] with 108 integer bits and 20 fractional bits.
fixed_conversion!(I108F20);
/// [`FixedI128`] with 107 integer bits and 21 fractional bits.
fixed_conversion!(I107F21);
/// [`FixedI128`] with 106 integer bits and 22 fractional bits.
fixed_conversion!(I106F22);
/// [`FixedI128`] with 105 integer bits and 23 fractional bits.
fixed_conversion!(I105F23);
/// [`FixedI128`] with 104 integer bits and 24 fractional bits.
fixed_conversion!(I104F24);
/// [`FixedI128`] with 103 integer bits and 25 fractional bits.
fixed_conversion!(I103F25);
/// [`FixedI128`] with 102 integer bits and 26 fractional bits.
fixed_conversion!(I102F26);
/// [`FixedI128`] with 101 integer bits and 27 fractional bits.
fixed_conversion!(I101F27);
/// [`FixedI128`] with 100 integer bits and 28 fractional bits.
fixed_conversion!(I100F28);
/// [`FixedI128`] with 99 integer bits and 29 fractional bits.
fixed_conversion!(I99F29);
/// [`FixedI128`] with 98 integer bits and 30 fractional bits.
fixed_conversion!(I98F30);
/// [`FixedI128`] with 97 integer bits and 31 fractional bits.
fixed_conversion!(I97F31);
/// [`FixedI128`] with 96 integer bits and 32 fractional bits.
fixed_conversion!(I96F32);
/// [`FixedI128`] with 95 integer bits and 33 fractional bits.
fixed_conversion!(I95F33);
/// [`FixedI128`] with 94 integer bits and 34 fractional bits.
fixed_conversion!(I94F34);
/// [`FixedI128`] with 93 integer bits and 35 fractional bits.
fixed_conversion!(I93F35);
/// [`FixedI128`] with 92 integer bits and 36 fractional bits.
fixed_conversion!(I92F36);
/// [`FixedI128`] with 91 integer bits and 37 fractional bits.
fixed_conversion!(I91F37);
/// [`FixedI128`] with 90 integer bits and 38 fractional bits.
fixed_conversion!(I90F38);
/// [`FixedI128`] with 89 integer bits and 39 fractional bits.
fixed_conversion!(I89F39);
/// [`FixedI128`] with 88 integer bits and 40 fractional bits.
fixed_conversion!(I88F40);
/// [`FixedI128`] with 87 integer bits and 41 fractional bits.
fixed_conversion!(I87F41);
/// [`FixedI128`] with 86 integer bits and 42 fractional bits.
fixed_conversion!(I86F42);
/// [`FixedI128`] with 85 integer bits and 43 fractional bits.
fixed_conversion!(I85F43);
/// [`FixedI128`] with 84 integer bits and 44 fractional bits.
fixed_conversion!(I84F44);
/// [`FixedI128`] with 83 integer bits and 45 fractional bits.
fixed_conversion!(I83F45);
/// [`FixedI128`] with 82 integer bits and 46 fractional bits.
fixed_conversion!(I82F46);
/// [`FixedI128`] with 81 integer bits and 47 fractional bits.
fixed_conversion!(I81F47);
/// [`FixedI128`] with 80 integer bits and 48 fractional bits.
fixed_conversion!(I80F48);
/// [`FixedI128`] with 79 integer bits and 49 fractional bits.
fixed_conversion!(I79F49);
/// [`FixedI128`] with 78 integer bits and 50 fractional bits.
fixed_conversion!(I78F50);
/// [`FixedI128`] with 77 integer bits and 51 fractional bits.
fixed_conversion!(I77F51);
/// [`FixedI128`] with 76 integer bits and 52 fractional bits.
fixed_conversion!(I76F52);
/// [`FixedI128`] with 75 integer bits and 53 fractional bits.
fixed_conversion!(I75F53);
/// [`FixedI128`] with 74 integer bits and 54 fractional bits.
fixed_conversion!(I74F54);
/// [`FixedI128`] with 73 integer bits and 55 fractional bits.
fixed_conversion!(I73F55);
/// [`FixedI128`] with 72 integer bits and 56 fractional bits.
fixed_conversion!(I72F56);
/// [`FixedI128`] with 71 integer bits and 57 fractional bits.
fixed_conversion!(I71F57);
/// [`FixedI128`] with 70 integer bits and 58 fractional bits.
fixed_conversion!(I70F58);
/// [`FixedI128`] with 69 integer bits and 59 fractional bits.
fixed_conversion!(I69F59);
/// [`FixedI128`] with 68 integer bits and 60 fractional bits.
fixed_conversion!(I68F60);
/// [`FixedI128`] with 67 integer bits and 61 fractional bits.
fixed_conversion!(I67F61);
/// [`FixedI128`] with 66 integer bits and 62 fractional bits.
fixed_conversion!(I66F62);
/// [`FixedI128`] with 65 integer bits and 63 fractional bits.
fixed_conversion!(I65F63);
/// [`FixedI128`] with 64 integer bits and 64 fractional bits.
fixed_conversion!(I64F64);
/// [`FixedI128`] with 63 integer bits and 65 fractional bits.
fixed_conversion!(I63F65);
/// [`FixedI128`] with 62 integer bits and 66 fractional bits.
fixed_conversion!(I62F66);
/// [`FixedI128`] with 61 integer bits and 67 fractional bits.
fixed_conversion!(I61F67);
/// [`FixedI128`] with 60 integer bits and 68 fractional bits.
fixed_conversion!(I60F68);
/// [`FixedI128`] with 59 integer bits and 69 fractional bits.
fixed_conversion!(I59F69);
/// [`FixedI128`] with 58 integer bits and 70 fractional bits.
fixed_conversion!(I58F70);
/// [`FixedI128`] with 57 integer bits and 71 fractional bits.
fixed_conversion!(I57F71);
/// [`FixedI128`] with 56 integer bits and 72 fractional bits.
fixed_conversion!(I56F72);
/// [`FixedI128`] with 55 integer bits and 73 fractional bits.
fixed_conversion!(I55F73);
/// [`FixedI128`] with 54 integer bits and 74 fractional bits.
fixed_conversion!(I54F74);
/// [`FixedI128`] with 53 integer bits and 75 fractional bits.
fixed_conversion!(I53F75);
/// [`FixedI128`] with 52 integer bits and 76 fractional bits.
fixed_conversion!(I52F76);
/// [`FixedI128`] with 51 integer bits and 77 fractional bits.
fixed_conversion!(I51F77);
/// [`FixedI128`] with 50 integer bits and 78 fractional bits.
fixed_conversion!(I50F78);
/// [`FixedI128`] with 49 integer bits and 79 fractional bits.
fixed_conversion!(I49F79);
/// [`FixedI128`] with 48 integer bits and 80 fractional bits.
fixed_conversion!(I48F80);
/// [`FixedI128`] with 47 integer bits and 81 fractional bits.
fixed_conversion!(I47F81);
/// [`FixedI128`] with 46 integer bits and 82 fractional bits.
fixed_conversion!(I46F82);
/// [`FixedI128`] with 45 integer bits and 83 fractional bits.
fixed_conversion!(I45F83);
/// [`FixedI128`] with 44 integer bits and 84 fractional bits.
fixed_conversion!(I44F84);
/// [`FixedI128`] with 43 integer bits and 85 fractional bits.
fixed_conversion!(I43F85);
/// [`FixedI128`] with 42 integer bits and 86 fractional bits.
fixed_conversion!(I42F86);
/// [`FixedI128`] with 41 integer bits and 87 fractional bits.
fixed_conversion!(I41F87);
/// [`FixedI128`] with 40 integer bits and 88 fractional bits.
fixed_conversion!(I40F88);
/// [`FixedI128`] with 39 integer bits and 89 fractional bits.
fixed_conversion!(I39F89);
/// [`FixedI128`] with 38 integer bits and 90 fractional bits.
fixed_conversion!(I38F90);
/// [`FixedI128`] with 37 integer bits and 91 fractional bits.
fixed_conversion!(I37F91);
/// [`FixedI128`] with 36 integer bits and 92 fractional bits.
fixed_conversion!(I36F92);
/// [`FixedI128`] with 35 integer bits and 93 fractional bits.
fixed_conversion!(I35F93);
/// [`FixedI128`] with 34 integer bits and 94 fractional bits.
fixed_conversion!(I34F94);
/// [`FixedI128`] with 33 integer bits and 95 fractional bits.
fixed_conversion!(I33F95);
/// [`FixedI128`] with 32 integer bits and 96 fractional bits.
fixed_conversion!(I32F96);
/// [`FixedI128`] with 31 integer bits and 97 fractional bits.
fixed_conversion!(I31F97);
/// [`FixedI128`] with 30 integer bits and 98 fractional bits.
fixed_conversion!(I30F98);
/// [`FixedI128`] with 29 integer bits and 99 fractional bits.
fixed_conversion!(I29F99);
/// [`FixedI128`] with 28 integer bits and 100 fractional bits.
fixed_conversion!(I28F100);
/// [`FixedI128`] with 27 integer bits and 101 fractional bits.
fixed_conversion!(I27F101);
/// [`FixedI128`] with 26 integer bits and 102 fractional bits.
fixed_conversion!(I26F102);
/// [`FixedI128`] with 25 integer bits and 103 fractional bits.
fixed_conversion!(I25F103);
/// [`FixedI128`] with 24 integer bits and 104 fractional bits.
fixed_conversion!(I24F104);
/// [`FixedI128`] with 23 integer bits and 105 fractional bits.
fixed_conversion!(I23F105);
/// [`FixedI128`] with 22 integer bits and 106 fractional bits.
fixed_conversion!(I22F106);
/// [`FixedI128`] with 21 integer bits and 107 fractional bits.
fixed_conversion!(I21F107);
/// [`FixedI128`] with 20 integer bits and 108 fractional bits.
fixed_conversion!(I20F108);
/// [`FixedI128`] with 19 integer bits and 109 fractional bits.
fixed_conversion!(I19F109);
/// [`FixedI128`] with 18 integer bits and 110 fractional bits.
fixed_conversion!(I18F110);
/// [`FixedI128`] with 17 integer bits and 111 fractional bits.
fixed_conversion!(I17F111);
/// [`FixedI128`] with 16 integer bits and 112 fractional bits.
fixed_conversion!(I16F112);
/// [`FixedI128`] with 15 integer bits and 113 fractional bits.
fixed_conversion!(I15F113);
/// [`FixedI128`] with 14 integer bits and 114 fractional bits.
fixed_conversion!(I14F114);
/// [`FixedI128`] with 13 integer bits and 115 fractional bits.
fixed_conversion!(I13F115);
/// [`FixedI128`] with 12 integer bits and 116 fractional bits.
fixed_conversion!(I12F116);
/// [`FixedI128`] with 11 integer bits and 117 fractional bits.
fixed_conversion!(I11F117);
/// [`FixedI128`] with 10 integer bits and 118 fractional bits.
fixed_conversion!(I10F118);
/// [`FixedI128`] with nine integer bits and 119 fractional bits.
fixed_conversion!(I9F119);
/// [`FixedI128`] with eight integer bits and 120 fractional bits.
fixed_conversion!(I8F120);
/// [`FixedI128`] with seven integer bits and 121 fractional bits.
fixed_conversion!(I7F121);
/// [`FixedI128`] with six integer bits and 122 fractional bits.
fixed_conversion!(I6F122);
/// [`FixedI128`] with five integer bits and 123 fractional bits.
fixed_conversion!(I5F123);
/// [`FixedI128`] with four integer bits and 124 fractional bits.
fixed_conversion!(I4F124);
/// [`FixedI128`] with three integer bits and 125 fractional bits.
fixed_conversion!(I3F125);
/// [`FixedI128`] with two integer bits and 126 fractional bits.
fixed_conversion!(I2F126);
/// [`FixedI128`] with one integer bit and 127 fractional bits.
fixed_conversion!(I1F127);
/// [`FixedI128`] with no integer bits and 128 fractional bits.
fixed_conversion!(I0F128);
/// [`FixedU8`] with eight integer bits and no fractional bits.
fixed_conversion!(U8F0);
/// [`FixedU8`] with seven integer bits and one fractional bit.
fixed_conversion!(U7F1);
/// [`FixedU8`] with six integer bits and two fractional bits.
fixed_conversion!(U6F2);
/// [`FixedU8`] with five integer bits and three fractional bits.
fixed_conversion!(U5F3);
/// [`FixedU8`] with four integer bits and four fractional bits.
fixed_conversion!(U4F4);
/// [`FixedU8`] with three integer bits and five fractional bits.
fixed_conversion!(U3F5);
/// [`FixedU8`] with two integer bits and six fractional bits.
fixed_conversion!(U2F6);
/// [`FixedU8`] with one integer bit and seven fractional bits.
fixed_conversion!(U1F7);
/// [`FixedU8`] with no integer bits and eight fractional bits.
fixed_conversion!(U0F8);
/// [`FixedU16`] with 16 integer bits and no fractional bits.
fixed_conversion!(U16F0);
/// [`FixedU16`] with 15 integer bits and one fractional bit.
fixed_conversion!(U15F1);
/// [`FixedU16`] with 14 integer bits and two fractional bits.
fixed_conversion!(U14F2);
/// [`FixedU16`] with 13 integer bits and three fractional bits.
fixed_conversion!(U13F3);
/// [`FixedU16`] with 12 integer bits and four fractional bits.
fixed_conversion!(U12F4);
/// [`FixedU16`] with 11 integer bits and five fractional bits.
fixed_conversion!(U11F5);
/// [`FixedU16`] with 10 integer bits and six fractional bits.
fixed_conversion!(U10F6);
/// [`FixedU16`] with nine integer bits and seven fractional bits.
fixed_conversion!(U9F7);
/// [`FixedU16`] with eight integer bits and eight fractional bits.
fixed_conversion!(U8F8);
/// [`FixedU16`] with seven integer bits and nine fractional bits.
fixed_conversion!(U7F9);
/// [`FixedU16`] with six integer bits and 10 fractional bits.
fixed_conversion!(U6F10);
/// [`FixedU16`] with five integer bits and 11 fractional bits.
fixed_conversion!(U5F11);
/// [`FixedU16`] with four integer bits and 12 fractional bits.
fixed_conversion!(U4F12);
/// [`FixedU16`] with three integer bits and 13 fractional bits.
fixed_conversion!(U3F13);
/// [`FixedU16`] with two integer bits and 14 fractional bits.
fixed_conversion!(U2F14);
/// [`FixedU16`] with one integer bit and 15 fractional bits.
fixed_conversion!(U1F15);
/// [`FixedU16`] with no integer bits and 16 fractional bits.
fixed_conversion!(U0F16);
/// [`FixedU32`] with 32 integer bits and no fractional bits.
fixed_conversion!(U32F0);
/// [`FixedU32`] with 31 integer bits and one fractional bit.
fixed_conversion!(U31F1);
/// [`FixedU32`] with 30 integer bits and two fractional bits.
fixed_conversion!(U30F2);
/// [`FixedU32`] with 29 integer bits and three fractional bits.
fixed_conversion!(U29F3);
/// [`FixedU32`] with 28 integer bits and four fractional bits.
fixed_conversion!(U28F4);
/// [`FixedU32`] with 27 integer bits and five fractional bits.
fixed_conversion!(U27F5);
/// [`FixedU32`] with 26 integer bits and six fractional bits.
fixed_conversion!(U26F6);
/// [`FixedU32`] with 25 integer bits and seven fractional bits.
fixed_conversion!(U25F7);
/// [`FixedU32`] with 24 integer bits and eight fractional bits.
fixed_conversion!(U24F8);
/// [`FixedU32`] with 23 integer bits and nine fractional bits.
fixed_conversion!(U23F9);
/// [`FixedU32`] with 22 integer bits and 10 fractional bits.
fixed_conversion!(U22F10);
/// [`FixedU32`] with 21 integer bits and 11 fractional bits.
fixed_conversion!(U21F11);
/// [`FixedU32`] with 20 integer bits and 12 fractional bits.
fixed_conversion!(U20F12);
/// [`FixedU32`] with 19 integer bits and 13 fractional bits.
fixed_conversion!(U19F13);
/// [`FixedU32`] with 18 integer bits and 14 fractional bits.
fixed_conversion!(U18F14);
/// [`FixedU32`] with 17 integer bits and 15 fractional bits.
fixed_conversion!(U17F15);
/// [`FixedU32`] with 16 integer bits and 16 fractional bits.
fixed_conversion!(U16F16);
/// [`FixedU32`] with 15 integer bits and 17 fractional bits.
fixed_conversion!(U15F17);
/// [`FixedU32`] with 14 integer bits and 18 fractional bits.
fixed_conversion!(U14F18);
/// [`FixedU32`] with 13 integer bits and 19 fractional bits.
fixed_conversion!(U13F19);
/// [`FixedU32`] with 12 integer bits and 20 fractional bits.
fixed_conversion!(U12F20);
/// [`FixedU32`] with 11 integer bits and 21 fractional bits.
fixed_conversion!(U11F21);
/// [`FixedU32`] with 10 integer bits and 22 fractional bits.
fixed_conversion!(U10F22);
/// [`FixedU32`] with nine integer bits and 23 fractional bits.
fixed_conversion!(U9F23);
/// [`FixedU32`] with eight integer bits and 24 fractional bits.
fixed_conversion!(U8F24);
/// [`FixedU32`] with seven integer bits and 25 fractional bits.
fixed_conversion!(U7F25);
/// [`FixedU32`] with six integer bits and 26 fractional bits.
fixed_conversion!(U6F26);
/// [`FixedU32`] with five integer bits and 27 fractional bits.
fixed_conversion!(U5F27);
/// [`FixedU32`] with four integer bits and 28 fractional bits.
fixed_conversion!(U4F28);
/// [`FixedU32`] with three integer bits and 29 fractional bits.
fixed_conversion!(U3F29);
/// [`FixedU32`] with two integer bits and 30 fractional bits.
fixed_conversion!(U2F30);
/// [`FixedU32`] with one integer bit and 31 fractional bits.
fixed_conversion!(U1F31);
/// [`FixedU32`] with no integer bits and 32 fractional bits.
fixed_conversion!(U0F32);
/// [`FixedU64`] with 64 integer bits and no fractional bits.
fixed_conversion!(U64F0);
/// [`FixedU64`] with 63 integer bits and one fractional bit.
fixed_conversion!(U63F1);
/// [`FixedU64`] with 62 integer bits and two fractional bits.
fixed_conversion!(U62F2);
/// [`FixedU64`] with 61 integer bits and three fractional bits.
fixed_conversion!(U61F3);
/// [`FixedU64`] with 60 integer bits and four fractional bits.
fixed_conversion!(U60F4);
/// [`FixedU64`] with 59 integer bits and five fractional bits.
fixed_conversion!(U59F5);
/// [`FixedU64`] with 58 integer bits and six fractional bits.
fixed_conversion!(U58F6);
/// [`FixedU64`] with 57 integer bits and seven fractional bits.
fixed_conversion!(U57F7);
/// [`FixedU64`] with 56 integer bits and eight fractional bits.
fixed_conversion!(U56F8);
/// [`FixedU64`] with 55 integer bits and nine fractional bits.
fixed_conversion!(U55F9);
/// [`FixedU64`] with 54 integer bits and 10 fractional bits.
fixed_conversion!(U54F10);
/// [`FixedU64`] with 53 integer bits and 11 fractional bits.
fixed_conversion!(U53F11);
/// [`FixedU64`] with 52 integer bits and 12 fractional bits.
fixed_conversion!(U52F12);
/// [`FixedU64`] with 51 integer bits and 13 fractional bits.
fixed_conversion!(U51F13);
/// [`FixedU64`] with 50 integer bits and 14 fractional bits.
fixed_conversion!(U50F14);
/// [`FixedU64`] with 49 integer bits and 15 fractional bits.
fixed_conversion!(U49F15);
/// [`FixedU64`] with 48 integer bits and 16 fractional bits.
fixed_conversion!(U48F16);
/// [`FixedU64`] with 47 integer bits and 17 fractional bits.
fixed_conversion!(U47F17);
/// [`FixedU64`] with 46 integer bits and 18 fractional bits.
fixed_conversion!(U46F18);
/// [`FixedU64`] with 45 integer bits and 19 fractional bits.
fixed_conversion!(U45F19);
/// [`FixedU64`] with 44 integer bits and 20 fractional bits.
fixed_conversion!(U44F20);
/// [`FixedU64`] with 43 integer bits and 21 fractional bits.
fixed_conversion!(U43F21);
/// [`FixedU64`] with 42 integer bits and 22 fractional bits.
fixed_conversion!(U42F22);
/// [`FixedU64`] with 41 integer bits and 23 fractional bits.
fixed_conversion!(U41F23);
/// [`FixedU64`] with 40 integer bits and 24 fractional bits.
fixed_conversion!(U40F24);
/// [`FixedU64`] with 39 integer bits and 25 fractional bits.
fixed_conversion!(U39F25);
/// [`FixedU64`] with 38 integer bits and 26 fractional bits.
fixed_conversion!(U38F26);
/// [`FixedU64`] with 37 integer bits and 27 fractional bits.
fixed_conversion!(U37F27);
/// [`FixedU64`] with 36 integer bits and 28 fractional bits.
fixed_conversion!(U36F28);
/// [`FixedU64`] with 35 integer bits and 29 fractional bits.
fixed_conversion!(U35F29);
/// [`FixedU64`] with 34 integer bits and 30 fractional bits.
fixed_conversion!(U34F30);
/// [`FixedU64`] with 33 integer bits and 31 fractional bits.
fixed_conversion!(U33F31);
/// [`FixedU64`] with 32 integer bits and 32 fractional bits.
fixed_conversion!(U32F32);
/// [`FixedU64`] with 31 integer bits and 33 fractional bits.
fixed_conversion!(U31F33);
/// [`FixedU64`] with 30 integer bits and 34 fractional bits.
fixed_conversion!(U30F34);
/// [`FixedU64`] with 29 integer bits and 35 fractional bits.
fixed_conversion!(U29F35);
/// [`FixedU64`] with 28 integer bits and 36 fractional bits.
fixed_conversion!(U28F36);
/// [`FixedU64`] with 27 integer bits and 37 fractional bits.
fixed_conversion!(U27F37);
/// [`FixedU64`] with 26 integer bits and 38 fractional bits.
fixed_conversion!(U26F38);
/// [`FixedU64`] with 25 integer bits and 39 fractional bits.
fixed_conversion!(U25F39);
/// [`FixedU64`] with 24 integer bits and 40 fractional bits.
fixed_conversion!(U24F40);
/// [`FixedU64`] with 23 integer bits and 41 fractional bits.
fixed_conversion!(U23F41);
/// [`FixedU64`] with 22 integer bits and 42 fractional bits.
fixed_conversion!(U22F42);
/// [`FixedU64`] with 21 integer bits and 43 fractional bits.
fixed_conversion!(U21F43);
/// [`FixedU64`] with 20 integer bits and 44 fractional bits.
fixed_conversion!(U20F44);
/// [`FixedU64`] with 19 integer bits and 45 fractional bits.
fixed_conversion!(U19F45);
/// [`FixedU64`] with 18 integer bits and 46 fractional bits.
fixed_conversion!(U18F46);
/// [`FixedU64`] with 17 integer bits and 47 fractional bits.
fixed_conversion!(U17F47);
/// [`FixedU64`] with 16 integer bits and 48 fractional bits.
fixed_conversion!(U16F48);
/// [`FixedU64`] with 15 integer bits and 49 fractional bits.
fixed_conversion!(U15F49);
/// [`FixedU64`] with 14 integer bits and 50 fractional bits.
fixed_conversion!(U14F50);
/// [`FixedU64`] with 13 integer bits and 51 fractional bits.
fixed_conversion!(U13F51);
/// [`FixedU64`] with 12 integer bits and 52 fractional bits.
fixed_conversion!(U12F52);
/// [`FixedU64`] with 11 integer bits and 53 fractional bits.
fixed_conversion!(U11F53);
/// [`FixedU64`] with 10 integer bits and 54 fractional bits.
fixed_conversion!(U10F54);
/// [`FixedU64`] with nine integer bits and 55 fractional bits.
fixed_conversion!(U9F55);
/// [`FixedU64`] with eight integer bits and 56 fractional bits.
fixed_conversion!(U8F56);
/// [`FixedU64`] with seven integer bits and 57 fractional bits.
fixed_conversion!(U7F57);
/// [`FixedU64`] with six integer bits and 58 fractional bits.
fixed_conversion!(U6F58);
/// [`FixedU64`] with five integer bits and 59 fractional bits.
fixed_conversion!(U5F59);
/// [`FixedU64`] with four integer bits and 60 fractional bits.
fixed_conversion!(U4F60);
/// [`FixedU64`] with three integer bits and 61 fractional bits.
fixed_conversion!(U3F61);
/// [`FixedU64`] with two integer bits and 62 fractional bits.
fixed_conversion!(U2F62);
/// [`FixedU64`] with one integer bit and 63 fractional bits.
fixed_conversion!(U1F63);
/// [`FixedU64`] with no integer bits and 64 fractional bits.
fixed_conversion!(U0F64);
/// [`FixedU128`] with 128 integer bits and no fractional bits.
fixed_conversion!(U128F0);
/// [`FixedU128`] with 127 integer bits and one fractional bit.
fixed_conversion!(U127F1);
/// [`FixedU128`] with 126 integer bits and two fractional bits.
fixed_conversion!(U126F2);
/// [`FixedU128`] with 125 integer bits and three fractional bits.
fixed_conversion!(U125F3);
/// [`FixedU128`] with 124 integer bits and four fractional bits.
fixed_conversion!(U124F4);
/// [`FixedU128`] with 123 integer bits and five fractional bits.
fixed_conversion!(U123F5);
/// [`FixedU128`] with 122 integer bits and six fractional bits.
fixed_conversion!(U122F6);
/// [`FixedU128`] with 121 integer bits and seven fractional bits.
fixed_conversion!(U121F7);
/// [`FixedU128`] with 120 integer bits and eight fractional bits.
fixed_conversion!(U120F8);
/// [`FixedU128`] with 119 integer bits and nine fractional bits.
fixed_conversion!(U119F9);
/// [`FixedU128`] with 118 integer bits and 10 fractional bits.
fixed_conversion!(U118F10);
/// [`FixedU128`] with 117 integer bits and 11 fractional bits.
fixed_conversion!(U117F11);
/// [`FixedU128`] with 116 integer bits and 12 fractional bits.
fixed_conversion!(U116F12);
/// [`FixedU128`] with 115 integer bits and 13 fractional bits.
fixed_conversion!(U115F13);
/// [`FixedU128`] with 114 integer bits and 14 fractional bits.
fixed_conversion!(U114F14);
/// [`FixedU128`] with 113 integer bits and 15 fractional bits.
fixed_conversion!(U113F15);
/// [`FixedU128`] with 112 integer bits and 16 fractional bits.
fixed_conversion!(U112F16);
/// [`FixedU128`] with 111 integer bits and 17 fractional bits.
fixed_conversion!(U111F17);
/// [`FixedU128`] with 110 integer bits and 18 fractional bits.
fixed_conversion!(U110F18);
/// [`FixedU128`] with 109 integer bits and 19 fractional bits.
fixed_conversion!(U109F19);
/// [`FixedU128`] with 108 integer bits and 20 fractional bits.
fixed_conversion!(U108F20);
/// [`FixedU128`] with 107 integer bits and 21 fractional bits.
fixed_conversion!(U107F21);
/// [`FixedU128`] with 106 integer bits and 22 fractional bits.
fixed_conversion!(U106F22);
/// [`FixedU128`] with 105 integer bits and 23 fractional bits.
fixed_conversion!(U105F23);
/// [`FixedU128`] with 104 integer bits and 24 fractional bits.
fixed_conversion!(U104F24);
/// [`FixedU128`] with 103 integer bits and 25 fractional bits.
fixed_conversion!(U103F25);
/// [`FixedU128`] with 102 integer bits and 26 fractional bits.
fixed_conversion!(U102F26);
/// [`FixedU128`] with 101 integer bits and 27 fractional bits.
fixed_conversion!(U101F27);
/// [`FixedU128`] with 100 integer bits and 28 fractional bits.
fixed_conversion!(U100F28);
/// [`FixedU128`] with 99 integer bits and 29 fractional bits.
fixed_conversion!(U99F29);
/// [`FixedU128`] with 98 integer bits and 30 fractional bits.
fixed_conversion!(U98F30);
/// [`FixedU128`] with 97 integer bits and 31 fractional bits.
fixed_conversion!(U97F31);
/// [`FixedU128`] with 96 integer bits and 32 fractional bits.
fixed_conversion!(U96F32);
/// [`FixedU128`] with 95 integer bits and 33 fractional bits.
fixed_conversion!(U95F33);
/// [`FixedU128`] with 94 integer bits and 34 fractional bits.
fixed_conversion!(U94F34);
/// [`FixedU128`] with 93 integer bits and 35 fractional bits.
fixed_conversion!(U93F35);
/// [`FixedU128`] with 92 integer bits and 36 fractional bits.
fixed_conversion!(U92F36);
/// [`FixedU128`] with 91 integer bits and 37 fractional bits.
fixed_conversion!(U91F37);
/// [`FixedU128`] with 90 integer bits and 38 fractional bits.
fixed_conversion!(U90F38);
/// [`FixedU128`] with 89 integer bits and 39 fractional bits.
fixed_conversion!(U89F39);
/// [`FixedU128`] with 88 integer bits and 40 fractional bits.
fixed_conversion!(U88F40);
/// [`FixedU128`] with 87 integer bits and 41 fractional bits.
fixed_conversion!(U87F41);
/// [`FixedU128`] with 86 integer bits and 42 fractional bits.
fixed_conversion!(U86F42);
/// [`FixedU128`] with 85 integer bits and 43 fractional bits.
fixed_conversion!(U85F43);
/// [`FixedU128`] with 84 integer bits and 44 fractional bits.
fixed_conversion!(U84F44);
/// [`FixedU128`] with 83 integer bits and 45 fractional bits.
fixed_conversion!(U83F45);
/// [`FixedU128`] with 82 integer bits and 46 fractional bits.
fixed_conversion!(U82F46);
/// [`FixedU128`] with 81 integer bits and 47 fractional bits.
fixed_conversion!(U81F47);
/// [`FixedU128`] with 80 integer bits and 48 fractional bits.
fixed_conversion!(U80F48);
/// [`FixedU128`] with 79 integer bits and 49 fractional bits.
fixed_conversion!(U79F49);
/// [`FixedU128`] with 78 integer bits and 50 fractional bits.
fixed_conversion!(U78F50);
/// [`FixedU128`] with 77 integer bits and 51 fractional bits.
fixed_conversion!(U77F51);
/// [`FixedU128`] with 76 integer bits and 52 fractional bits.
fixed_conversion!(U76F52);
/// [`FixedU128`] with 75 integer bits and 53 fractional bits.
fixed_conversion!(U75F53);
/// [`FixedU128`] with 74 integer bits and 54 fractional bits.
fixed_conversion!(U74F54);
/// [`FixedU128`] with 73 integer bits and 55 fractional bits.
fixed_conversion!(U73F55);
/// [`FixedU128`] with 72 integer bits and 56 fractional bits.
fixed_conversion!(U72F56);
/// [`FixedU128`] with 71 integer bits and 57 fractional bits.
fixed_conversion!(U71F57);
/// [`FixedU128`] with 70 integer bits and 58 fractional bits.
fixed_conversion!(U70F58);
/// [`FixedU128`] with 69 integer bits and 59 fractional bits.
fixed_conversion!(U69F59);
/// [`FixedU128`] with 68 integer bits and 60 fractional bits.
fixed_conversion!(U68F60);
/// [`FixedU128`] with 67 integer bits and 61 fractional bits.
fixed_conversion!(U67F61);
/// [`FixedU128`] with 66 integer bits and 62 fractional bits.
fixed_conversion!(U66F62);
/// [`FixedU128`] with 65 integer bits and 63 fractional bits.
fixed_conversion!(U65F63);
/// [`FixedU128`] with 64 integer bits and 64 fractional bits.
fixed_conversion!(U64F64);
/// [`FixedU128`] with 63 integer bits and 65 fractional bits.
fixed_conversion!(U63F65);
/// [`FixedU128`] with 62 integer bits and 66 fractional bits.
fixed_conversion!(U62F66);
/// [`FixedU128`] with 61 integer bits and 67 fractional bits.
fixed_conversion!(U61F67);
/// [`FixedU128`] with 60 integer bits and 68 fractional bits.
fixed_conversion!(U60F68);
/// [`FixedU128`] with 59 integer bits and 69 fractional bits.
fixed_conversion!(U59F69);
/// [`FixedU128`] with 58 integer bits and 70 fractional bits.
fixed_conversion!(U58F70);
/// [`FixedU128`] with 57 integer bits and 71 fractional bits.
fixed_conversion!(U57F71);
/// [`FixedU128`] with 56 integer bits and 72 fractional bits.
fixed_conversion!(U56F72);
/// [`FixedU128`] with 55 integer bits and 73 fractional bits.
fixed_conversion!(U55F73);
/// [`FixedU128`] with 54 integer bits and 74 fractional bits.
fixed_conversion!(U54F74);
/// [`FixedU128`] with 53 integer bits and 75 fractional bits.
fixed_conversion!(U53F75);
/// [`FixedU128`] with 52 integer bits and 76 fractional bits.
fixed_conversion!(U52F76);
/// [`FixedU128`] with 51 integer bits and 77 fractional bits.
fixed_conversion!(U51F77);
/// [`FixedU128`] with 50 integer bits and 78 fractional bits.
fixed_conversion!(U50F78);
/// [`FixedU128`] with 49 integer bits and 79 fractional bits.
fixed_conversion!(U49F79);
/// [`FixedU128`] with 48 integer bits and 80 fractional bits.
fixed_conversion!(U48F80);
/// [`FixedU128`] with 47 integer bits and 81 fractional bits.
fixed_conversion!(U47F81);
/// [`FixedU128`] with 46 integer bits and 82 fractional bits.
fixed_conversion!(U46F82);
/// [`FixedU128`] with 45 integer bits and 83 fractional bits.
fixed_conversion!(U45F83);
/// [`FixedU128`] with 44 integer bits and 84 fractional bits.
fixed_conversion!(U44F84);
/// [`FixedU128`] with 43 integer bits and 85 fractional bits.
fixed_conversion!(U43F85);
/// [`FixedU128`] with 42 integer bits and 86 fractional bits.
fixed_conversion!(U42F86);
/// [`FixedU128`] with 41 integer bits and 87 fractional bits.
fixed_conversion!(U41F87);
/// [`FixedU128`] with 40 integer bits and 88 fractional bits.
fixed_conversion!(U40F88);
/// [`FixedU128`] with 39 integer bits and 89 fractional bits.
fixed_conversion!(U39F89);
/// [`FixedU128`] with 38 integer bits and 90 fractional bits.
fixed_conversion!(U38F90);
/// [`FixedU128`] with 37 integer bits and 91 fractional bits.
fixed_conversion!(U37F91);
/// [`FixedU128`] with 36 integer bits and 92 fractional bits.
fixed_conversion!(U36F92);
/// [`FixedU128`] with 35 integer bits and 93 fractional bits.
fixed_conversion!(U35F93);
/// [`FixedU128`] with 34 integer bits and 94 fractional bits.
fixed_conversion!(U34F94);
/// [`FixedU128`] with 33 integer bits and 95 fractional bits.
fixed_conversion!(U33F95);
/// [`FixedU128`] with 32 integer bits and 96 fractional bits.
fixed_conversion!(U32F96);
/// [`FixedU128`] with 31 integer bits and 97 fractional bits.
fixed_conversion!(U31F97);
/// [`FixedU128`] with 30 integer bits and 98 fractional bits.
fixed_conversion!(U30F98);
/// [`FixedU128`] with 29 integer bits and 99 fractional bits.
fixed_conversion!(U29F99);
/// [`FixedU128`] with 28 integer bits and 100 fractional bits.
fixed_conversion!(U28F100);
/// [`FixedU128`] with 27 integer bits and 101 fractional bits.
fixed_conversion!(U27F101);
/// [`FixedU128`] with 26 integer bits and 102 fractional bits.
fixed_conversion!(U26F102);
/// [`FixedU128`] with 25 integer bits and 103 fractional bits.
fixed_conversion!(U25F103);
/// [`FixedU128`] with 24 integer bits and 104 fractional bits.
fixed_conversion!(U24F104);
/// [`FixedU128`] with 23 integer bits and 105 fractional bits.
fixed_conversion!(U23F105);
/// [`FixedU128`] with 22 integer bits and 106 fractional bits.
fixed_conversion!(U22F106);
/// [`FixedU128`] with 21 integer bits and 107 fractional bits.
fixed_conversion!(U21F107);
/// [`FixedU128`] with 20 integer bits and 108 fractional bits.
fixed_conversion!(U20F108);
/// [`FixedU128`] with 19 integer bits and 109 fractional bits.
fixed_conversion!(U19F109);
/// [`FixedU128`] with 18 integer bits and 110 fractional bits.
fixed_conversion!(U18F110);
/// [`FixedU128`] with 17 integer bits and 111 fractional bits.
fixed_conversion!(U17F111);
/// [`FixedU128`] with 16 integer bits and 112 fractional bits.
fixed_conversion!(U16F112);
/// [`FixedU128`] with 15 integer bits and 113 fractional bits.
fixed_conversion!(U15F113);
/// [`FixedU128`] with 14 integer bits and 114 fractional bits.
fixed_conversion!(U14F114);
/// [`FixedU128`] with 13 integer bits and 115 fractional bits.
fixed_conversion!(U13F115);
/// [`FixedU128`] with 12 integer bits and 116 fractional bits.
fixed_conversion!(U12F116);
/// [`FixedU128`] with 11 integer bits and 117 fractional bits.
fixed_conversion!(U11F117);
/// [`FixedU128`] with 10 integer bits and 118 fractional bits.
fixed_conversion!(U10F118);
/// [`FixedU128`] with nine integer bits and 119 fractional bits.
fixed_conversion!(U9F119);
/// [`FixedU128`] with eight integer bits and 120 fractional bits.
fixed_conversion!(U8F120);
/// [`FixedU128`] with seven integer bits and 121 fractional bits.
fixed_conversion!(U7F121);
/// [`FixedU128`] with six integer bits and 122 fractional bits.
fixed_conversion!(U6F122);
/// [`FixedU128`] with five integer bits and 123 fractional bits.
fixed_conversion!(U5F123);
/// [`FixedU128`] with four integer bits and 124 fractional bits.
fixed_conversion!(U4F124);
/// [`FixedU128`] with three integer bits and 125 fractional bits.
fixed_conversion!(U3F125);
/// [`FixedU128`] with two integer bits and 126 fractional bits.
fixed_conversion!(U2F126);
/// [`FixedU128`] with one integer bit and 127 fractional bits.
fixed_conversion!(U1F127);
/// [`FixedU128`] with no integer bits and 128 fractional bits.
fixed_conversion!(U0F128);
