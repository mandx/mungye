mod json;
mod yaml;

use std::fmt;

#[derive(Debug, Copy, Clone)]
pub(crate) enum ArrayMergeBehavior {
    Replace,
    Concat,
    // TODO: Add `Extend`: Like concat, but will remove duplicates.
    // Extend,
    // TODO: Add `Zip`: Merge together elements in the same indices.
    //       For arrays of unequal lengths, simply use the value
    //       that's already there.
    // Zip,
}

impl Default for ArrayMergeBehavior {
    fn default() -> Self {
        ArrayMergeBehavior::Replace
    }
}

impl std::str::FromStr for ArrayMergeBehavior {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<ArrayMergeBehavior, &'static str> {
        match s {
            "Replace" | "replace" => Ok(Self::Replace),
            "Concat" | "concat" => Ok(Self::Concat),
            _ => Err("Not `replace` | `concat`"),
        }
    }
}

impl fmt::Display for ArrayMergeBehavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Replace => write!(f, "replace"),
            Self::Concat => write!(f, "concat"),
        }
    }
}

// TODO: Find better names for `current` and `next` variables/symbols.

pub(crate) trait DeepMerge {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self;
}
