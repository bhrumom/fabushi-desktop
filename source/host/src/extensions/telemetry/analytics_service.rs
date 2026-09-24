use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ProductAnalyticsEvent {
    pub name: String,
    pub properties: BTreeMap<String, Value>,
}

pub fn product_analytics_event(name: &str, properties: &Value) -> ProductAnalyticsEvent {
    let mut clean = BTreeMap::new();
    if let Some(object) = properties.as_object() {
        for (key, value) in object {
            if matches!(
                value,
                Value::Bool(_) | Value::Number(_) | Value::String(_)
            ) {
                clean.insert(key.clone(), value.clone());
            }
        }
    }
    ProductAnalyticsEvent {
        name: name.to_string(),
        properties: clean,
    }
}
