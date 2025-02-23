use rand::Rng;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct SessionKey {
    pub play_ok1: i32,
    pub play_ok2: i32,
    pub login_ok1: i32,
    pub login_ok2: i32,
}

impl SessionKey {
    #[must_use]
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        SessionKey {
            play_ok1: rng.r#gen(),
            play_ok2: rng.r#gen(),
            login_ok1: rng.r#gen(),
            login_ok2: rng.r#gen(),
        }
    }
    #[must_use]
    pub fn check_session(&self, s_key_1: i32, s_key_2: i32) -> bool {
        s_key_1 == self.login_ok1 && s_key_2 == self.login_ok2
    }
    #[must_use]
    pub fn equals(&self, other: &SessionKey, show_license: bool) -> bool {
        let is_play_ok = self.play_ok1 == other.play_ok1 && self.play_ok2 == other.play_ok2;
        if show_license {
            is_play_ok && self.login_ok1 == other.login_ok1 && self.login_ok2 == other.login_ok2
        } else {
            is_play_ok
        }
    }
    #[must_use]
    pub fn get_play_session_id(&self) -> i32 {
        self.play_ok1
    }
}

#[cfg(test)]
mod tests {
    use crate::session::SessionKey;

    #[test]
    fn test_session_key_not_equals() {
        let session_key = SessionKey::new();
        assert!(!session_key.equals(&SessionKey::new(), false));
        assert!(!session_key.equals(&SessionKey::new(), true));
    }
    #[test]
    fn test_session_key_equals() {
        let session_key = SessionKey::new();
        let other = session_key.clone();
        assert!(session_key.equals(&other, false));
        assert!(session_key.equals(&other, true));
    }
    #[test]
    fn test_session_key_check_session() {
        let session_key = SessionKey::new();
        assert!(session_key.check_session(session_key.login_ok1, session_key.login_ok2));
    }
}
