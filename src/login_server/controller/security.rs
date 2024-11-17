use super::data::Login;
use crate::common::traits::IpBan;
use dashmap::DashMap;

impl Login {
    pub fn update_ip_ban_list(&self, ip: &str, ban_duration: i64) {
        let _ = self.ip_ban_list.remove(ip);
        self.ip_ban_list.insert(ip.to_string(), ban_duration);
    }
}
impl IpBan for Login {
    fn get_ban_list(&self) -> &DashMap<String, i64> {
        &self.ip_ban_list
    }
}
