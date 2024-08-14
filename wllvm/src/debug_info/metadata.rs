use core::slice;
use std::{marker::PhantomData, mem::MaybeUninit, ops::Deref};

use llvm_sys::{
    core::LLVMMetadataAsValue,
    debuginfo::{
        LLVMDIFileGetDirectory, LLVMDIFileGetFilename, LLVMDIFileGetSource, LLVMDIScopeGetFile,
        LLVMGetMetadataKind, LLVMMetadataKind,
    },
    LLVMOpaqueMetadata,
};

use crate::{value::Value, Context};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Metadata<'ctx> {
    ptr: *mut LLVMOpaqueMetadata,
    _phantomdata: PhantomData<&'ctx LLVMOpaqueMetadata>,
}

impl<'ctx> Metadata<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMOpaqueMetadata) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn as_value(&self, context: &'ctx Context) -> Value<'ctx> {
        unsafe { Value::from_raw(LLVMMetadataAsValue(context.raw(), self.ptr)) }
    }

    pub fn kind(&self) -> LLVMMetadataKind {
        unsafe { LLVMGetMetadataKind(self.ptr) }
    }
}

// maybe i should've just written the boilerplate instead of creating this
// giant, unholy declarative macro
metadata_types! {
    pub struct DICompileUnit
        kind: LLVMDICompileUnitMetadataKind
        supertype: DIScope; // `is_scope` flag is implied by the supertype

    /// A source code file
    pub struct DIFile
        kind: LLVMDIFileMetadataKind
        supertype: DIScope;

    /// A location in a source code file
    pub struct DILocation
        kind: LLVMDILocationMetadataKind;

    pub struct DILexicalBlock
        kind: LLVMDILexicalBlockMetadataKind
        supertype: DILocalScope
        flags: [is_scope];

    /// A function
    pub struct DISubprogram
        kind: LLVMDISubprogramMetadataKind
        supertype: DILocalScope
        flags: [is_scope];

    pub struct DIScope;

    /// A scope that can contain lexical blocks, local variables, and debug info locations.
    pub struct DILocalScope
        supertype: DIScope;

    pub struct DIType
        supertype: DIScope;

    pub struct DISubroutineType
        kind: LLVMDISubroutineTypeMetadataKind
        supertype: DIType
        flags: [is_scope];

    pub struct DIBasicType
        kind: LLVMDIBasicTypeMetadataKind
        supertype: DIType
        flags: [is_scope];

    pub struct DICompositeType
        kind: LLVMDICompositeTypeMetadataKind
        supertype: DIType
        flags: [is_scope];

    pub struct DIDerivedType
        kind: LLVMDIDerivedTypeMetadataKind
        supertype: DIType
        flags: [is_scope];
}

impl<'ctx> DIScope<'ctx> {
    pub fn file(&self) -> DIFile<'ctx> {
        unsafe { DIFile::from_raw(LLVMDIScopeGetFile(self.ptr)) }
    }

    pub fn filename(&self) -> &'ctx [u8] {
        unsafe {
            let mut len = MaybeUninit::uninit();

            let raw = LLVMDIFileGetFilename(self.ptr, len.as_mut_ptr()).cast::<u8>();

            slice::from_raw_parts(raw, len.assume_init() as usize)
        }
    }

    pub fn directory(&self) -> &'ctx [u8] {
        unsafe {
            let mut len = MaybeUninit::uninit();

            let raw = LLVMDIFileGetDirectory(self.ptr, len.as_mut_ptr()).cast::<u8>();

            slice::from_raw_parts(raw, len.assume_init() as usize)
        }
    }

    /// Gets the source code of the file. Returns `b""` if source code is not found.
    pub fn source(&self) -> &'ctx [u8] {
        unsafe {
            let mut len = MaybeUninit::uninit();

            let raw = LLVMDIFileGetSource(self.ptr, len.as_mut_ptr()).cast::<u8>();

            slice::from_raw_parts(raw, len.assume_init() as usize)
        }
    }
}

