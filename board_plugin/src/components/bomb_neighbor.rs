use bevy::prelude::Component;

/// Bomb neighbor component
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Debug, Copy, Clone, Component)]
pub struct BombNeighbor {
    /// Number of neighbor bombs
    pub count: u8,
}
