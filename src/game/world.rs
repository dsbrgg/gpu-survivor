pub use crate::attack::{EnemyKind, ProjectileKind};
use crate::attack::{BOSS_SHOT_RESERVE, MAX_FRIENDLY_SHOTS, MAX_HOSTILE_SHOTS, WINDUP};
use crate::config::{Edition, EventKind};
use crate::content::{AssetPack, PlayerModifiers, RivalTemplate, RECIPES};
use crate::founder::{founder, FOUNDERS};
use crate::terrain::{Climate, Terrain, START};
pub use crate::upgrade::Upgrade;
use macroquad::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

pub const STEP: f32 = 1.0 / 60.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Menu,
    Playing,
    Paused,
    Upgrade,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Facing {
    Down,
    DownLeft,
    Left,
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
}

impl Facing {
    pub fn direction(self) -> Vec2 {
        match self {
            Self::Down => vec2(0., 1.),
            Self::DownLeft => vec2(-1., 1.).normalize(),
            Self::Left => vec2(-1., 0.),
            Self::UpLeft => vec2(-1., -1.).normalize(),
            Self::Up => vec2(0., -1.),
            Self::UpRight => vec2(1., -1.).normalize(),
            Self::Right => vec2(1., 0.),
            Self::DownRight => vec2(1., 1.).normalize(),
        }
    }

    pub fn follow(self, movement: Vec2) -> Self {
        if movement.length_squared() < 0.0001 {
            return self;
        }
        // A small dead band prevents jitter between adjacent diagonal sectors.
        if movement.normalize().dot(self.direction()) > 0.9 {
            return self;
        }
        let sector = (movement.to_angle() / std::f32::consts::FRAC_PI_4).round() as i32;
        [
            Self::Right,
            Self::DownRight,
            Self::Down,
            Self::DownLeft,
            Self::Left,
            Self::UpLeft,
            Self::Up,
            Self::UpRight,
        ][sector.rem_euclid(8) as usize]
    }

    pub fn sprite_column(self, frame: usize) -> usize {
        self as usize * 4 + frame % 4
    }
}

pub struct Enemy {
    pub id: u32,
    pub asset_pack: AssetPack,
    pub reward_pressure: f32,
    pub position: Vec2,
    pub facing: Facing,
    pub moving: bool,
    pub walk_time: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub kind: EnemyKind,
    pub speed: f32,
    pub radius: f32,
    pub attack: f32,
    pub flash: f32,
    pub knockback: Vec2,
    pub dead: bool,
}

pub struct Shot {
    pub position: Vec2,
    pub previous: Vec2,
    pub velocity: Vec2,
    pub damage: f32,
    pub ttl: f32,
    pub kind: ProjectileKind,
    pub age: f32,
    pub radius: f32,
    pub pierce: u32,
    pub hit_ids: Vec<u32>,
    pub blast_radius: f32,
}

impl Shot {
    fn new(position: Vec2, velocity: Vec2, damage: f32, ttl: f32, kind: ProjectileKind) -> Self {
        Self {
            position,
            previous: position,
            velocity,
            damage,
            ttl,
            kind,
            age: 0.,
            radius: 5.,
            pierce: 1,
            hit_ids: Vec::new(),
            blast_radius: 0.,
        }
    }
}

pub struct Blast {
    pub position: Vec2,
    pub radius: f32,
    pub ttl: f32,
}

pub struct Pickup {
    pub asset_pack: AssetPack,
    pub position: Vec2,
    pub value: f32,
    pub cash: bool,
    pub age: f32,
}

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub ttl: f32,
    pub size: f32,
}

pub struct Floater {
    pub position: Vec2,
    pub text: String,
    pub color: Color,
    pub ttl: f32,
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub movement: Vec2,
    pub dash: bool,
    pub ability: bool,
}

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / 16_777_216.0
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + self.unit() * (high - low)
    }
}

pub struct World {
    pub phase: Phase,
    pub edition: Edition,
    pub character: u32,
    pub player: Vec2,
    pub facing: Facing,
    pub moving: bool,
    pub walk_time: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub xp: f32,
    pub level: u32,
    pub kills: u32,
    pub collected: u32,
    pub time: f32,
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub blasts: Vec<Blast>,
    pub pickups: Vec<Pickup>,
    pub particles: Vec<Particle>,
    pub floaters: Vec<Floater>,
    pub terrain: Terrain,
    pub climate: Climate,
    pub heat_exposure: f32,
    pub upgrades: [u32; Upgrade::COUNT],
    pub choices: Vec<Upgrade>,
    pub invulnerable: f32,
    pub dash_cooldown: f32,
    pub dash_time: f32,
    pub dash_direction: Vec2,
    pub ability_cooldown: f32,
    pub pulse: f32,
    pub shake: f32,
    pub blackout: f32,
    pub banner: String,
    pub banner_time: f32,
    pub boss_spawned: bool,
    pub boss_defeated: bool,
    pub sound_cues: Vec<&'static str>,
    pub status_serial: u32,
    pub live: crate::live::LiveState,
    pub next_miniboss: f32,
    pub signature_cooldown: f32,
    next_enemy_id: u32,
    minibosses: Vec<EnemyKind>,
    rng: Rng,
    spawn_clock: f32,
    shot_clock: f32,
    orbit_clock: f32,
    events_fired: Vec<bool>,
}

impl World {
    pub fn new(edition: Edition, character: u32) -> Self {
        let events_fired = vec![false; edition.events.len()];
        let seed = edition.seed;
        let mut encounters = Rng::new(seed ^ 0x9e3779b97f4a7c15);
        let next_miniboss = 40. + encounters.range(4., 18.);
        let mut minibosses = vec![EnemyKind::Bernie, EnemyKind::Aoc, EnemyKind::Spy];
        for i in (1..minibosses.len()).rev() {
            minibosses.swap(i, encounters.next() as usize % (i + 1));
        }
        let mut world = Self {
            phase: Phase::Playing,
            edition,
            character: character.min(FOUNDERS.len() as u32 - 1),
            player: START,
            facing: Facing::Down,
            moving: false,
            walk_time: 0.,
            hp: 100.,
            max_hp: 100.,
            xp: 0.,
            level: 1,
            kills: 0,
            collected: 0,
            time: 0.,
            enemies: Vec::new(),
            shots: Vec::new(),
            blasts: Vec::new(),
            pickups: Vec::new(),
            particles: Vec::new(),
            floaters: Vec::new(),
            terrain: Terrain::new(seed),
            climate: Climate::Normal,
            heat_exposure: 0.,
            upgrades: [0; Upgrade::COUNT],
            choices: Vec::new(),
            invulnerable: 0.,
            dash_cooldown: 0.,
            dash_time: 0.,
            dash_direction: Vec2::X,
            ability_cooldown: 0.,
            pulse: 0.,
            shake: 0.,
            blackout: 0.,
            banner: "SURVIVE THE NEXT FUNDING CYCLE".into(),
            banner_time: 5.,
            boss_spawned: false,
            boss_defeated: false,
            sound_cues: Vec::new(),
            status_serial: 0,
            live: crate::live::LiveState::default(),
            next_miniboss,
            signature_cooldown: 0.,
            next_enemy_id: 0,
            minibosses,
            rng: Rng::new(seed),
            spawn_clock: 1.0,
            shot_clock: 0.,
            orbit_clock: 0.,
            events_fired,
        };
        // A first pickup trail teaches collection before the arena fills.
        for i in 0..7 {
            world.pickups.push(Pickup {
                asset_pack: AssetPack::DataCenter,
                position: world.player + vec2(30. + i as f32 * 15., (i as f32 * 0.8).sin() * 24.),
                value: 2.,
                cash: false,
                age: 0.,
            });
        }
        world
    }

    pub fn xp_needed(&self) -> f32 {
        7. + self.level as f32 * 4.
    }

    pub fn attack_radius_multiplier(&self) -> f32 {
        1. + self.upgrades[Upgrade::WiderContext.index()] as f32 * 0.2
    }

    pub fn ability_radius(&self) -> f32 {
        founder(self.character).ability_radius * self.attack_radius_multiplier()
    }

    pub fn signature_rank(&self) -> u32 {
        self.upgrades[founder(self.character).signature.index()]
    }

    pub fn active_boss(&self) -> Option<&Enemy> {
        self.enemies
            .iter()
            .filter(|enemy| enemy.kind.boss())
            .max_by_key(|enemy| enemy.kind == EnemyKind::Bubble)
    }

    fn weapon_damage_multiplier(&self) -> f32 {
        (1. + self.upgrades[Upgrade::FineTuning.index()] as f32 * 0.3)
            * self.player_modifiers().damage
    }

    fn weapon_speed_multiplier(&self) -> f32 {
        1. + self.upgrades[Upgrade::Overclock.index()] as f32 * 0.22
    }

    pub fn ability_locked(&self) -> bool {
        self.enemies.iter().any(|enemy| {
            enemy.kind == EnemyKind::Aoc
                && enemy.hp > 0.
                && enemy.position.distance(self.player) < 100.
        })
    }

    pub fn event_remaining(&self, kind: EventKind) -> f32 {
        self.edition
            .events
            .iter()
            .zip(&self.events_fired)
            .filter(|(event, fired)| **fired && event.kind == kind)
            .map(|(event, _)| {
                (event.at_seconds + kind.recipe().duration_seconds - self.time).max(0.)
            })
            .fold(0., f32::max)
    }

    pub fn effective_threat(&self) -> f32 {
        let pressure = RECIPES
            .iter()
            .filter(|recipe| self.event_remaining(recipe.kind) > 0.)
            .map(|recipe| recipe.pressure_multiplier)
            .fold(1., f32::max);
        (self.edition.threat_multiplier * pressure).clamp(0.75, 1.35)
    }

    pub fn player_event(&self) -> Option<&crate::config::WorldEvent> {
        self.edition
            .events
            .iter()
            .zip(&self.events_fired)
            .filter(|(event, fired)| {
                **fired
                    && event.kind.recipe().player_modifiers.is_some()
                    && self.time < event.at_seconds + event.kind.recipe().duration_seconds
            })
            .map(|(event, _)| event)
            .max_by(|a, b| a.at_seconds.total_cmp(&b.at_seconds))
    }

    pub fn player_modifiers(&self) -> PlayerModifiers {
        self.player_event()
            .and_then(|event| event.kind.recipe().player_modifiers)
            .unwrap_or_default()
    }

