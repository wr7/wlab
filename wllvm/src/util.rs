use core::ffi::c_char;
use std::{ffi::CStr, fmt::Debug, ops::Deref};

use llvm_sys::{core::LLVMDisposeMessage, error::LLVMDisposeErrorMessage};

macro_rules! llvm_string_type {
    {$(#[doc = $doc:literal])* pub struct $name:ident; $destructor:ident} => {
        $(
            #[doc = $doc]
        )*
        #[repr(transparent)]
        pub struct $name {
            ptr: *mut c_char,
        }

        impl $name {
            /// Wraps a raw LLVM string. Upon dropping, `
            #[doc = ::std::stringify!($destructor)]
            ///` will be called on it.
            pub unsafe fn from_raw(ptr: *mut c_char) -> Self {
                Self { ptr }
            }

            pub fn as_bytes(&self) -> &[u8] {
                let cstr = &**self;

                cstr.to_bytes()
            }

            pub fn as_bytes_mut(&mut self) -> &[u8] {
                let cstr = &**self;
                let len = cstr.count_bytes();

                let ptr = self.ptr.cast::<u8>();

                unsafe { std::slice::from_raw_parts(ptr, len) }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe { $destructor(self.ptr) }
            }
        }

        impl Deref for $name {
            type Target = CStr;

            fn deref(&self) -> &Self::Target {
                unsafe { CStr::from_ptr(self.ptr) }
            }
        }

        impl AsRef<CStr> for $name {
            fn as_ref(&self) -> &CStr {
                &*self
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Debug::fmt(&**self, f)
            }
        }
    };
}

llvm_string_type! {
    /// An owned string obtained from LLVM. `LLVMDisposeMessage` is automatically used to free this string.
    pub struct LLVMString;
    LLVMDisposeMessage
}

llvm_string_type! {
    /// An owned error string obtained from LLVM. `LLVMDisposeErrorMessage` is automatically used to free this string.
    pub struct LLVMErrorString;
    LLVMDisposeErrorMessage
}