/// Generates the Metadata sub-types, the [`MetadataEnum`] type, and [`Metadata::downcast`]
macro_rules! metadata_types {
    {
        $(
            $(#[doc = $doc:literal])*
            $vis:vis struct $name:ident
                $(kind: $kind:ident)?
                $(supertype: $supertype:ident)?
                $(flags: [$($flag:ident),+ $(,)?])?
        );+ $(;)?
    } => {
        $(
            $(
                #[doc = $doc]
            )*

            #[repr(transparent)]
            #[derive(Clone, Copy)]
            $vis struct $name<'ctx> {
                inner: Metadata<'ctx>,
            }

            impl<'ctx> $name<'ctx> {
                pub unsafe fn from_raw(ptr: *mut LLVMOpaqueMetadata) -> Self {
                    unsafe {
                        Self {
                            inner: Metadata::from_raw(ptr),
                        }
                    }
                }

                pub fn raw(&self) -> *mut LLVMOpaqueMetadata {
                    self.ptr
                }
            }

            impl<'ctx> AsRef<Metadata<'ctx>> for $name<'ctx> {
                fn as_ref(&self) -> &Metadata<'ctx> {
                    &self
                }
            }

            // generates the `Deref` implementation for the type. The
            // `Metadata` type is assumed if no supertype is provided.
            add_metadata_deref!{$name $($supertype)?}

            impl<'ctx> From<$name<'ctx>> for Metadata<'ctx> {
                fn from(value: $name<'ctx>) -> Self {
                    value.inner
                }
            }

            $(
                #[doc = noop_ident!($kind)]
                impl<'ctx> From<$name<'ctx>> for MetadataEnum<'ctx> {
                    fn from(value: $name<'ctx>) -> Self {
                        MetadataEnum::$name(value)
                    }
                }
            )?

            $(
                $(metadata_flag!{$name $flag})+
            )?
        )+

        /// Returned by [`Metadata::downcast`]
        pub enum MetadataEnum<'ctx> {
            $($(
                // this doc comment does nothing. It's just needed to match the
                // $kind metavariable
                #[doc = noop_ident!($kind)]
                $name($name<'ctx>),
            )?)+
        }

        impl<'ctx> Metadata<'ctx> {
            /// Tries to convert the metadata into a more specialized type.
            pub fn downcast(&self) -> Option<MetadataEnum<'ctx>> {
                Some(unsafe {
                    match self.kind() {
                        $($(
                            LLVMMetadataKind::$kind=> $name::from_raw(self.ptr).into(),
                        )?)+
                        _ => return None,
                    }
                })
            }
        }
    };
}

/// Generates the deref implementation for a metadata type.
///
/// This is a helper macro for the [`metadata_types`] macro.
macro_rules! add_metadata_deref {
    {$name:ident} => {
        impl<'ctx> Deref for $name<'ctx> {
            type Target = Metadata<'ctx>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    };
    {$name:ident $supertype:ident} => {
        impl<'ctx> AsRef<$supertype<'ctx>> for $name<'ctx> {
            fn as_ref(&self) -> &$supertype<'ctx> {
                &self
            }
        }

        impl<'ctx> From<$name<'ctx>> for $supertype<'ctx> {
            fn from(value: $name<'ctx>) -> Self {
                unsafe {std::mem::transmute::<$name, $supertype>(value)}
            }
        }

        impl<'ctx> Deref for $name<'ctx> {
            type Target = $supertype<'ctx>;

            fn deref(&self) -> &Self::Target {
                unsafe {$crate::util::transmute_ref::<$name, $supertype>(self)}
            }
        }
    }
}

/// Generates code for a "flag". Helper macro for [`metadata_types`]
macro_rules! metadata_flag {
    {
        $name:ident is_scope
    } => {
        impl<'ctx> AsRef<DIScope<'ctx>> for $name<'ctx> {
            fn as_ref(&self) -> &DIScope<'ctx> {
                unsafe { $crate::util::transmute_ref::<$name<'ctx>, DIScope<'ctx>>(self) }
            }
        }

        impl<'ctx> From<$name<'ctx>> for DIScope<'ctx> {
            fn from(value: $name<'ctx>) -> Self {
                unsafe { DIScope::from_raw(value.ptr) }
            }
        }
    }
}

/// A macro that returns `""`. This is useful for matching metavariables without using them.
macro_rules! noop_ident {
    {$ident:ident} => {""}
}

pub(self) use add_metadata_deref;
pub(self) use metadata_flag;
pub(self) use metadata_types;
pub(self) use noop_ident;
