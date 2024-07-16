use rand::Rng;

#[derive(Clone, Debug)]
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
}
