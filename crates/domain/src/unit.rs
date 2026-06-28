/// PD↔hours↔PM conversion constants (design §4.1). Configurable; defaults 1 PD = 8h, 1 PM = 20 PD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitConfig {
    pub hours_per_pd: f64,   // default 8.0
    pub pd_per_pm: f64,      // default 20.0
}

impl UnitConfig {
    pub const DEFAULT: Self = Self { hours_per_pd: 8.0, pd_per_pm: 20.0 };

    pub fn pd_to_hours(self, pd: f64) -> f64 { pd * self.hours_per_pd }
    pub fn hours_to_pd(self, h: f64) -> f64 { h / self.hours_per_pd }
    pub fn pd_to_pm(self, pd: f64) -> f64 { pd / self.pd_per_pm }
    pub fn pm_to_pd(self, pm: f64) -> f64 { pm * self.pd_per_pm }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pd_hours_roundtrip() {
        let u = UnitConfig::DEFAULT;
        assert!((u.pd_to_hours(1.0) - 8.0).abs() < 1e-9);
        assert!((u.hours_to_pd(8.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pm_pd_roundtrip() {
        let u = UnitConfig::DEFAULT;
        assert!((u.pm_to_pd(1.0) - 20.0).abs() < 1e-9);
        assert!((u.pd_to_pm(20.0) - 1.0).abs() < 1e-9);
    }
}