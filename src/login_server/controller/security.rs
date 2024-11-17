use super::data::Login;
use chrono::Utc;
use dashmap::DashMap;

impl Login {
    pub fn update_ip_ban_list(&self, ip: &str, ban_duration: i64) {
        let _ = self.ip_ban_list.remove(ip);
        self.ip_ban_list.insert(ip.to_string(), ban_duration);
    }
    fn get_ban_list(&self) -> &DashMap<String, i64> {
        &self.ip_ban_list
    }
    pub fn is_ip_banned(&self, ip: &str) -> bool {
        let ban_list = self.get_ban_list();
        if let Some(res) = ban_list.get(ip) {
            let now = Utc::now().timestamp();
            if now < *res {
                return true;
            }
            ban_list.remove(ip);
        }
        false
    }
}
