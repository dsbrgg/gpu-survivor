use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Edition {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub seed: u64,
    pub duration_seconds: f32,
    pub boss_at_seconds: f32,
    pub regime: String,
    pub threat_multiplier: f32,
    pub xp_multiplier: f32,
    pub provenance: String,
    pub events: Vec<WorldEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorldEvent {
    pub at_seconds: f32,
    pub kind: EventKind,
    pub title: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Funding,
    Blackout,
    Liquidation,
    BlueWave,
    RegulatoryCrackdown,
    AiStockCrash,
    AiStockRally,
    Research,
    ComputeExpansion,
}

impl Edition {
    pub fn demo() -> Self {
        serde_json::from_str(include_str!("../web/public/editions/demo.json")).unwrap()
    }

    pub fn parse(json: &str) -> Result<Self, String> {
        let edition: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        edition.validate()?;
        Ok(edition)
    }

    pub fn validate(&self) -> Result<(), String> {
        let valid = self.schema_version == 1
            && !self.id.is_empty()
            && self.id.len() <= 80
            && self.title.len() <= 100
            && self.provenance.len() <= 500
            && matches!(self.regime.as_str(), "bull" | "neutral" | "bear")
            && (30.0..=600.0).contains(&self.duration_seconds)
            && self.boss_at_seconds == self.duration_seconds
            && (0.75..=1.35).contains(&self.threat_multiplier)
            && (0.85..=1.25).contains(&self.xp_multiplier)
            && self.events.len() <= 8
            && self.events.iter().all(|event| {
                (0.0..self.boss_at_seconds).contains(&event.at_seconds)
                    && event.title.len() <= 64
                    && event.title.is_ascii()
            });
        if valid {
            Ok(())
        } else {
            Err("Edition exceeds the supported version, timing, or content budget".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_edition_rejects_unbounded_pressure_and_unknown_behaviour() {
        let mut edition = Edition::demo();
        edition.threat_multiplier = 100.0;
        assert!(edition.validate().is_err());
        let json = include_str!("../web/public/editions/demo.json");
        assert!(Edition::parse(&json.replace("\"funding\"", "\"execute_code\"")).is_err());
        assert!(Edition::parse(json).is_ok());
    }
}
