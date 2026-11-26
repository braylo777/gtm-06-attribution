//! Cohort retention triangle. Each customer has a signup month and MRR per month index.
//! Logo retention = share of the cohort with MRR > 0; revenue retention = MRR vs month 0.
use crate::util::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn cohort_op(i: &Value) -> R<Value> {
    let now = f(i, "current_month")? as i64;
    let mut by: BTreeMap<i64, Vec<&Vec<Value>>> = BTreeMap::new();
    for c in arr(i, "customers")? {
        let m = f(c, "signup_month")? as i64;
        if m > now {
            return Err("signup_month after current_month".into());
        }
        by.entry(m).or_default().push(arr(c, "mrr_by_month")?);
    }
    let mut rows = vec![];
    for (m, cs) in &by {
        let age = (now - m) as usize;
        let n = cs.len() as f64;
        let m0: f64 = cs.iter().map(|v| v.first().and_then(Value::as_f64).unwrap_or(0.0)).sum();
        let mut logo = vec![];
        let mut rev = vec![];
        for k in 0..=age {
            let alive = cs.iter().filter(|v| v.get(k).and_then(Value::as_f64).unwrap_or(0.0) > 0.0).count() as f64;
            let mrr: f64 = cs.iter().map(|v| v.get(k).and_then(Value::as_f64).unwrap_or(0.0)).sum();
            logo.push(rd(alive / n * 100.0, 1));
            rev.push(if m0 > 0.0 { rd(mrr / m0 * 100.0, 1) } else { 0.0 });
        }
        rows.push(json!({"cohort_month": m, "customers": cs.len(), "logo_retention_pct": logo, "revenue_retention_pct": rev}));
    }
    // weighted month-k average across cohorts old enough
    let maxk = rows.iter().map(|r| r["logo_retention_pct"].as_array().unwrap().len()).max().unwrap_or(0);
    let mut avg = vec![];
    for k in 0..maxk {
        let (mut w, mut s) = (0.0, 0.0);
        for r in &rows {
            if let Some(x) = r["revenue_retention_pct"].get(k).and_then(Value::as_f64) {
                let n = r["customers"].as_f64().unwrap();
                w += n;
                s += x * n;
            }
        }
        avg.push(rd(s / w, 1));
    }
    Ok(json!({"cohorts": rows, "avg_revenue_retention_pct_by_month": avg}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn triangle() {
        let i = json!({"current_month":2,"customers":[{"signup_month":0,"mrr_by_month":[100,100,150]},{"signup_month":0,"mrr_by_month":[100,0,0]},{"signup_month":2,"mrr_by_month":[50]}]});
        let r = cohort_op(&i).unwrap();
        assert_eq!(r["cohorts"][0]["logo_retention_pct"], json!([100.0, 50.0, 50.0]));
        assert_eq!(r["cohorts"][0]["revenue_retention_pct"], json!([100.0, 50.0, 75.0]));
    }
}
