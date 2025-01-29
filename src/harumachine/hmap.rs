//! Provides a hashmap of String-NativeValue

use super::string::HaruString;
use super::value::Value;
use std::collections::HashMap;

/// A hashmap of String-NativeValue
pub type HaruHashMap = HashMap<HaruString, Value>;
