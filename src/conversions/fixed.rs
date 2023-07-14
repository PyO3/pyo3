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

fixed_conversion!(I8F0);
fixed_conversion!(I7F1);
fixed_conversion!(I6F2);
fixed_conversion!(I5F3);
fixed_conversion!(I4F4);
fixed_conversion!(I3F5);
fixed_conversion!(I2F6);
fixed_conversion!(I1F7);
fixed_conversion!(I0F8);
fixed_conversion!(I16F0);
fixed_conversion!(I15F1);
fixed_conversion!(I14F2);
fixed_conversion!(I13F3);
fixed_conversion!(I12F4);
fixed_conversion!(I11F5);
fixed_conversion!(I10F6);
fixed_conversion!(I9F7);
fixed_conversion!(I8F8);
fixed_conversion!(I7F9);
fixed_conversion!(I6F10);
fixed_conversion!(I5F11);
fixed_conversion!(I4F12);
fixed_conversion!(I3F13);
fixed_conversion!(I2F14);
fixed_conversion!(I1F15);
fixed_conversion!(I0F16);
fixed_conversion!(I32F0);
fixed_conversion!(I31F1);
fixed_conversion!(I30F2);
fixed_conversion!(I29F3);
fixed_conversion!(I28F4);
fixed_conversion!(I27F5);
fixed_conversion!(I26F6);
fixed_conversion!(I25F7);
fixed_conversion!(I24F8);
fixed_conversion!(I23F9);
fixed_conversion!(I22F10);
fixed_conversion!(I21F11);
fixed_conversion!(I20F12);
fixed_conversion!(I19F13);
fixed_conversion!(I18F14);
fixed_conversion!(I17F15);
fixed_conversion!(I16F16);
fixed_conversion!(I15F17);
fixed_conversion!(I14F18);
fixed_conversion!(I13F19);
fixed_conversion!(I12F20);
fixed_conversion!(I11F21);
fixed_conversion!(I10F22);
fixed_conversion!(I9F23);
fixed_conversion!(I8F24);
fixed_conversion!(I7F25);
fixed_conversion!(I6F26);
fixed_conversion!(I5F27);
fixed_conversion!(I4F28);
fixed_conversion!(I3F29);
fixed_conversion!(I2F30);
fixed_conversion!(I1F31);
fixed_conversion!(I0F32);
fixed_conversion!(I64F0);
fixed_conversion!(I63F1);
fixed_conversion!(I62F2);
fixed_conversion!(I61F3);
fixed_conversion!(I60F4);
fixed_conversion!(I59F5);
fixed_conversion!(I58F6);
fixed_conversion!(I57F7);
fixed_conversion!(I56F8);
fixed_conversion!(I55F9);
fixed_conversion!(I54F10);
fixed_conversion!(I53F11);
fixed_conversion!(I52F12);
fixed_conversion!(I51F13);
fixed_conversion!(I50F14);
fixed_conversion!(I49F15);
fixed_conversion!(I48F16);
fixed_conversion!(I47F17);
fixed_conversion!(I46F18);
fixed_conversion!(I45F19);
fixed_conversion!(I44F20);
fixed_conversion!(I43F21);
fixed_conversion!(I42F22);
fixed_conversion!(I41F23);
fixed_conversion!(I40F24);
fixed_conversion!(I39F25);
fixed_conversion!(I38F26);
fixed_conversion!(I37F27);
fixed_conversion!(I36F28);
fixed_conversion!(I35F29);
fixed_conversion!(I34F30);
fixed_conversion!(I33F31);
fixed_conversion!(I32F32);
fixed_conversion!(I31F33);
fixed_conversion!(I30F34);
fixed_conversion!(I29F35);
fixed_conversion!(I28F36);
fixed_conversion!(I27F37);
fixed_conversion!(I26F38);
fixed_conversion!(I25F39);
fixed_conversion!(I24F40);
fixed_conversion!(I23F41);
fixed_conversion!(I22F42);
fixed_conversion!(I21F43);
fixed_conversion!(I20F44);
fixed_conversion!(I19F45);
fixed_conversion!(I18F46);
fixed_conversion!(I17F47);
fixed_conversion!(I16F48);
fixed_conversion!(I15F49);
fixed_conversion!(I14F50);
fixed_conversion!(I13F51);
fixed_conversion!(I12F52);
fixed_conversion!(I11F53);
fixed_conversion!(I10F54);
fixed_conversion!(I9F55);
fixed_conversion!(I8F56);
fixed_conversion!(I7F57);
fixed_conversion!(I6F58);
fixed_conversion!(I5F59);
fixed_conversion!(I4F60);
fixed_conversion!(I3F61);
fixed_conversion!(I2F62);
fixed_conversion!(I1F63);
fixed_conversion!(I0F64);
fixed_conversion!(I128F0);
fixed_conversion!(I127F1);
fixed_conversion!(I126F2);
fixed_conversion!(I125F3);
fixed_conversion!(I124F4);
fixed_conversion!(I123F5);
fixed_conversion!(I122F6);
fixed_conversion!(I121F7);
fixed_conversion!(I120F8);
fixed_conversion!(I119F9);
fixed_conversion!(I118F10);
fixed_conversion!(I117F11);
fixed_conversion!(I116F12);
fixed_conversion!(I115F13);
fixed_conversion!(I114F14);
fixed_conversion!(I113F15);
fixed_conversion!(I112F16);
fixed_conversion!(I111F17);
fixed_conversion!(I110F18);
fixed_conversion!(I109F19);
fixed_conversion!(I108F20);
fixed_conversion!(I107F21);
fixed_conversion!(I106F22);
fixed_conversion!(I105F23);
fixed_conversion!(I104F24);
fixed_conversion!(I103F25);
fixed_conversion!(I102F26);
fixed_conversion!(I101F27);
fixed_conversion!(I100F28);
fixed_conversion!(I99F29);
fixed_conversion!(I98F30);
fixed_conversion!(I97F31);
fixed_conversion!(I96F32);
fixed_conversion!(I95F33);
fixed_conversion!(I94F34);
fixed_conversion!(I93F35);
fixed_conversion!(I92F36);
fixed_conversion!(I91F37);
fixed_conversion!(I90F38);
fixed_conversion!(I89F39);
fixed_conversion!(I88F40);
fixed_conversion!(I87F41);
fixed_conversion!(I86F42);
fixed_conversion!(I85F43);
fixed_conversion!(I84F44);
fixed_conversion!(I83F45);
fixed_conversion!(I82F46);
fixed_conversion!(I81F47);
fixed_conversion!(I80F48);
fixed_conversion!(I79F49);
fixed_conversion!(I78F50);
fixed_conversion!(I77F51);
fixed_conversion!(I76F52);
fixed_conversion!(I75F53);
fixed_conversion!(I74F54);
fixed_conversion!(I73F55);
fixed_conversion!(I72F56);
fixed_conversion!(I71F57);
fixed_conversion!(I70F58);
fixed_conversion!(I69F59);
fixed_conversion!(I68F60);
fixed_conversion!(I67F61);
fixed_conversion!(I66F62);
fixed_conversion!(I65F63);
fixed_conversion!(I64F64);
fixed_conversion!(I63F65);
fixed_conversion!(I62F66);
fixed_conversion!(I61F67);
fixed_conversion!(I60F68);
fixed_conversion!(I59F69);
fixed_conversion!(I58F70);
fixed_conversion!(I57F71);
fixed_conversion!(I56F72);
fixed_conversion!(I55F73);
fixed_conversion!(I54F74);
fixed_conversion!(I53F75);
fixed_conversion!(I52F76);
fixed_conversion!(I51F77);
fixed_conversion!(I50F78);
fixed_conversion!(I49F79);
fixed_conversion!(I48F80);
fixed_conversion!(I47F81);
fixed_conversion!(I46F82);
fixed_conversion!(I45F83);
fixed_conversion!(I44F84);
fixed_conversion!(I43F85);
fixed_conversion!(I42F86);
fixed_conversion!(I41F87);
fixed_conversion!(I40F88);
fixed_conversion!(I39F89);
fixed_conversion!(I38F90);
fixed_conversion!(I37F91);
fixed_conversion!(I36F92);
fixed_conversion!(I35F93);
fixed_conversion!(I34F94);
fixed_conversion!(I33F95);
fixed_conversion!(I32F96);
fixed_conversion!(I31F97);
fixed_conversion!(I30F98);
fixed_conversion!(I29F99);
fixed_conversion!(I28F100);
fixed_conversion!(I27F101);
fixed_conversion!(I26F102);
fixed_conversion!(I25F103);
fixed_conversion!(I24F104);
fixed_conversion!(I23F105);
fixed_conversion!(I22F106);
fixed_conversion!(I21F107);
fixed_conversion!(I20F108);
fixed_conversion!(I19F109);
fixed_conversion!(I18F110);
fixed_conversion!(I17F111);
fixed_conversion!(I16F112);
fixed_conversion!(I15F113);
fixed_conversion!(I14F114);
fixed_conversion!(I13F115);
fixed_conversion!(I12F116);
fixed_conversion!(I11F117);
fixed_conversion!(I10F118);
fixed_conversion!(I9F119);
fixed_conversion!(I8F120);
fixed_conversion!(I7F121);
fixed_conversion!(I6F122);
fixed_conversion!(I5F123);
fixed_conversion!(I4F124);
fixed_conversion!(I3F125);
fixed_conversion!(I2F126);
fixed_conversion!(I1F127);
fixed_conversion!(I0F128);
fixed_conversion!(U8F0);
fixed_conversion!(U7F1);
fixed_conversion!(U6F2);
fixed_conversion!(U5F3);
fixed_conversion!(U4F4);
fixed_conversion!(U3F5);
fixed_conversion!(U2F6);
fixed_conversion!(U1F7);
fixed_conversion!(U0F8);
fixed_conversion!(U16F0);
fixed_conversion!(U15F1);
fixed_conversion!(U14F2);
fixed_conversion!(U13F3);
fixed_conversion!(U12F4);
fixed_conversion!(U11F5);
fixed_conversion!(U10F6);
fixed_conversion!(U9F7);
fixed_conversion!(U8F8);
fixed_conversion!(U7F9);
fixed_conversion!(U6F10);
fixed_conversion!(U5F11);
fixed_conversion!(U4F12);
fixed_conversion!(U3F13);
fixed_conversion!(U2F14);
fixed_conversion!(U1F15);
fixed_conversion!(U0F16);
fixed_conversion!(U32F0);
fixed_conversion!(U31F1);
fixed_conversion!(U30F2);
fixed_conversion!(U29F3);
fixed_conversion!(U28F4);
fixed_conversion!(U27F5);
fixed_conversion!(U26F6);
fixed_conversion!(U25F7);
fixed_conversion!(U24F8);
fixed_conversion!(U23F9);
fixed_conversion!(U22F10);
fixed_conversion!(U21F11);
fixed_conversion!(U20F12);
fixed_conversion!(U19F13);
fixed_conversion!(U18F14);
fixed_conversion!(U17F15);
fixed_conversion!(U16F16);
fixed_conversion!(U15F17);
fixed_conversion!(U14F18);
fixed_conversion!(U13F19);
fixed_conversion!(U12F20);
fixed_conversion!(U11F21);
fixed_conversion!(U10F22);
fixed_conversion!(U9F23);
fixed_conversion!(U8F24);
fixed_conversion!(U7F25);
fixed_conversion!(U6F26);
fixed_conversion!(U5F27);
fixed_conversion!(U4F28);
fixed_conversion!(U3F29);
fixed_conversion!(U2F30);
fixed_conversion!(U1F31);
fixed_conversion!(U0F32);
fixed_conversion!(U64F0);
fixed_conversion!(U63F1);
fixed_conversion!(U62F2);
fixed_conversion!(U61F3);
fixed_conversion!(U60F4);
fixed_conversion!(U59F5);
fixed_conversion!(U58F6);
fixed_conversion!(U57F7);
fixed_conversion!(U56F8);
fixed_conversion!(U55F9);
fixed_conversion!(U54F10);
fixed_conversion!(U53F11);
fixed_conversion!(U52F12);
fixed_conversion!(U51F13);
fixed_conversion!(U50F14);
fixed_conversion!(U49F15);
fixed_conversion!(U48F16);
fixed_conversion!(U47F17);
fixed_conversion!(U46F18);
fixed_conversion!(U45F19);
fixed_conversion!(U44F20);
fixed_conversion!(U43F21);
fixed_conversion!(U42F22);
fixed_conversion!(U41F23);
fixed_conversion!(U40F24);
fixed_conversion!(U39F25);
fixed_conversion!(U38F26);
fixed_conversion!(U37F27);
fixed_conversion!(U36F28);
fixed_conversion!(U35F29);
fixed_conversion!(U34F30);
fixed_conversion!(U33F31);
fixed_conversion!(U32F32);
fixed_conversion!(U31F33);
fixed_conversion!(U30F34);
fixed_conversion!(U29F35);
fixed_conversion!(U28F36);
fixed_conversion!(U27F37);
fixed_conversion!(U26F38);
fixed_conversion!(U25F39);
fixed_conversion!(U24F40);
fixed_conversion!(U23F41);
fixed_conversion!(U22F42);
fixed_conversion!(U21F43);
fixed_conversion!(U20F44);
fixed_conversion!(U19F45);
fixed_conversion!(U18F46);
fixed_conversion!(U17F47);
fixed_conversion!(U16F48);
fixed_conversion!(U15F49);
fixed_conversion!(U14F50);
fixed_conversion!(U13F51);
fixed_conversion!(U12F52);
fixed_conversion!(U11F53);
fixed_conversion!(U10F54);
fixed_conversion!(U9F55);
fixed_conversion!(U8F56);
fixed_conversion!(U7F57);
fixed_conversion!(U6F58);
fixed_conversion!(U5F59);
fixed_conversion!(U4F60);
fixed_conversion!(U3F61);
fixed_conversion!(U2F62);
fixed_conversion!(U1F63);
fixed_conversion!(U0F64);
fixed_conversion!(U128F0);
fixed_conversion!(U127F1);
fixed_conversion!(U126F2);
fixed_conversion!(U125F3);
fixed_conversion!(U124F4);
fixed_conversion!(U123F5);
fixed_conversion!(U122F6);
fixed_conversion!(U121F7);
fixed_conversion!(U120F8);
fixed_conversion!(U119F9);
fixed_conversion!(U118F10);
fixed_conversion!(U117F11);
fixed_conversion!(U116F12);
fixed_conversion!(U115F13);
fixed_conversion!(U114F14);
fixed_conversion!(U113F15);
fixed_conversion!(U112F16);
fixed_conversion!(U111F17);
fixed_conversion!(U110F18);
fixed_conversion!(U109F19);
fixed_conversion!(U108F20);
fixed_conversion!(U107F21);
fixed_conversion!(U106F22);
fixed_conversion!(U105F23);
fixed_conversion!(U104F24);
fixed_conversion!(U103F25);
fixed_conversion!(U102F26);
fixed_conversion!(U101F27);
fixed_conversion!(U100F28);
fixed_conversion!(U99F29);
fixed_conversion!(U98F30);
fixed_conversion!(U97F31);
fixed_conversion!(U96F32);
fixed_conversion!(U95F33);
fixed_conversion!(U94F34);
fixed_conversion!(U93F35);
fixed_conversion!(U92F36);
fixed_conversion!(U91F37);
fixed_conversion!(U90F38);
fixed_conversion!(U89F39);
fixed_conversion!(U88F40);
fixed_conversion!(U87F41);
fixed_conversion!(U86F42);
fixed_conversion!(U85F43);
fixed_conversion!(U84F44);
fixed_conversion!(U83F45);
fixed_conversion!(U82F46);
fixed_conversion!(U81F47);
fixed_conversion!(U80F48);
fixed_conversion!(U79F49);
fixed_conversion!(U78F50);
fixed_conversion!(U77F51);
fixed_conversion!(U76F52);
fixed_conversion!(U75F53);
fixed_conversion!(U74F54);
fixed_conversion!(U73F55);
fixed_conversion!(U72F56);
fixed_conversion!(U71F57);
fixed_conversion!(U70F58);
fixed_conversion!(U69F59);
fixed_conversion!(U68F60);
fixed_conversion!(U67F61);
fixed_conversion!(U66F62);
fixed_conversion!(U65F63);
fixed_conversion!(U64F64);
fixed_conversion!(U63F65);
fixed_conversion!(U62F66);
fixed_conversion!(U61F67);
fixed_conversion!(U60F68);
fixed_conversion!(U59F69);
fixed_conversion!(U58F70);
fixed_conversion!(U57F71);
fixed_conversion!(U56F72);
fixed_conversion!(U55F73);
fixed_conversion!(U54F74);
fixed_conversion!(U53F75);
fixed_conversion!(U52F76);
fixed_conversion!(U51F77);
fixed_conversion!(U50F78);
fixed_conversion!(U49F79);
fixed_conversion!(U48F80);
fixed_conversion!(U47F81);
fixed_conversion!(U46F82);
fixed_conversion!(U45F83);
fixed_conversion!(U44F84);
fixed_conversion!(U43F85);
fixed_conversion!(U42F86);
fixed_conversion!(U41F87);
fixed_conversion!(U40F88);
fixed_conversion!(U39F89);
fixed_conversion!(U38F90);
fixed_conversion!(U37F91);
fixed_conversion!(U36F92);
fixed_conversion!(U35F93);
fixed_conversion!(U34F94);
fixed_conversion!(U33F95);
fixed_conversion!(U32F96);
fixed_conversion!(U31F97);
fixed_conversion!(U30F98);
fixed_conversion!(U29F99);
fixed_conversion!(U28F100);
fixed_conversion!(U27F101);
fixed_conversion!(U26F102);
fixed_conversion!(U25F103);
fixed_conversion!(U24F104);
fixed_conversion!(U23F105);
fixed_conversion!(U22F106);
fixed_conversion!(U21F107);
fixed_conversion!(U20F108);
fixed_conversion!(U19F109);
fixed_conversion!(U18F110);
fixed_conversion!(U17F111);
fixed_conversion!(U16F112);
fixed_conversion!(U15F113);
fixed_conversion!(U14F114);
fixed_conversion!(U13F115);
fixed_conversion!(U12F116);
fixed_conversion!(U11F117);
fixed_conversion!(U10F118);
fixed_conversion!(U9F119);
fixed_conversion!(U8F120);
fixed_conversion!(U7F121);
fixed_conversion!(U6F122);
fixed_conversion!(U5F123);
fixed_conversion!(U4F124);
fixed_conversion!(U3F125);
fixed_conversion!(U2F126);
fixed_conversion!(U1F127);
fixed_conversion!(U0F128);

