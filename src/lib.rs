//! OpenTelemetry integration for Langfuse observability.
//!
//! This crate provides a bridge between OpenTelemetry and Langfuse,
//! allowing you to export OpenTelemetry traces to the Langfuse platform
//! for LLM observability and monitoring.

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
