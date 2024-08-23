use std::{
    ffi::{c_char, CStr},
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    ptr, slice,
};

use llvm_sys::{
    core::{
        LLVMAddAttributeAtIndex, LLVMAddIncoming, LLVMAppendBasicBlockInContext, LLVMCountParams,
        LLVMGetLinkage, LLVMGetParam, LLVMGetTypeContext, LLVMGetValueKind, LLVMGetValueName2,
        LLVMGlobalGetValueType, LLVMIsDeclaration, LLVMIsGlobalConstant, LLVMPrintValueToString,
        LLVMSetGlobalConstant, LLVMSetInitializer, LLVMSetLinkage, LLVMSetValueName2, LLVMTypeOf,
    },
    debuginfo::LLVMSetSubprogram,
    prelude::LLVMBool,
    LLVMBasicBlock, LLVMTypeKind, LLVMValue, LLVMValueKind,
};

use crate::{
    attribute::Attribute,
    basic_block::BasicBlock,
    debug_info::DISubprogram,
    type_::{ArrayType, FnType, IntType, PtrType, StructType},
    util::LLVMString,
    Type,
};

mod re_exports;
pub use re_exports::*;

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

    pub fn raw(&self) -> *mut LLVMValue {
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

    pub fn type_(&self) -> Type<'ctx> {
        unsafe { Type::from_raw(LLVMTypeOf(self.ptr)) }
    }

    pub fn kind(&self) -> LLVMValueKind {
        unsafe { LLVMGetValueKind(self.ptr) }
    }

    pub fn print_to_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMPrintValueToString(self.ptr)) }
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self.print_to_string();
        let cstr = String::from_utf8_lossy(str.as_bytes());

        std::fmt::Write::write_str(f, &*cstr)
    }
}

specialized_values! {
    /// An LLVM function value reference
    pub struct FnValue
        supertype: GlobalValue
        value_kind: LLVMFunctionValueKind;

    /// An LLVM integer value reference
    pub struct IntValue: IntType @ LLVMIntegerTypeKind;

    /// An LLVM pointer value reference
    pub struct PtrValue: PtrType @ LLVMPointerTypeKind;

    /// An LLVM integer value reference
    pub struct StructValue: StructType @ LLVMStructTypeKind;

    pub struct ArrayValue: ArrayType @ LLVMArrayTypeKind;

    /// An LLVM phi value reference
    pub struct PhiValue
        value_kind: LLVMMemoryPhiValueKind;

    /// An LLVM global value reference
    pub struct GlobalValue;

    /// An LLVM global variable reference
    pub struct GlobalVariable
        supertype: GlobalValue;
}

impl<'ctx> FnValue<'ctx> {
    pub fn add_basic_block(&self, name: &CStr) -> BasicBlock<'ctx> {
        unsafe {
            let context = LLVMGetTypeContext(LLVMTypeOf(self.ptr));
            BasicBlock::from_raw(LLVMAppendBasicBlockInContext(
                context,
                self.ptr,
                name.as_ptr(),
            ))
        }
    }

    pub fn add_attribute(&self, attr: Attribute<'ctx>) {
        unsafe { LLVMAddAttributeAtIndex(self.ptr, u32::MAX, attr.raw()) }
    }

    pub fn num_params(&self) -> u32 {
        unsafe { LLVMCountParams(self.ptr) }
    }

    pub fn param(&self, idx: u32) -> Option<Value<'ctx>> {
        (idx < self.num_params()).then(|| unsafe { Value::from_raw(LLVMGetParam(self.ptr, idx)) })
    }

    pub fn set_subprogram(&self, subprogram: DISubprogram<'ctx>) {
        unsafe { LLVMSetSubprogram(self.ptr, subprogram.raw()) }
    }

    pub fn type_(&self) -> FnType<'ctx> {
        unsafe { FnType::from_raw(LLVMGlobalGetValueType(self.ptr)) }
    }
}

