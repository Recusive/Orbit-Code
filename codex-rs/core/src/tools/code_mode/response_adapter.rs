//! Adapters for code-mode response payloads.
//!
//! TODO(upstream-0.118.0): replace this local JSON conversion helper with the
//! upstream typed `orbit_code_code_mode` adapter once that crate lands here.

use orbit_code_protocol::models::FunctionCallOutputContentItem;
use serde_json::Value as JsonValue;

pub(super) fn into_function_call_output_content_items(
    items: Vec<JsonValue>,
) -> Vec<FunctionCallOutputContentItem> {
    items.into_iter()
        .filter_map(|item| serde_json::from_value(item).ok())
        .collect()
}
