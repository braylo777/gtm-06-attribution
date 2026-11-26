//! Funnel leakage: conversion between adjacent stages, the stage with the most lost volume vs
//! benchmark, and the revenue that gap costs at the final-stage value.
use crate::util::*;
use serde_json::{json, Value};

pub fn leakage_op(i: &Value) -> R<Value> {
    let stages = arr(i, "stages")?;
    if stages.len() < 2 {
        return Err("need at least 2 stages".into());
    }
    let deal = f_or(i, "avg_deal_value", 0.0);
    let mut rows = vec![];
    let mut worst: Option<(String, f64)> = None;
    let counts: Vec<f64> = stages.iter().map(|s| f(s, "count")).collect::<R<_>>()?;
    for k in 1..stages.len() {
        let (a, b) = (counts[k - 1], counts[k]);
        if b > a {
            return Err(format!("stage {} has more than the stage before it", s_or(&stages[k], "name", "?")));
        }
        let rate = if a > 0.0 { b / a } else { 0.0 };
        let bench = stages[k].get("benchmark_rate").and_then(Value::as_f64);
        // extra final-stage wins if this step hit benchmark and later steps stayed the same
        let downstream: f64 = (k + 1..stages.len()).map(|j| if counts[j - 1] > 0.0 { counts[j] / counts[j - 1] } else { 0.0 }).product();
        let gain = bench.filter(|bm| *bm > rate).map(|bm| (bm - rate) * a * downstream).unwrap_or(0.0);
        if gain > worst.as_ref().map_or(0.0, |w| w.1) {
            worst = Some((format!("{} → {}", s_or(&stages[k - 1], "name", "?"), s_or(&stages[k], "name", "?")), gain));
        }
        rows.push(json!({"step": format!("{} → {}", s_or(&stages[k - 1], "name", "?"), s_or(&stages[k], "name", "?")),
            "rate_pct": rd(rate * 100.0, 1), "benchmark_pct": bench.map(|b| rd(b * 100.0, 1)), "lost": a - b,
            "extra_wins_at_benchmark": rd(gain, 1), "revenue_gap": rd(gain * deal, 0)}));
    }
    let overall = counts.last().unwrap() / counts[0].max(1e-9);
    Ok(json!({"steps": rows, "overall_rate_pct": rd(overall * 100.0, 2),
        "biggest_leak": worst.map(|w| json!({"step": w.0, "extra_wins": rd(w.1, 1), "revenue": rd(w.1 * deal, 0)}))}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn finds_leak() {
        let i = json!({"avg_deal_value":1000,"stages":[{"name":"a","count":100},{"name":"b","count":20,"benchmark_rate":0.4},{"name":"c","count":10,"benchmark_rate":0.5}]});
        let r = leakage_op(&i).unwrap();
        assert_eq!(r["biggest_leak"]["step"], "a → b");
        assert_eq!(r["biggest_leak"]["revenue"], 10000.0);
        assert!(leakage_op(&json!({"stages":[{"count":1},{"count":2}]})).is_err());
    }
}
