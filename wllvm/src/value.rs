use std::{marker::PhantomData, ops::Deref};

use llvm_sys::LLVMValue;

/// An LLVM value reference
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Value<'ctx> {
    ptr: *mut LLVMValue,
    _phantomdata: PhantomData<&'ctx LLVMValue>,
}

impl<'ctx> Value<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMValue) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }
}

macro_rules! specialized_type {
    {$(#[doc = $doc:literal])* pub struct $name:ident} => {
        $(#[doc = $doc])*
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct $name<'ctx> {
            value: Value<'ctx>,
        }

        impl<'ctx> $name<'ctx> {
            pub unsafe fn from_raw(raw: *mut LLVMValue) -> Self {
                Self {value: Value::from_raw(raw)}
            }
        }

        impl<'ctx> Deref for $name<'ctx> {
            type Target = Value<'ctx>;

            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }

        impl<'ctx> AsRef<Value<'ctx>> for $name<'ctx> {
            fn as_ref(&self) -> &Value<'ctx> {
                &**self
            }
        }

        impl<'ctx> From<$name<'ctx>> for Value<'ctx> {
            fn from(value: $name<'ctx>) -> Self {
                value.value
            }
        }
    };
}

specialized_type! {
    /// An LLVM function value reference
    pub struct FnValue
}
