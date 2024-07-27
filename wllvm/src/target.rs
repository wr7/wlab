use std::{ffi::CStr, mem::MaybeUninit};

use llvm_sys::{
    target::{
        LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter,
        LLVM_InitializeNativeDisassembler, LLVM_InitializeNativeTarget,
    },
    target_machine::{
        LLVMCreateTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures,
        LLVMGetHostCPUName, LLVMGetTargetDescription, LLVMGetTargetFromTriple, LLVMGetTargetName,
        LLVMTarget,
    },
};

use crate::util::{LLVMErrorString, LLVMString};

pub use re_exports::{CodeModel, OptLevel, RelocMode};

pub use target_data::TargetData;
pub use target_machine::TargetMachine;

mod re_exports;
mod target_data;
mod target_machine;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Target {
    ptr: *mut LLVMTarget,
}

impl Target {
    pub unsafe fn from_raw(ptr: *mut LLVMTarget) -> Self {
        Self { ptr }
    }

    /// Initializes the native target.
    ///
    /// Returns `false` if there is no native target.
    pub fn initialize_native(asm_parser: bool, asm_printer: bool, disassembler: bool) -> bool {
        let ret_val = unsafe { LLVM_InitializeNativeTarget() == 0 };

        if asm_parser {
            unsafe { LLVM_InitializeNativeAsmParser() };
        }
        if asm_printer {
            unsafe { LLVM_InitializeNativeAsmPrinter() };
        }
        if disassembler {
            unsafe { LLVM_InitializeNativeDisassembler() };
        }

        ret_val
    }

    /// Obtains a target.
    ///
    /// The corresponding `initialize` function should be called first or else
    /// an error will be returned.
    pub fn from_triple(triple: &CStr) -> Result<Self, LLVMErrorString> {
        unsafe {
            let mut err_msg = MaybeUninit::uninit();
            let mut target = MaybeUninit::uninit();

            let success =
                LLVMGetTargetFromTriple(triple.as_ptr(), target.as_mut_ptr(), err_msg.as_mut_ptr())
                    == 0;

            if success {
                Ok(Self::from_raw(target.assume_init()))
            } else {
                Err(LLVMErrorString::from_raw(err_msg.assume_init()))
            }
        }
    }

    pub fn create_target_machine(
        &self,
        triple: &CStr,
        cpu: &CStr,
        cpu_features: &CStr,
        opt_level: OptLevel,
        reloc_mode: RelocMode,
        code_model: CodeModel,
    ) -> TargetMachine {
        unsafe {
            TargetMachine::from_raw(LLVMCreateTargetMachine(
                self.ptr,
                triple.as_ptr(),
                cpu.as_ptr(),
                cpu_features.as_ptr(),
                opt_level.into(),
                reloc_mode.into(),
                code_model.into(),
            ))
        }
    }

    pub fn description(&self) -> &'static CStr {
        unsafe { CStr::from_ptr(LLVMGetTargetDescription(self.ptr)) }
    }

    pub fn name(&self) -> &'static CStr {
        unsafe { CStr::from_ptr(LLVMGetTargetName(self.ptr)) }
    }
}

pub fn host_target_triple() -> LLVMString {
    unsafe { LLVMString::from_raw(LLVMGetDefaultTargetTriple()) }
}

pub fn host_cpu() -> LLVMString {
    unsafe { LLVMString::from_raw(LLVMGetHostCPUName()) }
}

pub fn host_cpu_features() -> LLVMString {
    unsafe { LLVMString::from_raw(LLVMGetHostCPUFeatures()) }
}
