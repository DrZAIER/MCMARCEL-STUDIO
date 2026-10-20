use crate::types::{
    AnalysisResult, CountryPack, IcerRow, InmbRow, PstarRow, ScenarioSummary,
};

#[derive(Debug, Clone)]
struct Strategy {
    id: String,
    name: String,
    cost: f64,
    effect: f64,
    doses: f64,
    price: f64,
    wastage: f64,
    is_reference: bool,
}

pub fn analyze(pack: &CountryPack, summaries: &[ScenarioSummary]) -> AnalysisResult {
    let thresholds = pack.wtp_thresholds();
    let mut strategies: Vec<Strategy> = summaries
        .iter()
        .map(|s| Strategy {
            id: s.scenario_id.clone(),
            name: s.scenario_name.clone(),
            cost: if s.is_reference { 0.0 } else { s.mean_incremental_cost },
            effect: if s.is_reference { 0.0 } else { s.mean_qaly_averted },
            doses: s.mean_doses,
            price: s.mean_price,
            wastage: s.mean_wastage,
            is_reference: s.is_reference,
        })
        .collect();

    // Reference always anchors the origin.
    if let Some(reference) = strategies.iter_mut().find(|s| s.is_reference) {
        reference.cost = 0.0;
        reference.effect = 0.0;
    } else if let Some(first) = strategies.first_mut() {
        first.cost = 0.0;
        first.effect = 0.0;
    }

    // Dominance uses cost-effectiveness space. Sort by cost, then by effect.
    let mut sorted = strategies.clone();
    sorted.sort_by(|a, b| a.cost.total_cmp(&b.cost).then_with(|| b.effect.total_cmp(&a.effect)));

    // Simple dominance first: a strategy is dominated when another strategy
    // costs no more and achieves no less. Dominated strategies have to be
    // removed before the frontier is built, otherwise they distort the ICER
    // sequence of the remaining points.
    let mut dominated_by: Vec<(String, String)> = sorted
        .iter()
        .filter_map(|strategy| {
            sorted
                .iter()
                .find(|other| {
                    other.id != strategy.id
                        && other.cost <= strategy.cost + 1e-9
                        && other.effect >= strategy.effect - 1e-9
                        && (other.cost < strategy.cost - 1e-9 || other.effect > strategy.effect + 1e-9)
                })
                .map(|other| (strategy.id.clone(), other.id.clone()))
        })
        .collect();
    let dominated_ids: Vec<String> = dominated_by.iter().map(|(id, _)| id.clone()).collect();

    // The frontier is the increasing-effect sequence of the surviving strategies.
    let mut frontier: Vec<Strategy> = Vec::new();
    let mut max_effect = f64::NEG_INFINITY;
    for strategy in &sorted {
        if dominated_ids.contains(&strategy.id) {
            continue;
        }
        if strategy.effect > max_effect + 1e-12 {
            frontier.push(strategy.clone());
            max_effect = strategy.effect;
        }
    }

    // Remove extended dominance: a point whose ICER is lower than the previous
    // point's ICER makes the previous point redundant.
    loop {
        if frontier.len() < 3 {
            break;
        }
        let mut remove_index = None;
        for index in 1..frontier.len() - 1 {
            let left = &frontier[index - 1];
            let middle = &frontier[index];
            let right = &frontier[index + 1];
            let icer_middle = ratio(middle.cost - left.cost, middle.effect - left.effect);
            let icer_right = ratio(right.cost - middle.cost, right.effect - middle.effect);
            if let (Some(a), Some(b)) = (icer_middle, icer_right) {
                if b < a - 1e-12 {
                    remove_index = Some(index);
                    break;
                }
            }
        }
        match remove_index {
            Some(index) => {
                let removed = frontier.remove(index);
                dominated_by.push((removed.id.clone(), "an adjacent strategy".to_string()));
            }
            None => break,
        }
    }

    let frontier_ids: Vec<String> = frontier.iter().map(|s| s.id.clone()).collect();
    // Every strategy that is not on the frontier is dominated: either by a
    // single cheaper or more effective strategy, or by the two strategies that
    // surround it on the frontier.
    let dominated_by: Vec<(String, String)> = dominated_by
        .into_iter()
        .chain(
            strategies
                .iter()
                .filter(|s| !frontier_ids.contains(&s.id))
                .filter(|s| !dominated_ids.contains(&s.id))
                .map(|s| (s.id.clone(), "the strategies around it on the frontier".to_string())),
        )
        .collect();

    let mut icer_table = Vec::new();
    for strategy in &strategies {
        if strategy.is_reference {
            icer_table.push(IcerRow {
                strategy_id: strategy.id.clone(),
                strategy: strategy.name.clone(),
                cost: strategy.cost,
                effect: strategy.effect,
                incremental_cost: None,
                incremental_effect: None,
                icer: None,
                status: "reference".into(),
                note: "Comparator".into(),
            });
            continue;
        }
        let position = frontier.iter().position(|s| s.id == strategy.id);
        if let Some(index) = position {
            if index == 0 {
                icer_table.push(IcerRow {
                    strategy_id: strategy.id.clone(),
                    strategy: strategy.name.clone(),
                    cost: strategy.cost,
                    effect: strategy.effect,
                    incremental_cost: None,
                    incremental_effect: None,
                    icer: None,
                    status: "frontier".into(),
                    note: "Below the reference cost; dominates the comparator.".into(),
                });
            } else {
                let previous = &frontier[index - 1];
                let incremental_cost = strategy.cost - previous.cost;
                let incremental_effect = strategy.effect - previous.effect;
                let icer = ratio(incremental_cost, incremental_effect);
                icer_table.push(IcerRow {
                    strategy_id: strategy.id.clone(),
                    strategy: strategy.name.clone(),
                    cost: strategy.cost,
                    effect: strategy.effect,
                    incremental_cost: Some(incremental_cost),
                    incremental_effect: Some(incremental_effect),
                    icer,
                    status: if icer.map(|v| v < 0.0).unwrap_or(false) { "dominant".into() } else { "frontier".into() },
                    note: format!("Incremental versus {}.", previous.name),
                });
            }
        } else {
            let dominated = dominated_by
                .iter()
                .find(|(id, _)| id == &strategy.id)
                .map(|(_, by)| by.clone())
                .unwrap_or_else(|| "a more efficient strategy".into());
            icer_table.push(IcerRow {
                strategy_id: strategy.id.clone(),
                strategy: strategy.name.clone(),
                cost: strategy.cost,
                effect: strategy.effect,
                incremental_cost: None,
                incremental_effect: None,
                icer: None,
                status: "dominated".into(),
                note: format!("Dominated by {dominated}."),
            });
        }
    }

    let mut inmb_table = Vec::new();
    for threshold in &thresholds {
        for strategy in &strategies {
            inmb_table.push(InmbRow {
                threshold: *threshold,
                strategy_id: strategy.id.clone(),
                strategy: strategy.name.clone(),
                inmb: net_benefit(strategy.effect, strategy.cost, *threshold),
            });
        }
    }

    let central_wtp = thresholds.get(1).copied().or_else(|| thresholds.first().copied()).unwrap_or(0.0);
    let pstar_table = strategies
        .iter()
        .filter(|s| !s.is_reference)
        .map(|strategy| {
            let slope = strategy.doses * (1.0 + strategy.wastage);
            if slope <= 1e-12 {
                return PstarRow {
                    strategy_id: strategy.id.clone(),
                    strategy: strategy.name.clone(),
                    wtp: central_wtp,
                    pstar: None,
                    current_price: strategy.price,
                    price_floor: 0.0,
                    slope,
                    doses: strategy.doses,
                    fvp: 0.0,
                    note: "No doses in this scenario; price is irrelevant.".into(),
                };
            }
            let cost_at_zero_price = strategy.cost - strategy.price * slope;
            let pstar = (central_wtp * strategy.effect - cost_at_zero_price) / slope;
            PstarRow {
                strategy_id: strategy.id.clone(),
                strategy: strategy.name.clone(),
                wtp: central_wtp,
                pstar: Some(pstar),
                current_price: strategy.price,
                price_floor: 0.0,
                slope,
                doses: strategy.doses,
                fvp: 0.0,
                note: if pstar < 0.0 {
                    "Negative threshold price: the intervention does not break even even at zero dose price.".into()
                } else {
                    "Threshold dose price at the central WTP value.".into()
                },
            }
        })
        .collect();

    AnalysisResult {
        thresholds,
        icer_table,
        inmb_table,
        pstar_table,
        frontier: frontier_ids,
    }
}

