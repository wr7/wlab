use std::ops::BitOr;

use llvm_sys::debuginfo::LLVMDIFlags;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DIFlags(LLVMDIFlags);

#[allow(non_upper_case_globals)]
impl DIFlags {
    pub const Zero: Self = Self(0);
    pub const Private: Self = Self(1);
    pub const Protected: Self = Self(2);
    pub const Public: Self = Self(3);
    pub const FwdDecl: Self = Self(1 << 2);
    pub const AppleBlock: Self = Self(1 << 3);
    pub const ReservedBit4: Self = Self(1 << 4);
    pub const Virtual: Self = Self(1 << 5);
    pub const Artificial: Self = Self(1 << 6);
    pub const Explicit: Self = Self(1 << 7);
    pub const Prototyped: Self = Self(1 << 8);
    pub const ObjcClassComplete: Self = Self(1 << 9);
    pub const ObjectPointer: Self = Self(1 << 10);
    pub const Vector: Self = Self(1 << 11);
    pub const StaticMember: Self = Self(1 << 12);
    pub const LValueReference: Self = Self(1 << 13);
    pub const RValueReference: Self = Self(1 << 14);
    pub const Reserved: Self = Self(1 << 15);
    pub const SingleInheritance: Self = Self(1 << 16);
    pub const MultipleInheritance: Self = Self(2 << 16);
    pub const VirtualInheritance: Self = Self(3 << 16);
    pub const IntroducedVirtual: Self = Self(1 << 18);
    pub const BitField: Self = Self(1 << 19);
    pub const NoReturn: Self = Self(1 << 20);
    pub const TypePassByValue: Self = Self(1 << 22);
    pub const TypePassByReference: Self = Self(1 << 23);
    pub const EnumClass: Self = Self(1 << 24);
    pub const Thunk: Self = Self(1 << 25);
    pub const NonTrivial: Self = Self(1 << 26);
    pub const Bigendian: Self = Self(1 << 27);
    pub const LittleEndian: Self = Self(1 << 28);
    pub const IndirectVirtualBase: Self = Self((1 << 2) | (1 << 5));
    pub const Accessibility: Self = Self(Self::Protected.0 | Self::Private.0 | Self::Public.0);
    pub const PtrToMemberRep: Self =
        Self(Self::SingleInheritance.0 | Self::MultipleInheritance.0 | Self::VirtualInheritance.0);
}

#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum TypeEncoding {
    address = 0x01,       // v2 DWARF
    boolean = 0x02,       // v2 DWARF
    complex_float = 0x03, // v2 DWARF
    float = 0x04,         // v2 DWARF
    signed = 0x05,        // v2 DWARF
    signed_char = 0x06,   // v2 DWARF
    unsigned = 0x07,      // v2 DWARF
    unsigned_char = 0x08, // v2 DWARF
    // New in DWARF v3:
    imaginary_float = 0x09, // v3 DWARF
    packed_decimal = 0x0a,  // v3 DWARF
    numeric_string = 0x0b,  // v3 DWARF
    edited = 0x0c,          // v3 DWARF
    signed_fixed = 0x0d,    // v3 DWARF
    unsigned_fixed = 0x0e,  // v3 DWARF
    decimal_float = 0x0f,   // v3 DWARF
    // New in DWARF v4:
    UTF = 0x10, // v4 DWARF
    // New in DWARF v5:
    UCS = 0x11,   // v5 DWARF
    ASCII = 0x12, // v5 DWARF

    // The version numbers of all vendor extensions >0x80 were guessed.
    HP_complex_float = 0x81,      // v2 HP
    HP_float128 = 0x82,           // v2 HP
    HP_complex_float128 = 0x83,   // v2 HP
    HP_floathpintel = 0x84,       // v2 HP
    HP_imaginary_float90 = 0x85,  // v2 HP
    HP_imaginary_float128 = 0x86, // v2 HP
}

impl BitOr for DIFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl From<DIFlags> for LLVMDIFlags {
    fn from(value: DIFlags) -> Self {
        value.0
    }
}

