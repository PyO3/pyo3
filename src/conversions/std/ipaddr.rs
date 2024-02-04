use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::exceptions::PyValueError;
use crate::instance::Bound;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::PyType;
use crate::{intern, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};

impl FromPyObject<'_> for IpAddr {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        match obj.getattr(intern!(obj.py(), "packed")) {
            Ok(packed) => {
                if let Ok(packed) = packed.extract::<[u8; 4]>() {
                    Ok(IpAddr::V4(Ipv4Addr::from(packed)))
                } else if let Ok(packed) = packed.extract::<[u8; 16]>() {
                    Ok(IpAddr::V6(Ipv6Addr::from(packed)))
                } else {
                    Err(PyValueError::new_err("invalid packed length"))
                }
            }
            Err(_) => {
                // We don't have a .packed attribute, so we try to construct an IP from str().
                obj.str()?.to_cow()?.parse().map_err(PyValueError::new_err)
            }
        }
    }
}

impl ToPyObject for Ipv4Addr {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        static IPV4_ADDRESS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        IPV4_ADDRESS
            .get_or_try_init_type_ref(py, "ipaddress", "IPv4Address")
            .expect("failed to load ipaddress.IPv4Address")
            .call1((u32::from_be_bytes(self.octets()),))
            .expect("failed to construct ipaddress.IPv4Address")
            .to_object(py)
    }
}

impl ToPyObject for Ipv6Addr {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        static IPV6_ADDRESS: GILOnceCell<Py<PyType>> = GILOnceCell::new();
        IPV6_ADDRESS
            .get_or_try_init_type_ref(py, "ipaddress", "IPv6Address")
            .expect("failed to load ipaddress.IPv6Address")
            .call1((u128::from_be_bytes(self.octets()),))
            .expect("failed to construct ipaddress.IPv6Address")
            .to_object(py)
    }
}

impl ToPyObject for IpAddr {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            IpAddr::V4(ip) => ip.to_object(py),
            IpAddr::V6(ip) => ip.to_object(py),
        }
    }
}

impl IntoPy<PyObject> for IpAddr {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

#[cfg(test)]
mod test_ipaddr {
    use std::str::FromStr;

    use crate::types::PyString;

    use super::*;

    #[test]
    fn test_roundtrip() {
        Python::with_gil(|py| {
            fn roundtrip(py: Python<'_>, ip: &str) {
                let ip = IpAddr::from_str(ip).unwrap();
                let py_cls = if ip.is_ipv4() {
                    "IPv4Address"
                } else {
                    "IPv6Address"
                };

                let pyobj = ip.into_py(py);
                let repr = pyobj.as_ref(py).repr().unwrap().to_string_lossy();
                assert_eq!(repr, format!("{}('{}')", py_cls, ip));

                let ip2: IpAddr = pyobj.extract(py).unwrap();
                assert_eq!(ip, ip2);
            }
            roundtrip(py, "127.0.0.1");
            roundtrip(py, "::1");
            roundtrip(py, "0.0.0.0");
        });
    }

    #[test]
    fn test_from_pystring() {
        Python::with_gil(|py| {
            let py_str = PyString::new_bound(py, "0:0:0:0:0:0:0:1");
            let ip: IpAddr = py_str.to_object(py).extract(py).unwrap();
            assert_eq!(ip, IpAddr::from_str("::1").unwrap());

            let py_str = PyString::new_bound(py, "invalid");
            assert!(py_str.to_object(py).extract::<IpAddr>(py).is_err());
        });
    }
}
