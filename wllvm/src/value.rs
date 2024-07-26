use std::{
    ffi::{c_char, CStr},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    slice,
};

use llvm_sys::{
    core::{
        LLVMAddIncoming, LLVMAppendBasicBlockInContext, LLVMCountParams, LLVMGetParam,
        LLVMGetTypeContext, LLVMGetValueName2, LLVMSetValueName2, LLVMTypeOf,
    },
    LLVMBasicBlock, LLVMTypeKind, LLVMValue,
};

use crate::{
    basic_block::BasicBlock,
    type_::{FnType, IntType, PtrType, StructType},
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

    // TODO: test with no name and with inline asm value
    pub fn name(&self) -> &'ctx [u8] {
        unsafe {
            let mut len = MaybeUninit::uninit();
            let ptr = LLVMGetValueName2(self.ptr, len.as_mut_ptr());

            let len = len.assume_init();
            slice::from_raw_parts(ptr.cast::<u8>(), len)
        }
    }

    pub fn set_name<S>(&self, name: &S)
    where
        S: ?Sized + AsRef<[u8]>,
    {
        let name = name.as_ref();
        unsafe { LLVMSetValueName2(self.ptr, name.as_ptr().cast::<c_char>(), name.len()) }
    }

    pub fn type_(self) -> Type<'ctx> {
        unsafe { Type::from_raw(LLVMTypeOf(self.ptr)) }
    }
}

macro_rules! noop_ident {
    {$ident:ident} => {
        ""
    }
}

macro_rules! specialized_values {
    {
        $(
            $(#[doc = $doc:literal])*
            pub struct $name:ident $( : $type:ident @ $type_kind:ident)?;
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

                $(
                    pub fn type_(self) -> $type<'ctx> {
                        unsafe { $type::from_raw((*self).type_().raw()) }
                    }
                )?
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
            $($(
                #[doc = noop_ident!($type_kind)] // to match $type_kind
                $name($name<'ctx>),
            )?)+
        }

        impl<'ctx> Value<'ctx> {
            /// Tries to downcast a generic `Value` into a more-specific value type.
            pub fn downcast(self) -> Option<ValueEnum<'ctx>> {
                Some(match self.type_().kind() {
                    $($(
                        LLVMTypeKind::$type_kind => {
                            ValueEnum::$name(unsafe { $name::from_raw(self.raw()) })
                        }
                    )?)+
                    _ => return None,
                })
            }
        }
    };
}

specialized_values! {
    /// An LLVM function value reference
    pub struct FnValue: FnType @ LLVMFunctionTypeKind;

    /// An LLVM integer value reference
    pub struct IntValue: IntType @ LLVMIntegerTypeKind;

    /// An LLVM pointer value reference
    pub struct PtrValue: PtrType @ LLVMPointerTypeKind;

    /// An LLVM integer value reference
    pub struct StructValue: StructType @ LLVMStructTypeKind;

    /// An LLVM phi value reference
    pub struct PhiValue;
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

impl<'ctx> PhiValue<'ctx> {
    /// Adds an incoming block and value.
    ///
    /// Returns `false` iff `values.len()` != `block.len()`
    pub fn add_incoming(self, values: &[Value<'ctx>], blocks: &[BasicBlock<'ctx>]) -> bool {
        if values.len() != blocks.len() {
            return false;
        }

        let values_ptr = values.as_ptr().cast::<*mut LLVMValue>().cast_mut();
        let blocks_ptr = blocks.as_ptr().cast::<*mut LLVMBasicBlock>().cast_mut();

        unsafe { LLVMAddIncoming(self.ptr, values_ptr, blocks_ptr, values.len() as u32) }

        true
    }
}