// Copied from `llvm/include/llvm/BinaryFormat/Dwarf.def`
#[allow(non_camel_case_types)]
#[repr(u64)]
pub enum DwarfOperator {
    // DWARF Expression operators.
    addr = 0x03,
    deref = 0x06,
    const1u = 0x08,
    const1s = 0x09,
    const2u = 0x0a,
    const2s = 0x0b,
    const4u = 0x0c,
    const4s = 0x0d,
    const8u = 0x0e,
    const8s = 0x0f,
    constu = 0x10,
    consts = 0x11,
    dup = 0x12,
    drop = 0x13,
    over = 0x14,
    pick = 0x15,
    swap = 0x16,
    rot = 0x17,
    xderef = 0x18,
    abs = 0x19,
    and = 0x1a,
    div = 0x1b,
    minus = 0x1c,
    mod_ = 0x1d,
    mul = 0x1e,
    neg = 0x1f,
    not = 0x20,
    or = 0x21,
    plus = 0x22,
    plus_uconst = 0x23,
    shl = 0x24,
    shr = 0x25,
    shra = 0x26,
    xor = 0x27,
    bra = 0x28,
    eq = 0x29,
    ge = 0x2a,
    gt = 0x2b,
    le = 0x2c,
    lt = 0x2d,
    ne = 0x2e,
    skip = 0x2f,
    lit0 = 0x30,
    lit1 = 0x31,
    lit2 = 0x32,
    lit3 = 0x33,
    lit4 = 0x34,
    lit5 = 0x35,
    lit6 = 0x36,
    lit7 = 0x37,
    lit8 = 0x38,
    lit9 = 0x39,
    lit10 = 0x3a,
    lit11 = 0x3b,
    lit12 = 0x3c,
    lit13 = 0x3d,
    lit14 = 0x3e,
    lit15 = 0x3f,
    lit16 = 0x40,
    lit17 = 0x41,
    lit18 = 0x42,
    lit19 = 0x43,
    lit20 = 0x44,
    lit21 = 0x45,
    lit22 = 0x46,
    lit23 = 0x47,
    lit24 = 0x48,
    lit25 = 0x49,
    lit26 = 0x4a,
    lit27 = 0x4b,
    lit28 = 0x4c,
    lit29 = 0x4d,
    lit30 = 0x4e,
    lit31 = 0x4f,
    reg0 = 0x50,
    reg1 = 0x51,
    reg2 = 0x52,
    reg3 = 0x53,
    reg4 = 0x54,
    reg5 = 0x55,
    reg6 = 0x56,
    reg7 = 0x57,
    reg8 = 0x58,
    reg9 = 0x59,
    reg10 = 0x5a,
    reg11 = 0x5b,
    reg12 = 0x5c,
    reg13 = 0x5d,
    reg14 = 0x5e,
    reg15 = 0x5f,
    reg16 = 0x60,
    reg17 = 0x61,
    reg18 = 0x62,
    reg19 = 0x63,
    reg20 = 0x64,
    reg21 = 0x65,
    reg22 = 0x66,
    reg23 = 0x67,
    reg24 = 0x68,
    reg25 = 0x69,
    reg26 = 0x6a,
    reg27 = 0x6b,
    reg28 = 0x6c,
    reg29 = 0x6d,
    reg30 = 0x6e,
    reg31 = 0x6f,
    breg0 = 0x70,
    breg1 = 0x71,
    breg2 = 0x72,
    breg3 = 0x73,
    breg4 = 0x74,
    breg5 = 0x75,
    breg6 = 0x76,
    breg7 = 0x77,
    breg8 = 0x78,
    breg9 = 0x79,
    breg10 = 0x7a,
    breg11 = 0x7b,
    breg12 = 0x7c,
    breg13 = 0x7d,
    breg14 = 0x7e,
    breg15 = 0x7f,
    breg16 = 0x80,
    breg17 = 0x81,
    breg18 = 0x82,
    breg19 = 0x83,
    breg20 = 0x84,
    breg21 = 0x85,
    breg22 = 0x86,
    breg23 = 0x87,
    breg24 = 0x88,
    breg25 = 0x89,
    breg26 = 0x8a,
    breg27 = 0x8b,
    breg28 = 0x8c,
    breg29 = 0x8d,
    breg30 = 0x8e,
    breg31 = 0x8f,
    regx = 0x90,
    fbreg = 0x91,
    bregx = 0x92,
    piece = 0x93,
    deref_size = 0x94,
    xderef_size = 0x95,
    nop = 0x96,
    // New in DWARF v3:
    push_object_address = 0x97,
    call2 = 0x98,
    call4 = 0x99,
    call_ref = 0x9a,
    form_tls_address = 0x9b,
    call_frame_cfa = 0x9c,
    bit_piece = 0x9d,
    // New in DWARF v4:
    implicit_value = 0x9e,
    stack_value = 0x9f,
    // New in DWARF v5:
    implicit_pointer = 0xa0,
    addrx = 0xa1,
    constx = 0xa2,
    entry_value = 0xa3,
    const_type = 0xa4,
    regval_type = 0xa5,
    deref_type = 0xa6,
    xderef_type = 0xa7,
    convert = 0xa8,
    reinterpret = 0xa9,
    // Vendor extensions:
    // Extensions for GNU-style thread-local storage.
    GNU_push_tls_address = 0xe0,
    // Conflicting:
    // HANDLE_DW_OP(0xe0, HP_unknown, 0, HP)
    HP_is_value = 0xe1,
    HP_fltconst4 = 0xe2,
    HP_fltconst8 = 0xe3,
    HP_mod_range = 0xe4,
    HP_unmod_range = 0xe5,
    HP_tls = 0xe6,
    INTEL_bit_piece = 0xe8,

    // Extensions for WebAssembly.
    WASM_location = 0xed,
    WASM_location_int = 0xee,
    // Historic and not implemented in LLVM.
    APPLE_uninit = 0xf0,
    // The GNU entry value extension.
    GNU_entry_value = 0xf3,
    PGI_omp_thread_num = 0xf8,
    // Extensions for Fission proposal.
    GNU_addr_index = 0xfb,
    GNU_const_index = 0xfc,

    // DW_OP_LLVM_user has two operands:
    //   (1) An unsigned LEB128 "LLVM Vendor Extension Opcode".
    //   (2) Zero or more literal operands, the number and type of which are
    //       implied by the opcode (1).
    // DW_OP_LLVM_user acts as an extension multiplexer, opening up the encoding
    // space to accommodate an infinite number of extensions. This better reflects
    // the de-facto permanent allocation of extensions.
    LLVM_user = 0xe9,
}