/// Net monetary benefit of a strategy, mirroring `get_net_benefit` in the R
/// implementation (`functions/MISC_CEA.R`).
pub fn net_benefit(qaly_averted: f64, incremental_cost: f64, wtp: f64) -> f64 {
    wtp * qaly_averted - incremental_cost
}

fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator.abs() <= 1e-12 || !numerator.is_finite() || !denominator.is_finite() {
        None
    } else {
        Some(numerator / denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::demo_pack;
    use crate::types::ScenarioSummary;

    fn summary(id: &str, cost: f64, effect: f64, reference: bool) -> ScenarioSummary {
        ScenarioSummary {
            scenario_id: id.into(),
            scenario_name: id.into(),
            is_reference: reference,
            mean_incremental_cost: cost,
            mean_qaly_averted: effect,
            mean_doses: 1000.0,
            mean_price: 10.0,
            mean_wastage: 0.05,
            ..Default::default()
        }
    }

    #[test]
    fn dominated_strategy_is_removed() {
        let pack = demo_pack();
        let summaries = vec![
            summary("none", 0.0, 0.0, true),
            summary("a", 100.0, 1.0, false),
            summary("b", 200.0, 0.5, false),
        ];
        let result = analyze(&pack, &summaries);
        assert!(result.frontier.contains(&"a".into()));
        assert!(!result.frontier.contains(&"b".into()));
        let b = result.icer_table.iter().find(|r| r.strategy_id == "b").unwrap();
        assert_eq!(b.status, "dominated");
    }
}
