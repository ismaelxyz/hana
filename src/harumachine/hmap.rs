//! Provides a hashmap of String-NativeValue

use super::value::Value;
use super::string::HaruString;
use std::collections::HashMap;

/// A hashmap of String-NativeValue
pub type HaruHashMap = HashMap<HaruString, Value>;
