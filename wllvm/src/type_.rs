use std::{ffi::CStr, fmt::Debug, marker::PhantomData, ops::Deref};

use llvm_sys::{
    core::{
        LLVMCountParamTypes, LLVMGetIntTypeWidth, LLVMGetParamTypes, LLVMGetReturnType,
        LLVMIsFunctionVarArg, LLVMPrintTypeToString,
    },
    LLVMType,
};

use crate::util::LLVMString;

/// An LLVM type reference
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Type<'ctx> {
    ptr: *mut LLVMType,
    _phantomdata: PhantomData<&'ctx LLVMType>,
}

impl<'ctx> Type<'ctx> {
    /// Creates a type from a raw `LLVMType` pointer.
    /// # Safety
    /// - The context of the pointed-to type must live for `'ctx`.
    pub unsafe fn from_raw(ptr: *mut LLVMType) -> Self {
        Type {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn into_raw(self) -> *mut LLVMType {
        self.ptr
    }

    /// Prints the type into an [`LLVMString`].
    pub fn as_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMPrintTypeToString(self.ptr)) }
    }
}

impl Debug for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cstr: &CStr = &*self.as_string();

        Debug::fmt(cstr, f)
    }
}

macro_rules! specialized_type {
    {$(#[doc = $doc:literal])* pub struct $name:ident;} => {
        $(#[doc = $doc])*
        #[repr(transparent)]
        #[derive(Clone, Copy)]
        pub struct $name<'ctx> {
            type_: Type<'ctx>,
        }

        impl<'ctx> $name<'ctx> {
            pub unsafe fn from_raw(raw: *mut LLVMType) -> Self {
                Self {type_: Type::from_raw(raw)}
            }
        }

        impl<'ctx> Deref for $name<'ctx> {
            type Target = Type<'ctx>;

            fn deref(&self) -> &Self::Target {
                &self.type_
            }
        }

        impl<'ctx> AsRef<Type<'ctx>> for $name<'ctx> {
            fn as_ref(&self) -> &Type<'ctx> {
                &**self
            }
        }

        impl<'ctx> From<$name<'ctx>> for Type<'ctx> {
            fn from(value: $name<'ctx>) -> Self {
                value.type_
            }
        }

        impl<'ctx> Debug for $name<'ctx> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Debug::fmt(&**self, f)
            }
        }
    };
}

specialized_type! {
    /// An LLVM integer type reference
    pub struct IntType;
}

specialized_type! {
    /// An LLVM function type reference
    pub struct FnType;
}

specialized_type! {
    /// An LLVM struct type reference
    pub struct StructType;
}

impl<'ctx> IntType<'ctx> {
    /// Gets the width (in bits) of the integer type
    pub fn width(self) -> u32 {
        unsafe { LLVMGetIntTypeWidth(self.ptr) }
    }
}

impl<'ctx> FnType<'ctx> {
    pub fn var_args(self) -> bool {
        unsafe { LLVMIsFunctionVarArg(self.ptr) != 0 }
    }

    pub fn return_type(self) -> Type<'ctx> {
        unsafe { Type::from_raw(LLVMGetReturnType(self.ptr)) }
    }

    pub fn num_params(self) -> u32 {
        unsafe { LLVMCountParamTypes(self.ptr) }
    }

    pub fn params(self) -> Vec<Type<'ctx>> {
        let num_params = self.num_params() as usize;

        let mut params = Vec::<Type<'ctx>>::with_capacity(num_params);

        unsafe {
            LLVMGetParamTypes(self.ptr, params.as_mut_ptr().cast::<*mut LLVMType>());
            params.set_len(num_params);
        }

        params
    }
}
