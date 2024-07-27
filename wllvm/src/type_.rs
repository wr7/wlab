use std::{
    ffi::{c_char, c_ulonglong, CStr},
    fmt::Debug,
    marker::PhantomData,
    ops::Deref,
};

use llvm_sys::{
    core::{
        LLVMConstInt, LLVMConstIntOfArbitraryPrecision, LLVMConstIntOfStringAndSize, LLVMConstNull,
        LLVMCountParamTypes, LLVMGetInlineAsm, LLVMGetIntTypeWidth, LLVMGetParamTypes,
        LLVMGetReturnType, LLVMGetTypeKind, LLVMIsFunctionVarArg, LLVMPrintTypeToString,
    },
    prelude::LLVMBool,
    LLVMType,
};

use crate::{
    util::LLVMString,
    value::{FnValue, IntValue, PtrValue, StructValue, Value},
};

pub use llvm_sys::{LLVMInlineAsmDialect, LLVMTypeKind};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AsmDialect {
    ATT,
    Intel,
}

impl From<AsmDialect> for LLVMInlineAsmDialect {
    fn from(value: AsmDialect) -> Self {
        match value {
            AsmDialect::ATT => LLVMInlineAsmDialect::LLVMInlineAsmDialectATT,
            AsmDialect::Intel => LLVMInlineAsmDialect::LLVMInlineAsmDialectIntel,
        }
    }
}

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

    pub fn raw(&self) -> *mut LLVMType {
        self.ptr
    }

    pub fn kind(&self) -> LLVMTypeKind {
        unsafe { LLVMGetTypeKind(self.ptr) }
    }

    pub fn const_null(&self) -> Value<'ctx> {
        unsafe { Value::from_raw(LLVMConstNull(self.ptr)) }
    }

    /// Prints the type into an [`LLVMString`].
    pub fn to_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMPrintTypeToString(self.ptr)) }
    }
}

impl Debug for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cstr: &CStr = &*self.to_string();

        Debug::fmt(cstr, f)
    }
}

specialized_type! {
    /// An LLVM integer type reference
    pub struct IntType: IntValue
}

specialized_type! {
    /// An LLVM integer type reference
    pub struct PtrType: PtrValue
}

specialized_type! {
    /// An LLVM function type reference
    pub struct FnType: FnValue
}

specialized_type! {
    /// An LLVM struct type reference
    pub struct StructType: StructValue
}

impl<'ctx> IntType<'ctx> {
    /// Gets the width (in bits) of the integer type
    pub fn width(&self) -> u32 {
        unsafe { LLVMGetIntTypeWidth(self.ptr) }
    }

    pub fn const_(&self, val: c_ulonglong, sign_extend: bool) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMConstInt(self.ptr, val, sign_extend as LLVMBool)) }
    }

    pub fn const_arbitrary_precision(&self, num: &[u64]) -> IntValue<'ctx> {
        unsafe {
            IntValue::from_raw(LLVMConstIntOfArbitraryPrecision(
                self.ptr,
                num.len() as u32,
                num.as_ptr(),
            ))
        }
    }

    pub fn const_from_string<S>(
        &self,
        str: &(impl ?Sized + AsRef<[u8]>),
        radix: u8,
    ) -> IntValue<'ctx> {
        let s = str.as_ref();

        let ptr = s.as_ptr().cast::<c_char>();
        let len = s.len();

        unsafe {
            IntValue::from_raw(LLVMConstIntOfStringAndSize(
                self.ptr, ptr, len as u32, radix,
            ))
        }
    }
}

impl<'ctx> FnType<'ctx> {
    pub fn inline_asm(
        &self,
        asm: &(impl ?Sized + AsRef<[u8]>),
        constraints: &(impl ?Sized + AsRef<[u8]>),
        side_effects: bool,
        align_stack: bool,
        dialect: AsmDialect,
        can_throw: bool,
    ) -> PtrValue<'ctx> {
        let asm = asm.as_ref();
        let constraints = constraints.as_ref();
        let side_effects = side_effects as LLVMBool;
        let align_stack = align_stack as LLVMBool;
        let can_throw = can_throw as LLVMBool;

        let asm_ptr = asm.as_ptr().cast::<c_char>();
        let constraints_ptr = constraints.as_ptr().cast::<c_char>();

        unsafe {
            PtrValue::from_raw(LLVMGetInlineAsm(
                self.ptr,
                asm_ptr,
                asm.len(),
                constraints_ptr,
                constraints.len(),
                side_effects,
                align_stack,
                dialect.into(),
                can_throw,
            ))
        }
    }

    pub fn var_args(&self) -> bool {
        unsafe { LLVMIsFunctionVarArg(self.ptr) != 0 }
    }

    pub fn return_type(&self) -> Type<'ctx> {
        unsafe { Type::from_raw(LLVMGetReturnType(self.ptr)) }
    }

    pub fn num_params(&self) -> u32 {
        unsafe { LLVMCountParamTypes(self.ptr) }
    }

    pub fn params(&self) -> Vec<Type<'ctx>> {
        let num_params = self.num_params() as usize;

        let mut params = Vec::<Type<'ctx>>::with_capacity(num_params);

        unsafe {
            LLVMGetParamTypes(self.ptr, params.as_mut_ptr().cast::<*mut LLVMType>());
            params.set_len(num_params);
        }

        params
    }
}

macro_rules! specialized_type {
    {
        $(#[doc = $doc:literal])*
        pub struct $name:ident $(: $value:ident)?
    } => {
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

            $(
                pub fn const_null(&self) -> $value<'ctx> {
                    unsafe { $value::from_raw(LLVMConstNull(self.ptr)) }
                }
            )?
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

pub(self) use specialized_type;
