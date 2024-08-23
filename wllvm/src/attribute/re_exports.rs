use std::ptr;

use crate::Type;

pub(super) enum UnpackedAttrKind<'ctx> {
    Enum(u32),
    Type(u32, Type<'ctx>),
    Int(u32, u64),
}

macro_rules! define_attr_kind {
    {
        $(#[$attr:meta])*
        $vis:vis enum AttrKind<'ctx> {
            Enum {
                $($enum_name:ident = $enum_num:literal),* $(,)?
            },
            Type {
                $($type_name:ident = $type_num:literal),* $(,)?
            },
            Int {
                $($int_name:ident = $int_num:literal),* $(,)?
            } $(,)?
        }
    } => {
        $(#[$attr])*
        $vis enum AttrKind<'ctx> {
            $($enum_name() = $enum_num,)*
            $($type_name(Type<'ctx>) = $type_num,)*
            $($int_name(u64) = $int_num,)*
        }

        impl<'ctx> AttrKind<'ctx> {
            pub(super) fn unpack(self) -> UnpackedAttrKind<'ctx> {
                match self {
                    $(Self::$enum_name() => UnpackedAttrKind::Enum(self.id()),)*
                    $(Self::$type_name(ty) => UnpackedAttrKind::Type(self.id(), ty),)*
                    $(Self::$int_name(int) => UnpackedAttrKind::Int(self.id(), int),)*
                }
            }
        }
    };
}

impl<'ctx> AttrKind<'ctx> {
    fn id(&self) -> u32 {
        // SAFETY: see https://doc.rust-lang.org/reference/type-layout.html#primitive-representation-of-enums-with-fields
        unsafe { *ptr::from_ref(self).cast::<u32>() }
    }
}

define_attr_kind! {
    /// A regular, non-string attribute
    #[derive(Clone, Copy)]
    #[repr(u32)]
    pub enum AttrKind<'ctx> {
        Enum {
            AllocAlign = 1,
            AllocatedPointer = 2,
            AlwaysInline = 3,
            Builtin = 4,
            Cold = 5,
            Convergent = 6,
            CoroDestroyOnlyWhenComplete = 7,
            DeadOnUnwind = 8,
            DisableSanitizerInstrumentation = 9,
            FnRetThunkExtern = 10,
            Hot = 11,
            ImmArg = 12,
            InReg = 13,
            InlineHint = 14,
            JumpTable = 15,
            MinSize = 16,
            MustProgress = 17,
            Naked = 18,
            Nest = 19,
            NoAlias = 20,
            NoBuiltin = 21,
            NoCallback = 22,
            NoCapture = 23,
            NoCfCheck = 24,
            NoDuplicate = 25,
            NoFree = 26,
            NoImplicitFloat = 27,
            NoInline = 28,
            NoMerge = 29,
            NoProfile = 30,
            NoRecurse = 31,
            NoRedZone = 32,
            NoReturn = 33,
            NoSanitizeBounds = 34,
            NoSanitizeCoverage = 35,
            NoSync = 36,
            NoUndef = 37,
            NoUnwind = 38,
            NonLazyBind = 39,
            NonNull = 40,
            NullPointerIsValid = 41,
            OptForFuzzing = 42,
            OptimizeForDebugging = 43,
            OptimizeForSize = 44,
            OptimizeNone = 45,
            PresplitCoroutine = 46,
            ReadNone = 47,
            ReadOnly = 48,
            Returned = 49,
            ReturnsTwice = 50,
            SExt = 51,
            SafeStack = 52,
            SanitizeAddress = 53,
            SanitizeHWAddress = 54,
            SanitizeMemTag = 55,
            SanitizeMemory = 56,
            SanitizeThread = 57,
            ShadowCallStack = 58,
            SkipProfile = 59,
            Speculatable = 60,
            SpeculativeLoadHardening = 61,
            StackProtect = 62,
            StackProtectReq = 63,
            StackProtectStrong = 64,
            StrictFP = 65,
            SwiftAsync = 66,
            SwiftError = 67,
            SwiftSelf = 68,
            WillReturn = 69,
            Writable = 70,
            WriteOnly = 71,
            ZExt = 72,
        },
        Type {
            ByRef = 73,
            ByVal = 74,
            ElementType = 75,
            InAlloca = 76,
            Preallocated = 77,
            StructRet = 78,
        },
        Int {
            Alignment = 79,
            AllocKind = 80,
            AllocSize = 81,
            Dereferenceable = 82,
            DereferenceableOrNull = 83,
            Memory = 84,
            NoFPClass = 85,
            StackAlignment = 86,
            UWTable = 87,
            VScaleRange = 88,
        },
    }
}
