use core::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum VmaType {
    Code,
    Heap,
    Stack,
}

impl Display for VmaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Code => write!(f, "Code"),
            Self::Heap => write!(f, "Heap"),
            Self::Stack => write!(f, "Stack")
        }
    }
}

/// Virtual Memory Area (VMA)
pub struct VMA {
    pub start: u64,
    pub end: u64,
    pub typ: VmaType,
}

impl VMA {
    /// Create a new VMA with a start and end address and a given type.
    pub fn new(start: u64, end: u64, typ: VmaType) -> Self {
        VMA { start, end, typ }
    }

    /// Check if this VMA overlaps with another one.
    pub fn overlaps(&self, other: &VMA) -> bool {
        (self.end > other.start && self.end < other.end) || (other.end > self.start && other.end < self.end)
    }
}

impl fmt::Debug for VMA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VMA {{ start: 0x{:016x}, end: {:#016x}, type: {:?} }}", self.start, self.end, self.typ)
    }
}