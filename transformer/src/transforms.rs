use serde::Serialize;

/// A transformed signal value, ready to serve.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SignalValue {
    Float(f64),
    Bool(bool),
    Text(String),
}

/// Apply the named transform to a value.
/// Unknown names and type mismatches act as passthrough.
pub fn apply(name: &str, value: SignalValue) -> SignalValue {
    match (name, value) {
        ("kmh_to_mph", SignalValue::Float(v)) => SignalValue::Float(v * 0.621371),
        (_, v) => v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kmh_to_mph_converts() {
        match apply("kmh_to_mph", SignalValue::Float(100.0)) {
            SignalValue::Float(v) => assert!((v - 62.1371).abs() < 1e-6),
            _ => panic!("expected float"),
        }
    }

    #[test]
    fn passthrough_keeps_value() {
        match apply("passthrough", SignalValue::Bool(true)) {
            SignalValue::Bool(b) => assert!(b),
            _ => panic!("expected bool"),
        }
    }

    #[test]
    fn unknown_transform_is_passthrough() {
        match apply("no_such_transform", SignalValue::Float(7.0)) {
            SignalValue::Float(v) => assert_eq!(v, 7.0),
            _ => panic!("expected float"),
        }
    }
}
