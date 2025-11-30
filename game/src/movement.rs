use kameo::actor::ActorRef;
use std::time::Instant;
use tokio::task::JoinHandle;

/// Represents the current movement state of a player
#[derive(Debug)]
pub struct MovementState {
    /// Starting position (x, y, z)
    pub source_x: i32,
    pub source_y: i32,
    pub source_z: i32,
    
    /// Destination position (x, y, z)
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_z: i32,
    
    /// When the movement started
    pub start_time: Instant,
    
    /// Movement speed in game units per second
    pub speed: f64,
    
    /// Handle to the periodic broadcast task
    pub task_handle: Option<JoinHandle<()>>,
}

impl MovementState {
    /// Create a new movement state
    pub fn new(
        source_x: i32,
        source_y: i32,
        source_z: i32,
        dest_x: i32,
        dest_y: i32,
        dest_z: i32,
        speed: u16,
    ) -> Self {
        Self {
            source_x,
            source_y,
            source_z,
            dest_x,
            dest_y,
            dest_z,
            start_time: Instant::now(),
            speed: f64::from(speed),
            task_handle: None,
        }
    }

    /// Calculate the total distance to travel
    fn calculate_distance(&self) -> f64 {
        let dx = f64::from(self.dest_x - self.source_x);
        let dy = f64::from(self.dest_y - self.source_y);
        let dz = f64::from(self.dest_z - self.source_z);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate how long the entire journey should take (in seconds)
    pub fn calculate_travel_duration(&self) -> f64 {
        let distance = self.calculate_distance();
        if self.speed > 0.0 {
            distance / self.speed
        } else {
            0.0
        }
    }

    /// Calculate the current interpolated position based on elapsed time
    pub fn calculate_current_position(&self) -> (i32, i32, i32) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let duration = self.calculate_travel_duration();

        if duration <= 0.0 || elapsed >= duration {
            // Already arrived or instant movement
            return (self.dest_x, self.dest_y, self.dest_z);
        }

        let progress = (elapsed / duration).min(1.0);

        let current_x = self.source_x + ((self.dest_x - self.source_x) as f64 * progress) as i32;
        let current_y = self.source_y + ((self.dest_y - self.source_y) as f64 * progress) as i32;
        let current_z = self.source_z + ((self.dest_z - self.source_z) as f64 * progress) as i32;

        (current_x, current_y, current_z)
    }

    /// Check if the player has arrived at the destination
    pub fn has_arrived(&self) -> bool {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let duration = self.calculate_travel_duration();
        elapsed >= duration
    }

    /// Cancel the periodic broadcast task if it exists
    pub fn cancel_task(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

impl Drop for MovementState {
    fn drop(&mut self) {
        self.cancel_task();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_distance() {
        let state = MovementState::new(0, 0, 0, 300, 400, 0, 100);
        let distance = state.calculate_distance();
        assert_eq!(distance, 500.0); // 3-4-5 triangle
    }

    #[test]
    fn test_calculate_travel_duration() {
        let state = MovementState::new(0, 0, 0, 500, 0, 0, 100);
        let duration = state.calculate_travel_duration();
        assert_eq!(duration, 5.0); // 500 units at 100 units/sec = 5 seconds
    }

    #[test]
    fn test_calculate_current_position_at_start() {
        let state = MovementState::new(0, 0, 0, 1000, 0, 0, 100);
        let (x, y, z) = state.calculate_current_position();
        // Should be at or very near start position
        assert!(x >= 0 && x < 50); // Allow small movement
        assert_eq!(y, 0);
        assert_eq!(z, 0);
    }

    #[test]
    fn test_has_arrived_immediately() {
        let state = MovementState::new(0, 0, 0, 0, 0, 0, 100);
        assert!(state.has_arrived()); // No distance = instant arrival
    }
}
