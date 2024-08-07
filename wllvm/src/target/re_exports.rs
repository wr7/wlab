use llvm_sys::target_machine::{LLVMCodeGenOptLevel, LLVMCodeModel, LLVMRelocMode};

use crate::util::wrap_c_enum;

wrap_c_enum! {
    pub enum OptLevel: LLVMCodeGenOptLevel {
        LLVMCodeGenLevelNone => None = 0,
        LLVMCodeGenLevelLess => Less = 1,
        LLVMCodeGenLevelDefault => Default = 2,
        LLVMCodeGenLevelAggressive => Aggressive = 3,
    }
}

wrap_c_enum! {
    #[allow(non_camel_case_types)]
    pub enum RelocMode: LLVMRelocMode {
        LLVMRelocDefault => Default = 0,
        LLVMRelocStatic => Static = 1,
        LLVMRelocPIC => PIC = 2,
        LLVMRelocDynamicNoPic => DynamicNoPic = 3,
        LLVMRelocROPI => ROPI = 4,
        LLVMRelocRWPI => RWPI = 5,
        LLVMRelocROPI_RWPI => ROPI_RWPI = 6,
    }
}

wrap_c_enum! {
    pub enum CodeModel: LLVMCodeModel {
        LLVMCodeModelDefault => Default = 0,
        LLVMCodeModelJITDefault => JITDefault = 1,
        LLVMCodeModelTiny => Tiny = 2,
        LLVMCodeModelSmall => Small = 3,
        LLVMCodeModelKernel => Kernel = 4,
        LLVMCodeModelMedium => Medium = 5,
        LLVMCodeModelLarge => Large = 6,
    }
}
