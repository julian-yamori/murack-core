pub mod filter_target;
pub use filter_target::FilterTarget;

pub mod range_artwork;
pub use range_artwork::ArtworkFilterRange;

pub mod range_bool;
pub use range_bool::BoolFilterRange;

pub mod range_date;
pub use range_date::DateFilterRange;

pub mod range_group;
pub use range_group::GroupOperand;

pub mod range_int;
pub use range_int::IntFilterRange;

pub mod range_string;
pub use range_string::StringFilterRange;

pub mod range_tags;
pub use range_tags::TagsFilterRange;

#[cfg(test)]
mod tests;

/// 最上位フィルタ (GUI では、最上位は FilterGroup である必要がある)
pub type RootFilter = FilterTarget;
