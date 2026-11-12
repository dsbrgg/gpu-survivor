use crate::config::EventKind;
use serde::Serialize;

pub const CONTENT_VERSION: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetPack {
    DataCenter,
    BlueWave,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RivalTemplate {
    Investor,
    Inspector,
    Auditor,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PlayerModifiers {
    pub damage: f32,
    pub attack_rate: f32,
    pub movement: f32,
}

impl Default for PlayerModifiers {
    fn default() -> Self {
        Self {
            damage: 1.,
            attack_rate: 1.,
            movement: 1.,
        }
    }
}

#[derive(Serialize)]
pub struct DropBurst {
    pub count: usize,
    pub cash: bool,
    pub value: f32,
}

#[derive(Serialize)]
pub struct Reinforcements {
    pub template: RivalTemplate,
    pub count: usize,
}

#[derive(Serialize)]
pub struct EventRecipe {
    pub kind: EventKind,
    pub label: &'static str,
    pub description: &'static str,
    pub assets: &'static str,
    pub asset_pack: AssetPack,
    pub duration_seconds: f32,
    pub pressure_multiplier: f32,
    pub inspector_share: f32,
    pub blackout: bool,
    pub drops: Option<DropBurst>,
    pub reinforcements: Option<Reinforcements>,
    pub player_modifiers: Option<PlayerModifiers>,
}

pub const RECIPES: [EventRecipe; 9] = [
    EventRecipe {
        kind: EventKind::Funding,
        label: "FUNDING ROUND",
        description: "Five cash pickups restore runway. Five investors arrive to collect their return.",
        assets: "CASH / INVESTORS",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 0., pressure_multiplier: 1., inspector_share: 0., blackout: false,
        drops: Some(DropBurst { count: 5, cash: true, value: 3. }),
        reinforcements: Some(Reinforcements { template: RivalTemplate::Investor, count: 5 }),
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::Blackout,
        label: "ROLLING BLACKOUT",
        description: "The data center loses power for 13 seconds. Visibility drops; combat continues.",
        assets: "DARKNESS / UNLIT RACKS",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 13., pressure_multiplier: 1., inspector_share: 0., blackout: true,
        drops: None, reinforcements: None,
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::Liquidation,
        label: "LIQUIDATION SALE",
        description: "Fifteen GPU pickups appear nearby. Each grants three XP toward your next upgrade.",
        assets: "GPU CACHE",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 0., pressure_multiplier: 1., inspector_share: 0., blackout: false,
        drops: Some(DropBurst { count: 15, cash: false, value: 3. }), reinforcements: None,
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::BlueWave,
        label: "BLUE WAVE",
        description: "45 seconds of inspectors and Democratic Party banners. Spawn pressure rises 20%, capped at 1.35x. Inspectors drop blue GPU vouchers.",
        assets: "DEMOCRATS / INSPECTORS / VOUCHERS",
        asset_pack: AssetPack::BlueWave,
        duration_seconds: 45., pressure_multiplier: 1.2, inspector_share: 0.28, blackout: false,
        drops: None,
        reinforcements: Some(Reinforcements { template: RivalTemplate::Inspector, count: 4 }),
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::Research,
        label: "RESEARCH DIVIDEND",
        description: "Eight research GPU pickups appear nearby. Each grants three XP toward your next upgrade.",
        assets: "RESEARCH / GPU CACHE",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 0., pressure_multiplier: 1., inspector_share: 0., blackout: false,
        drops: Some(DropBurst { count: 8, cash: false, value: 3. }), reinforcements: None,
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::ComputeExpansion,
        label: "COMPUTE EXPANSION",
        description: "Twelve GPU pickups arrive. Compute demand raises spawn pressure 10% for 30 seconds, capped at 1.35x.",
        assets: "GPU SUPPLY / DEMAND SURGE",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 30., pressure_multiplier: 1.1, inspector_share: 0., blackout: false,
        drops: Some(DropBurst { count: 12, cash: false, value: 2. }), reinforcements: None,
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::RegulatoryCrackdown,
        label: "REGULATORY CRACKDOWN",
        description: "35 seconds of audits and subpoena volleys. Five lawyers arrive; spawn pressure rises 15%, capped at 1.35x.",
        assets: "AUDITORS / SUBPOENAS",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 35., pressure_multiplier: 1.15, inspector_share: 0., blackout: false,
        drops: None, reinforcements: Some(Reinforcements { template: RivalTemplate::Auditor, count: 5 }),
        player_modifiers: None,
    },
    EventRecipe {
        kind: EventKind::AiStockCrash,
        label: "AI STOCK CRASH",
        description: "30 seconds: damage -20%, fire rate -15%, movement -10%. Eight salvage GPUs appear. A new stock event replaces these modifiers.",
        assets: "RED TICKERS / SALVAGE GPUS",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 30., pressure_multiplier: 1., inspector_share: 0., blackout: false,
        drops: Some(DropBurst { count: 8, cash: false, value: 2. }), reinforcements: None,
        player_modifiers: Some(PlayerModifiers { damage: 0.8, attack_rate: 0.85, movement: 0.9 }),
    },
    EventRecipe {
        kind: EventKind::AiStockRally,
        label: "AI STOCK RALLY",
        description: "30 seconds: damage +25%, fire rate +20%, movement +10%. A new stock event replaces these modifiers; upgrades stay permanent.",
        assets: "GREEN TICKERS / PLAYER OVERDRIVE",
        asset_pack: AssetPack::DataCenter,
        duration_seconds: 30., pressure_multiplier: 1., inspector_share: 0., blackout: false,
        drops: None, reinforcements: None,
        player_modifiers: Some(PlayerModifiers { damage: 1.25, attack_rate: 1.2, movement: 1.1 }),
    },
];

impl EventKind {
    pub fn recipe(self) -> &'static EventRecipe {
        RECIPES
            .iter()
            .find(|recipe| recipe.kind == self)
            .expect("Every event has a recipe")
    }
}
