use chrono::Utc;
use dashmap::DashMap;

pub trait IpBan {
    fn get_ban_list(&self) -> &DashMap<String, i64>;
    fn is_ip_banned(&self, ip: &str) -> bool {
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
