//! Sprite Management Module
//!
//! Manages the lifecycle of server-controlled sprites:
//! - CREATE: Spawn new sprites periodically
//! - UPDATE: Move sprites with random walk
//! - DELETE: Remove expired sprites after lifetime
//! - READ: Provide state snapshots

use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::Rng;

use crate::types::{SpriteData, SpriteMessage, SpritePosition, SpriteType};

const MAP_SIZE: i16 = 128; // 128x128 pixel map
const LIFETIME_SECS: u64 = 10; // Sprite lifetime in seconds
const SPAWN_INTERVAL_SECS: u64 = 3; // Spawn new sprite every N seconds
const SPRITE_STEP_SIZE: i16 = 10; // Move every N ticks (10 = 10x slower)

/// Manages all server-side sprites and their lifecycle
pub struct SpriteManager {
    sprites: HashMap<uuid::Uuid, SpriteData>,
    next_spawn_time: Instant,
}

impl SpriteManager {
    /// Create a new sprite manager
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            next_spawn_time: Instant::now(), // Spawn immediately on first tick
        }
    }

    /// Check if it's time to spawn a new sprite
    pub fn should_spawn(&self) -> bool {
        Instant::now() >= self.next_spawn_time
    }

    /// Spawn a new sprite at a random position
    /// Returns the Create message to send to clients
    pub fn spawn_sprite(&mut self) -> SpriteMessage {
        let id = uuid::Uuid::now_v7(); // Use v7 per rules
        let sprite_type = SpriteType::Serrif;
        let position = (0, 0); // Spawn at origin
        let spawn_time = Instant::now();
        let lifetime = Duration::from_secs(LIFETIME_SECS);

        let sprite_data = SpriteData {
            id,
            sprite_type,
            position,
            spawn_time,
            lifetime,
        };

        self.sprites.insert(id, sprite_data.clone());
        self.next_spawn_time = Instant::now() + Duration::from_secs(SPAWN_INTERVAL_SECS);

        tracing::info!(
            "[Sprite Create] serrif_{:?} at ({}, {})",
            id,
            position.0,
            position.1
        );

        SpriteMessage::create(sprite_type, id, position.0, position.1)
    }

    /// Update all active sprites with random walk
    /// Returns a list of Update messages for clients
    pub fn update_sprites(&mut self) -> Vec<SpriteMessage> {
        let mut updates = Vec::new();

        for sprite in self.sprites.values_mut() {
            let old_position = sprite.position;
            sprite.position = Self::random_walk(sprite.position);

            // Only send update if position changed
            if sprite.position != old_position {
                updates.push(SpriteMessage::update(
                    sprite.id,
                    sprite.position.0,
                    sprite.position.1,
                ));

                tracing::debug!(
                    "[Sprite Update] {} moved to ({}, {})",
                    sprite.id,
                    sprite.position.0,
                    sprite.position.1
                );
            }
        }

        updates
    }

    /// Remove sprites that have exceeded their lifetime
    /// Returns a list of Delete messages for clients
    pub fn cleanup_expired_sprites(&mut self) -> Vec<SpriteMessage> {
        let now = Instant::now();
        let mut deletes = Vec::new();
        let mut expired_ids = Vec::new();

        // Find expired sprites
        for (id, sprite) in &self.sprites {
            if now.duration_since(sprite.spawn_time) >= sprite.lifetime {
                expired_ids.push(*id);
            }
        }

        // Remove and create delete messages
        for id in expired_ids {
            if let Some(sprite) = self.sprites.remove(&id) {
                deletes.push(SpriteMessage::delete(id));

                tracing::info!(
                    "[Sprite Delete] serrif_{:?} at ({}, {})",
                    id,
                    sprite.position.0,
                    sprite.position.1
                );
            }
        }

        deletes
    }

    /// Get current state of all sprites
    /// Returns a Snapshot message for READ verification
    pub fn get_state_snapshot(&self) -> SpriteMessage {
        let sprite_count = self.sprites.len();

        tracing::info!("[Sprite Snapshot] {} active sprites", sprite_count);

        SpriteMessage::snapshot()
    }

    /// Get count of active sprites
    pub fn active_count(&self) -> usize {
        self.sprites.len()
    }

    /// Get all active sprite IDs
    pub fn active_ids(&self) -> Vec<uuid::Uuid> {
        self.sprites.keys().copied().collect()
    }

    /// Apply random walk: move 1 pixel in a random direction
    /// Only moves 1 out of SPRITE_STEP_SIZE ticks (10 = 10x slower)
    /// Clamps to 128x128 bounds
    fn random_walk(current: SpritePosition) -> SpritePosition {
        let (x, y) = current;
        let mut rng = rand::thread_rng();

        // Only move 1 out of SPRITE_STEP_SIZE ticks (slower movement)
        if rng.gen_range(0..SPRITE_STEP_SIZE) != 0 {
            return (x, y);
        }

        // Choose direction: 0=up, 1=down, 2=left, 3=right
        let direction = rng.gen_range(0..4);

        match direction {
            0 => {
                // up
                if y < MAP_SIZE - 1 {
                    (x, y + 1)
                } else {
                    (x, y)
                }
            }
            1 => {
                // down
                if y > 0 {
                    (x, y - 1)
                } else {
                    (x, y)
                }
            }
            2 => {
                // left
                if x > 0 {
                    (x - 1, y)
                } else {
                    (x, y)
                }
            }
            3 => {
                // right
                if x < MAP_SIZE - 1 {
                    (x + 1, y)
                } else {
                    (x, y)
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Default for SpriteManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SpriteOp;

    use std::time::Duration;

    #[test]
    fn test_sprite_manager_creation() {
        let manager = SpriteManager::new();
        assert_eq!(manager.active_count(), 0);
        assert!(manager.should_spawn());
    }

    #[test]
    fn test_spawn_sprite() {
        let mut manager = SpriteManager::new();

        let create_msg = manager.spawn_sprite();
        assert_eq!(manager.active_count(), 1);

        match create_msg.get_operation() {
            Some(SpriteOp::Create) => {
                // Verify position is in bounds
                assert!(create_msg.x >= 0 && create_msg.x < MAP_SIZE);
                assert!(create_msg.y >= 0 && create_msg.y < MAP_SIZE);

                // Verify sprite exists
                assert!(manager.sprites.contains_key(&create_msg.get_id()));
            }
            _ => panic!("Expected Create message"),
        }
    }

    #[test]
    fn test_random_walk_bounds() {
        let _manager = SpriteManager::new();

        // Test corners
        let positions = vec![
            (0, 0),                       // bottom-left
            (MAP_SIZE - 1, 0),            // bottom-right
            (0, MAP_SIZE - 1),            // top-left
            (MAP_SIZE - 1, MAP_SIZE - 1), // top-right
        ];

        for pos in positions {
            let new_pos = SpriteManager::random_walk(pos);
            assert!(new_pos.0 >= 0 && new_pos.0 < MAP_SIZE);
            assert!(new_pos.1 >= 0 && new_pos.1 < MAP_SIZE);
        }
    }

    #[test]
    fn test_cleanup_expired_sprites() {
        let mut manager = SpriteManager::new();

        // Spawn a sprite with very short lifetime
        let id = uuid::Uuid::now_v7();
        let sprite = SpriteData {
            id,
            sprite_type: SpriteType::Serrif,
            position: (10, 10),
            spawn_time: Instant::now() - Duration::from_secs(20), // 20s ago
            lifetime: Duration::from_secs(10),
        };

        manager.sprites.insert(id, sprite);
        assert_eq!(manager.active_count(), 1);

        // Cleanup should remove it
        let deletes = manager.cleanup_expired_sprites();
        assert_eq!(deletes.len(), 1);
        assert_eq!(manager.active_count(), 0);

        match deletes[0].get_operation() {
            Some(SpriteOp::Delete) => {
                assert_eq!(deletes[0].get_id(), id);
            }
            _ => panic!("Expected Delete message"),
        }
    }

    #[test]
    fn test_state_snapshot() {
        let mut _manager = SpriteManager::new();
        _manager.spawn_sprite();
        _manager.spawn_sprite();

        let snapshot = _manager.get_state_snapshot();
        match snapshot.get_operation() {
            Some(SpriteOp::Snapshot) => {
                // Snapshot messages don't contain detailed data in this implementation
                // Just verify it's a snapshot
            }
            _ => panic!("Expected Snapshot message"),
        }
    }

    #[test]
    fn test_update_sprites() {
        let mut manager = SpriteManager::new();
        manager.spawn_sprite();

        // Multiple updates should work
        for _ in 0..10 {
            let updates = manager.update_sprites();
            // May or may not have updates (random walk might stay in place)
            // But should never crash
            assert!(updates.len() <= 1);
        }

        assert_eq!(manager.active_count(), 1);
    }
}
