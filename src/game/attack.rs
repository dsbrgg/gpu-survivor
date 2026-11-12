use crate::content::AssetPack;
use serde::Serialize;

pub const WINDUP: f32 = 0.65;
pub const MAX_HOSTILE_SHOTS: usize = 128;
pub const BOSS_SHOT_RESERVE: usize = 24;
pub const MAX_FRIENDLY_SHOTS: usize = 96;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyKind {
    Bot,
    Crawler,
    Lawyer,
    Investor,
    Bernie,
    Aoc,
    Spy,
    Bubble,
    Engineer,
    Politician,
    Glm,
    Kimi,
    DeepSeek,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectileKind {
    Founder,
    Legislation,
    Subpoena,
    Citation,
    Flag,
    Wrench,
    Coin,
    Bolt,
    Scrap,
    Cybertruck,
    LeatherJacket,
    Capital,
    SafetyRail,
}

impl ProjectileKind {
    pub fn hostile(self) -> bool {
        !matches!(
            self,
            Self::Founder
                | Self::Cybertruck
                | Self::LeatherJacket
                | Self::Capital
                | Self::SafetyRail
        )
    }

    pub fn sprite_column(self) -> usize {
        self as usize - 1
    }
}

pub struct Attack {
    pub projectile: ProjectileKind,
    pub count: usize,
    pub speed: f32,
    pub damage: f32,
    pub cooldown: f32,
    pub range: f32,
    pub spread: f32,
}

impl EnemyKind {
    pub fn miniboss(self) -> bool {
        matches!(self, Self::Bernie | Self::Aoc | Self::Spy)
    }

    pub fn boss(self) -> bool {
        self.miniboss() || self == Self::Bubble
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Bernie => "BERNIE // ONCE AGAIN ASKING",
            Self::Aoc => "AOC // BUDGET FREEZE",
            Self::Spy => "ALICE HUANG - (ALLEGED) GPU ESPIONAGE",
            Self::Bubble => "AI BUBBLE // FINAL SHOWDOWN",
            _ => "",
        }
    }

    pub fn standoff(self) -> f32 {
        match self {
            Self::Lawyer | Self::Politician | Self::Glm => 155.,
            Self::Engineer => 110.,
            Self::Investor => 125.,
            Self::Aoc => 85.,
            Self::Spy => 130.,
            Self::Bubble => 100.,
            _ => 0.,
        }
    }

    pub fn attack(self, pack: AssetPack) -> Attack {
        use ProjectileKind::*;
        let (projectile, count, speed, damage, cooldown, range, spread) = match self {
            Self::Lawyer => (
                if pack == AssetPack::BlueWave {
                    Citation
                } else {
                    Subpoena
                },
                1,
                95.,
                10.,
                4.2,
                285.,
                0.,
            ),
            Self::Politician => (Legislation, 2, 100., 10., 4.5, 285., 0.24),
            Self::Bernie => (
                Legislation,
                12,
                84.,
                15.,
                2.6,
                330.,
                std::f32::consts::TAU / 12.,
            ),
            Self::Glm => (Flag, 3, 100., 9., 5.2, 285., 0.24),
            Self::Aoc => (Subpoena, 5, 112., 12., 3.1, 310., 0.22),
            Self::Spy => (Bolt, 3, 155., 11., 2.8, 300., 0.16),
            Self::Bubble => (Coin, 18, 100., 18., 2.1, 420., std::f32::consts::TAU / 18.),
            Self::Kimi => (Flag, 1, 135., 7., 5.8, 210., 0.),
            Self::DeepSeek => (Flag, 2, 110., 9., 5., 230., 0.30),
            Self::Engineer => (Wrench, 1, 125., 10., 5.4, 240., 0.),
            Self::Investor => (Coin, 3, 100., 8., 6., 240., 0.18),
            Self::Bot => (Bolt, 1, 145., 7., 6., 170., 0.),
            Self::Crawler => (Scrap, 2, 85., 6., 6.5, 105., 0.34),
        };
        Attack {
            projectile,
            count,
            speed,
            damage,
            cooldown,
            range,
            spread,
        }
    }
}
