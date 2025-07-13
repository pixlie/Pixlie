pub use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub trait ToolParameterSchema: Serialize + for<'de> Deserialize<'de> + TS + JsonSchema {}

pub trait ToolResponseSchema: Serialize + for<'de> Deserialize<'de> + TS + JsonSchema {}
