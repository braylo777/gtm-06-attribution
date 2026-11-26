//! Stage 6 — Multi-Touch Attribution & RevOps.
//!
//! * `attribution` — first-touch, last-touch, linear, time-decay and U-shaped (40/20/40)
//!   credit per channel from the same journeys, plus CAC and ROAS per model when spend is given.
//! * `funnel`      — stage-to-stage conversion, the biggest leak, and lost revenue vs benchmark.
//! * `cohorts`     — signup-month retention triangle (logo and revenue).

mod api;
pub mod attribution;
pub mod cohorts;
pub mod funnel;
pub mod util;

pub use api::dispatch;
use serde_json::Value;

pub const OPS: &[&str] = &["attribute", "funnel_leakage", "cohort_retention"];

pub fn handle(op: &str, i: &Value) -> Result<Value, String> {
    match op {
        "attribute" => attribution::attribute_op(i),
        "funnel_leakage" => funnel::leakage_op(i),
        "cohort_retention" => cohorts::cohort_op(i),
        _ => Err(format!("unknown op `{op}`; known: {}", OPS.join(", "))),
    }
}
