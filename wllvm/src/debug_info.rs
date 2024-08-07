use std::{ffi::c_char, marker::PhantomData};

use llvm_sys::{
    debuginfo::{
        LLVMCreateDIBuilder, LLVMDIBuilderCreateBasicType, LLVMDIBuilderCreateFile,
        LLVMDIBuilderCreateFunction, LLVMDIBuilderCreateSubroutineType, LLVMDIBuilderFinalize,
        LLVMDisposeDIBuilder,
    },
    prelude::LLVMBool,
    LLVMOpaqueDIBuilder, LLVMOpaqueMetadata,
};

pub use compile_unit::*;
pub use metadata::*;
pub use re_exports::*;

use crate::Module;
mod compile_unit;
mod metadata;
mod re_exports;

#[repr(transparent)]
pub struct DIBuilder<'ctx> {
    ptr: *mut LLVMOpaqueDIBuilder,
    _phantomdata: PhantomData<&'ctx LLVMOpaqueDIBuilder>,
}

impl<'ctx> DIBuilder<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMOpaqueDIBuilder) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn new(module: &Module) -> Self {
        let builder = unsafe { LLVMCreateDIBuilder(module.raw()) };

        // TODO: compile unit may be necessary

        unsafe { Self::from_raw(builder) }
    }

    pub fn subprogram(
        &self,
        scope: DIScope<'ctx>,
        name: &(impl ?Sized + AsRef<[u8]>),
        linkage_name: &(impl ?Sized + AsRef<[u8]>),
        file: DIFile<'ctx>,
        line_no: u32,
        scope_line_no: u32,
        ty: DISubroutineType<'ctx>,
        local_to_unit: bool,
        is_definition: bool,
        is_optimized: bool,
        flags: DIFlags,
    ) -> DISubprogram<'ctx> {
        let linkage_name = linkage_name.as_ref();
        let name = name.as_ref();

        let name_ptr = name.as_ptr().cast::<c_char>();
        let linkage_name_ptr = linkage_name.as_ptr().cast::<c_char>();

        unsafe {
            DISubprogram::from_raw(LLVMDIBuilderCreateFunction(
                self.ptr,
                scope.raw(),
                name_ptr,
                name.len(),
                linkage_name_ptr,
                linkage_name.len(),
                file.raw(),
                line_no,
                ty.raw(),
                local_to_unit as LLVMBool,
                is_definition as LLVMBool,
                scope_line_no,
                flags.into(),
                is_optimized as LLVMBool,
            ))
        }
    }

    /// Creates a subroutine type
    ///
    /// * `file` - The file that the function resides in
    /// * `params` - A list of parameter types; the 0th type is the return value
    /// * `flags` - Flags of the subroutine type
    pub fn subroutine_type(
        &self,
        file: DIFile<'ctx>,
        params: &[DIType<'ctx>],
        flags: DIFlags,
    ) -> DISubroutineType<'ctx> {
        let params_ptr = params.as_ptr().cast::<*mut LLVMOpaqueMetadata>().cast_mut();

        unsafe {
            DISubroutineType::from_raw(LLVMDIBuilderCreateSubroutineType(
                self.ptr,
                file.raw(),
                params_ptr,
                params.len() as u32,
                flags.into(),
            ))
        }
    }

    /// Create debugging information entry for a basic type.
    ///
    /// * `Name` - Type name.
    /// * `SizeInBits` - Size of the type.
    /// * `Encoding` - DWARF encoding code, e.g., `TypeEncoding::float`.
    /// * `Flags` - Optional DWARF attributes, e.g., `DIFlags::Bigendian`.
    pub fn basic_type(
        &self,
        name: &(impl ?Sized + AsRef<[u8]>),
        size_bits: u64,
        encoding: Option<TypeEncoding>,
        flags: DIFlags,
    ) -> DIBasicType<'ctx> {
        let name = name.as_ref();
        let name_ptr = name.as_ptr().cast::<c_char>();

        unsafe {
            DIBasicType::from_raw(LLVMDIBuilderCreateBasicType(
                self.ptr,
                name_ptr,
                name.len(),
                size_bits,
                encoding.map_or(0, |e| e as u32),
                flags.into(),
            ))
        }
    }

    /// Creates a builder for a DICompileUnit.
    ///
    /// All builder options that aren't parameters of this function are optional.
    ///
    /// * `file` - File info
    /// * `language` - Source programming language, eg. dwarf::DW_LANG_C99
    /// * `producer` - Identify the producer of debugging information and code. Usually this is a compiler version string
    /// * `optimized` - A boolean flag which indicates whether optimization is enabled or not
    /// * `flags` - This string lists command line options. This string is directly embedded in debug info output which may be used by a tool analyzing generated debugging information
    /// * `runtime_ver` - This indicates runtime version for languages like Objective-C
    pub fn build_compile_unit<'a>(
        &'a self,
        file: DIFile<'ctx>,
        language: SourceLanguage,
        producer: &'a (impl ?Sized + AsRef<[u8]>),
        optimized: bool,
        flags: &'a (impl ?Sized + AsRef<[u8]>),
        runtime_ver: u32,
    ) -> DICompileUnitBuilder<'a, 'ctx> {
        DICompileUnitBuilder {
            di_builder: &self,
            file,
            language,
            producer: producer.as_ref(),
            optimized,
            flags: flags.as_ref(),
            runtime_ver,
            split_name: b"",
            kind: EmissionKind::Full,
            dwoid: 0,
            split_debug_inlining: true,
            debug_info_for_profiling: false,
            sysroot: b"",
            sdk: b"",
        }
    }

    pub fn file(
        &self,
        basename: &(impl ?Sized + AsRef<[u8]>),
        directory: &(impl ?Sized + AsRef<[u8]>),
    ) -> DIFile {
        let basename = basename.as_ref();
        let directory = directory.as_ref();

        let basename_ptr = basename.as_ptr().cast::<c_char>();
        let dir_ptr = directory.as_ptr().cast::<c_char>();

        unsafe {
            DIFile::from_raw(LLVMDIBuilderCreateFile(
                self.ptr,
                basename_ptr,
                basename.len(),
                dir_ptr,
                directory.len(),
            ))
        }
    }

    pub fn finalize(&self) {
        unsafe { LLVMDIBuilderFinalize(self.ptr) }
    }
}

impl<'ctx> Drop for DIBuilder<'ctx> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeDIBuilder(self.ptr) }
    }
}