    pub fn pickup_radius(&self) -> f32 {
        48. + self.upgrades[Upgrade::Acquisition.index()] as f32 * 35.
    }

    pub fn heat_damage_per_second(&self) -> f32 {
        if self.climate == Climate::Hot && self.heat_exposure > 0.75 {
            (2. + (self.heat_exposure - 0.75) * 1.4).min(22.)
        } else {
            0.
        }
    }

    fn update_thermal(&mut self) {
        self.climate = self.terrain.climate_at(self.player);
        if self.climate == Climate::Hot {
            self.heat_exposure = (self.heat_exposure + STEP).min(20.);
            // Environmental damage neither grants nor consumes combat invulnerability.
            self.hp -= self.heat_damage_per_second() * STEP;
        } else {
            self.heat_exposure = (self.heat_exposure - STEP * 3.).max(0.);
        }
    }

    pub fn orbit_positions(&self) -> Vec<Vec2> {
        let count = self.upgrades[Upgrade::OpenWeights.index()];
        (0..count)
            .map(|i| {
                let angle = self.time * 2.6 + i as f32 * std::f32::consts::TAU / count as f32;
                self.player + Vec2::from_angle(angle) * 44. * self.attack_radius_multiplier()
            })
            .collect()
    }

    pub fn toggle_pause(&mut self) {
        self.phase = match self.phase {
            Phase::Playing => Phase::Paused,
            Phase::Paused => Phase::Playing,
            other => other,
        };
        self.status_serial += 1;
    }

    pub fn choose(&mut self, index: usize) {
        if self.phase != Phase::Upgrade || index >= self.choices.len() {
            return;
        }
        let upgrade = self.choices[index];
        self.upgrades[upgrade.index()] += 1;
        match upgrade {
            Upgrade::Runway => {
                self.max_hp += 25.;
                self.hp = (self.hp + 40.).min(self.max_hp);
            }
            Upgrade::Buyback => self.hp = (self.hp + 60.).min(self.max_hp),
            _ => {}
        }
        self.choices.clear();
        self.phase = Phase::Playing;
        self.sound_cues.push("upgrade");
        self.status_serial += 1;
        self.check_level();
    }

    fn check_level(&mut self) {
        if self.phase != Phase::Playing || self.xp < self.xp_needed() {
            return;
        }
        self.xp -= self.xp_needed();
        self.level += 1;
        self.phase = Phase::Upgrade;
        let mut pool = vec![
            Upgrade::Overclock,
            Upgrade::Parallelism,
            Upgrade::FineTuning,
            Upgrade::Acquisition,
            Upgrade::Runway,
            Upgrade::OpenWeights,
            Upgrade::LiquidCooling,
            Upgrade::WiderContext,
            founder(self.character).signature,
        ];
        pool.retain(|u| self.upgrades[u.index()] < u.max_rank());
        pool.push(Upgrade::Buyback);
        self.choices.clear();
        if self.level == 2 {
            self.choices
                .extend([Upgrade::WiderContext, founder(self.character).signature]);
            pool.retain(|u| !self.choices.contains(u));
        }
        while !pool.is_empty() && self.choices.len() < 3 {
            let index = self.rng.next() as usize % pool.len();
            self.choices.push(pool.swap_remove(index));
        }
        self.sound_cues.push("level");
        self.status_serial += 1;
    }

    pub fn update(&mut self, input: Input) {
        if self.phase != Phase::Playing {
            return;
        }
        if self
            .live
            .apply_pending(&mut self.edition, &mut self.events_fired, self.time)
        {
            self.status_serial += 1;
        }
        let dt = STEP;
        self.time += dt;
        self.trigger_events();
        let modifiers = self.player_modifiers();
        self.invulnerable = (self.invulnerable - dt).max(0.);
        self.dash_cooldown = (self.dash_cooldown - dt).max(0.);
        self.ability_cooldown = (self.ability_cooldown - dt).max(0.);
        self.dash_time = (self.dash_time - dt).max(0.);
        self.pulse = (self.pulse - dt).max(0.);
        self.shake = (self.shake - dt * 15.).max(0.);
        self.blackout = (self.blackout - dt).max(0.);
        self.banner_time = (self.banner_time - dt).max(0.);
        self.terrain.stream(self.player);
        self.climate = self.terrain.climate_at(self.player);
        let movement = input.movement.normalize_or_zero();
        self.moving = movement.length_squared() > 0.;
        self.facing = self.facing.follow(movement);
        if input.dash && self.dash_cooldown <= 0. {
            self.dash_direction = if self.moving {
                movement
            } else {
                self.facing.direction()
            };
            self.dash_time = 0.17;
            self.dash_cooldown =
                3. * 0.85_f32.powi(self.upgrades[Upgrade::LiquidCooling.index()] as i32);
            self.invulnerable = 0.3;
            self.sound_cues.push("dash");
        }
        let velocity = (if self.dash_time > 0. {
            self.dash_direction * 440.
        } else {
            movement * (104. + 10. * self.upgrades[Upgrade::LiquidCooling.index()] as f32)
        }) * self.climate.movement_multiplier()
            * modifiers.movement;
        self.facing = self.facing.follow(velocity);
        let previous = self.player;
        self.player = self.terrain.move_actor(self.player, velocity * dt, 8.);
        self.terrain.stream(self.player);
        self.update_thermal();
        self.moving = self.player.distance_squared(previous) > 0.0001;
        self.walk_time = if self.moving { self.walk_time + dt } else { 0. };
        if self.dash_time > 0. {
            self.burst(self.player, 1, Color::from_rgba(167, 220, 180, 180));
        }
        if input.ability && self.ability_cooldown <= 0. && !self.ability_locked() {
            let founder = founder(self.character);
            let radius = self.ability_radius();
            self.ability_cooldown = 15.;
            self.pulse = 0.55;
            self.shake = 4.;
            self.sound_cues.push("ability");
            for enemy in &mut self.enemies {
                if enemy.position.distance(self.player) < radius {
                    enemy.hp -= founder.ability_damage * modifiers.damage;
                    enemy.flash = 0.2;
                    enemy.knockback = (enemy.position - self.player).normalize_or_zero() * 250.;
                }
            }
            self.shots
                .retain(|s| !s.kind.hostile() || s.position.distance(self.player) >= radius);
            self.hp = (self.hp + founder.heal).min(self.max_hp);
            self.invulnerable = self.invulnerable.max(founder.shield);
        }
        self.enemies
            .retain(|enemy| enemy.kind.boss() || enemy.position.distance(self.player) < 900.);
        self.pickups
            .retain(|pickup| pickup.position.distance(self.player) < 1100.);
        for enemy in &mut self.enemies {
            if enemy.kind.boss() && enemy.position.distance(self.player) > 800. {
                enemy.position = self.terrain.open_position(
                    self.player + (enemy.position - self.player).normalize_or_zero() * 400.,
                    enemy.radius,
                );
            }
        }
        self.spawn_clock -= dt;
        if !self.boss_spawned && self.spawn_clock <= 0. && self.enemies.len() < 220 {
            let progress = (self.time / self.edition.boss_at_seconds).min(1.2);
            let count = 1 + (progress * 4.) as usize;
            for _ in 0..count {
                let roll = self.rng.unit();
                if self.event_remaining(EventKind::BlueWave) > 0.
                    && roll < EventKind::BlueWave.recipe().inspector_share
                {
                    self.spawn_template(RivalTemplate::Inspector);
                    continue;
                }
                let kind = match roll {
                    r if r < 0.12 => EnemyKind::Bot,
                    r if r < 0.32 => EnemyKind::Engineer,
                    r if r < 0.44 => EnemyKind::Crawler,
                    r if r < 0.58 => EnemyKind::Kimi,
                    r if r < 0.72 => EnemyKind::DeepSeek,
                    r if r < 0.84 => EnemyKind::Glm,
                    r if r < 0.94 => EnemyKind::Politician,
                    _ => EnemyKind::Lawyer,
                };
                self.spawn_enemy(kind);
            }
            self.spawn_clock = (1.7 - progress * 1.12).max(0.38) / self.effective_threat();
        }
        self.update_encounters();
        self.update_enemies();
        self.fire_weapon();
        self.fire_signature();
        self.update_shots();
        self.update_orbits();
        self.remove_dead();
        self.collect();
        self.update_effects();
        // Losing takes priority over progression and victory in the same tick.
        if self.hp <= 0. {
            self.phase = Phase::Lost;
            self.sound_cues.push("lose");
            self.status_serial += 1;
        } else if self.boss_defeated {
            self.phase = Phase::Won;
            self.sound_cues.push("win");
            self.status_serial += 1;
        } else {
            self.check_level();
        }
    }

    fn update_encounters(&mut self) {
        if self.boss_spawned {
            return;
        }
        let kind = if self.time >= self.edition.duration_seconds {
            self.boss_spawned = true;
            self.shots.retain(|shot| !shot.kind.hostile());
            EnemyKind::Bubble
        } else if self.time >= self.next_miniboss
            && self.time > 40.
            && self.time < self.edition.duration_seconds - 15.
            && !self.enemies.iter().any(|enemy| enemy.kind.miniboss())
        {
            let Some(kind) = self.minibosses.pop() else {
                return;
            };
            kind
        } else {
            return;
        };
        self.spawn_enemy(kind);
        self.banner = kind.label().into();
        self.banner_time = 6.;
        self.sound_cues.push("boss");
        self.status_serial += 1;
    }

    fn trigger_events(&mut self) {
        for i in 0..self.edition.events.len() {
            if self.events_fired[i] || self.time < self.edition.events[i].at_seconds {
                continue;
            }
            self.events_fired[i] = true;
            let event = self.edition.events[i].clone();
            self.banner = event.title;
            self.banner_time = 5.;
            self.status_serial += 1;
            self.sound_cues.push("event");
            let recipe = event.kind.recipe();
            if recipe.blackout {
                self.blackout = self.event_remaining(event.kind);
            }
            if let Some(drops) = &recipe.drops {
                for _ in 0..drops.count {
                    let angle = self.rng.range(0., std::f32::consts::TAU);
                    let position =
                        self.player + Vec2::from_angle(angle) * self.rng.range(50., 120.);
                    self.pickups.push(Pickup {
                        asset_pack: recipe.asset_pack,
                        position: self.terrain.open_position(position, 8.),
                        value: drops.value,
                        cash: drops.cash,
                        age: 0.,
                    });
                }
            }
            if let Some(reinforcements) = &recipe.reinforcements {
                for _ in 0..reinforcements.count {
                    self.spawn_template(reinforcements.template);
                }
            }
        }
    }

