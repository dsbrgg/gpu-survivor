use crate::upgrade::Upgrade;
use serde::Serialize;

#[derive(Serialize)]
pub struct Founder {
    pub name: &'static str,
    pub company: &'static str,
    pub actor_row: usize,
    pub projectiles: u32,
    pub damage: f32,
    pub shot_cooldown: f32,
    pub shot_speed: f32,
    pub ability: &'static str,
    pub ability_damage: f32,
    pub ability_radius: f32,
    pub heal: f32,
    pub shield: f32,
    pub signature: Upgrade,
}

pub const FOUNDERS: [Founder; 4] = [
    Founder {
        name: "JENSEN HUANG",
        company: "NVIDIA",
        actor_row: 0,
        projectiles: 1,
        damage: 22.,
        shot_cooldown: 0.43,
        shot_speed: 280.,
        ability: "CUDA BURST",
        ability_damage: 60.,
        ability_radius: 155.,
        heal: 0.,
        shield: 0.,
        signature: Upgrade::LeatherJacket,
    },
    Founder {
        name: "SAM ALTMAN",
        company: "OpenAI",
        actor_row: 1,
        projectiles: 2,
        damage: 14.,
        shot_cooldown: 0.62,
        shot_speed: 280.,
        ability: "FUNDRAISE",
        ability_damage: 42.,
        ability_radius: 155.,
        heal: 16.,
        shield: 0.,
        signature: Upgrade::CapitalInjection,
    },
    Founder {
        name: "DARIO AMODEI",
        company: "Anthropic",
        actor_row: 8,
        projectiles: 1,
        damage: 32.,
        shot_cooldown: 0.60,
        shot_speed: 250.,
        ability: "GUARDRAILS",
        ability_damage: 36.,
        ability_radius: 155.,
        heal: 0.,
        shield: 2.5,
        signature: Upgrade::SafetyRails,
    },
    Founder {
        name: "ELON MUSK",
        company: "xAI",
        actor_row: 9,
        projectiles: 3,
        damage: 10.,
        shot_cooldown: 0.64,
        shot_speed: 350.,
        ability: "SCALE UP",
        ability_damage: 75.,
        ability_radius: 185.,
        heal: 0.,
        shield: 0.,
        signature: Upgrade::Cybertruck,
    },
];

pub fn founder(character: u32) -> &'static Founder {
    &FOUNDERS[character.min(FOUNDERS.len() as u32 - 1) as usize]
}