#[cfg(test)]
mod test_fixed {
    use super::*;
    use crate::err::PyErr;
    use crate::types::PyDict;
    use fixed::types::*;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() {
                Python::with_gil(|py| {
                    let rs_orig = $rs;
                    let rs_dec = rs_orig.into_py(py);
                    let locals = PyDict::new(py);
                    locals.set_item("rs_dec", &rs_dec).unwrap();
                    // Checks if Rust I64F64 -> Python Decimal conversion is correct
                    py.run(
                        &format!(
                            "import decimal\npy_dec = decimal.Decimal({})\nassert py_dec == rs_dec",
                            $py
                        ),
                        None,
                        Some(locals),
                    )
                    .unwrap();
                    // Checks if Python Decimal -> Rust I64F64 conversion is correct
                    let py_dec = locals.get_item("py_dec").unwrap();
                    let py_result: I64F64 = FromPyObject::extract(py_dec).unwrap();
                    assert_eq!(rs_orig, py_result);
                })
            }
        };
    }

    convert_constants!(convert_zero, I64F64::from_str("0").unrwap(), "0");
    convert_constants!(convert_one, I64F64::from_str("1").unrwap(), "1");
    convert_constants!(convert_neg_one, I64F64::from_str("-1").unrwap(), "-1");
    convert_constants!(
        convert_one_thousand,
        I64F64::from_str("1000").unrwap(),
        "1000"
    );
    convert_constants!(
        convert_decimal,
        I64F64::from_str("999.999").unrwap(),
        "999.999"
    );
    convert_constants!(
        convert_neg_decimal,
        I64F64::from_str("-999.999").unrwap(),
        "-999.999"
    );

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_roundtrip(
            val in any::<f64>()) {
            let num = I64F64::from_num(x);
            Python::with_gil(|py| {
                let rs_dec = num.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("rs_dec", &rs_dec).unwrap();
                py.run(
                    &format!(
                       "import decimal\npy_dec = decimal.Decimal(\"{}\")\nassert py_dec == rs_dec",
                     num.to_string()),
                None, Some(locals)).unwrap();
                let roundtripped: I64F64 = rs_dec.extract(py).unwrap();
                assert_eq!(num, roundtripped);
            })
        }

        #[test]
        fn test_integers(num in any::<i64>()) {
            Python::with_gil(|py| {
                let py_num = num.into_py(py);
                let roundtripped: I64F64 = py_num.extract(py).unwrap();
                let rs_dec = I64F64::from_num(num);
                assert_eq!(rs_dec, roundtripped);
            })
        }
    }
}
