use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
use crate::instance::Bound;
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::PyType;
use crate::{intern, FromPyObject, Py, PyAny, PyErr, PyResult, Python};

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

impl<'py> IntoPyObject<'py> for Ipv4Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static IPV4_ADDRESS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        IPV4_ADDRESS
            .import(py, "ipaddress", "IPv4Address")?
            .call1((u32::from_be_bytes(self.octets()),))
    }
}

impl<'py> IntoPyObject<'py> for &Ipv4Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for Ipv6Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        static IPV6_ADDRESS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        IPV6_ADDRESS
            .import(py, "ipaddress", "IPv6Address")?
            .call1((u128::from_be_bytes(self.octets()),))
    }
}

impl<'py> IntoPyObject<'py> for &Ipv6Addr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

impl<'py> IntoPyObject<'py> for IpAddr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            IpAddr::V4(ip) => ip.into_pyobject(py),
            IpAddr::V6(ip) => ip.into_pyobject(py),
        }
    }
}

impl<'py> IntoPyObject<'py> for &IpAddr {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

#[cfg(test)]
mod test_ipaddr {
    use std::str::FromStr;

    use crate::types::PyString;

    use super::*;

    #[test]
    fn test_roundtrip() {
        Python::attach(|py| {
            fn roundtrip(py: Python<'_>, ip: &str) {
                let ip = IpAddr::from_str(ip).unwrap();
                let py_cls = if ip.is_ipv4() {
                    "IPv4Address"
                } else {
                    "IPv6Address"
                };

                let pyobj = ip.into_pyobject(py).unwrap();
                let repr = pyobj.repr().unwrap();
                let repr = repr.to_string_lossy();
                assert_eq!(repr, format!("{py_cls}('{ip}')"));

                let ip2: IpAddr = pyobj.extract().unwrap();
                assert_eq!(ip, ip2);
            }
            roundtrip(py, "127.0.0.1");
            roundtrip(py, "::1");
            roundtrip(py, "0.0.0.0");
        });
    }

    #[test]
    fn test_from_pystring() {
        Python::attach(|py| {
            let py_str = PyString::new(py, "0:0:0:0:0:0:0:1");
            let ip: IpAddr = py_str.extract().unwrap();
            assert_eq!(ip, IpAddr::from_str("::1").unwrap());

            let py_str = PyString::new(py, "invalid");
            assert!(py_str.extract::<IpAddr>().is_err());
        });
    }
}
