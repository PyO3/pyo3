use crate::type_object::PyBorrowFlagLayout;
use crate::{Py, PyClass, PyClassInitializer, PyTypeInfo, Python};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};

impl<T> Serialize for Py<T>
where
    T: Serialize + PyClass,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        Python::with_gil(|py| {
            self.try_borrow(py)
                .map_err(|e| ser::Error::custom(e.to_string()))?
                .serialize(serializer)
        })
    }
}

impl<'de, T> Deserialize<'de> for Py<T>
where
    T: Into<PyClassInitializer<T>> + PyClass + Deserialize<'de>,
    <T as PyTypeInfo>::BaseLayout: PyBorrowFlagLayout<<T as PyTypeInfo>::BaseType>,
{
    fn deserialize<D>(deserializer: D) -> Result<Py<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialized = T::deserialize(deserializer)?;

        Python::with_gil(|py| {
            Py::new(py, deserialized).map_err(|e| de::Error::custom(e.to_string()))
        })
    }
}
