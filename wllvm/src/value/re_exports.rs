use llvm_sys::LLVMLinkage;

use crate::util::wrap_c_enum;

wrap_c_enum! {
    #[derive(Default)]
    pub enum Linkage: LLVMLinkage {
        #[default]
        LLVMExternalLinkage => External = 0,
        LLVMAvailableExternallyLinkage => AvailableExternally = 1,
        LLVMLinkOnceAnyLinkage => LinkOnceAny = 2,
        LLVMLinkOnceODRLinkage => LinkOnceODR = 3,
        LLVMLinkOnceODRAutoHideLinkage => LinkOnceODRAutoHide = 4,
        LLVMWeakAnyLinkage => WeakAny = 5,
        LLVMWeakODRLinkage => WeakODR = 6,
        LLVMAppendingLinkage => Appending = 7,
        LLVMInternalLinkage => Internal = 8,
        LLVMPrivateLinkage => Private = 9,
        LLVMDLLImportLinkage => DLLImport = 10,
        LLVMDLLExportLinkage => DLLExport = 11,
        LLVMExternalWeakLinkage => ExternalWeak = 12,
        LLVMGhostLinkage => Ghost = 13,
        LLVMCommonLinkage => Common = 14,
        LLVMLinkerPrivateLinkage => LinkerPrivate = 15,
        LLVMLinkerPrivateWeakLinkage => LinkerPrivateWeak = 16,
    }
}
