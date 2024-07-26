use std::{ffi::CStr, marker::PhantomData, ops::Deref};

use llvm_sys::{
    core::{
        LLVMAppendBasicBlockInContext, LLVMCountParams, LLVMGetParam, LLVMGetTypeContext,
        LLVMTypeOf,
    },
    LLVMTypeKind, LLVMValue,
};

use crate::{
    basic_block::BasicBlock,
    type_::{FnType, IntType, StructType},
    Type,
};

/// A generic LLVM value reference
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

    pub fn raw(self) -> *mut LLVMValue {
        self.ptr
    }

    pub fn type_(self) -> Type<'ctx> {
        unsafe { Type::from_raw(LLVMTypeOf(self.ptr)) }
    }
}

macro_rules! specialized_values {
    {
        $(
            $(#[doc = $doc:literal])*
            pub struct $name:ident : $type:ident @ $type_kind:ident
        )+
    } => {
        $(
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

                pub fn type_(self) -> $type<'ctx> {
                    unsafe { $type::from_raw((*self).type_().raw()) }
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
        )+

        /// Returned by [`Value::downcast`]
        pub enum ValueEnum<'ctx> {
            $(
                $name($name<'ctx>),
            )+
        }

        impl<'ctx> Value<'ctx> {
            /// Tries to downcast a generic `Value` into a more-specific value type.
            pub fn downcast(self) -> Option<ValueEnum<'ctx>> {
                Some(match self.type_().kind() {
                    $(
                        LLVMTypeKind::$type_kind => {
                            ValueEnum::$name(unsafe { $name::from_raw(self.raw()) })
                        }
                    )+
                    _ => return None,
                })
            }
        }
    };
}

specialized_values! {
    /// An LLVM function value reference
    pub struct FnValue: FnType @ LLVMFunctionTypeKind

    /// An LLVM integer value reference
    pub struct IntValue: IntType @ LLVMIntegerTypeKind

    /// An LLVM integer value reference
    pub struct StructValue: StructType @ LLVMStructTypeKind
}

impl<'ctx> FnValue<'ctx> {
    pub fn add_basic_block(self, name: &CStr) -> BasicBlock<'ctx> {
        unsafe {
            let context = LLVMGetTypeContext(LLVMTypeOf(self.ptr));
            BasicBlock::from_raw(LLVMAppendBasicBlockInContext(
                context,
                self.ptr,
                name.as_ptr(),
            ))
        }
    }

    pub fn num_params(self) -> u32 {
        unsafe { LLVMCountParams(self.ptr) }
    }

    pub fn param(self, idx: u32) -> Option<Value<'ctx>> {
        (idx < self.num_params()).then(|| unsafe { Value::from_raw(LLVMGetParam(self.ptr, idx)) })
    }
}