    fn spawn_enemy(&mut self, kind: EnemyKind) {
        self.spawn_actor(kind, AssetPack::DataCenter);
    }

    fn spawn_template(&mut self, template: RivalTemplate) {
        let (kind, pack) = match template {
            RivalTemplate::Investor => (EnemyKind::Investor, AssetPack::DataCenter),
            RivalTemplate::Inspector => (EnemyKind::Lawyer, AssetPack::BlueWave),
            RivalTemplate::Auditor => (EnemyKind::Lawyer, AssetPack::DataCenter),
        };
        self.spawn_actor(kind, pack);
    }

    fn spawn_actor(&mut self, kind: EnemyKind, asset_pack: AssetPack) {
        if self.enemies.len() >= 220 && !kind.boss() {
            return;
        }
        let angle = self.rng.range(0., std::f32::consts::TAU);
        let direction = Vec2::from_angle(angle);
        let spawn_distance =
            (350. / direction.x.abs().max(0.01)).min(210. / direction.y.abs().max(0.01));
        let (hp, speed, radius) = match kind {
            EnemyKind::Bot => (17., 29., 10.),
            EnemyKind::Crawler => (12., 55., 10.),
            EnemyKind::Lawyer => (34., 24., 10.),
            EnemyKind::Investor => (52., 35., 11.),
            EnemyKind::Bernie => (270., 24., 20.),
            EnemyKind::Aoc => (250., 30., 18.),
            EnemyKind::Spy => (220., 46., 16.),
            EnemyKind::Bubble => (1800., 32., 34.),
            EnemyKind::Engineer => (26., 42., 10.),
            EnemyKind::Politician => (38., 23., 10.),
            EnemyKind::Glm => (48., 22., 12.),
            EnemyKind::Kimi => (19., 52., 10.),
            EnemyKind::DeepSeek => (30., 30., 12.),
        };
        let position = self
            .terrain
            .open_position(self.player + direction * spawn_distance, radius);
        let hp = hp * (1. + (self.time / self.edition.duration_seconds).min(1.) * 0.65);
        self.enemies.push(Enemy {
            id: self.next_enemy_id,
            asset_pack,
            reward_pressure: self.effective_threat(),
            position,
            facing: Facing::Down.follow(self.player - position),
            moving: false,
            walk_time: 0.,
            hp,
            max_hp: hp,
            kind,
            speed,
            radius,
            attack: self.rng.range(1.5, 3.),
            flash: 0.,
            knockback: Vec2::ZERO,
            dead: false,
        });
        self.next_enemy_id += 1;
    }

