//! Attribution models. A journey is an ordered list of touches `{channel, day}` ending in a
//! conversion with a `value` on `converted_day`. Every model gives out exactly 100% of value.
use crate::util::*;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub const MODELS: &[&str] = &["first_touch", "last_touch", "linear", "time_decay", "u_shaped"];

pub fn weights(model: &str, days_before: &[f64], half_life: f64) -> R<Vec<f64>> {
    let n = days_before.len();
    if n == 0 {
        return Ok(vec![]);
    }
    let mut w = vec![0.0; n];
    match model {
        "first_touch" => w[0] = 1.0,
        "last_touch" => w[n - 1] = 1.0,
        "linear" => w.iter_mut().for_each(|x| *x = 1.0 / n as f64),
        "time_decay" => {
            for (k, d) in days_before.iter().enumerate() {
                w[k] = 0.5f64.powf(d / half_life);
            }
            let s: f64 = w.iter().sum();
            w.iter_mut().for_each(|x| *x /= s);
        }
        "u_shaped" => match n {
            1 => w[0] = 1.0,
            2 => {
                w[0] = 0.5;
                w[1] = 0.5
            }
            _ => {
                w[0] = 0.4;
                w[n - 1] = 0.4;
                for x in w.iter_mut().take(n - 1).skip(1) {
                    *x = 0.2 / (n - 2) as f64;
                }
            }
        },
        _ => return Err(format!("unknown model `{model}`")),
    }
    Ok(w)
}

pub fn attribute_op(i: &Value) -> R<Value> {
    let hl = f_or(i, "half_life_days", 7.0);
    if hl <= 0.0 {
        return Err("half_life_days must be > 0".into());
    }
    let mut credit: BTreeMap<&str, BTreeMap<String, f64>> = BTreeMap::new();
    let mut total = 0.0;
    let mut conv = 0;
    for j in arr(i, "journeys")? {
        let v = f_or(j, "value", 0.0);
        let Some(cd) = j.get("converted_day").and_then(Value::as_f64) else { continue };
        let mut touches: Vec<(String, f64)> = vec![];
        for t in arr(j, "touches")? {
            let d = f(t, "day")?;
            if d <= cd {
                touches.push((s(t, "channel")?.to_string(), d));
            }
        }
        if touches.is_empty() {
            continue;
        }
        touches.sort_by(|a, b| a.1.total_cmp(&b.1));
        conv += 1;
        total += v;
        let before: Vec<f64> = touches.iter().map(|t| cd - t.1).collect();
        for m in MODELS {
            let w = weights(m, &before, hl)?;
            let e = credit.entry(m).or_default();
            for (k, t) in touches.iter().enumerate() {
                *e.entry(t.0.clone()).or_default() += w[k] * v;
            }
        }
    }
    let spend = i.get("spend").and_then(Value::as_object).cloned().unwrap_or_default();
    let mut models = Map::new();
    for (m, per) in &credit {
        let mut rows = Map::new();
        for (ch, val) in per {
            let mut r = json!({"revenue": rd(*val, 2), "share_pct": rd(val / total.max(1e-9) * 100.0, 1)});
            if let Some(sp) = spend.get(ch).and_then(Value::as_f64) {
                r["roas"] = json!(if sp > 0.0 { Some(rd(val / sp, 2)) } else { None });
            }
            rows.insert(ch.clone(), r);
        }
        models.insert(m.to_string(), Value::Object(rows));
    }
    // spread: how much each channel's share moves across models
    let chans: std::collections::BTreeSet<&String> = credit.values().flat_map(|m| m.keys()).collect();
    let mut spread = Map::new();
    for ch in chans {
        let shares: Vec<f64> = credit.values().map(|m| m.get(ch).copied().unwrap_or(0.0) / total.max(1e-9) * 100.0).collect();
        let (lo, hi) = shares.iter().fold((f64::MAX, f64::MIN), |a, &x| (a.0.min(x), a.1.max(x)));
        spread.insert(ch.clone(), json!({"min_share_pct": rd(lo, 1), "max_share_pct": rd(hi, 1)}));
    }
    Ok(json!({"conversions": conv, "revenue": rd(total, 2), "models": models, "share_range_by_channel": spread,
        "read_me": "a channel with a wide share range is where the model choice changes the budget decision"}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn models_sum_to_one() {
        for m in MODELS {
            let w = weights(m, &[10.0, 5.0, 2.0, 0.0], 7.0).unwrap();
            assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-9, "{m}");
        }
        assert_eq!(weights("u_shaped", &[3.0, 2.0, 1.0], 7.0).unwrap(), vec![0.4, 0.2, 0.4]);
    }
    #[test]
    fn ignores_touches_after_conversion() {
        let i = json!({"journeys":[{"value":100,"converted_day":5,"touches":[{"channel":"ads","day":1},{"channel":"email","day":9}]}],"spend":{"ads":50}});
        let r = attribute_op(&i).unwrap();
        assert_eq!(r["models"]["last_touch"]["ads"]["revenue"], 100.0);
        assert_eq!(r["models"]["linear"]["ads"]["roas"], 2.0);
    }
}
