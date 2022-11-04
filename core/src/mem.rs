use std::ffi::c_char;

use serde::{de::DeserializeOwned, Serialize};

pub trait ToMem {
    type Type;
    fn to_mem(&self) -> Self::Type;
}

pub trait FromMem
where
    Self: ToMem,
{
    fn from_mem(value: <Self as ToMem>::Type) -> Self;
}

impl<T> ToMem for T
where
    T: Serialize,
{
    type Type = *mut c_char;

    fn to_mem(&self) -> Self::Type {
        let rt = serde_json::to_string(&self).unwrap();
        let rt = std::ffi::CString::new(rt).unwrap();
        rt.into_raw()
    }
}

impl<T> FromMem for T
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn from_mem(value: <Self as ToMem>::Type) -> Self {
        unsafe {
            let st = std::ffi::CString::from_raw(value).into_string().unwrap();
            let sv = serde_json::from_str::<T>(&st).unwrap();
            sv
        }
    }
}
