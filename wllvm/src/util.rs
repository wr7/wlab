use core::slice;
use std::{ops::Deref, ptr};

use llvm_sys::{
    core::{LLVMDisposeMemoryBuffer, LLVMDisposeMessage, LLVMGetBufferSize, LLVMGetBufferStart},
    error::LLVMDisposeErrorMessage,
    LLVMMemoryBuffer,
};

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

pub struct MemoryBuffer {
    raw: *mut LLVMMemoryBuffer,
    ptr: *const [u8],
}

impl MemoryBuffer {
    pub unsafe fn from_raw(raw: *mut LLVMMemoryBuffer) -> Self {
        unsafe {
            let len = LLVMGetBufferSize(raw);
            let ptr = LLVMGetBufferStart(raw).cast::<u8>();

            let ptr = ptr::from_ref(slice::from_raw_parts(ptr, len));

            Self { raw, ptr }
        }
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe { LLVMDisposeMemoryBuffer(self.raw) }
    }
}

impl AsRef<[u8]> for MemoryBuffer {
    fn as_ref(&self) -> &[u8] {
        unsafe { &*self.ptr }
    }
}

impl Deref for MemoryBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

macro_rules! llvm_string_type {
    {$(#[doc = $doc:literal])* $vis:vis struct $name:ident; $destructor:ident} => {
        $(
            #[doc = $doc]
        )*
        #[repr(transparent)]
        $vis struct $name {
            ptr: *mut ::core::ffi::c_char,
        }

        impl $name {
            /// Wraps a raw LLVM string. Upon dropping, `
            #[doc = ::core::stringify!($destructor)]
            ///` will be called on it.
            pub unsafe fn from_raw(ptr: *mut ::core::ffi::c_char) -> Self {
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

                unsafe { ::core::slice::from_raw_parts(ptr, len) }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe { $destructor(self.ptr) }
            }
        }

        impl ::core::ops::Deref for $name {
            type Target = ::core::ffi::CStr;

            fn deref(&self) -> &Self::Target {
                unsafe { ::core::ffi::CStr::from_ptr(self.ptr) }
            }
        }

        impl AsRef<::core::ffi::CStr> for $name {
            fn as_ref(&self) -> &::core::ffi::CStr {
                &*self
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(&**self, f)
            }
        }
    };
}

pub(crate) use llvm_string_type;
