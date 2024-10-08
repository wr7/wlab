use std::{
    ffi::{c_char, CStr},
    marker::PhantomData,
    mem::MaybeUninit,
};

use llvm_sys::{
    analysis::LLVMVerifyModule,
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMDisposeModule, LLVMGetNamedFunction,
        LLVMPrintModuleToFile, LLVMPrintModuleToString,
    },
    target_machine::{
        LLVMCodeGenFileType, LLVMTargetMachineEmitToFile, LLVMTargetMachineEmitToMemoryBuffer,
    },
    LLVMModule,
};

use crate::{
    target::TargetMachine,
    type_::FnType,
    util::{self, LLVMErrorString, LLVMString, MemoryBuffer},
    value::{FnValue, GlobalVariable},
    Type,
};

/// An LLVM Module
#[repr(transparent)]
pub struct Module<'ctx> {
    ptr: *mut LLVMModule,
    _phantomdata: PhantomData<&'ctx LLVMModule>,
}

impl<'ctx> Module<'ctx> {
    /// Wraps a raw LLVMModule pointer,
    /// # Safety
    /// `'ctx` cannot outlive the context of the module
    pub unsafe fn from_raw(ptr: *mut LLVMModule) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn raw(&self) -> *mut LLVMModule {
        self.ptr
    }

    /// Creates a function in the module
    pub fn add_function(&self, name: &CStr, function_type: FnType<'ctx>) -> FnValue<'ctx> {
        unsafe {
            FnValue::from_raw(LLVMAddFunction(
                self.ptr,
                name.as_ptr(),
                function_type.raw(),
            ))
        }
    }

    /// Gets a function by its name
    /// NOTE: `name` currently cannot contain any null bytes
    pub fn get_function(&self, name: &(impl ?Sized + AsRef<[u8]>)) -> Option<FnValue<'ctx>> {
        let name = util::get_cstr_of(name.as_ref()).unwrap();

        unsafe {
            let raw = LLVMGetNamedFunction(self.ptr, name.as_ptr().cast::<c_char>());

            util::recycle_cstr(name);

            if raw.is_null() {
                return None;
            }

            Some(FnValue::from_raw(raw))
        }
    }

    pub fn add_global(&self, type_: Type<'ctx>, name: &CStr) -> GlobalVariable<'ctx> {
        unsafe {
            GlobalVariable::from_raw(LLVMAddGlobal(
                self.ptr,
                type_.raw(),
                name.as_ptr().cast::<c_char>(),
            ))
        }
    }

    pub fn compile_to_buffer(
        &self,
        target: &TargetMachine,
        dont_assemble: bool,
    ) -> Result<MemoryBuffer, LLVMString> {
        unsafe {
            let mut buf = MaybeUninit::uninit();
            let mut err_msg = MaybeUninit::uninit();

            let file_type = if dont_assemble {
                LLVMCodeGenFileType::LLVMAssemblyFile
            } else {
                LLVMCodeGenFileType::LLVMObjectFile
            };

            let succ = LLVMTargetMachineEmitToMemoryBuffer(
                target.raw(),
                self.ptr,
                file_type,
                err_msg.as_mut_ptr(),
                buf.as_mut_ptr(),
            ) == 0;

            if succ {
                Ok(MemoryBuffer::from_raw(buf.assume_init()))
            } else {
                Err(LLVMString::from_raw(err_msg.assume_init()))
            }
        }
    }

    pub fn compile(
        &self,
        target: &TargetMachine,
        file_name: &CStr,
        dont_assemble: bool,
    ) -> Result<(), LLVMString> {
        let file_type = if dont_assemble {
            LLVMCodeGenFileType::LLVMAssemblyFile
        } else {
            LLVMCodeGenFileType::LLVMObjectFile
        };

        unsafe {
            let mut err_msg = MaybeUninit::uninit();

            let succ = LLVMTargetMachineEmitToFile(
                target.raw(),
                self.ptr,
                file_name.as_ptr(),
                file_type,
                err_msg.as_mut_ptr(),
            ) == 0;

            if succ {
                Ok(())
            } else {
                Err(LLVMString::from_raw(err_msg.assume_init()))
            }
        }
    }

    pub fn verify(&self) -> Result<(), LLVMString> {
        let mut string = MaybeUninit::<*mut i8>::uninit();

        let result = unsafe {
            LLVMVerifyModule(
                self.ptr,
                llvm_sys::analysis::LLVMVerifierFailureAction::LLVMReturnStatusAction,
                string.as_mut_ptr(),
            )
        };

        if result != 0 {
            Err(unsafe { LLVMString::from_raw(string.assume_init_read()) })
        } else {
            Ok(())
        }
    }

    /// Converts the module to human-readable LLVM IR and then prints it to a string
    pub fn print_to_string(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMPrintModuleToString(self.ptr)) }
    }

    /// Converts the module to human-readable LLVM IR and then writes it to a file
    pub fn print_to_file(&self, filename: &CStr) -> Result<(), LLVMErrorString> {
        let mut err_msg: MaybeUninit<*mut c_char> = MaybeUninit::uninit();

        let result =
            unsafe { LLVMPrintModuleToFile(self.ptr, filename.as_ptr(), err_msg.as_mut_ptr()) };

        if result != 0 {
            unsafe { Err(LLVMErrorString::from_raw(err_msg.assume_init())) }
        } else {
            Ok(())
        }
    }
}

impl<'ctx> Drop for Module<'ctx> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeModule(self.ptr) }
    }
}