impl<'ctx> PhiValue<'ctx> {
    /// Adds an incoming block and value.
    ///
    /// Returns `false` iff `values.len()` != `block.len()`
    pub fn add_incoming(&self, values: &[Value<'ctx>], blocks: &[BasicBlock<'ctx>]) -> bool {
        if values.len() != blocks.len() {
            return false;
        }

        let values_ptr = values.as_ptr().cast::<*mut LLVMValue>().cast_mut();
        let blocks_ptr = blocks.as_ptr().cast::<*mut LLVMBasicBlock>().cast_mut();

        unsafe { LLVMAddIncoming(self.ptr, values_ptr, blocks_ptr, values.len() as u32) }

        true
    }
}

impl<'ctx> GlobalValue<'ctx> {
    pub fn linkage(&self) -> Linkage {
        unsafe { LLVMGetLinkage(self.ptr) }.into()
    }

    pub fn set_linkage(&self, linkage: Linkage) {
        unsafe { LLVMSetLinkage(self.ptr, linkage.into()) }
    }

    pub fn is_declaration(&self) -> bool {
        unsafe { LLVMIsDeclaration(self.ptr) != 0 }
    }
}

impl<'ctx> GlobalVariable<'ctx> {
    pub fn set_initializer(&self, initializer: Option<Value<'ctx>>) {
        let initializer = initializer.map_or(ptr::null_mut(), |i| i.raw());

        unsafe { LLVMSetInitializer(self.ptr, initializer) }
    }

    pub fn set_constant(&self, constant: bool) {
        unsafe { LLVMSetGlobalConstant(self.ptr, constant as LLVMBool) }
    }

    pub fn is_constant(&self) -> bool {
        unsafe { LLVMIsGlobalConstant(self.ptr) != 0 }
    }

    pub fn as_ptr(&self) -> PtrValue<'ctx> {
        unsafe { PtrValue::from_raw(self.ptr) }
    }
}

macro_rules! noop_ident {
    {$ident:ident} => {
        ""
    }
}

macro_rules! generate_deref {
    {$name:ident} => {
        impl<'ctx> Deref for $name<'ctx> {
            type Target = Value<'ctx>;

            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
    };

    {$name:ident $supertype:ident} => {
        impl<'ctx> Deref for $name<'ctx> {
            type Target = $supertype<'ctx>;

            fn deref(&self) -> &Self::Target {
                unsafe {::wutil::transmute_ref::<Self, Self::Target>(self)}
            }
        }
    };
}

macro_rules! specialized_values {
    {
        $(
            $(#[doc = $doc:literal])*
            pub struct $name:ident $( : $type:ident)? $(@ $type_kind:ident)?
                $(supertype: $supertype:ident)?
                $(value_kind: $value_kind:ident)?;
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
                    pub fn type_(&self) -> $type<'ctx> {
                        unsafe { $type::from_raw(Value::from(*self).type_().raw()) }
                    }
                )?
            }

            generate_deref!($name $($supertype)?);

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

            impl Debug for $name<'_> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let val: &Value = self.as_ref();
                    Debug::fmt(val, f)
                }
            }

            $(
                impl<'ctx> TryFrom<Value<'ctx>> for $name<'ctx> {
                    type Error = ();

                    fn try_from(value: Value<'ctx>) -> Result<Self, ()> {
                        #[allow(unused_doc_comments)]
                        #[doc = noop_ident!($type_kind)] // to match $type_kind
                        {}

                        if let Some(ValueEnum::$name(val)) = value.downcast() {
                            Ok(val)
                        } else {
                            Err(())
                        }
                    }
                }
            )?

            $(
                impl<'ctx> TryFrom<Value<'ctx>> for $name<'ctx> {
                    type Error = ();
                    fn try_from(value: Value<'ctx>) -> Result<Self, ()> {
                        if value.kind() == LLVMValueKind::$value_kind {
                            Ok(Self {value})
                        } else {
                            Err(())
                        }
                    }
                }
            )?
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
            pub fn downcast(&self) -> Option<ValueEnum<'ctx>> {
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

pub(self) use generate_deref;
pub(self) use noop_ident;
pub(self) use specialized_values;
