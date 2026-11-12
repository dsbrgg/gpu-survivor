use serde::Serialize;

#[derive(Clone, Copy, Serialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Upgrade {
    Overclock,
    Parallelism,
    FineTuning,
    Acquisition,
    Runway,
    OpenWeights,
    LiquidCooling,
    Buyback,
    WiderContext,
    LeatherJacket,
    CapitalInjection,
    SafetyRails,
    Cybertruck,
}

#[derive(Serialize)]
pub struct UpgradeCard {
    pub kind: Upgrade,
    pub name: &'static str,
    pub description: &'static str,
    pub tag: &'static str,
    pub level: u32,
}

impl Upgrade {
    pub const COUNT: usize = Self::Cybertruck as usize + 1;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn max_rank(self) -> u32 {
        match self {
            Self::LeatherJacket | Self::CapitalInjection | Self::SafetyRails | Self::Cybertruck => {
                3
            }
            Self::Parallelism | Self::OpenWeights | Self::WiderContext => 4,
            Self::Buyback => u32::MAX,
            _ => 6,
        }
    }

    pub fn card(self, level: u32) -> UpgradeCard {
        let (name, description, tag) = match self {
            Self::Overclock => (
                "OVERCLOCK",
                "+22% attack speed. The warranty is a suggestion.",
                "HARDWARE",
            ),
            Self::Parallelism => (
                "PARALLELISM",
                "+1 projectile per burst. More is a strategy.",
                "SCALING",
            ),
            Self::FineTuning => (
                "FINE-TUNING",
                "+30% weapon damage. Now with better benchmarks.",
                "RESEARCH",
            ),
            Self::Acquisition => (
                "ACQUISITION",
                "+35 pickup radius. Your GPUs are our GPUs.",
                "MONOPOLY",
            ),
            Self::Runway => (
                "SERIES C",
                "+25 max runway. Restore 40 health. Pre-revenue, forever.",
                "FUNDING",
            ),
            Self::OpenWeights => (
                "OPEN WEIGHTS",
                "Add an orbiting model. Contact damage, zero moat.",
                "OPEN SOURCE",
            ),
            Self::LiquidCooling => (
                "LIQUID COOLING",
                "+10% movement. Dash recharges 15% faster.",
                "INFRASTRUCTURE",
            ),
            Self::Buyback => (
                "STOCK BUYBACK",
                "Heal 60 runway. Absolutely a sustainable strategy.",
                "RECOVERY",
            ),
            Self::WiderContext => (
                "WIDER CONTEXT",
                "+20% attack reach, projectile size, special and orbit radius.",
                "AREA / ALL FOUNDERS",
            ),
            Self::LeatherJacket => (
                "LEATHER JACKET",
                if level == 1 {
                    "Auto-throw piercing, returning jackets alongside NVIDIA shots."
                } else {
                    "+40% base jacket damage and +1 rival pierced per throw."
                },
                "JENSEN / STACKING WEAPON",
            ),
            Self::CapitalInjection => (
                "CAPITAL INJECTION",
                if level == 1 {
                    "Auto-fire exploding cash bundles alongside OpenAI shots."
                } else {
                    "+40% base cash damage and +10 explosion radius."
                },
                "SAM / STACKING WEAPON",
            ),
            Self::SafetyRails => (
                "SAFETY RAILS",
                if level == 1 {
                    "Auto-fire barriers that pierce rivals and block hostile shots."
                } else {
                    "+40% base barrier damage and +1 rival pierced per barrier."
                },
                "DARIO / STACKING WEAPON",
            ),
            Self::Cybertruck => (
                "CYBERTRUCK",
                if level == 1 {
                    "Auto-launch a piercing Cybertruck alongside Grok shots."
                } else {
                    "+40% base truck damage and +1 rival pierced per charge."
                },
                "ELON / STACKING WEAPON",
            ),
        };
        UpgradeCard {
            kind: self,
            name,
            description,
            tag,
            level,
        }
    }
}
