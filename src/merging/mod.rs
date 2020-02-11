mod json;
mod yaml;

use strum_macros::{Display, EnumString, EnumVariantNames};

#[derive(Debug, Copy, Clone, Display, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab-case")]
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

pub(crate) trait DeepMerge {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self;
}
