use std::ffi::c_char;

use llvm_sys::{debuginfo::LLVMDIBuilderCreateCompileUnit, prelude::LLVMBool};

use super::{DIBuilder, DICompileUnit, DIFile};

mod re_exports;

pub use re_exports::*;

pub struct DICompileUnitBuilder<'a, 'ctx> {
    pub di_builder: &'a DIBuilder<'ctx>,
    pub file: DIFile<'ctx>,
    pub language: SourceLanguage,
    pub producer: &'a [u8],
    pub optimized: bool,
    pub flags: &'a [u8],
    pub runtime_ver: u32,
    pub split_name: &'a [u8],
    pub kind: EmissionKind,
    pub dwoid: u32,
    pub split_debug_inlining: bool,
    pub debug_info_for_profiling: bool,
    pub sysroot: &'a [u8],
    pub sdk: &'a [u8],
}

impl<'a, 'ctx> DICompileUnitBuilder<'a, 'ctx> {
    /// The name of the file that we'll split debug info out into.
    pub fn split_name(mut self, split_name: &'a (impl ?Sized + AsRef<[u8]>)) -> Self {
        self.split_name = split_name.as_ref();
        self
    }

    /// The kind of debug information to generate.
    pub fn kind(mut self, kind: EmissionKind) -> Self {
        self.kind = kind;
        self
    }

    /// The DWOId if this is a split skeleton compile unit.
    pub fn dwoid(mut self, dwoid: u32) -> Self {
        self.dwoid = dwoid;
        self
    }

    /// Whether to emit inline debug info.
    pub fn split_debug_inlining(mut self, split_debug_inlining: bool) -> Self {
        self.split_debug_inlining = split_debug_inlining;
        self
    }

    /// Whether to emit extra debug info for profile collection.
    pub fn debug_info_for_profiling(mut self, debug_info_for_profiling: bool) -> Self {
        self.debug_info_for_profiling = debug_info_for_profiling;
        self
    }

    /// The clang system root (value of -isysroot).
    pub fn sysroot(mut self, sysroot: &'a (impl ?Sized + AsRef<[u8]>)) -> Self {
        self.sysroot = sysroot.as_ref();
        self
    }

    /// The SDK name. On Darwin, this is the last component of the sysroot.
    pub fn sdk(mut self, sdk: &'a (impl ?Sized + AsRef<[u8]>)) -> Self {
        self.sdk = sdk.as_ref();
        self
    }

    pub fn build(self) -> DICompileUnit<'ctx> {
        {
            let producer_ptr = self.producer.as_ptr().cast::<c_char>();
            let flags_ptr = self.flags.as_ptr().cast::<c_char>();
            let split_name_ptr = self.split_name.as_ptr().cast::<c_char>();
            let sysroot_ptr = self.sysroot.as_ptr().cast::<c_char>();
            let sdk_ptr = self.sdk.as_ptr().cast::<c_char>();

            unsafe {
                DICompileUnit::from_raw(LLVMDIBuilderCreateCompileUnit(
                    self.di_builder.ptr,
                    self.language.into(),
                    self.file.raw(),
                    producer_ptr,
                    self.producer.len(),
                    self.optimized as LLVMBool,
                    flags_ptr,
                    self.flags.len(),
                    self.runtime_ver,
                    split_name_ptr,
                    self.split_name.len(),
                    self.kind.into(),
                    self.dwoid,
                    self.split_debug_inlining as LLVMBool,
                    self.debug_info_for_profiling as LLVMBool,
                    sysroot_ptr,
                    self.sysroot.len(),
                    sdk_ptr,
                    self.sdk.len(),
                ))
            }
        }
    }
}
