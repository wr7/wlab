use llvm_sys::target_machine::{
    LLVMCreateTargetDataLayout, LLVMDisposeTargetMachine, LLVMGetTargetMachineCPU,
    LLVMGetTargetMachineFeatureString, LLVMGetTargetMachineTarget, LLVMGetTargetMachineTriple,
    LLVMOpaqueTargetMachine,
};

use crate::util::LLVMString;

use super::{Target, TargetData};

#[repr(transparent)]
pub struct TargetMachine {
    ptr: *mut LLVMOpaqueTargetMachine,
}

impl TargetMachine {
    pub unsafe fn from_raw(ptr: *mut LLVMOpaqueTargetMachine) -> Self {
        Self { ptr }
    }

    pub fn raw(&self) -> *mut LLVMOpaqueTargetMachine {
        self.ptr
    }

    pub fn target(&self) -> Target {
        unsafe { Target::from_raw(LLVMGetTargetMachineTarget(self.ptr)) }
    }

    pub fn create_target_data(&self) -> TargetData {
        unsafe { TargetData::from_raw(LLVMCreateTargetDataLayout(self.ptr)) }
    }

    pub fn cpu(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMGetTargetMachineCPU(self.ptr)) }
    }

    pub fn cpu_features(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMGetTargetMachineFeatureString(self.ptr)) }
    }

    pub fn target_triple(&self) -> LLVMString {
        unsafe { LLVMString::from_raw(LLVMGetTargetMachineTriple(self.ptr)) }
    }
}

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe { LLVMDisposeTargetMachine(self.ptr) }
    }
}
