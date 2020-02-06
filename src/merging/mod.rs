mod json;
mod yaml;

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

// TODO: Find better names for `current` and `next` variables/symbols.

pub(crate) trait DeepMerge {
    fn deep_merge(self, with: Self, array_merge: ArrayMergeBehavior) -> Self;
}
