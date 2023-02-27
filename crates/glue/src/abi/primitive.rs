use super::{FromWasmAbi, ToWasmAbi};

macro_rules! impl_wasm_abi_for_primitive {
    ($prim:ty) => {
        impl FromWasmAbi for $prim {
            type Type = $prim;

            #[inline]
            fn from_wasm_abi(value: Self::Type) -> Self {
                value
            }
        }

        impl ToWasmAbi for $prim {
            type Type = $prim;

            #[inline]
            fn to_wasm_abi(self) -> Self::Type {
                self
            }
        }
    };
}

impl_wasm_abi_for_primitive!(i32);
