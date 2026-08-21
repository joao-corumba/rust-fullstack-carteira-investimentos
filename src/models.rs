use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub quantity: f64,
}

impl Asset {
    /// Total value currently held of this asset (unit_value * quantity).
    pub fn total_value(&self) -> f64 {
        self.unit_value * self.quantity
    }
}

pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}
