use crate::attack::WINDUP;
use crate::world::*;
use crate::{config::EventKind, content::AssetPack};
use crate::{
    founder::founder,
    terrain::{Climate, CHUNK_SIZE},
};
use macroquad::prelude::*;

pub const WIDTH: f32 = 640.;
pub const HEIGHT: f32 = 360.;
const RENDER_SCALE: u32 = 2;
const ACTOR_CELL: f32 = 128.;
const ACTORS_PER_PAGE: usize = 4;
const ACTOR_COLUMNS: usize = 16;
const CREAM: Color = Color::new(0.90, 0.90, 0.79, 1.);
const GREEN: Color = Color::new(0.76, 0.92, 0.49, 1.);
const MUTED: Color = Color::new(0.51, 0.61, 0.57, 1.);
const INK: Color = Color::new(0.065, 0.10, 0.12, 1.);

pub struct Renderer {
    atlas: Texture2D,
    actors: Vec<Texture2D>,
    packs: Texture2D,
    logos: Texture2D,
    environment: Texture2D,
    flooring: Texture2D,
    projectiles: Texture2D,
    bubble: Texture2D,
    font: Font,
    target: RenderTarget,
}

impl Renderer {
    pub async fn load() -> Result<Self, String> {
        let atlas = load_texture("assets/sprites.png")
            .await
            .map_err(|e| e.to_string())?;
        atlas.set_filter(FilterMode::Nearest);
        let mut actors = Vec::with_capacity(5);
        for page in 0..5 {
            let texture = load_texture(&format!("assets/actors-{page}.png"))
                .await
                .map_err(|e| e.to_string())?;
            texture.set_filter(FilterMode::Nearest);
            actors.push(texture);
        }
        let packs = load_texture("assets/packs.png")
            .await
            .map_err(|e| e.to_string())?;
        packs.set_filter(FilterMode::Nearest);
        let logos = load_texture("assets/logos.png")
            .await
            .map_err(|e| e.to_string())?;
        logos.set_filter(FilterMode::Nearest);
        let environment = load_texture("assets/environment.png")
            .await
            .map_err(|e| e.to_string())?;
        let flooring = load_texture("assets/floor.png")
            .await
            .map_err(|e| e.to_string())?;
        let projectiles = load_texture("assets/projectiles.png")
            .await
            .map_err(|e| e.to_string())?;
        let bubble = load_texture("assets/bubble.png")
            .await
            .map_err(|e| e.to_string())?;
        for texture in [&environment, &flooring, &projectiles, &bubble] {
            texture.set_filter(FilterMode::Nearest);
        }
        let font = load_ttf_font("assets/pixel.ttf")
            .await
            .map_err(|e| e.to_string())?;
        let target = render_target_ex(
            WIDTH as u32 * RENDER_SCALE,
            HEIGHT as u32 * RENDER_SCALE,
            RenderTargetParams {
                sample_count: 0,
                depth: false,
            },
        );
        target.texture.set_filter(FilterMode::Nearest);
        Ok(Self {
            atlas,
            actors,
            packs,
            logos,
            environment,
            flooring,
            projectiles,
            bubble,
            font,
            target,
        })
    }

