use super::{Column, SLOTS};

// titles an examine strings. same slot index as nav, different table.
pub type TitlePage = Column<Option<String>, SLOTS>;
