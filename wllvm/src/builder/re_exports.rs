use crate::util::wrap_c_enum;
use llvm_sys::LLVMIntPredicate;

wrap_c_enum! {
    pub enum IntPredicate: LLVMIntPredicate {
        LLVMIntEQ => EQ = 32,
        LLVMIntNE => NE = 33,
        LLVMIntUGT => UGT = 34,
        LLVMIntUGE => UGE = 35,
        LLVMIntULT => ULT = 36,
        LLVMIntULE => ULE = 37,
        LLVMIntSGT => SGT = 38,
        LLVMIntSGE => SGE = 39,
        LLVMIntSLT => SLT = 40,
        LLVMIntSLE => SLE = 41,
    }
}
