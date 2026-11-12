use crate::config::{Edition, EventKind, WorldEvent};
use serde::{Deserialize, Serialize};

const MAX_RUN_EVENTS: usize = 8;
const WARNING_SECONDS: f32 = 8.;
const EVENT_SPACING: f32 = 20.;
const RECIPE_COOLDOWN: f32 = 60.;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Bull,
    #[default]
    Neutral,
    Bear,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_recipes_do_not_fill_the_opening_schedule() {
        let mut edition = Edition::demo();
        edition.events.clear();
        let mut fired = Vec::new();
        let mut state = LiveState::default();
        state.queue(LiveUpdate {
            revision: "policy-heavy".into(),
            sequence: 1,
            regime: Regime::Neutral,
            confidence: 1.,
            stories: [
                EventKind::BlueWave,
                EventKind::BlueWave,
                EventKind::Funding,
                EventKind::Research,
            ]
            .iter()
            .enumerate()
            .map(|(id, kind)| Story {
                id: id.to_string(),
                kind: *kind,
                title: "NEWS".into(),
            })
            .collect(),
        });
        state.apply_pending(&mut edition, &mut fired, 0.);
        assert_eq!(
            edition
                .events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![EventKind::BlueWave, EventKind::Funding, EventKind::Research]
        );
    }

    #[test]
    fn live_input_is_validated_and_schedule_leaves_room_for_future_news() {
        assert!(LiveUpdate::parse(
            r#"{"revision":"a","sequence":1,"regime":"panic","confidence":1,"stories":[]}"#
        )
        .is_err());
        let mut state = LiveState::default();
        let mut edition = Edition::demo();
        edition.events.clear();
        let mut fired = Vec::new();
        for sequence in 1..=4 {
            let update = LiveUpdate {
                revision: sequence.to_string(),
                sequence,
                regime: Regime::Bear,
                confidence: 0.5,
                stories: (0..8)
                    .map(|i| Story {
                        id: format!("{sequence}-{i}"),
                        kind: [EventKind::Research, EventKind::Funding, EventKind::Blackout][i % 3],
                        title: "NEWS".into(),
                    })
                    .collect(),
            };
            state.queue(update);
            state.apply_pending(&mut edition, &mut fired, 0.);
            if sequence == 1 {
                assert_eq!(edition.events.len(), 3);
            }
        }
        assert_eq!(edition.regime, "neutral");
        assert_eq!(edition.events.len(), 8);
        assert_eq!(fired.len(), 8);
        assert!(edition
            .events
            .windows(2)
            .all(|events| events[1].at_seconds - events[0].at_seconds >= 20.));
        assert!(edition
            .events
            .iter()
            .all(|event| event.at_seconds < edition.duration_seconds));
    }
}

impl Regime {
    fn parameters(self) -> (&'static str, f32, f32) {
        match self {
            Self::Bull => ("bull", 0.9, 1.15),
            Self::Neutral => ("neutral", 1., 1.),
            Self::Bear => ("bear", 1.2, 0.9),
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Story {
    pub id: String,
    pub kind: EventKind,
    pub title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveUpdate {
    pub revision: String,
    pub sequence: u64,
    pub regime: Regime,
    pub confidence: f32,
    pub stories: Vec<Story>,
}

impl LiveUpdate {
    pub fn parse(json: &str) -> Result<Self, String> {
        let update: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let valid = !update.revision.is_empty()
            && update.revision.len() <= 64
            && update.sequence > 0
            && (0.0..=1.0).contains(&update.confidence)
            && update.stories.len() <= MAX_RUN_EVENTS
            && update.stories.iter().all(|story| {
                !story.id.is_empty()
                    && story.id.len() <= 80
                    && story.title.is_ascii()
                    && story.title.len() <= 64
            });
        if !valid {
            return Err("Live update exceeds the supported signal budget".into());
        }
        Ok(update)
    }
}

#[derive(Serialize)]
pub struct ScheduledStory {
    pub id: String,
    pub kind: EventKind,
    pub title: String,
    pub at_seconds: f32,
}

#[derive(Default, Serialize)]
pub struct LiveState {
    pub sequence: u64,
    pub revision: String,
    pub regime: Regime,
    pub confidence: f32,
    pub events: Vec<ScheduledStory>,
    #[serde(skip)]
    pending: Option<LiveUpdate>,
}

impl LiveState {
    pub fn queue(&mut self, update: LiveUpdate) {
        if update.sequence <= self.sequence
            || self
                .pending
                .as_ref()
                .is_some_and(|pending| update.sequence <= pending.sequence)
        {
            return;
        }
        self.pending = Some(update);
    }

    pub fn apply_pending(
        &mut self,
        edition: &mut Edition,
        fired: &mut Vec<bool>,
        time: f32,
    ) -> bool {
        let Some(update) = self.pending.take() else {
            return false;
        };
        self.sequence = update.sequence;
        self.revision = update.revision;
        self.confidence = update.confidence;
        self.regime = if update.confidence >= 0.6 {
            update.regime
        } else {
            Regime::Neutral
        };
        let (regime, threat, xp) = self.regime.parameters();
        edition.regime = regime.into();
        edition.threat_multiplier = threat;
        edition.xp_multiplier = xp;
        let mut next = edition
            .events
            .iter()
            .map(|event| event.at_seconds + EVENT_SPACING)
            .fold(time + WARNING_SECONDS, f32::max);
        let mut accepted = 0;
        for story in update.stories {
            if accepted >= 3 {
                break;
            }
            if edition.events.len() >= MAX_RUN_EVENTS || next >= edition.boss_at_seconds {
                break;
            }
            if self.events.iter().any(|event| event.id == story.id) {
                continue;
            }
            if edition.events.iter().any(|event| {
                event.kind == story.kind
                    && next
                        < event.at_seconds
                            + RECIPE_COOLDOWN.max(story.kind.recipe().duration_seconds)
            }) {
                continue;
            }
            edition.events.push(WorldEvent {
                at_seconds: next,
                kind: story.kind,
                title: story.title.clone(),
            });
            fired.push(false);
            self.events.push(ScheduledStory {
                id: story.id,
                kind: story.kind,
                title: story.title,
                at_seconds: next,
            });
            next += EVENT_SPACING;
            accepted += 1;
        }
        true
    }
}
