use std::{borrow::Borrow, fmt::Display, ops::Range};

use crate::logic::asm::{
    codegen::{AsmCodeGenerationContext, AsmCodeGenerationError},
    expressions::{AsParsedLogicArgument, LogicArgument, ParsedLogicArgument},
};

#[derive(Debug, Clone, Eq, Ord)]
pub struct ScriptLocation {
    pub offset: usize,
}

impl PartialEq for ScriptLocation {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
    }
}

impl PartialOrd for ScriptLocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.offset.cmp(&other.offset))
    }
}

impl Display for ScriptLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("offset {}", self.offset))
    }
}

impl Default for ScriptLocation {
    fn default() -> Self {
        Self {
            offset: Default::default(),
        }
    }
}

pub type ScriptLocationRange = Range<ScriptLocation>;

pub fn location_range(start: usize, end: usize) -> ScriptLocationRange {
    ScriptLocation { offset: start }..ScriptLocation { offset: end }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithLocation<T> {
    pub value: T,
    pub location: ScriptLocationRange,
}

impl<T> WithLocation<T> {
    pub fn new(value: T, location: ScriptLocationRange) -> Self {
        Self { value, location }
    }
}

impl<T> Borrow<T> for WithLocation<T> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

impl<T: LogicArgument> LogicArgument for WithLocation<T> {
    fn value_eq(&self, other: &Self) -> bool {
        self.value.value_eq(&other.value)
    }

    fn try_parse(
        &self,
        context: &AsmCodeGenerationContext,
    ) -> Result<ParsedLogicArgument, AsmCodeGenerationError> {
        self.value.try_parse(context)
    }
}

impl<T: AsParsedLogicArgument> AsParsedLogicArgument for WithLocation<T> {
    fn as_parsed(&self) -> &ParsedLogicArgument {
        self.value.as_parsed()
    }
}

pub trait Locatable
where
    Self: Sized,
{
    fn with_location(self, location: ScriptLocationRange) -> WithLocation<Self> {
        WithLocation::new(self, location)
    }
}

impl<T> Locatable for T where T: Sized {}