    fn update_enemies(&mut self) {
        let mut hostile_shots = Vec::new();
        let existing_shots = self.shots.iter().filter(|shot| shot.kind.hostile()).count();
        let player = self.player;
        let time = self.time;
        let pressure = 0.93 + 0.07 * self.effective_threat();
        let mut contact = false;
        // Spatial buckets keep local separation bounded as waves grow.
        let mut grid: HashMap<(i32, i32), Vec<Vec2>> = HashMap::new();
        for e in &self.enemies {
            grid.entry(cell(e.position)).or_default().push(e.position);
        }
        for enemy in &mut self.enemies {
            if enemy.hp <= 0. {
                continue;
            }
            enemy.flash = (enemy.flash - STEP).max(0.);
            let delta = player - enemy.position;
            let distance = delta.length();
            let mut attack = enemy.kind.attack(enemy.asset_pack);
            let radial = enemy.kind == EnemyKind::Bernie
                || (enemy.kind == EnemyKind::Bubble && (time as u32 / 3).is_multiple_of(2));
            if enemy.kind == EnemyKind::Bubble {
                let rage = 1. - enemy.hp / enemy.max_hp;
                attack.cooldown *= 1. - rage * 0.4;
                attack.speed *= 1. + rage * 0.35;
                if !radial {
                    attack.count = 7;
                    attack.spread = 0.14;
                }
            }
            if distance < attack.range {
                enemy.attack -= STEP;
            } else {
                enemy.attack = (enemy.attack - STEP).max(WINDUP);
            }
            let mut direction = delta.normalize_or_zero();
            if distance < enemy.kind.standoff() {
                direction *= -0.25;
            }
            if matches!(enemy.kind, EnemyKind::Kimi | EnemyKind::Spy) {
                direction += vec2(-direction.y, direction.x) * (time * 2.2).sin() * 0.65;
                direction = direction.normalize_or_zero();
            }
            let (cx, cy) = cell(enemy.position);
            let mut separation = Vec2::ZERO;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if let Some(neighbors) = grid.get(&(cx + dx, cy + dy)) {
                        for other in neighbors.iter().take(20) {
                            let away = enemy.position - *other;
                            let length = away.length();
                            if length > 0.01 && length < enemy.radius * 1.8 {
                                separation += away / length * 20.;
                            }
                        }
                    }
                }
            }
            let velocity = direction * enemy.speed * pressure + separation + enemy.knockback;
            let previous = enemy.position;
            enemy.position =
                self.terrain
                    .move_actor(enemy.position, velocity * STEP, enemy.radius * 0.65);
            if enemy.speed > 0.
                && enemy.position.distance_squared(previous)
                    < velocity.length_squared() * STEP * STEP * 0.1
            {
                enemy.position = self.terrain.move_actor(
                    enemy.position,
                    vec2(-direction.y, direction.x) * enemy.speed * STEP,
                    enemy.radius * 0.65,
                );
            }
            let displacement = enemy.position - previous;
            enemy.moving = displacement.length_squared() > 0.0001;
            enemy.facing = enemy.facing.follow(displacement);
            enemy.walk_time = if enemy.moving {
                enemy.walk_time + STEP
            } else {
                0.
            };
            enemy.knockback *= 0.86;
            if distance < enemy.radius + 8. && enemy.hp > 0. {
                contact = true;
            }
            if enemy.attack <= 0. && distance < attack.range {
                let boss = enemy.kind.boss();
                let budget = MAX_HOSTILE_SHOTS - if boss { 0 } else { BOSS_SHOT_RESERVE };
                if existing_shots + hostile_shots.len() + attack.count <= budget {
                    for i in 0..attack.count {
                        let angle = if radial {
                            i as f32 * attack.spread + time * 0.15
                        } else {
                            delta.to_angle()
                                + (i as f32 - (attack.count - 1) as f32 / 2.) * attack.spread
                        };
                        let direction = Vec2::from_angle(angle);
                        let position = enemy.position + direction * (enemy.radius + 5.);
                        hostile_shots.push(Shot::new(
                            position,
                            direction * attack.speed,
                            attack.damage,
                            attack.range / attack.speed + 0.5,
                            attack.projectile,
                        ));
                    }
                }
                if enemy.kind == EnemyKind::DeepSeek {
                    enemy.knockback = delta.normalize_or_zero() * 310.;
                }
                enemy.attack = attack.cooldown;
            }
        }
        self.shots.extend(hostile_shots);
        if contact {
            self.hurt(12.);
        }
    }

    fn fire_weapon(&mut self) {
        let modifiers = self.player_modifiers();
        self.shot_clock -= STEP * modifiers.attack_rate;
        if self.shot_clock > 0. {
            return;
        }
        let area = self.attack_radius_multiplier();
        let Some(target) = self.closest_target(300. * area) else {
            return;
        };
        let direction = (target - self.player).normalize_or_zero();
        let founder = founder(self.character);
        let count = founder.projectiles + self.upgrades[Upgrade::Parallelism.index()];
        if !self.has_shot_capacity(count as usize) {
            return;
        }
        let damage = founder.damage * self.weapon_damage_multiplier();
        for i in 0..count {
            let spread = i as f32 - (count - 1) as f32 / 2.;
            let angle = direction.to_angle() + spread * 0.15;
            let position =
                self.player + direction * 18. + vec2(-direction.y, direction.x) * spread * 20.;
            let mut shot = Shot::new(
                position,
                Vec2::from_angle(angle) * founder.shot_speed,
                damage,
                1.3 * area,
                ProjectileKind::Founder,
            );
            shot.radius *= area;
            self.shots.push(shot);
        }
        self.shot_clock = founder.shot_cooldown / self.weapon_speed_multiplier();
        self.sound_cues.push("shoot");
    }

    fn closest_target(&self, range: f32) -> Option<Vec2> {
        self.enemies
            .iter()
            .filter(|enemy| enemy.hp > 0. && enemy.position.distance(self.player) <= range)
            .min_by(|a, b| {
                a.position
                    .distance_squared(self.player)
                    .total_cmp(&b.position.distance_squared(self.player))
            })
            .map(|enemy| enemy.position)
    }

    fn has_shot_capacity(&self, count: usize) -> bool {
        self.shots
            .iter()
            .filter(|shot| !shot.kind.hostile())
            .count()
            + count
            <= MAX_FRIENDLY_SHOTS
    }

    fn fire_signature(&mut self) {
        let rank = self.signature_rank();
        if rank == 0 {
            return;
        }
        self.signature_cooldown -= STEP * self.player_modifiers().attack_rate;
        if self.signature_cooldown > 0. {
            return;
        }
        use ProjectileKind::*;
        let (kind, cooldown, speed, damage, radius, pierce, ttl, range, count) =
            match founder(self.character).signature {
                Upgrade::LeatherJacket => (LeatherJacket, 2.2, 280., 24., 10., 5, 2.4, 300., 1),
                Upgrade::CapitalInjection => (Capital, 2.8, 220., 32., 8., 1, 1.8, 300., 1),
                Upgrade::SafetyRails => (SafetyRail, 3.4, 135., 20., 18., 4, 1.8, 240., 2),
                Upgrade::Cybertruck => (Cybertruck, 2.6, 330., 38., 13., 8, 1.4, 360., 1),
                _ => unreachable!("Each founder has a signature weapon"),
            };
        let area = self.attack_radius_multiplier();
        let Some(target) = self.closest_target(range * area) else {
            return;
        };
        if !self.has_shot_capacity(count) {
            return;
        }
        let damage = damage * (1. + (rank - 1) as f32 * 0.4) * self.weapon_damage_multiplier();
        let direction = (target - self.player).normalize_or_zero();
        for i in 0..count {
            let spread = (i as f32 - (count - 1) as f32 / 2.) * 30.;
            let position = self.player + direction * 18. + vec2(-direction.y, direction.x) * spread;
            let mut shot = Shot::new(position, direction * speed, damage, ttl * area, kind);
            shot.radius = radius * area;
            shot.pierce = pierce + rank - 1;
            if kind == Capital {
                shot.blast_radius = (55. + (rank - 1) as f32 * 10.) * area;
            }
            self.shots.push(shot);
        }
        self.signature_cooldown = cooldown / self.weapon_speed_multiplier();
        self.sound_cues.push("shoot");
    }

    fn update_shots(&mut self) {
        let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        let mut max_radius: f32 = 0.;
        for (i, enemy) in self.enemies.iter().enumerate() {
            grid.entry(cell(enemy.position)).or_default().push(i);
            max_radius = max_radius.max(enemy.radius);
        }
        let mut hits = Vec::new();
        let mut explosions = Vec::new();
        let mut damage_taken: f32 = 0.;
        for shot in &mut self.shots {
            if shot.ttl <= 0. {
                continue;
            }
            if shot.kind == ProjectileKind::LeatherJacket && shot.age > 1.05 * shot.radius / 10. {
                shot.velocity = (self.player - shot.position).normalize_or_zero() * 280.;
                if shot.position.distance(self.player) < shot.radius + 8. {
                    shot.ttl = 0.;
                    continue;
                }
            }
            shot.previous = shot.position;
            shot.position += shot.velocity * STEP;
            shot.ttl -= STEP;
            shot.age += STEP;
            if shot.ttl <= 0. && shot.blast_radius > 0. {
                explosions.push((shot.position, shot.blast_radius, shot.damage));
            }
        }
        let shields: Vec<_> = self
            .shots
            .iter()
            .filter(|shot| shot.kind == ProjectileKind::SafetyRail && shot.ttl > 0.)
            .map(|shot| (shot.position, shot.radius))
            .collect();
        for shot in &mut self.shots {
            if shot.ttl <= 0. {
                continue;
            }
            if shot.kind.hostile() {
                if shields.iter().any(|(position, radius)| {
                    segment_distance(*position, shot.previous, shot.position) < radius + shot.radius
                }) {
                    shot.ttl = 0.;
                } else if segment_distance(self.player, shot.previous, shot.position) < 10. {
                    damage_taken = damage_taken.max(shot.damage);
                    shot.ttl = 0.;
                }
                continue;
            }
            let padding = Vec2::splat(shot.radius + max_radius);
            let (min_x, min_y) = cell(shot.previous.min(shot.position) - padding);
            let (max_x, max_y) = cell(shot.previous.max(shot.position) + padding);
            'collision: for x in min_x..=max_x {
                for y in min_y..=max_y {
                    if let Some(neighbors) = grid.get(&(x, y)) {
                        for &index in neighbors {
                            let enemy = &mut self.enemies[index];
                            if enemy.hp > 0.
                                && !shot.hit_ids.contains(&enemy.id)
                                && segment_distance(enemy.position, shot.previous, shot.position)
                                    < enemy.radius + shot.radius
                            {
                                if shot.blast_radius > 0. {
                                    explosions.push((
                                        shot.position,
                                        shot.blast_radius,
                                        shot.damage,
                                    ));
                                    shot.ttl = 0.;
                                    break 'collision;
                                }
                                enemy.hp -= shot.damage;
                                enemy.flash = 0.1;
                                enemy.knockback += shot.velocity.normalize_or_zero()
                                    * if shot.kind == ProjectileKind::Cybertruck {
                                        130.
                                    } else {
                                        38.
                                    };
                                shot.hit_ids.push(enemy.id);
                                shot.pierce -= 1;
                                hits.push((enemy.position, shot.damage));
                                if shot.pierce == 0 {
                                    shot.ttl = 0.;
                                    break 'collision;
                                }
                            }
                        }
                    }
                }
            }
        }
        for (position, radius, damage) in explosions {
            for enemy in &mut self.enemies {
                if enemy.hp > 0. && enemy.position.distance(position) <= radius + enemy.radius {
                    enemy.hp -= damage;
                    enemy.flash = 0.15;
                    enemy.knockback += (enemy.position - position).normalize_or_zero() * 75.;
                    hits.push((enemy.position, damage));
                }
            }
            if self.blasts.len() < 32 {
                self.blasts.push(Blast {
                    position,
                    radius,
                    ttl: 0.35,
                });
            }
            self.burst(position, 14, Color::from_rgba(235, 201, 112, 255));
        }
        self.shots.retain(|s| s.ttl > 0.);
        for (position, damage) in hits {
            self.burst(position, 3, Color::from_rgba(223, 218, 164, 255));
            if self.floaters.len() < 70 {
                self.floaters.push(Floater {
                    position: position + vec2(0., -18.),
                    text: format!("{}", damage as u32),
                    color: Color::from_rgba(224, 226, 188, 255),
                    ttl: 0.6,
                });
            }
        }
        if damage_taken > 0. {
            self.hurt(damage_taken);
        }
    }

    fn update_orbits(&mut self) {
        self.orbit_clock -= STEP;
        if self.orbit_clock > 0. {
            return;
        }
        self.orbit_clock = 0.24;
        let positions = self.orbit_positions();
        let radius = 9. * self.attack_radius_multiplier();
        let damage = 16. * self.player_modifiers().damage;
        for enemy in &mut self.enemies {
            if positions
                .iter()
                .any(|p| p.distance(enemy.position) < enemy.radius + radius)
            {
                enemy.hp -= damage;
                enemy.flash = 0.12;
            }
        }
    }

    fn remove_dead(&mut self) {
        let mut deaths = Vec::new();
        for enemy in &mut self.enemies {
            if enemy.hp <= 0. && !enemy.dead {
                enemy.dead = true;
                deaths.push((
                    enemy.position,
                    enemy.kind,
                    enemy.asset_pack,
                    enemy.reward_pressure,
                ));
            }
        }
        self.enemies.retain(|e| !e.dead);
        for (position, kind, asset_pack, reward_pressure) in deaths {
            self.kills += 1;
            let value = match kind {
                EnemyKind::Investor => 4.,
                EnemyKind::Lawyer | EnemyKind::Politician | EnemyKind::Glm => 3.,
                EnemyKind::DeepSeek | EnemyKind::Engineer => 2.,
                EnemyKind::Bernie | EnemyKind::Aoc | EnemyKind::Spy => 24.,
                EnemyKind::Bubble => 50.,
                _ => 1.5,
            };
            // The reward target offsets the extra spawn density in harsh editions.
            let value = value * self.edition.xp_multiplier / reward_pressure;
            self.pickups.push(Pickup {
                asset_pack,
                position,
                value,
                cash: false,
                age: 0.,
            });
            if self.rng.unit() < 0.09 {
                self.pickups.push(Pickup {
                    asset_pack: AssetPack::DataCenter,
                    position: position + vec2(8., 0.),
                    value: 1.,
                    cash: true,
                    age: 0.,
                });
            }
            self.burst(position, 9, Color::from_rgba(161, 194, 125, 255));
            if kind == EnemyKind::Bubble {
                self.boss_defeated = true;
            }
            if kind.miniboss() {
                self.next_miniboss = self.time + self.rng.range(18., 30.);
                self.pickups.push(Pickup {
                    asset_pack: AssetPack::DataCenter,
                    position: position + vec2(-10., 0.),
                    value: 2.,
                    cash: true,
                    age: 0.,
                });
            }
        }
        // Merge old loot rather than allowing an unbounded entity count.
        while self.pickups.len() > 450 {
            let old = self.pickups.remove(0);
            if let Some(target) = self.pickups.iter_mut().find(|p| p.cash == old.cash) {
                target.value += old.value;
            }
        }
    }

    fn collect(&mut self) {
        let radius = self.pickup_radius();
        let mut collected = 0;
        for pickup in &mut self.pickups {
            pickup.age += STEP;
            let delta = self.player - pickup.position;
            let distance = delta.length();
            if distance < radius {
                pickup.position +=
                    delta.normalize_or_zero() * (100. + (radius - distance) * 5.) * STEP;
            }
            if distance < 12. {
                if pickup.cash {
                    self.hp = (self.hp + pickup.value * 10.).min(self.max_hp);
                } else {
                    self.xp += pickup.value;
                    self.collected += pickup.value.ceil() as u32;
                }
                pickup.value = 0.;
                collected += 1;
            }
        }
        self.pickups.retain(|p| p.value > 0.);
        if collected > 0 {
            self.sound_cues.push("pickup");
        }
    }

    fn hurt(&mut self, damage: f32) {
        if self.invulnerable > 0. || self.hp <= 0. {
            return;
        }
        self.hp -= damage;
        self.invulnerable = 0.8;
        self.shake = 3.;
        self.sound_cues.push("hit");
        self.burst(self.player, 10, Color::from_rgba(222, 117, 110, 255));
    }

    fn burst(&mut self, position: Vec2, count: usize, color: Color) {
        for _ in 0..count {
            if self.particles.len() >= 450 {
                break;
            }
            let angle = self.rng.range(0., std::f32::consts::TAU);
            let speed = self.rng.range(12., 60.);
            self.particles.push(Particle {
                position,
                velocity: Vec2::from_angle(angle) * speed,
                color,
                ttl: self.rng.range(0.2, 0.55),
                size: self.rng.range(1., 3.),
            });
        }
    }

    fn update_effects(&mut self) {
        for blast in &mut self.blasts {
            blast.ttl -= STEP;
        }
        self.blasts.retain(|blast| blast.ttl > 0.);
        for p in &mut self.particles {
            p.position += p.velocity * STEP;
            p.velocity *= 0.96;
            p.ttl -= STEP;
        }
        self.particles.retain(|p| p.ttl > 0.);
        for f in &mut self.floaters {
            f.position.y -= 19. * STEP;
            f.ttl -= STEP;
        }
        self.floaters.retain(|f| f.ttl > 0.);
    }
}

fn cell(position: Vec2) -> (i32, i32) {
    (
        (position.x / 40.).floor() as i32,
        (position.y / 40.).floor() as i32,
    )
}

