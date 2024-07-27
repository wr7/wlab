use std::{ffi::CStr, marker::PhantomData};

use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildAnd, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr,
        LLVMBuildExtractValue, LLVMBuildICmp, LLVMBuildMul, LLVMBuildNot, LLVMBuildOr,
        LLVMBuildPhi, LLVMBuildRet, LLVMBuildSDiv, LLVMBuildSub, LLVMBuildUDiv,
        LLVMBuildUnreachable, LLVMBuildXor, LLVMBuildZExt, LLVMCountStructElementTypes,
        LLVMDisposeBuilder, LLVMGetInsertBlock, LLVMPositionBuilderAtEnd,
    },
    LLVMBuilder, LLVMValue,
};

use crate::{
    basic_block::BasicBlock,
    type_::{FnType, IntType},
    value::{FnValue, IntValue, PhiValue, PtrValue, StructValue, Value},
    Type,
};

pub use llvm_sys::LLVMIntPredicate;

#[repr(transparent)]
pub struct Builder<'ctx> {
    ptr: *mut LLVMBuilder,
    _phantomdata: PhantomData<&'ctx LLVMBuilder>,
}

impl<'ctx> Builder<'ctx> {
    pub unsafe fn from_raw(ptr: *mut LLVMBuilder) -> Self {
        Self {
            ptr,
            _phantomdata: PhantomData,
        }
    }

    pub fn build_ptr_call(
        &self,
        fn_type: FnType<'ctx>,
        ptr: PtrValue<'ctx>,
        args: &[Value<'ctx>],
        name: &CStr,
    ) -> Value<'ctx> {
        let args_ptr = args.as_ptr().cast::<*mut LLVMValue>().cast_mut();

        unsafe {
            Value::from_raw(LLVMBuildCall2(
                self.ptr,
                fn_type.raw(),
                ptr.raw(),
                args_ptr,
                args.len() as u32,
                name.as_ptr(),
            ))
        }
    }

    pub fn build_fn_call(
        &self,
        fn_: FnValue<'ctx>,
        args: &[Value<'ctx>],
        name: &CStr,
    ) -> Value<'ctx> {
        let fn_type = fn_.type_();
        let args_ptr = args.as_ptr().cast::<*mut LLVMValue>().cast_mut();

        unsafe {
            Value::from_raw(LLVMBuildCall2(
                self.ptr,
                fn_type.raw(),
                fn_.raw(),
                args_ptr,
                args.len() as u32,
                name.as_ptr(),
            ))
        }
    }

    pub fn build_cond_br(
        &self,
        if_: IntValue<'ctx>,
        then: BasicBlock<'ctx>,
        else_: BasicBlock<'ctx>,
    ) {
        unsafe { LLVMBuildCondBr(self.ptr, if_.raw(), then.raw(), else_.raw()) };
    }

    pub fn build_br(&self, block: BasicBlock<'ctx>) {
        unsafe { LLVMBuildBr(self.ptr, block.raw()) };
    }

    pub fn build_icmp(
        &self,
        op: LLVMIntPredicate,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe {
            IntValue::from_raw(LLVMBuildICmp(
                self.ptr,
                op,
                lhs.raw(),
                rhs.raw(),
                name.as_ptr(),
            ))
        }
    }

    pub fn build_add(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildAdd(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_sub(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildSub(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_mul(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildMul(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_sdiv(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildSDiv(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_udiv(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildUDiv(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_and(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildAnd(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_or(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildOr(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_xor(
        &self,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildXor(self.ptr, lhs.raw(), rhs.raw(), name.as_ptr())) }
    }

    pub fn build_not(&self, val: IntValue<'ctx>, name: &CStr) -> IntValue<'ctx> {
        unsafe { IntValue::from_raw(LLVMBuildNot(self.ptr, val.raw(), name.as_ptr())) }
    }

    pub fn build_zext(
        &self,
        val: IntValue<'ctx>,
        target: IntType<'ctx>,
        name: &CStr,
    ) -> IntValue<'ctx> {
        unsafe {
            IntValue::from_raw(LLVMBuildZExt(
                self.ptr,
                val.raw(),
                target.raw(),
                name.as_ptr(),
            ))
        }
    }

    pub fn build_extract_value(
        &self,
        val: StructValue<'ctx>,
        idx: u32,
        name: &CStr,
    ) -> Option<Value<'ctx>> {
        let num_elements = unsafe { LLVMCountStructElementTypes(val.type_().raw()) };

        if idx >= num_elements {
            return None;
        }

        Some(unsafe {
            Value::from_raw(LLVMBuildExtractValue(
                self.ptr,
                val.raw(),
                idx,
                name.as_ptr(),
            ))
        })
    }

    pub fn build_phi(&self, type_: Type<'ctx>, name: &CStr) -> PhiValue<'ctx> {
        unsafe { PhiValue::from_raw(LLVMBuildPhi(self.ptr, type_.raw(), name.as_ptr())) }
    }

    pub fn build_ret(&self, val: Value<'ctx>) {
        unsafe { LLVMBuildRet(self.ptr, val.raw()) };
    }

    pub fn build_unreachable(&self) {
        unsafe { LLVMBuildUnreachable(self.ptr) };
    }

    pub fn position_at_end(&self, block: BasicBlock<'ctx>) {
        unsafe { LLVMPositionBuilderAtEnd(self.ptr, block.raw()) }
    }

    pub fn current_block(&self) -> Option<BasicBlock<'ctx>> {
        unsafe {
            let raw = LLVMGetInsertBlock(self.ptr);

            if !raw.is_null() {
                Some(BasicBlock::from_raw(raw))
            } else {
                None
            }
        }
    }
}

impl<'ctx> Drop for Builder<'ctx> {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.ptr) }
    }
}
