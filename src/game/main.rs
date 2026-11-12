#![recursion_limit = "256"]

mod attack;
mod bridge;
mod config;
mod content;
mod founder;
mod live;
mod render;
mod terrain;
mod upgrade;
mod world;

use config::Edition;
use macroquad::prelude::*;
use serde_json::json;
use world::{Input, Phase, World, STEP};

fn window_conf() -> Conf {
    Conf {
        window_title: "GPU Survivor".into(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    set_pc_assets_folder("src/web/public");
    let renderer = match render::Renderer::load().await {
        Ok(renderer) => renderer,
        Err(error) => {
            bridge::emit(
                &json!({"type": "error", "message": format!("Could not load game assets: {error}")}),
            );
            return;
        }
    };
    let mut world = World::new(Edition::demo(), 0);
    world.phase = Phase::Menu;
    let mut accumulator = 0.;
    let mut report_clock = 0.;
    let mut serial = u32::MAX;
    let mut queued_dash = false;
    let mut queued_ability = false;
    bridge::emit(
        &json!({"type": "ready", "version": env!("CARGO_PKG_VERSION"),
        "contentVersion": content::CONTENT_VERSION, "recipes": content::RECIPES}),
    );
    loop {
        let commands = bridge::take_commands();
        if let Some(character) = commands.start {
            match Edition::parse(&bridge::configuration()) {
                Ok(edition) => {
                    world = World::new(edition, character);
                    accumulator = 0.;
                    queued_dash = false;
                    queued_ability = false;
                    serial = u32::MAX;
                }
                Err(message) => bridge::emit(&json!({"type": "error", "message": message})),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if world.phase == Phase::Menu && is_key_pressed(KeyCode::Enter) {
            world = World::new(Edition::demo(), 0);
        }
        if commands.menu {
            world.phase = Phase::Menu;
            world.status_serial += 1;
        }
        if commands.live_update
            && matches!(world.phase, Phase::Playing | Phase::Paused | Phase::Upgrade)
        {
            match live::LiveUpdate::parse(&bridge::live_configuration()) {
                Ok(update) => world.live.queue(update),
                Err(message) => bridge::emit(&json!({"type": "live_error", "message": message})),
            }
        }
        if commands.pause || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
            world.toggle_pause();
        }
        if let Some(choice) = commands.upgrade {
            world.choose(choice);
        }
        for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3]
            .iter()
            .enumerate()
        {
            if is_key_pressed(*key) {
                world.choose(i);
            }
        }
        let movement = vec2(
            (is_key_down(KeyCode::D) || is_key_down(KeyCode::Right)) as u8 as f32
                - (is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)) as u8 as f32,
            (is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)) as u8 as f32
                - (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)) as u8 as f32,
        );
        let frame = get_frame_time().min(0.1);
        if world.phase == Phase::Playing {
            queued_dash |= is_key_pressed(KeyCode::Space);
            queued_ability |= is_key_pressed(KeyCode::E);
            accumulator += frame;
            while accumulator >= STEP {
                world.update(Input {
                    movement,
                    dash: queued_dash,
                    ability: queued_ability,
                });
                queued_dash = false;
                queued_ability = false;
                accumulator -= STEP;
                if world.phase != Phase::Playing {
                    accumulator = 0.;
                    break;
                }
            }
        } else {
            accumulator = 0.;
        }
        renderer.draw(&world, get_time() as f32);
        report_clock += frame;
        if report_clock >= 0.2 || serial != world.status_serial {
            serial = world.status_serial;
            report_clock = 0.;
            let phase = match world.phase {
                Phase::Menu => "menu",
                Phase::Playing => "playing",
                Phase::Paused => "paused",
                Phase::Upgrade => "upgrade",
                Phase::Won => "won",
                Phase::Lost => "lost",
            };
            let choices: Vec<_> = world
                .choices
                .iter()
                .map(|u| u.card(world.upgrades[u.index()] + 1))
                .collect();
            bridge::emit(&json!({
                "type": "state", "phase": phase, "character": world.character,
                "time": world.time, "duration": world.edition.duration_seconds,
                "hp": world.hp.max(0.), "maxHp": world.max_hp,
                "level": world.level, "xp": world.xp, "xpNeeded": world.xp_needed(),
                "kills": world.kills, "gpus": world.collected,
                "position": {"x": world.player.x, "y": world.player.y},
                "facing": world.facing, "moving": world.moving,
                "company": founder::founder(world.character).company,
                "climate": world.climate, "heatExposure": world.heat_exposure,
                "heatDamagePerSecond": world.heat_damage_per_second(),
                "movementMultiplier": world.climate.movement_multiplier(),
                "chunk": terrain::chunk_at(world.player), "loadedChunks": world.terrain.chunks.len(),
                "rivals": world.enemies.iter().map(|enemy| enemy.kind).collect::<Vec<_>>(),
                "effectiveThreat": world.effective_threat(),
                "xpMultiplier": world.edition.xp_multiplier, "marketRegime": world.edition.regime,
                "playerModifiers": world.player_modifiers(), "playerEvent": world.player_event().map(|event| event.kind),
                "live": world.live,
                "blueWaveRemaining": world.event_remaining(config::EventKind::BlueWave),
                "inspectorCount": world.enemies.iter().filter(|enemy| enemy.asset_pack == content::AssetPack::BlueWave).count(),
                "enemyCount": world.enemies.len(), "projectileCount": world.shots.len(),
                "hostileProjectiles": world.shots.iter().filter(|shot| shot.kind.hostile()).map(|shot| shot.kind).collect::<Vec<_>>(),
                "dashCooldown": world.dash_cooldown, "abilityCooldown": world.ability_cooldown,
                "attackRadiusMultiplier": world.attack_radius_multiplier(),
                "friendlyProjectiles": world.shots.iter().filter(|shot| !shot.kind.hostile()).map(|shot| shot.kind).collect::<Vec<_>>(),
                "signature": {"kind": founder::founder(world.character).signature, "rank": world.signature_rank(), "cooldown": world.signature_cooldown},
                "choices": choices, "edition": world.edition.id,
                "bossSpawned": world.boss_spawned, "bossDefeated": world.boss_defeated,
                "abilityLocked": world.ability_locked(),
                "boss": world.active_boss().map(|enemy|
                    json!({"kind": enemy.kind, "hp": enemy.hp, "maxHp": enemy.max_hp, "label": enemy.kind.label()})),
                "banner": world.banner, "upgrades": world.upgrades,
                "fps": get_fps()
            }));
        }
        if !world.sound_cues.is_empty() {
            bridge::emit(&json!({"type": "sounds", "cues": world.sound_cues}));
            world.sound_cues.clear();
        }
        next_frame().await;
    }
}