fn segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let t = ((point - start).dot(segment) / segment.length_squared().max(0.0001)).clamp(0., 1.);
    point.distance(start + segment * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weapon_world(character: u32, positions: &[Vec2]) -> World {
        let mut edition = Edition::demo();
        edition.events.clear();
        let mut world = World::new(edition, character);
        world.pickups.clear();
        world.spawn_clock = 1000.;
        for position in positions {
            world.spawn_enemy(EnemyKind::Lawyer);
            let enemy = world.enemies.last_mut().unwrap();
            enemy.position = world.player + *position;
            enemy.hp = 1000.;
            enemy.speed = 0.;
            enemy.attack = 1000.;
        }
        world
    }

    #[test]
    fn upgrade_choices_offer_universal_area_and_only_the_current_founders_signature() {
        let signatures: Vec<_> = FOUNDERS.iter().map(|founder| founder.signature).collect();
        for character in 0..4 {
            let mut world = weapon_world(character, &[]);
            world.xp = world.xp_needed();
            world.check_level();
            let signature = founder(character).signature;
            assert!(world.choices.contains(&Upgrade::WiderContext));
            assert!(world.choices.contains(&signature));
            assert_eq!(world.choices.len(), 3);
            world.choose(
                world
                    .choices
                    .iter()
                    .position(|choice| *choice == signature)
                    .unwrap(),
            );
            assert_eq!(world.signature_rank(), 1);
            world.upgrades[signature.index()] = signature.max_rank();
            world.upgrades[Upgrade::WiderContext.index()] = Upgrade::WiderContext.max_rank();
            for _ in 0..30 {
                world.xp = world.xp_needed();
                world.check_level();
                assert!(!world.choices.contains(&Upgrade::WiderContext));
                assert!(world
                    .choices
                    .iter()
                    .all(|choice| !signatures.contains(choice)));
                world.choose(0);
            }
        }
    }

    #[test]
    fn wider_context_extends_actual_targeting_special_hits_and_projectile_collision_for_every_founder(
    ) {
        for character in 0..4 {
            let mut world = weapon_world(character, &[vec2(330., 0.)]);
            world.fire_weapon();
            assert!(world.shots.is_empty());
            let pickup_radius = world.pickup_radius();
            world.upgrades[Upgrade::WiderContext.index()] = 1;
            world.fire_weapon();
            assert_eq!(world.shots.len(), founder(character).projectiles as usize);
            assert!((world.shots[0].radius - 6.).abs() < 0.001);
            assert_eq!(world.pickup_radius(), pickup_radius);
            world.shots.truncate(1);
            world.shots[0].position = world.enemies[0].position + vec2(0., 15.5);
            world.shots[0].velocity = Vec2::ZERO;
            world.update_shots();
            assert!(world.enemies[0].hp < 1000.);
            for (rank, hits) in [(0, false), (1, true)] {
                let distance = founder(character).ability_radius + 10.;
                let mut special = weapon_world(character, &[vec2(distance, 0.)]);
                special.upgrades[Upgrade::WiderContext.index()] = rank;
                special.shot_clock = 1000.;
                special.update(Input {
                    ability: true,
                    ..Input::default()
                });
                assert_eq!(special.enemies[0].hp < 1000., hits);
            }
        }
    }

    #[test]
    fn signatures_stack_with_starters_respect_stock_modifiers_pause_and_reset() {
        for (character, kind) in [
            (0, ProjectileKind::LeatherJacket),
            (1, ProjectileKind::Capital),
            (2, ProjectileKind::SafetyRail),
            (3, ProjectileKind::Cybertruck),
        ] {
            let mut world = weapon_world(character, &[vec2(120., 0.)]);
            world.fire_signature();
            assert!(world.shots.is_empty());
            world.upgrades[founder(character).signature.index()] = 1;
            world.fire_weapon();
            world.fire_signature();
            assert_eq!(
                world
                    .shots
                    .iter()
                    .filter(|shot| shot.kind == ProjectileKind::Founder)
                    .count(),
                founder(character).projectiles as usize
            );
            let base_damage = world
                .shots
                .iter()
                .find(|shot| shot.kind == kind)
                .unwrap()
                .damage;
            assert!(world.shots.iter().all(|shot| !shot.kind.hostile()));
            world.shots.clear();
            world.edition.events.push(crate::config::WorldEvent {
                at_seconds: 0.,
                kind: EventKind::AiStockRally,
                title: "RALLY".into(),
            });
            world.events_fired.push(false);
            world.trigger_events();
            world.signature_cooldown = 0.;
            world.fire_signature();
            assert!((world.shots[0].damage / base_damage - 1.25).abs() < 0.001);
            let frozen = (
                world.time,
                world.signature_cooldown,
                world.shots[0].position,
                world.shots[0].age,
            );
            world.toggle_pause();
            for _ in 0..120 {
                world.update(Input::default());
            }
            assert_eq!(
                (
                    world.time,
                    world.signature_cooldown,
                    world.shots[0].position,
                    world.shots[0].age
                ),
                frozen
            );
            let fresh = World::new(world.edition.clone(), character);
            assert_eq!(fresh.signature_rank(), 0);
            assert_eq!(fresh.attack_radius_multiplier(), 1.);
            assert!(fresh.shots.is_empty());
        }
    }

    #[test]
    fn cybertruck_pierces_distinct_rivals_without_repeat_hits_when_enemy_indices_change() {
        let mut world = weapon_world(3, &[vec2(24., 0.), vec2(30., 0.)]);
        world.upgrades[Upgrade::Cybertruck.index()] = 1;
        world.fire_signature();
        let damage = world.shots[0].damage;
        world.update_shots();
        assert!(world.enemies.iter().all(|enemy| enemy.hp == 1000. - damage));
        world.update_shots();
        assert!(world.enemies.iter().all(|enemy| enemy.hp == 1000. - damage));
        world.enemies.remove(0);
        world.spawn_enemy(EnemyKind::Lawyer);
        world.enemies[1].position = world.shots[0].position;
        world.enemies[1].hp = 1000.;
        world.update_shots();
        assert!(world.enemies.iter().all(|enemy| enemy.hp == 1000. - damage));
        assert_eq!(world.shots[0].hit_ids.len(), 3);
    }

    #[test]
    fn cash_explodes_once_and_area_upgrades_reach_more_rivals() {
        for (rank, outer_hit) in [(0, false), (1, true)] {
            let mut world = weapon_world(1, &[vec2(100., 0.), vec2(100., 70.), vec2(100., 130.)]);
            world.upgrades[Upgrade::CapitalInjection.index()] = 1;
            world.upgrades[Upgrade::WiderContext.index()] = rank;
            world.fire_signature();
            let damage = world.shots[0].damage;
            world.shots[0].position = world.enemies[0].position;
            world.shots[0].velocity = Vec2::ZERO;
            world.update_shots();
            assert_eq!(world.enemies[0].hp, 1000. - damage);
            assert_eq!(world.enemies[1].hp < 1000., outer_hit);
            assert_eq!(world.enemies[2].hp, 1000.);
            world.update_shots();
            assert_eq!(world.enemies[0].hp, 1000. - damage);
            assert_eq!(world.blasts.len(), 1);
        }
    }

    #[test]
    fn safety_rails_intercept_hostile_shots_and_jackets_return_without_hurting_the_player() {
        let mut world = weapon_world(2, &[vec2(30., 15.)]);
        world.upgrades[Upgrade::SafetyRails.index()] = 1;
        world.fire_signature();
        world.shots.push(Shot::new(
            world.player + vec2(9., 0.),
            Vec2::ZERO,
            20.,
            1.,
            ProjectileKind::Bolt,
        ));
        world.shots.push(Shot::new(
            world.player,
            Vec2::ZERO,
            20.,
            1.,
            ProjectileKind::Founder,
        ));
        world.update_shots();
        assert!(world.shots.iter().all(|shot| !shot.kind.hostile()));
        assert!(world
            .shots
            .iter()
            .any(|shot| shot.kind == ProjectileKind::Founder));
        assert!(world.enemies[0].hp < 1000.);
        assert_eq!(world.hp, 100.);

        let mut jacket = weapon_world(0, &[vec2(200., 0.)]);
        jacket.upgrades[Upgrade::LeatherJacket.index()] = 1;
        jacket.fire_signature();
        for _ in 0..75 {
            jacket.update_shots();
        }
        let shot = &jacket.shots[0];
        assert!(shot.velocity.dot(jacket.player - shot.position) > 0.);
        for _ in 0..100 {
            jacket.update_shots();
        }
        assert!(jacket.shots.is_empty());
        assert_eq!(jacket.hp, 100.);
    }

    #[test]
    fn friendly_weapon_budget_and_large_projectile_collision_span_multiple_grid_cells() {
        let mut world = weapon_world(3, &[vec2(65., 0.)]);
        let mut shot = Shot::new(
            world.player,
            Vec2::ZERO,
            20.,
            1.,
            ProjectileKind::Cybertruck,
        );
        shot.radius = 60.;
        world.shots.push(shot);
        world.update_shots();
        assert_eq!(world.enemies[0].hp, 980.);
        world.upgrades[Upgrade::Cybertruck.index()] = 3;
        for _ in 0..100 {
            world.shot_clock = 0.;
            world.signature_cooldown = 0.;
            world.fire_weapon();
            world.fire_signature();
        }
        assert!(world.shots.len() <= MAX_FRIENDLY_SHOTS);
    }

    #[test]
    fn every_rival_emits_its_contextual_attack_including_inspectors() {
        use EnemyKind::*;
        for (enemy, projectile, count) in [
            (Bot, ProjectileKind::Bolt, 1),
            (Crawler, ProjectileKind::Scrap, 2),
            (Lawyer, ProjectileKind::Subpoena, 1),
            (Investor, ProjectileKind::Coin, 3),
            (Bernie, ProjectileKind::Legislation, 12),
            (Engineer, ProjectileKind::Wrench, 1),
            (Politician, ProjectileKind::Legislation, 2),
            (Glm, ProjectileKind::Flag, 3),
            (Kimi, ProjectileKind::Flag, 1),
            (DeepSeek, ProjectileKind::Flag, 2),
        ] {
            let mut world = World::new(Edition::demo(), 0);
            world.spawn_enemy(enemy);
            world.enemies[0].position = world.player + vec2(80., 0.);
            world.enemies[0].attack = 0.;
            world.update_enemies();
            assert_eq!(world.shots.len(), count, "{enemy:?}");
            assert!(world
                .shots
                .iter()
                .all(|shot| shot.kind == projectile && shot.velocity.length() > 80.));
            if enemy != Bernie {
                assert!(world.shots.iter().all(|shot| shot.velocity.x < 0.));
            }
            world.update_enemies();
            assert_eq!(world.shots.len(), count);
        }
        let mut world = World::new(Edition::demo(), 0);
        world.spawn_template(RivalTemplate::Inspector);
        world.enemies[0].position = world.player + vec2(80., 0.);
        world.enemies[0].attack = 0.;
        world.update_enemies();
        assert_eq!(world.shots[0].kind, ProjectileKind::Citation);
    }

    #[test]
    fn attacks_require_warning_on_entry_and_dead_rivals_cannot_fire() {
        let mut world = World::new(Edition::demo(), 0);
        world.spawn_enemy(EnemyKind::Engineer);
        world.enemies[0].position = world.player + vec2(500., 0.);
        world.enemies[0].speed = 0.;
        for _ in 0..600 {
            world.update_enemies();
        }
        assert!(world.shots.is_empty());
        assert_eq!(world.enemies[0].attack, WINDUP);
        world.enemies[0].position = world.player + vec2(80., 0.);
        for _ in 0..30 {
            world.update_enemies();
        }
        assert!(world.shots.is_empty());
        for _ in 0..12 {
            world.update_enemies();
        }
        assert_eq!(world.shots.len(), 1);
        world.enemies[0].hp = 0.;
        world.enemies[0].attack = 0.;
        world.update_enemies();
        assert_eq!(world.shots.len(), 1);
    }

    #[test]
    fn projectile_budget_reserves_boss_volley_and_expired_attacks_are_removed() {
        let mut world = World::new(Edition::demo(), 0);
        for _ in 0..220 {
            world.spawn_enemy(EnemyKind::Glm);
        }
        world.spawn_enemy(EnemyKind::Bernie);
        for enemy in &mut world.enemies {
            enemy.position = world.player + vec2(100., 0.);
            enemy.speed = 0.;
            enemy.attack = 0.;
        }
        world.update_enemies();
        assert!(world.shots.len() <= MAX_HOSTILE_SHOTS);
        assert_eq!(
            world
                .shots
                .iter()
                .filter(|shot| shot.kind == ProjectileKind::Legislation)
                .count(),
            12
        );
        for enemy in &mut world.enemies {
            enemy.attack = 0.;
        }
        world.update_enemies();
        assert!(world.shots.len() <= MAX_HOSTILE_SHOTS);
        for shot in &mut world.shots {
            shot.ttl = STEP / 2.;
        }
        world.update_shots();
        assert!(world.shots.is_empty());
    }

    #[test]
    fn thermal_zones_slow_motion_ramp_damage_and_recover_without_advancing_while_paused() {
        let mut world = World::new(Edition::demo(), 0);
        world.pickups.clear();
        world.spawn_clock = 1000.;
        let walk = Input {
            movement: Vec2::X,
            ..Input::default()
        };
        let start = world.player;
        world.update(walk);
        let normal_step = world.player.x - start.x;
        world.player = vec2(-192., 192.);
        let start = world.player;
        world.update(walk);
        assert_eq!(world.climate, Climate::Cold);
        assert!(((world.player.x - start.x) / normal_step - 0.62).abs() < 0.001);
        assert_eq!(world.hp, 100.);
        world.player = vec2(576., 192.);
        world.invulnerable = 100.;
        for _ in 0..60 {
            world.update(Input::default());
        }
        let first = 100. - world.hp;
        let hp = world.hp;
        for _ in 0..60 {
            world.update(Input::default());
        }
        let second = hp - world.hp;
        let hp = world.hp;
        for _ in 0..60 {
            world.update(Input::default());
        }
        let third = hp - world.hp;
        assert!(first > 0. && second > first && third > second);
        let frozen = (world.hp, world.heat_exposure, world.time);
        for phase in [Phase::Paused, Phase::Upgrade, Phase::Menu] {
            world.phase = phase;
            for _ in 0..120 {
                world.update(walk);
            }
            assert_eq!((world.hp, world.heat_exposure, world.time), frozen);
        }
        world.phase = Phase::Playing;
        world.player = START;
        for _ in 0..120 {
            world.update(Input::default());
        }
        assert_eq!(world.hp, frozen.0);
        assert_eq!(world.heat_exposure, 0.);
        assert_eq!(world.heat_damage_per_second(), 0.);
        world.player = vec2(576., 192.);
        world.update(Input::default());
        assert_eq!(world.hp, frozen.0);
        world.hp = 0.01;
        world.heat_exposure = 5.;
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Lost);
    }

    #[test]
    fn four_starters_have_independent_bursts_and_specials() {
        for (id, shots, damage) in [(0, 1, 22.), (1, 2, 14.), (2, 1, 32.), (3, 3, 10.)] {
            let mut world = World::new(Edition::demo(), id);
            world.pickups.clear();
            world.spawn_enemy(EnemyKind::Glm);
            world.enemies[0].position = world.player + vec2(100., 0.);
            world.fire_weapon();
            assert_eq!(world.shots.len(), shots);
            assert!(world
                .shots
                .iter()
                .all(|shot| shot.damage == damage && !shot.kind.hostile()));
            world.upgrades[Upgrade::Parallelism.index()] = 1;
            world.shot_clock = 0.;
            world.shots.clear();
            world.fire_weapon();
            assert_eq!(world.shots.len(), shots + 1);
            world.hp = 50.;
            world.update(Input {
                ability: true,
                ..Input::default()
            });
            assert_eq!(world.hp, if id == 1 { 66. } else { 50. });
            assert_eq!(world.invulnerable, if id == 2 { 2.5 } else { 0. });
            assert!(world.ability_cooldown > 14.);
        }
        assert_eq!(World::new(Edition::demo(), u32::MAX).character, 3);
    }

    #[test]
    fn rival_models_have_fan_flank_and_charge_behaviors() {
        for (kind, count) in [(EnemyKind::Glm, 3), (EnemyKind::Politician, 2)] {
            let mut world = World::new(Edition::demo(), 0);
            world.spawn_enemy(kind);
            world.enemies[0].position = world.player + vec2(120., 0.);
            world.enemies[0].attack = 0.;
            world.update_enemies();
            assert_eq!(world.shots.len(), count);
            assert!(world.shots.iter().all(|shot| shot.kind.hostile()));
            assert!(world.shots[0].velocity.y > 0. && world.shots[count - 1].velocity.y < 0.);
        }
        let mut world = World::new(Edition::demo(), 0);
        world.spawn_enemy(EnemyKind::DeepSeek);
        world.enemies[0].position = world.player + vec2(120., 0.);
        world.enemies[0].attack = 0.;
        world.update_enemies();
        assert!(world.enemies[0].knockback.x < -300.);
        assert_eq!(world.shots.len(), 2);
        assert!(world
            .shots
            .iter()
            .all(|shot| shot.kind == ProjectileKind::Flag));
        world.enemies.clear();
        world.time = 0.7;
        world.spawn_enemy(EnemyKind::Kimi);
        world.enemies[0].position = world.player + vec2(120., 0.);
        world.update_enemies();
        assert!((world.enemies[0].position.y - world.player.y).abs() > 0.1);
    }

    #[test]
    fn abandoned_entities_are_bounded_and_boss_stays_reachable() {
        let mut world = World::new(Edition::demo(), 0);
        world.spawn_enemy(EnemyKind::Bot);
        world.spawn_enemy(EnemyKind::Bernie);
        world.boss_spawned = true;
        let hp = world.enemies[1].hp;
        world.player += vec2(4000., 0.);
        world.update(Input::default());
        assert_eq!(world.enemies.len(), 1);
        assert_eq!(world.enemies[0].kind, EnemyKind::Bernie);
        assert_eq!(world.enemies[0].hp, hp);
        assert!(world.enemies[0].position.distance(world.player) < 500.);
        assert!(world.pickups.is_empty());
        assert_eq!(world.kills, 0);
        assert_eq!(world.terrain.chunks.len(), 25);
    }

    #[test]
    fn blue_wave_builds_themed_actors_and_expires_without_stacking_pressure() {
        let mut edition = Edition::demo();
        edition.threat_multiplier = 1.2;
        edition.events = vec![
            crate::config::WorldEvent {
                at_seconds: 0.1,
                kind: EventKind::BlueWave,
                title: "FIRST WAVE".into(),
            },
            crate::config::WorldEvent {
                at_seconds: 0.2,
                kind: EventKind::BlueWave,
                title: "SECOND WAVE".into(),
            },
        ];
        let mut world = World::new(edition.clone(), 0);
        world.pickups.clear();
        world.time = 0.1;
        world.trigger_events();
        assert_eq!(world.enemies.len(), 4);
        assert!(world.enemies.iter().all(
            |enemy| enemy.asset_pack == AssetPack::BlueWave && enemy.kind == EnemyKind::Lawyer
        ));
        assert_eq!(world.effective_threat(), 1.35);
        world.time = 0.2;
        world.trigger_events();
        assert_eq!(world.enemies.len(), 8);
        assert_eq!(world.effective_threat(), 1.35);
        world.toggle_pause();
        for _ in 0..120 {
            world.update(Input::default());
        }
        assert_eq!(world.event_remaining(EventKind::BlueWave), 45.);
        world.toggle_pause();
        world.time = 45.15;
        assert!(world.event_remaining(EventKind::BlueWave) > 0.);
        world.time = 45.3;
        assert_eq!(world.event_remaining(EventKind::BlueWave), 0.);
        assert_eq!(world.effective_threat(), 1.2);
        world.enemies[0].hp = 0.;
        world.remove_dead();
        let voucher = world.pickups.iter().find(|pickup| !pickup.cash).unwrap();
        assert_eq!(voucher.asset_pack, AssetPack::BlueWave);
        assert!((voucher.value - 3. / 1.35).abs() < 0.001);
        let mut restarted = World::new(edition, 0);
        assert_eq!(restarted.event_remaining(EventKind::BlueWave), 0.);
        restarted.spawn_enemy(EnemyKind::Lawyer);
        assert_eq!(restarted.enemies[0].asset_pack, AssetPack::DataCenter);
    }

    #[test]
    fn catalogue_drives_pickup_bursts_and_blackout_duration() {
        for kind in [
            EventKind::Funding,
            EventKind::Liquidation,
            EventKind::Blackout,
        ] {
            let mut edition = Edition::demo();
            edition.events = vec![crate::config::WorldEvent {
                at_seconds: 0.,
                kind,
                title: "RECIPE TEST".into(),
            }];
            let mut world = World::new(edition, 0);
            world.pickups.clear();
            world.update(Input::default());
            match kind {
                EventKind::Funding => {
                    assert_eq!(world.pickups.len(), 5);
                    assert!(world.pickups.iter().all(|pickup| pickup.cash));
                    assert_eq!(world.enemies.len(), 5);
                }
                EventKind::Liquidation => {
                    assert_eq!(world.pickups.len(), 15);
                    assert!(world
                        .pickups
                        .iter()
                        .all(|pickup| !pickup.cash && pickup.value == 3.));
                }
                EventKind::Blackout => assert!(world.blackout > 12.9 && world.blackout <= 13.),
                _ => unreachable!(),
            }
        }
    }

    const DIRECTIONS: [(Facing, Vec2); 8] = [
        (Facing::Down, Vec2::new(0., 1.)),
        (Facing::DownLeft, Vec2::new(-1., 1.)),
        (Facing::Left, Vec2::new(-1., 0.)),
        (Facing::UpLeft, Vec2::new(-1., -1.)),
        (Facing::Up, Vec2::new(0., -1.)),
        (Facing::UpRight, Vec2::new(1., -1.)),
        (Facing::Right, Vec2::new(1., 0.)),
        (Facing::DownRight, Vec2::new(1., 1.)),
    ];

    #[test]
    fn founders_walk_idle_and_dash_in_all_eight_directions() {
        for character in 0..FOUNDERS.len() as u32 {
            for (facing, movement) in DIRECTIONS {
                let mut world = World::new(Edition::demo(), character);
                for chunk in world.terrain.chunks.values_mut() {
                    chunk.props.clear();
                }
                world.pickups.clear();
                let start = world.player;
                for _ in 0..12 {
                    world.update(Input {
                        movement,
                        ..Input::default()
                    });
                }
                assert_eq!(world.facing, facing);
                assert!(world.moving);
                assert!(world.walk_time > 0.19);
                assert!((world.player - start).normalize().dot(movement.normalize()) > 0.999);
                world.update(Input::default());
                assert_eq!(world.facing, facing);
                assert!(!world.moving);
                assert_eq!(world.walk_time, 0.);
                let start = world.player;
                world.update(Input {
                    dash: true,
                    ..Input::default()
                });
                assert_eq!(world.facing, facing);
                assert!(world.moving);
                assert!((world.player - start).normalize().dot(movement.normalize()) > 0.999);
                assert!(world.player.distance(start) > 5.);
            }
        }
    }

    #[test]
    fn every_enemy_faces_its_movement_and_stops_animating_when_idle() {
        for kind in [
            EnemyKind::Bot,
            EnemyKind::Crawler,
            EnemyKind::Lawyer,
            EnemyKind::Investor,
            EnemyKind::Bernie,
            EnemyKind::Engineer,
            EnemyKind::Politician,
            EnemyKind::Glm,
            EnemyKind::Kimi,
            EnemyKind::DeepSeek,
        ] {
            for (facing, movement) in DIRECTIONS {
                let mut world = World::new(Edition::demo(), 0);
                for chunk in world.terrain.chunks.values_mut() {
                    chunk.props.clear();
                }
                world.spawn_enemy(kind);
                world.enemies[0].position = world.player - movement.normalize() * 250.;
                world.enemies[0].facing = Facing::Down;
                for _ in 0..12 {
                    world.update_enemies();
                }
                let enemy = &mut world.enemies[0];
                assert_eq!(enemy.facing, facing);
                assert!(enemy.moving);
                assert!(enemy.walk_time > 0.19);
                enemy.speed = 0.;
                world.update_enemies();
                assert_eq!(world.enemies[0].facing, facing);
                assert!(!world.enemies[0].moving);
                assert_eq!(world.enemies[0].walk_time, 0.);
            }
        }
        let mut world = World::new(Edition::demo(), 0);
        for chunk in world.terrain.chunks.values_mut() {
            chunk.props.clear();
        }
        world.spawn_enemy(EnemyKind::Lawyer);
        world.enemies[0].position = world.player + vec2(60., -60.);
        world.update_enemies();
        assert_eq!(world.enemies[0].facing, Facing::UpRight);
    }

    #[test]
    fn collecting_loot_pauses_for_upgrade_and_preserves_overflow() {
        let mut world = World::new(Edition::demo(), 0);
        world.pickups.clear();
        world.pickups.push(Pickup {
            asset_pack: AssetPack::DataCenter,
            position: world.player,
            value: 40.,
            cash: false,
            age: 0.,
        });
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Upgrade);
        assert_eq!(world.level, 2);
        let time = world.time;
        world.update(Input {
            movement: Vec2::X,
            ..Input::default()
        });
        assert_eq!(world.time, time);
        world.choose(0);
        assert_eq!(world.level, 3);
        assert_eq!(world.phase, Phase::Upgrade);
        world.choose(0);
        assert_eq!(world.phase, Phase::Playing);
    }

    #[test]
    fn simulation_is_seeded_and_streams_beyond_former_boundaries() {
        let mut a = World::new(Edition::demo(), 0);
        let mut b = World::new(Edition::demo(), 0);
        a.pickups.clear();
        b.pickups.clear();
        a.hp = 10000.;
        b.hp = 10000.;
        for _ in 0..1200 {
            let input = Input {
                movement: vec2(0., -1.),
                dash: true,
                ability: false,
            };
            a.update(input);
            b.update(input);
        }
        assert_eq!(a.player, b.player);
        assert!(a.player.y < -1440.);
        assert_eq!(a.terrain.chunks.len(), 25);
        assert_eq!(a.enemies.len(), b.enemies.len());
        assert_eq!(a.enemies[0].position, b.enemies[0].position);
    }

    #[test]
    fn only_bubble_death_wins_and_deadline_starts_overtime() {
        let mut world = World::new(Edition::demo(), 0);
        world.spawn_enemy(EnemyKind::Bernie);
        world.enemies[0].hp = 0.;
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Playing);
        world.spawn_enemy(EnemyKind::Spy);
        let spy = world.enemies.last().unwrap().id;
        world.time = world.edition.duration_seconds;
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Playing);
        assert!(world.boss_spawned);
        assert!(world.enemies.iter().any(|enemy| enemy.id == spy));
        assert_eq!(
            world
                .enemies
                .iter()
                .filter(|enemy| enemy.kind == EnemyKind::Bubble)
                .count(),
            1
        );
        world.time += 100.;
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Playing);
        world
            .enemies
            .iter_mut()
            .find(|enemy| enemy.kind == EnemyKind::Bubble)
            .unwrap()
            .hp = 0.;
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Won);
        world.hp = 0.;
        world.phase = Phase::Playing;
        world.update(Input::default());
        assert_eq!(world.phase, Phase::Lost);
    }

    #[test]
    fn finale_preserves_surviving_rivals_and_minibosses_in_combat() {
        let mut world = weapon_world(0, &[vec2(120., 0.)]);
        world.spawn_enemy(EnemyKind::Spy);
        world.enemies[1].position = world.player + vec2(150., 0.);
        for enemy in &mut world.enemies {
            enemy.attack = 0.;
        }
        let survivors: Vec<_> = world
            .enemies
            .iter()
            .map(|enemy| (enemy.id, enemy.position, enemy.hp))
            .collect();
        world.time = world.edition.duration_seconds;
        world.update_encounters();
        assert_eq!(world.enemies.len(), survivors.len() + 1);
        for (id, position, hp) in survivors {
            let enemy = world.enemies.iter().find(|enemy| enemy.id == id).unwrap();
            assert_eq!(enemy.position, position);
            assert_eq!(enemy.hp, hp);
        }
        world.update_encounters();
        assert_eq!(world.enemies.len(), 3);
        assert_eq!(world.active_boss().unwrap().kind, EnemyKind::Bubble);
        world.update_enemies();
        assert!(world
            .shots
            .iter()
            .any(|shot| shot.kind == ProjectileKind::Subpoena));
        assert!(world
            .shots
            .iter()
            .any(|shot| shot.kind == ProjectileKind::Bolt));
    }

    #[test]
    fn minibosses_are_seeded_random_nonoverlapping_and_never_early_or_after_finale() {
        let mut first_kinds = Vec::new();
        let mut arrival_times = Vec::new();
        for seed in 1..30 {
            let mut edition = Edition::demo();
            edition.seed = seed * 7937;
            let mut world = World::new(edition.clone(), 0);
            let replay = World::new(edition, 0);
            assert_eq!(world.minibosses, replay.minibosses);
            assert_eq!(world.next_miniboss, replay.next_miniboss);
            world.time = 40.;
            world.update_encounters();
            assert!(world.enemies.is_empty());
            arrival_times.push(world.next_miniboss);
            world.time = world.next_miniboss;
            world.update_encounters();
            first_kinds.push(world.enemies[0].kind);
            world.update_encounters();
            assert_eq!(world.enemies.len(), 1);
            let mut encountered = Vec::new();
            for _ in 0..3 {
                let kind = world.enemies[0].kind;
                assert!(!encountered.contains(&kind));
                encountered.push(kind);
                world.enemies[0].hp = 0.;
                world.remove_dead();
                assert!(!world.boss_defeated);
                world.time = world.next_miniboss;
                world.update_encounters();
            }
            assert!(world.enemies.is_empty());
            world.time = world.edition.duration_seconds;
            world.update_encounters();
            world.time += 60.;
            world.update_encounters();
            assert_eq!(world.enemies.len(), 1);
            assert_eq!(world.enemies[0].kind, EnemyKind::Bubble);
        }
        for kind in [EnemyKind::Bernie, EnemyKind::Aoc, EnemyKind::Spy] {
            assert!(first_kinds.contains(&kind));
        }
        assert!(arrival_times.iter().any(|time| *time != arrival_times[0]));
    }

    #[test]
    fn aoc_freezes_specials_and_bubble_is_stronger_than_minibosses() {
        let mut world = World::new(Edition::demo(), 0);
        world.pickups.clear();
        world.spawn_clock = 1000.;
        world.spawn_enemy(EnemyKind::Aoc);
        world.enemies[0].position = world.player + vec2(60., 0.);
        world.update(Input {
            ability: true,
            ..Input::default()
        });
        assert_eq!(world.ability_cooldown, 0.);
        world.enemies[0].position = world.player + vec2(150., 0.);
        world.update(Input {
            ability: true,
            ..Input::default()
        });
        assert!(world.ability_cooldown > 0.);
        for kind in [EnemyKind::Bernie, EnemyKind::Spy, EnemyKind::Bubble] {
            world.spawn_enemy(kind);
        }
        let bubble = world.enemies.last().unwrap();
        for miniboss in &world.enemies[..3] {
            assert!(bubble.max_hp > miniboss.max_hp * 4.);
        }
        let index = world.enemies.len() - 1;
        world.enemies[index].position = world.player + vec2(130., 0.);
        world.enemies[index].attack = 0.;
        world.time = 0.;
        world.update_enemies();
        assert_eq!(
            world
                .shots
                .iter()
                .filter(|shot| shot.kind == ProjectileKind::Coin)
                .count(),
            18
        );
        world.shots.clear();
        world.enemies[index].hp *= 0.25;
        world.enemies[index].attack = 0.;
        world.time = 3.;
        world.update_enemies();
        assert_eq!(
            world
                .shots
                .iter()
                .filter(|shot| shot.kind == ProjectileKind::Coin)
                .count(),
            7
        );
        assert!(world.enemies[index].attack < 2.);
    }

    #[test]
    fn live_updates_wait_for_resume_and_schedule_real_effects_without_retriggering() {
        let mut edition = Edition::demo();
        edition.events.clear();
        let mut world = World::new(edition, 0);
        world.pickups.clear();
        let update = |sequence, regime| {
            crate::live::LiveUpdate::parse(&format!(r#"{{"revision":"revision","sequence":{sequence},"regime":"{regime}","confidence":0.9,"stories":[{{"id":"article","kind":"research","title":"RESEARCH"}}]}}"#)).unwrap()
        };
        world.phase = Phase::Paused;
        world.live.queue(update(2, "bear"));
        world.live.queue(update(1, "bull"));
        world.update(Input::default());
        assert_eq!(world.live.sequence, 0);
        assert!(world.edition.events.is_empty());
        world.phase = Phase::Playing;
        world.update(Input::default());
        assert_eq!(world.edition.regime, "bear");
        assert_eq!(world.effective_threat(), 1.2);
        assert_eq!(world.edition.xp_multiplier, 0.9);
        assert_eq!(world.edition.events[0].at_seconds, 8.);
        world.time = 8.;
        world.trigger_events();
        assert_eq!(world.pickups.len(), 8);
        world.live.queue(update(3, "bull"));
        world.update(Input::default());
        world.trigger_events();
        assert_eq!(world.edition.events.len(), 1);
        assert_eq!(world.pickups.len(), 8);
        assert_eq!(world.edition.xp_multiplier, 1.15);
    }

    #[test]
    fn stock_events_change_combat_and_motion_then_restore_upgraded_stats() {
        let setup = |kind: Option<EventKind>| {
            let mut edition = Edition::demo();
            edition.events = kind
                .map(|kind| crate::config::WorldEvent {
                    kind,
                    title: "STOCK NEWS".into(),
                    at_seconds: 0.,
                })
                .into_iter()
                .collect();
            let mut world = World::new(edition, 0);
            world.pickups.clear();
            world.spawn_clock = 1000.;
            world.upgrades[Upgrade::FineTuning.index()] = 2;
            world.spawn_enemy(EnemyKind::Lawyer);
            world.enemies[0].position = world.player + vec2(120., 0.);
            world.enemies[0].hp = 10000.;
            world.enemies[0].attack = 1000.;
            world.enemies[0].speed = 0.;
            world
        };
        let mut base = setup(None);
        let input = Input {
            movement: Vec2::X,
            ability: true,
            ..Input::default()
        };
        base.update(input);
        let base_motion = base.player.x - START.x;
        let base_damage = base.shots[0].damage;
        let base_special = 10000. - base.enemies[0].hp;
        base.shots.clear();
        base.shot_clock = 0.;
        for _ in 0..120 {
            base.fire_weapon();
        }
        for (kind, damage, speed, faster) in [
            (EventKind::AiStockRally, 1.25, 1.1, true),
            (EventKind::AiStockCrash, 0.8, 0.9, false),
        ] {
            let mut world = setup(Some(kind));
            world.update(input);
            assert!(((world.player.x - START.x) / base_motion - speed).abs() < 0.001);
            assert!((world.shots[0].damage / base_damage - damage).abs() < 0.001);
            assert!(((10000. - world.enemies[0].hp) / base_special - damage).abs() < 0.001);
            world.shots.clear();
            world.shot_clock = 0.;
            for _ in 0..120 {
                world.fire_weapon();
            }
            assert_eq!(world.shots.len() > base.shots.len(), faster);
            if !faster {
                assert!(world.shots.len() < base.shots.len());
            }
            let frozen = (world.time, world.player_modifiers());
            world.toggle_pause();
            for _ in 0..180 {
                world.update(input);
            }
            assert_eq!((world.time, world.player_modifiers()), frozen);
            world.toggle_pause();
            world.time = 30.;
            world.player = START;
            world.shots.clear();
            world.shot_clock = 0.;
            world.update(Input {
                movement: Vec2::X,
                ..Input::default()
            });
            assert_eq!(world.player_modifiers(), PlayerModifiers::default());
            assert!((world.player.x - START.x - base_motion).abs() < 0.001);
            assert_eq!(world.shots[0].damage, base_damage);
            assert_eq!(world.upgrades[Upgrade::FineTuning.index()], 2);
        }
    }

    #[test]
    fn newer_stock_news_replaces_modifiers_and_generic_audits_have_no_party_pack() {
        let mut edition = Edition::demo();
        edition.events = [
            (0., EventKind::AiStockRally),
            (1., EventKind::AiStockRally),
            (10., EventKind::AiStockCrash),
            (10., EventKind::RegulatoryCrackdown),
        ]
        .into_iter()
        .map(|(at_seconds, kind)| crate::config::WorldEvent {
            at_seconds,
            kind,
            title: "NEWS".into(),
        })
        .collect();
        let mut world = World::new(edition, 0);
        world.pickups.clear();
        world.time = 1.;
        world.trigger_events();
        assert_eq!(world.player_modifiers().damage, 1.25);
        world.time = 10.;
        world.trigger_events();
        assert_eq!(world.player_event().unwrap().kind, EventKind::AiStockCrash);
        assert_eq!(world.player_modifiers().damage, 0.8);
        assert_eq!(world.enemies.len(), 5);
        assert!(world
            .enemies
            .iter()
            .all(|enemy| enemy.asset_pack == AssetPack::DataCenter
                && enemy.kind == EnemyKind::Lawyer));
        assert_eq!(world.effective_threat(), 1.15);
        assert_eq!(world.pickups.len(), 8);
        world.time = 40.;
        assert_eq!(world.player_modifiers(), PlayerModifiers::default());
    }

    #[test]
    fn paused_world_cannot_spawn_damage_or_advance_edition_events() {
        let mut world = World::new(Edition::demo(), 1);
        world.time = 34.99;
        world.toggle_pause();
        for _ in 0..120 {
            world.update(Input {
                ability: true,
                ..Input::default()
            });
        }
        assert_eq!(world.time, 34.99);
        assert!(world.events_fired.iter().all(|f| !f));
        assert_eq!(world.hp, 100.);
    }

    #[test]
    #[ignore = "Manual full-run balance probe; cargo test balance_probe -- --ignored --nocapture"]
    fn balance_probe() {
        for character in 0..FOUNDERS.len() as u32 {
            for (regime, threat, xp) in
                [("bull", 0.9, 1.15), ("neutral", 1., 1.), ("bear", 1.2, 0.9)]
            {
                let mut edition = Edition::demo();
                edition.regime = regime.into();
                edition.threat_multiplier = threat;
                edition.xp_multiplier = xp;
                let mut world = World::new(edition, character);
                for tick in 0..15000 {
                    if matches!(world.phase, Phase::Won | Phase::Lost) {
                        break;
                    }
                    if world.phase == Phase::Upgrade {
                        let index = world
                            .choices
                            .iter()
                            .position(|u| *u == Upgrade::Runway && world.hp < 60.)
                            .or_else(|| {
                                world
                                    .choices
                                    .iter()
                                    .position(|u| *u == Upgrade::OpenWeights)
                            })
                            .or_else(|| {
                                world
                                    .choices
                                    .iter()
                                    .position(|u| *u == Upgrade::Parallelism)
                            })
                            .or_else(|| {
                                world.choices.iter().position(|u| *u == Upgrade::FineTuning)
                            })
                            .unwrap_or(0);
                        world.choose(index);
                        continue;
                    }
                    let nearest = world.enemies.iter().min_by(|a, b| {
                        a.position
                            .distance_squared(world.player)
                            .total_cmp(&b.position.distance_squared(world.player))
                    });
                    let danger = nearest.map_or(999., |e| e.position.distance(world.player));
                    let target = world.pickups.iter().min_by(|a, b| {
                        a.position
                            .distance_squared(world.player)
                            .total_cmp(&b.position.distance_squared(world.player))
                    });
                    let orbit = START + Vec2::from_angle(tick as f32 * STEP * 0.55) * 115.;
                    let goal = target
                        .filter(|p| p.position.distance(world.player) < 95.)
                        .map_or(orbit, |p| p.position);
                    let mut movement = (goal - world.player).normalize_or_zero();
                    for enemy in &world.enemies {
                        let delta = world.player - enemy.position;
                        if delta.length() < 65. {
                            movement +=
                                delta.normalize_or_zero() * (1. - delta.length() / 65.) * 3.;
                        }
                    }
                    world.update(Input {
                        movement,
                        dash: danger < 22.,
                        ability: danger < 125.,
                    });
                }
                println!("character={character} regime={regime} outcome={:?} time={:.1} level={} kills={} hp={:.0} boss_hp={:.0}", world.phase, world.time, world.level, world.kills, world.hp, world.enemies.iter().find(|e| e.kind == EnemyKind::Bubble).map_or(0., |e| e.hp));
                assert!(matches!(world.phase, Phase::Won | Phase::Lost));
                assert!(world.enemies.len() <= 225);
            }
        }
    }
}