    pub fn draw(&self, world: &World, clock: f32) {
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., WIDTH, HEIGHT));
        camera.render_target = Some(self.target.clone());
        set_camera(&camera);
        clear_background(INK);
        let offset = (world.player - vec2(WIDTH / 2., HEIGHT / 2.)
            + vec2((clock * 51.).sin(), (clock * 73.).cos()) * world.shake)
            .floor();
        self.floor(world, offset, clock);
        for pickup in &world.pickups {
            let p = pickup.position - offset;
            if !visible(p, 20.) {
                continue;
            }
            let bob = (clock * 3. + pickup.position.x * 0.1).sin() * 1.5;
            draw_ellipse(p.x, p.y + 2., 7., 2., 0., Color::new(0., 0., 0., 0.20));
            if pickup.cash {
                self.sprite(5, p + vec2(0., -bob), 0.5, WHITE);
            } else if pickup.asset_pack == AssetPack::BlueWave {
                self.pack_sprite(1, p + vec2(0., -bob), 0.75);
            } else {
                draw_rectangle(
                    p.x - 4.,
                    p.y - 6. - bob,
                    9.,
                    7.,
                    Color::from_rgba(31, 68, 48, 255),
                );
                draw_rectangle(
                    p.x - 3.,
                    p.y - 5. - bob,
                    7.,
                    5.,
                    Color::from_rgba(139, 187, 96, 255),
                );
                draw_rectangle(
                    p.x - 1.,
                    p.y - 4. - bob,
                    3.,
                    3.,
                    Color::from_rgba(49, 83, 57, 255),
                );
                for x in [-2., 0., 2.] {
                    draw_rectangle(
                        p.x + x,
                        p.y + 1. - bob,
                        1.,
                        2.,
                        Color::from_rgba(226, 196, 110, 255),
                    );
                }
                if pickup.value >= 3. {
                    draw_rectangle(p.x - 2., p.y - 8. - bob, 5., 1., GREEN);
                }
            }
        }

        // One depth-sorted list handles scenery, enemies, and the player.
        let props: Vec<_> = world
            .terrain
            .chunks
            .values()
            .flat_map(|chunk| &chunk.props)
            .collect();
        let mut objects: Vec<(f32, u8, usize)> = props
            .iter()
            .enumerate()
            .map(|(i, p)| (p.position.y, 0, i))
            .collect();
        objects.extend(
            world
                .enemies
                .iter()
                .enumerate()
                .map(|(i, e)| (e.position.y, 1, i)),
        );
        objects.push((world.player.y, 2, 0));
        objects.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        for (_, kind, index) in objects {
            match kind {
                0 => {
                    let prop = props[index];
                    let p = prop.position - offset;
                    if !visible(p, 90.) {
                        continue;
                    }
                    draw_ellipse(
                        p.x + 5.,
                        p.y + 2.,
                        21.,
                        6.,
                        0.,
                        Color::new(0., 0., 0., 0.23),
                    );
                    let obscures_player = prop.position.y > world.player.y
                        && prop.position.y - world.player.y < 78.
                        && (prop.position.x - world.player.x).abs() < 35.;
                    draw_texture_ex(
                        &self.environment,
                        p.x - 32.,
                        p.y - 74.,
                        Color::new(1., 1., 1., if obscures_player { 0.35 } else { 1. }),
                        DrawTextureParams {
                            source: Some(Rect::new(
                                (prop.sprite % 4) as f32 * 128.,
                                (prop.sprite / 4) as f32 * 160.,
                                128.,
                                160.,
                            )),
                            dest_size: Some(vec2(64., 80.)),
                            ..Default::default()
                        },
                    );
                    if matches!(prop.sprite, 0 | 1 | 4 | 6) && world.blackout <= 0. {
                        let intensity = 0.05 + (clock * 2. + index as f32).sin().abs() * 0.025;
                        draw_ellipse(
                            p.x,
                            p.y + 2.,
                            13.,
                            3.,
                            0.,
                            Color::new(0.4, 0.9, 0.7, intensity),
                        );
                    }
                }
                1 => {
                    let enemy = &world.enemies[index];
                    let p = enemy.position - offset;
                    if !visible(p, 90.) {
                        continue;
                    }
                    let frame = if enemy.moving {
                        (enemy.walk_time * 7.) as usize % 4
                    } else {
                        0
                    };
                    let (row, scale) = match enemy.kind {
                        EnemyKind::Bot => (4, 0.86),
                        EnemyKind::Crawler => (5, 0.82),
                        EnemyKind::Lawyer => (
                            if enemy.asset_pack == AssetPack::BlueWave {
                                7
                            } else {
                                2
                            },
                            0.94,
                        ),
                        EnemyKind::Investor => (3, 1.),
                        EnemyKind::Bernie => (6, 1.55),
                        EnemyKind::Aoc => (15, 1.35),
                        EnemyKind::Spy => (16, 1.3),
                        EnemyKind::Bubble => (0, 1.8),
                        EnemyKind::Engineer => (10, 0.98),
                        EnemyKind::Politician => (11, 1.),
                        EnemyKind::Glm => (12, 1.05),
                        EnemyKind::Kimi => (13, 0.9),
                        EnemyKind::DeepSeek => (14, 1.),
                    };
                    if enemy.kind.boss() {
                        draw_ellipse(p.x, p.y, 31., 12., 0., Color::new(0.8, 0.3, 0.25, 0.14));
                    }
                    self.shadow(p, 10. * scale);
                    if enemy.kind == EnemyKind::Aoc {
                        draw_circle_lines(p.x, p.y, 100., 1.5, Color::new(0.35, 0.65, 1., 0.5));
                        self.center(
                            "BUDGET FREEZE / E DISABLED",
                            p.x,
                            p.y + 20.,
                            5.,
                            Color::new(0.55, 0.8, 1., 1.),
                        );
                    }
                    if enemy.kind == EnemyKind::Bubble {
                        let size = 108. + (world.time * 2.).sin() * 4.;
                        draw_texture_ex(
                            &self.bubble,
                            p.x - size / 2.,
                            p.y - size / 2.,
                            if enemy.flash > 0. {
                                Color::new(1., 0.65, 0.65, 1.)
                            } else {
                                WHITE
                            },
                            DrawTextureParams {
                                dest_size: Some(vec2(size, size)),
                                ..Default::default()
                            },
                        );
                        if enemy.hp < enemy.max_hp * 0.5 {
                            draw_circle_lines(p.x, p.y, 54., 1.5, Color::new(1., 0.35, 0.65, 0.6));
                        }
                    } else {
                        self.actor(
                            row,
                            enemy.facing,
                            frame,
                            p,
                            scale,
                            if enemy.flash > 0. {
                                Color::new(1., 0.65, 0.55, 1.)
                            } else if enemy.kind == EnemyKind::Spy
                                && world.time % 6. < 2.
                                && enemy.attack > WINDUP
                            {
                                Color::new(0.45, 0.9, 1., 0.35)
                            } else {
                                WHITE
                            },
                        );
                    }
                    if enemy.hp < enemy.max_hp && !enemy.kind.boss() {
                        draw_rectangle(p.x - 9., p.y + 3., 18., 2., INK);
                        draw_rectangle(
                            p.x - 9.,
                            p.y + 3.,
                            18. * (enemy.hp / enemy.max_hp).max(0.),
                            2.,
                            Color::from_rgba(217, 131, 112, 255),
                        );
                    }
                    let label = match enemy.kind {
                        EnemyKind::Glm => "GLM",
                        EnemyKind::Kimi => "KIMI",
                        EnemyKind::DeepSeek => "DEEPSEEK",
                        _ => "",
                    };
                    if !label.is_empty() {
                        self.center(label, p.x, p.y - 56., 5., CREAM);
                    }
                    if enemy.attack < WINDUP && enemy.attack > 0. {
                        let badge = p - vec2(0., if enemy.kind.boss() { 87. } else { 65. });
                        draw_circle(badge.x, badge.y, 10., Color::new(0.08, 0.10, 0.13, 0.85));
                        draw_circle_lines(
                            badge.x,
                            badge.y,
                            10.,
                            1.,
                            Color::new(1., 0.72, 0.3, 1. - enemy.attack / WINDUP),
                        );
                        self.projectile(
                            enemy.kind.attack(enemy.asset_pack).projectile,
                            badge,
                            0.,
                            0.,
                            16.,
                        );
                    }
                }
                _ => {
                    let p = world.player - offset;
                    if let Some(event) = world.player_event() {
                        let color = if event.kind == EventKind::AiStockRally {
                            Color::new(0.6, 1., 0.4, 0.65)
                        } else {
                            Color::new(1., 0.35, 0.3, 0.65)
                        };
                        draw_ellipse_lines(
                            p.x,
                            p.y,
                            22. + (world.time * 4.).sin() * 2.,
                            9.,
                            0.,
                            1.5,
                            color,
                        );
                    }
                    draw_ellipse(p.x, p.y, 15., 6., 0., Color::new(0.61, 0.81, 0.42, 0.2));
                    self.shadow(p, 10.);
                    let frame = if world.moving {
                        (world.walk_time * 9.) as usize % 4
                    } else {
                        0
                    };
                    let alpha =
                        if world.invulnerable > 0. && ((clock * 20.) as u32).is_multiple_of(2) {
                            0.45
                        } else {
                            1.
                        };
                    self.actor(
                        founder(world.character).actor_row,
                        world.facing,
                        frame,
                        p,
                        1.,
                        Color::new(1., 1., 1., alpha),
                    );
                    draw_triangle(
                        vec2(p.x, p.y - 56.),
                        vec2(p.x - 3., p.y - 60.),
                        vec2(p.x + 3., p.y - 60.),
                        GREEN,
                    );
                }
            }
        }

        for position in world.orbit_positions() {
            let p = position - offset;
            let size = world.attack_radius_multiplier();
            draw_rectangle(
                p.x - 5. * size,
                p.y - 7. - 5. * size,
                10. * size,
                10. * size,
                Color::from_rgba(79, 126, 124, 255),
            );
            draw_rectangle(
                p.x - 3. * size,
                p.y - 7. - 3. * size,
                6. * size,
                6. * size,
                Color::from_rgba(163, 222, 192, 255),
            );
            draw_rectangle(p.x - 1., p.y - 8., 2., 2., INK);
        }
        for shot in &world.shots {
            let p = shot.position - offset + vec2(0., -7.);
            if shot.kind.hostile() {
                self.shadow(p + vec2(0., 7.), 4.);
                let rotation = match shot.kind {
                    ProjectileKind::Wrench | ProjectileKind::Scrap => shot.age * 9.,
                    ProjectileKind::Bolt => shot.velocity.to_angle(),
                    ProjectileKind::Flag | ProjectileKind::Coin => 0.,
                    _ => (shot.age * 7.).sin() * 0.3,
                };
                self.projectile(
                    shot.kind,
                    p,
                    shot.age,
                    rotation,
                    if shot.kind == ProjectileKind::Flag {
                        26.
                    } else {
                        22.
                    },
                );
            } else if shot.kind != ProjectileKind::Founder {
                let (size, radius, rotation) = match shot.kind {
                    ProjectileKind::Cybertruck => (48., 13., shot.velocity.to_angle()),
                    ProjectileKind::LeatherJacket => (36., 10., shot.age * 8.),
                    ProjectileKind::Capital => (32., 8., (shot.age * 5.).sin() * 0.2),
                    ProjectileKind::SafetyRail => (52., 18., shot.velocity.to_angle()),
                    _ => unreachable!(),
                };
                self.shadow(p + vec2(0., 8.), shot.radius);
                self.projectile(
                    shot.kind,
                    p,
                    shot.age,
                    rotation,
                    size * shot.radius / radius,
                );
            } else {
                let color = [
                    GREEN,
                    Color::from_rgba(170, 235, 214, 255),
                    Color::from_rgba(227, 169, 137, 255),
                    CREAM,
                ][world.character as usize];
                let tail = p - shot.velocity.normalize_or_zero() * 9.;
                draw_line(
                    tail.x,
                    tail.y,
                    p.x,
                    p.y,
                    2.,
                    Color::new(color.r, color.g, color.b, 0.3),
                );
                draw_texture_ex(
                    &self.logos,
                    (p.x - 12. * shot.radius / 5.).floor(),
                    (p.y - 12. * shot.radius / 5.).floor(),
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(world.character as f32 * 32., 0., 32., 32.)),
                        dest_size: Some(vec2(24., 24.) * shot.radius / 5.),
                        ..Default::default()
                    },
                );
            }
        }
        for blast in &world.blasts {
            let p = blast.position - offset;
            let progress = 1. - blast.ttl / 0.35;
            draw_circle(
                p.x,
                p.y - 7.,
                blast.radius,
                Color::new(0.95, 0.77, 0.32, blast.ttl * 0.3),
            );
            draw_circle_lines(
                p.x,
                p.y - 7.,
                blast.radius * (0.5 + progress * 0.5),
                2.,
                Color::new(0.95, 0.77, 0.32, blast.ttl * 2.),
            );
        }
        for particle in &world.particles {
            let p = particle.position - offset;
            draw_rectangle(
                p.x.floor(),
                p.y.floor(),
                particle.size.ceil(),
                particle.size.ceil(),
                Color::new(
                    particle.color.r,
                    particle.color.g,
                    particle.color.b,
                    (particle.ttl * 4.).min(1.),
                ),
            );
        }
        for floater in &world.floaters {
            let p = floater.position - offset;
            self.text(&floater.text, p.x, p.y, 6., floater.color);
        }
        if world.pulse > 0. {
            let p = world.player - offset;
            let radius = (1. - world.pulse / 0.55) * world.ability_radius();
            draw_circle_lines(
                p.x,
                p.y - 12.,
                radius,
                3.,
                Color::new(0.75, 0.91, 0.55, world.pulse / 0.55),
            );
            draw_circle_lines(
                p.x,
                p.y - 12.,
                radius * 0.82,
                1.,
                Color::new(0.75, 0.91, 0.55, world.pulse / 0.7),
            );
        }
        if world.blackout > 0. {
            draw_rectangle(0., 0., WIDTH, HEIGHT, Color::new(0.015, 0.03, 0.06, 0.38));
            self.text(
                "POWER OUTAGE",
                250.,
                70.,
                8.,
                Color::from_rgba(233, 175, 106, 255),
            );
        }
        if world.phase != Phase::Menu {
            self.hud(world);
        }

        set_default_camera();
        clear_background(INK);
        let scale = (screen_width() / WIDTH).min(screen_height() / HEIGHT);
        draw_texture_ex(
            &self.target.texture,
            (screen_width() - WIDTH * scale) / 2.,
            (screen_height() - HEIGHT * scale) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(WIDTH * scale, HEIGHT * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    fn actor(
        &self,
        row: usize,
        facing: Facing,
        frame: usize,
        position: Vec2,
        scale: f32,
        color: Color,
    ) {
        let column = facing.sprite_column(frame);
        draw_texture_ex(
            &self.actors[row / ACTORS_PER_PAGE],
            (position.x - 32. * scale).floor(),
            (position.y - 54. * scale).floor(),
            color,
            DrawTextureParams {
                source: Some(Rect::new(
                    (column % ACTOR_COLUMNS) as f32 * ACTOR_CELL,
                    ((row % ACTORS_PER_PAGE) * 2 + column / ACTOR_COLUMNS) as f32 * ACTOR_CELL,
                    ACTOR_CELL,
                    ACTOR_CELL,
                )),
                dest_size: Some(vec2(64. * scale, 64. * scale)),
                ..Default::default()
            },
        );
    }

    fn pack_sprite(&self, column: usize, position: Vec2, scale: f32) {
        draw_texture_ex(
            &self.packs,
            (position.x - 32. * scale).floor(),
            (position.y - 50. * scale).floor(),
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(column as f32 * 64., 0., 64., 64.)),
                dest_size: Some(vec2(64. * scale, 64. * scale)),
                ..Default::default()
            },
        );
    }

    fn floor(&self, world: &World, offset: Vec2, clock: f32) {
        let start_x = (offset.x / 32.).floor() as i32 - 1;
        let start_y = (offset.y / 32.).floor() as i32 - 1;
        for y in start_y..start_y + 15 {
            for x in start_x..start_x + 23 {
                let hash = (x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663)).unsigned_abs();
                let px = x as f32 * 32. - offset.x;
                let py = y as f32 * 32. - offset.y;
                let vent =
                    matches!(x.rem_euclid(12), 5 | 6) && matches!(y.rem_euclid(12), 1..=3 | 8..=10);
                let tile = if vent { 3 } else { hash % 3 };
                draw_texture_ex(
                    &self.flooring,
                    px,
                    py,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect::new(tile as f32 * 64., 0., 64., 64.)),
                        dest_size: Some(vec2(32., 32.)),
                        ..Default::default()
                    },
                );
            }
        }
        for (&(x, y), chunk) in &world.terrain.chunks {
            let p = chunk.origin - offset;
            if p.x > WIDTH || p.y > HEIGHT || p.x + CHUNK_SIZE < 0. || p.y + CHUNK_SIZE < 0. {
                continue;
            }
            let (color, label) = match chunk.climate {
                Climate::Normal => (Color::from_rgba(96, 132, 130, 255), "COMPUTE HALL / ONLINE"),
                Climate::Cold => (
                    Color::from_rgba(116, 202, 237, 255),
                    "COLD AISLE / SLOW -38%",
                ),
                Climate::Hot => (
                    Color::from_rgba(239, 136, 96, 255),
                    "COOLING FAILURE / HEAT",
                ),
            };
            // Inset service lanes and raised-floor rack foundations.
            for py in [80., 136., 280., 336.] {
                for px in [48., 224.] {
                    draw_rectangle(
                        p.x + px,
                        p.y + py - 14.,
                        112.,
                        22.,
                        Color::new(0.04, 0.08, 0.10, 0.30),
                    );
                    draw_line(
                        p.x + px,
                        p.y + py + 10.,
                        p.x + px + 112.,
                        p.y + py + 10.,
                        0.5,
                        Color::new(0.6, 0.68, 0.67, 0.24),
                    );
                }
            }
            if chunk.climate != Climate::Normal {
                let z = Rect::new(
                    chunk.zone.x - offset.x,
                    chunk.zone.y - offset.y,
                    chunk.zone.w,
                    chunk.zone.h,
                );
                draw_rectangle(
                    z.x,
                    z.y,
                    z.w,
                    z.h,
                    Color::new(color.r, color.g, color.b, 0.13),
                );
                draw_rectangle_lines(
                    z.x,
                    z.y,
                    z.w,
                    z.h,
                    2.,
                    Color::new(color.r, color.g, color.b, 0.7),
                );
                for i in 0..(z.w / 16.) as i32 {
                    let px = z.x + i as f32 * 16.;
                    draw_line(px, z.y, px + 6., z.y + 6., 2., color);
                    draw_line(px, z.bottom(), px + 6., z.bottom() - 6., 2., color);
                }
                for i in 0..10 {
                    let px = z.x + 16. + ((i * 43) as f32 % (z.w - 32.));
                    let py = z.y
                        + (i as f32 * 37.
                            + clock
                                * if chunk.climate == Climate::Hot {
                                    -12.
                                } else {
                                    8.
                                })
                        .rem_euclid(z.h);
                    if chunk.climate == Climate::Cold {
                        draw_line(
                            px - 3.,
                            py,
                            px + 3.,
                            py,
                            1.,
                            Color::new(color.r, color.g, color.b, 0.5),
                        );
                        draw_line(
                            px,
                            py - 3.,
                            px,
                            py + 3.,
                            1.,
                            Color::new(color.r, color.g, color.b, 0.5),
                        );
                    } else {
                        draw_rectangle(px, py, 4., 7., Color::new(color.r, color.g, color.b, 0.22));
                    }
                }
            }
            // Cable trays and floor vents repeat at world coordinates, including negative chunks.
            for py in [18., 366.] {
                draw_rectangle(
                    p.x,
                    p.y + py,
                    CHUNK_SIZE,
                    6.,
                    Color::from_rgba(24, 33, 40, 255),
                );
                draw_line(
                    p.x,
                    p.y + py + 2.,
                    p.x + CHUNK_SIZE,
                    p.y + py + 2.,
                    1.,
                    Color::from_rgba(84, 101, 99, 255),
                );
                for px in (0..384).step_by(16) {
                    draw_rectangle(p.x + px as f32, p.y + py, 2., 6., INK);
                }
            }
            for px in [16., 364.] {
                for py in (48..345).step_by(32) {
                    draw_rectangle(p.x + px, p.y + py as f32, 6., 18., INK);
                    for slit in 0..4 {
                        draw_rectangle(
                            p.x + px + 1.,
                            p.y + py as f32 + slit as f32 * 4.,
                            4.,
                            1.,
                            MUTED,
                        );
                    }
                }
            }
            self.center(label, p.x + 192., p.y + 230., 6., color);
            self.center(
                &format!("SECTOR {x}:{y}"),
                p.x + 192.,
                p.y + 245.,
                5.,
                MUTED,
            );
            if world.event_remaining(EventKind::BlueWave) > 0. {
                self.pack_sprite(0, p + vec2(36., 190.), 1.);
            }
            if world.event_remaining(EventKind::RegulatoryCrackdown) > 0. {
                self.pack_sprite(2, p + vec2(345., 190.), 1.);
            }
            if let Some(event) = world.player_event() {
                self.pack_sprite(
                    if event.kind == EventKind::AiStockCrash {
                        3
                    } else {
                        4
                    },
                    p + vec2(108., 190.),
                    1.2,
                );
            }
        }
    }

    fn hud(&self, world: &World) {
        let (status, color) = match world.climate {
            Climate::Hot => (
                format!(
                    "HEAT {:.1}s / -{:.1} HP/s / LEAVE ZONE",
                    world.heat_exposure,
                    world.heat_damage_per_second()
                ),
                Color::from_rgba(245, 153, 112, 255),
            ),
            Climate::Cold => (
                "COLD AISLE / MOVEMENT -38%".into(),
                Color::from_rgba(133, 211, 244, 255),
            ),
            Climate::Normal if world.heat_exposure > 0. => (
                format!("COOLING DOWN / {:.1}s HEAT", world.heat_exposure),
                GREEN,
            ),
            Climate::Normal => ("DATA CENTER / THERMALS NORMAL".into(), MUTED),
        };
        draw_rectangle(8., 292., 302., 20., Color::new(0.055, 0.09, 0.10, 0.94));
        self.text(&status, 14., 305., 6., color);
        let timed = world
            .player_event()
            .map(|event| event.kind.recipe())
            .or_else(|| {
                crate::content::RECIPES
                    .iter()
                    .find(|recipe| world.event_remaining(recipe.kind) > 0.)
            });
        if let Some(recipe) = timed {
            let remaining = world.event_remaining(recipe.kind);
            let modifiers = world.player_modifiers();
            let color = match recipe.kind {
                EventKind::AiStockCrash => Color::from_rgba(245, 140, 120, 255),
                EventKind::AiStockRally => GREEN,
                _ => Color::from_rgba(148, 208, 249, 255),
            };
            draw_rectangle(8., 314., 302., 25., Color::new(0.055, 0.09, 0.10, 0.94));
            self.text(
                &format!("{} / {}s", recipe.label, remaining.ceil() as u32),
                14.,
                324.,
                6.,
                color,
            );
            let details = if recipe.player_modifiers.is_some() {
                format!(
                    "DMG {:.2}x / FIRE {:.2}x / MOVE {:.2}x",
                    modifiers.damage, modifiers.attack_rate, modifiers.movement
                )
            } else {
                format!("SPAWN PRESSURE {:.2}x", world.effective_threat())
            };
            self.text(&details, 14., 334., 5., color);
        }
        draw_rectangle(8., 8., 164., 38., Color::new(0.055, 0.09, 0.10, 0.94));
        draw_rectangle(8., 8., 2., 38., GREEN);
        self.text(founder(world.character).name, 17., 21., 8., CREAM);
        self.text("RUNWAY", 17., 34., 6., MUTED);
        draw_rectangle(61., 28., 99., 7., Color::from_rgba(47, 63, 56, 255));
        draw_rectangle(
            62.,
            29.,
            97. * (world.hp / world.max_hp).clamp(0., 1.),
            5.,
            if world.hp < 30. {
                Color::from_rgba(227, 135, 115, 255)
            } else {
                GREEN
            },
        );
        let seconds = world.time as u32;
        draw_rectangle(278., 8., 84., 29., Color::new(0.055, 0.09, 0.10, 0.94));
        self.center(
            &format!("{:02}:{:02}", seconds / 60, seconds % 60),
            320.,
            26.,
            12.,
            CREAM,
        );
        draw_rectangle(481., 8., 151., 38., Color::new(0.055, 0.09, 0.10, 0.94));
        self.text(
            &format!("LVL {:02} / {} GPU", world.level, world.collected),
            491.,
            21.,
            7.,
            GREEN,
        );
        self.text(
            &format!("{} COMPETITORS DISRUPTED", world.kills),
            491.,
            35.,
            5.,
            MUTED,
        );
        // A horizontal timeline makes the end condition legible.
        draw_rectangle(8., 351., 624., 3., Color::from_rgba(31, 48, 45, 255));
        draw_rectangle(
            8.,
            351.,
            624. * (world.xp / world.xp_needed()).min(1.),
            3.,
            GREEN,
        );
        self.text(
            &format!(
                "NEXT ROUND: {} / {} XP",
                world.xp as u32,
                world.xp_needed() as u32
            ),
            10.,
            346.,
            5.,
            MUTED,
        );
        draw_rectangle(420., 292., 212., 28., Color::new(0.055, 0.09, 0.10, 0.94));
        self.text(
            &format!("ATTACK RADIUS {:.2}x", world.attack_radius_multiplier()),
            426.,
            302.,
            5.,
            GREEN,
        );
        let signature = founder(world.character)
            .signature
            .card(world.signature_rank());
        let label = if world.signature_rank() > 0 {
            format!(
                "{} / R{} / {:.1}s",
                signature.name,
                signature.level,
                world.signature_cooldown.max(0.)
            )
        } else {
            format!("{} / LOCKED", signature.name)
        };
        self.text(&label, 426., 313., 5., CREAM);
        self.ability_box(420., "SPACE", "PIVOT", world.dash_cooldown, 3.);
        self.ability_box(
            526.,
            "E",
            if world.ability_locked() {
                "FROZEN"
            } else {
                founder(world.character).ability
            },
            world.ability_cooldown,
            15.,
        );
        if world.banner_time > 0. && !world.enemies.iter().any(|enemy| enemy.kind.boss()) {
            let text = &world.banner;
            let width = (text.len() as f32 * 6. + 24.).min(620.);
            draw_rectangle(
                (WIDTH - width) / 2.,
                55.,
                width,
                23.,
                Color::new(0.08, 0.13, 0.13, 0.95),
            );
            self.center(text, WIDTH / 2., 70., 6., CREAM);
        }
        if let Some(boss) = world.active_boss() {
            draw_rectangle(178., 44., 284., 27., INK);
            self.center(boss.kind.label(), 320., 55., 6., CREAM);
            draw_rectangle(186., 61., 268., 4., Color::from_rgba(75, 48, 48, 255));
            draw_rectangle(
                186.,
                61.,
                268. * (boss.hp / boss.max_hp).max(0.),
                4.,
                Color::from_rgba(222, 144, 121, 255),
            );
        }
        if world.hp < 25. && ((world.time * 3.) as u32).is_multiple_of(2) {
            draw_rectangle_lines(
                1.,
                1.,
                WIDTH - 2.,
                HEIGHT - 2.,
                3.,
                Color::new(0.8, 0.31, 0.25, 0.7),
            );
        }
    }

    fn ability_box(&self, x: f32, key: &str, name: &str, cooldown: f32, full: f32) {
        draw_rectangle(x, 322., 104., 25., Color::new(0.055, 0.09, 0.10, 0.94));
        self.text(key, x + 6., 332., 5., MUTED);
        self.text(
            name,
            x + 6.,
            342.,
            6.,
            if cooldown <= 0. { GREEN } else { MUTED },
        );
        draw_rectangle(
            x,
            347.,
            104. * (1. - cooldown / full).clamp(0., 1.),
            1.,
            GREEN,
        );
    }

    fn shadow(&self, position: Vec2, size: f32) {
        draw_ellipse(
            position.x,
            position.y,
            size,
            size * 0.35,
            0.,
            Color::new(0., 0., 0., 0.30),
        );
    }

    fn projectile(&self, kind: ProjectileKind, position: Vec2, age: f32, rotation: f32, size: f32) {
        draw_texture_ex(
            &self.projectiles,
            position.x - size / 2.,
            position.y - size / 2.,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    kind.sprite_column() as f32 * 64.,
                    ((age * 10.) as usize % 4) as f32 * 64.,
                    64.,
                    64.,
                )),
                dest_size: Some(vec2(size, size)),
                rotation,
                ..Default::default()
            },
        );
    }

    fn sprite(&self, column: usize, position: Vec2, scale: f32, color: Color) {
        draw_texture_ex(
            &self.atlas,
            (position.x - 32. * scale).floor(),
            (position.y - 50. * scale).floor(),
            color,
            DrawTextureParams {
                source: Some(Rect::new(column as f32 * 64., 0., 64., 64.)),
                dest_size: Some(vec2(64. * scale, 64. * scale)),
                ..Default::default()
            },
        );
    }

    pub fn text(&self, value: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(
            value,
            x.floor(),
            y.floor(),
            TextParams {
                font: Some(&self.font),
                font_size: 8,
                font_scale: size / 8.,
                color,
                ..Default::default()
            },
        );
    }

    fn center(&self, value: &str, x: f32, y: f32, size: f32, color: Color) {
        let measured = measure_text(value, Some(&self.font), 8, size / 8.);
        self.text(value, x - measured.width / 2., y, size, color);
    }
}

fn visible(p: Vec2, margin: f32) -> bool {
    p.x > -margin && p.y > -margin && p.x < WIDTH + margin && p.y < HEIGHT + margin
}
