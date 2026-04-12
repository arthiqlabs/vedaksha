// Quick accuracy check: osculating node vs Meeus true node at various dates
fn main() {
    let dates = vec![
        (2451545.0, "2000-01-01 J2000"),
        (2452275.0, "2002-01-01"),
        (2453006.0, "2004-01-01"),
        (2455197.5, "2010-01-01"),
        (2457388.5, "2016-01-01"),
        (2459580.5, "2022-01-01"),
        (2460676.5, "2025-01-01"),
        (2461041.5, "2026-01-01"),
        (2461142.5, "2026-04-12 today"),
    ];
    
    println!("{:<25} {:>12} {:>12} {:>12} {:>8}", "Date", "Mean", "True(M)", "Osculating", "Diff");
    println!("{}", "-".repeat(75));
    
    for (jd, label) in &dates {
        let mean = vedaksha_ephem_core::nodes::mean_node(*jd);
        let true_m = vedaksha_ephem_core::nodes::true_node(*jd);
        let osc = vedaksha_ephem_core::nodes::true_node_osculating(*jd);
        let mut diff = osc - true_m;
        if diff > 180.0 { diff -= 360.0; }
        if diff < -180.0 { diff += 360.0; }
        println!("{:<25} {:>12.4}° {:>12.4}° {:>12.4}° {:>8.4}°", label, mean, true_m, osc, diff);
    }
}
