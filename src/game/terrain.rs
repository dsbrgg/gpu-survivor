use macroquad::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;

pub const CHUNK_SIZE: f32 = 384.;
pub const STREAM_RADIUS: i32 = 2;
pub const START: Vec2 = Vec2::new(192., 192.);

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Climate {
    Normal,
    Cold,
    Hot,
}

impl Climate {
    pub fn movement_multiplier(self) -> f32 {
        if self == Self::Cold {
            0.62
        } else {
            1.
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub position: Vec2,
    pub sprite: usize,
}

impl Prop {
    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x - 17., self.position.y - 12., 34., 20.)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub origin: Vec2,
    pub climate: Climate,
    pub zone: Rect,
    pub props: Vec<Prop>,
}

pub struct Terrain {
    seed: u64,
    center: Option<(i32, i32)>,
    pub chunks: BTreeMap<(i32, i32), Chunk>,
}

pub fn chunk_at(position: Vec2) -> (i32, i32) {
    (
        (position.x / CHUNK_SIZE).floor() as i32,
        (position.y / CHUNK_SIZE).floor() as i32,
    )
}

fn mix(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

impl Terrain {
    pub fn new(seed: u64) -> Self {
        let mut terrain = Self {
            seed,
            center: None,
            chunks: BTreeMap::new(),
        };
        terrain.stream(START);
        terrain
    }

    fn generate(&self, x: i32, y: i32) -> Chunk {
        // Coordinate hashing never consumes the combat RNG.
        let hash = mix(self.seed
            ^ mix(x as i64 as u64)
            ^ mix((y as i64 as u64).wrapping_add(0x9e3779b97f4a7c15)));
        let origin = vec2(x as f32, y as f32) * CHUNK_SIZE;
        let climate = match (x, y) {
            (0, 0) => Climate::Normal,
            (-1, 0) => Climate::Cold,
            (1, 0) => Climate::Hot,
            _ => match hash % 10 {
                0..=2 => Climate::Cold,
                3..=5 => Climate::Hot,
                _ => Climate::Normal,
            },
        };
        let inset = 24. + ((hash >> 8) % 3) as f32 * 8.;
        let zone = Rect::new(
            origin.x + inset,
            origin.y + 40.,
            CHUNK_SIZE - inset * 2.,
            304.,
        );
        let mut props = Vec::new();
        for (row, py) in [80., 136., 280., 336.].into_iter().enumerate() {
            for (column, px) in [72., 112., 152., 248., 288., 328.].into_iter().enumerate() {
                let variant = mix(hash.wrapping_add((row * 6 + column) as u64));
                props.push(Prop {
                    position: origin + vec2(px, py),
                    sprite: match variant % 32 {
                        0..=2 => 2,
                        3..=4 => 3,
                        5..=7 => 4,
                        8..=9 => 5,
                        10..=12 => 6,
                        13 => 7,
                        _ => (variant % 2) as usize,
                    },
                });
            }
        }
        Chunk {
            origin,
            climate,
            zone,
            props,
        }
    }

    pub fn stream(&mut self, position: Vec2) {
        let center = chunk_at(position);
        if self.center == Some(center) {
            return;
        }
        self.center = Some(center);
        self.chunks.retain(|&(x, y), _| {
            (x - center.0).abs() <= STREAM_RADIUS && (y - center.1).abs() <= STREAM_RADIUS
        });
        for y in center.1 - STREAM_RADIUS..=center.1 + STREAM_RADIUS {
            for x in center.0 - STREAM_RADIUS..=center.0 + STREAM_RADIUS {
                if !self.chunks.contains_key(&(x, y)) {
                    let chunk = self.generate(x, y);
                    self.chunks.insert((x, y), chunk);
                }
            }
        }
    }

    pub fn climate_at(&self, position: Vec2) -> Climate {
        self.chunks
            .get(&chunk_at(position))
            .filter(|chunk| chunk.zone.contains(position))
            .map_or(Climate::Normal, |chunk| chunk.climate)
    }

    pub fn move_actor(&self, position: Vec2, delta: Vec2, radius: f32) -> Vec2 {
        let mut result = position;
        for axis in 0..2 {
            result[axis] += delta[axis];
            // Racks are inset farther than the largest actor radius from chunk seams.
            if let Some(chunk) = self.chunks.get(&chunk_at(result)) {
                for prop in &chunk.props {
                    let b = prop.bounds();
                    if result.x > b.x - radius
                        && result.x < b.right() + radius
                        && result.y > b.y - radius
                        && result.y < b.bottom() + radius
                    {
                        result[axis] = if delta[axis] > 0. {
                            if axis == 0 {
                                b.x - radius
                            } else {
                                b.y - radius
                            }
                        } else if delta[axis] < 0. {
                            if axis == 0 {
                                b.right() + radius
                            } else {
                                b.bottom() + radius
                            }
                        } else {
                            result[axis]
                        };
                    }
                }
            }
        }
        result
    }

    pub fn open_position(&self, position: Vec2, radius: f32) -> Vec2 {
        if let Some(chunk) = self.chunks.get(&chunk_at(position)) {
            if chunk.props.iter().any(|prop| {
                let bounds = prop.bounds();
                Rect::new(
                    bounds.x - radius,
                    bounds.y - radius,
                    bounds.w + radius * 2.,
                    bounds.h + radius * 2.,
                )
                .contains(position)
            }) {
                return vec2(position.x, chunk.origin.y + 208.);
            }
        }
        position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_regenerate_independently_of_visit_order_with_bounded_residency() {
        let mut terrain = Terrain::new(42);
        let original = terrain.chunks.clone();
        assert_eq!(chunk_at(vec2(-0.1, -384.1)), (-1, -2));
        for i in -50..50 {
            terrain.stream(vec2(i as f32 * CHUNK_SIZE, i as f32 * -CHUNK_SIZE));
            assert_eq!(terrain.chunks.len(), 25);
            assert_eq!(
                terrain
                    .chunks
                    .values()
                    .map(|c| c.props.len())
                    .sum::<usize>(),
                600
            );
        }
        terrain.stream(START);
        assert_eq!(terrain.chunks, original);
        assert_ne!(Terrain::new(43).chunks, original);
    }

    #[test]
    fn aisles_cross_chunk_seams_and_racks_block_both_axes() {
        let mut terrain = Terrain::new(42);
        for direction in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            let mut position = START;
            for _ in 0..2000 {
                terrain.stream(position);
                position = terrain.move_actor(position, direction * 3., 8.);
            }
            assert!(position.distance(START + direction * 6000.) < 0.01);
        }
        terrain.stream(START);
        let rack = &terrain.chunks[&(0, 0)].props[0];
        let start = rack.position + vec2(0., 40.);
        let mut position = start;
        for _ in 0..60 {
            position = terrain.move_actor(position, vec2(0., -2.), 8.);
        }
        assert!(position.y >= rack.bounds().bottom() + 8.);
    }
}
