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
