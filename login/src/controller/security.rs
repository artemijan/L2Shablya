use super::data::Login;
use l2_core::traits::IpBan;
use chrono::Utc;

impl Login {
    pub fn update_ip_ban_list(&self, ip: &str, ban_duration: i64) {
        let _ = self.ip_ban_list.remove(ip);
        self.ip_ban_list.insert(ip.to_string(), ban_duration);
    }
}
impl IpBan for Login {
    fn is_ip_banned(&self, ip: &str) -> bool {
        if let Some(res) = self.ip_ban_list.get(ip) {
            let now = Utc::now().timestamp();
            if now < *res {
                return true;
            }
            self.ip_ban_list.remove(ip);
        }
        false
    }
}
