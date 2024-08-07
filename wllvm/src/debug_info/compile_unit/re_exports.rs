use llvm_sys::debuginfo::{LLVMDWARFEmissionKind, LLVMDWARFSourceLanguage};

use crate::util::wrap_c_enum;

wrap_c_enum! {
    #[allow(non_camel_case_types)]
    pub enum SourceLanguage: LLVMDWARFSourceLanguage {
        LLVMDWARFSourceLanguageC89 => C89,
        LLVMDWARFSourceLanguageC => C,
        LLVMDWARFSourceLanguageAda83 => Ada83,
        LLVMDWARFSourceLanguageC_plus_plus => C_plus_plus,
        LLVMDWARFSourceLanguageCobol74 => Cobol74,
        LLVMDWARFSourceLanguageCobol85 => Cobol85,
        LLVMDWARFSourceLanguageFortran77 => Fortran77,
        LLVMDWARFSourceLanguageFortran90 => Fortran90,
        LLVMDWARFSourceLanguagePascal83 => Pascal83,
        LLVMDWARFSourceLanguageModula2 => Modula2,
        // New in DWARF v3:
        LLVMDWARFSourceLanguageJava => Java,
        LLVMDWARFSourceLanguageC99 => C99,
        LLVMDWARFSourceLanguageAda95 => Ada95,
        LLVMDWARFSourceLanguageFortran95 => Fortran95,
        LLVMDWARFSourceLanguagePLI => PLI,
        LLVMDWARFSourceLanguageObjC => ObjC,
        LLVMDWARFSourceLanguageObjC_plus_plus => ObjC_plus_plus,
        LLVMDWARFSourceLanguageUPC => UPC,
        LLVMDWARFSourceLanguageD => D,
        // New in DWARF v4:
        LLVMDWARFSourceLanguagePython => Python,
        // New in DWARF v5:
        LLVMDWARFSourceLanguageOpenCL => OpenCL,
        LLVMDWARFSourceLanguageGo => Go,
        LLVMDWARFSourceLanguageModula3 => Modula3,
        LLVMDWARFSourceLanguageHaskell => Haskell,
        LLVMDWARFSourceLanguageC_plus_plus_03 => C_plus_plus_03,
        LLVMDWARFSourceLanguageC_plus_plus_11 => C_plus_plus_11,
        LLVMDWARFSourceLanguageOCaml => OCaml,
        LLVMDWARFSourceLanguageRust => Rust,
        LLVMDWARFSourceLanguageC11 => C11,
        LLVMDWARFSourceLanguageSwift => Swift,
        LLVMDWARFSourceLanguageJulia => Julia,
        LLVMDWARFSourceLanguageDylan => Dylan,
        LLVMDWARFSourceLanguageC_plus_plus_14 => C_plus_plus_14,
        LLVMDWARFSourceLanguageFortran03 => Fortran03,
        LLVMDWARFSourceLanguageFortran08 => Fortran08,
        LLVMDWARFSourceLanguageRenderScript => RenderScript,
        LLVMDWARFSourceLanguageBLISS => BLISS,
        LLVMDWARFSourceLanguageKotlin => Kotlin,
        LLVMDWARFSourceLanguageZig => Zig,
        LLVMDWARFSourceLanguageCrystal => Crystal,
        LLVMDWARFSourceLanguageC_plus_plus_17 => C_plus_plus_17,
        LLVMDWARFSourceLanguageC_plus_plus_20 => C_plus_plus_20,
        LLVMDWARFSourceLanguageC17 => C17,
        LLVMDWARFSourceLanguageFortran18 => Fortran18,
        LLVMDWARFSourceLanguageAda2005 => Ada2005,
        LLVMDWARFSourceLanguageAda2012 => Ada2012,
        LLVMDWARFSourceLanguageMojo => Mojo,
        // Vendor extensions:
        LLVMDWARFSourceLanguageMips_Assembler => Mips_Assembler,
        LLVMDWARFSourceLanguageGOOGLE_RenderScript => GOOGLE_RenderScript,
        LLVMDWARFSourceLanguageBORLAND_Delphi => BORLAND_Delphi,
    }
}

wrap_c_enum! {
    pub enum EmissionKind: LLVMDWARFEmissionKind {
        LLVMDWARFEmissionKindNone => None = 0,
        LLVMDWARFEmissionKindFull => Full,
        LLVMDWARFEmissionKindLineTablesOnly => LineTablesOnly,
    }
}
