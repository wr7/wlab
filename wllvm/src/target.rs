use std::{ffi::CStr, mem::MaybeUninit};

use llvm_sys::target_machine::{
    LLVMCreateTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures,
    LLVMGetHostCPUName, LLVMGetTargetDescription, LLVMGetTargetFromTriple, LLVMGetTargetName,
    LLVMTarget,
};

use crate::util::{LLVMErrorString, LLVMString};

pub use llvm_sys::target_machine::{LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode};

pub use target_data::TargetData;
pub use target_machine::TargetMachine;

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
        self,
        triple: &CStr,
        cpu: &CStr,
        cpu_features: &CStr,
        opt_level: LLVMCodeGenOptLevel,
        reloc_mode: LLVMRelocMode,
        code_model: LLVMCodeModel,
    ) -> TargetMachine {
        unsafe {
            TargetMachine::from_raw(LLVMCreateTargetMachine(
                self.ptr,
                triple.as_ptr(),
                cpu.as_ptr(),
                cpu_features.as_ptr(),
                opt_level,
                reloc_mode,
                code_model,
            ))
        }
    }

    pub fn description(self) -> &'static CStr {
        unsafe { CStr::from_ptr(LLVMGetTargetDescription(self.ptr)) }
    }

    pub fn name(self) -> &'static CStr {
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
