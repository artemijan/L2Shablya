use rand::Rng;

#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SessionKey {
    pub play_ok1: i32,
    pub play_ok2: i32,
    pub login_ok1: i32,
    pub login_ok2: i32,
}

impl SessionKey {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        SessionKey {
            play_ok1: rng.gen(),
            play_ok2: rng.gen(),
            login_ok1: rng.gen(),
            login_ok2: rng.gen(),
        }
    }
    pub fn equals(&self, other: &SessionKey, show_license: bool) -> bool {
        let is_play_ok = self.play_ok1 == other.play_ok1 && self.play_ok2 == other.play_ok2;
        if show_license {
            is_play_ok && self.login_ok1 == other.login_ok1 && self.login_ok2 == other.login_ok2
        } else {
            is_play_ok
        }
    }
}
