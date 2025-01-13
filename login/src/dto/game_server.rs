use l2_core::dto::ServerHost;
use l2_core::shared_packets::common::ServerType;
use anyhow::bail;
use num::BigInt;
use pnet::ipnetwork::Ipv4Network;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct GSInfo {
    id: u8,
    accept_alternative_id: bool,
    host_reserved: bool,
    port: u16,
    is_authed: bool,
    status: i32,
    is_pvp: bool,
    server_type: i32,
    age_limit: u8,
    show_brackets: bool,
    max_players: u32,
    hex_id: Vec<u8>,
    hosts: Vec<ServerHost>,
}
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]
impl GSInfo {
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        id: u8,
        accept_alternative_id: bool,
        host_reserved: bool,
        port: u16,
        is_authed: bool,
        status: i32,
        is_pvp: bool,
        server_type: i32,
        age_limit: u8,
        show_brackets: bool,
        max_players: u32,
        hex_id: Vec<u8>,
        hosts: &[String],
    ) -> anyhow::Result<Self> {
        let validated_hosts = Self::validate_server_hosts(hosts)?;
        Ok(GSInfo {
            id,
            accept_alternative_id,
            host_reserved,
            port,
            is_authed,
            status,
            is_pvp,
            server_type,
            age_limit,
            show_brackets,
            max_players,
            hex_id,
            hosts: validated_hosts,
        })
    }

    pub fn get_host_ip(&self, client_ip: Ipv4Addr) -> Ipv4Addr {
        for s in &self.hosts {
            if s.subnet.contains(client_ip) {
                return s.ip;
            }
        }
        Ipv4Addr::new(127, 0, 0, 1)
    }
    pub fn hex(&self) -> String {
        BigInt::from_signed_bytes_be(&self.hex_id).to_str_radix(16)
    }
    pub fn hex_int(&self) -> BigInt {
        BigInt::from_signed_bytes_be(&self.hex_id)
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }
    pub fn accept_alternative_id(&self) -> bool {
        self.accept_alternative_id
    }
    pub fn is_host_reserved(&self) -> bool {
        self.host_reserved
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn is_authed(&self) -> bool {
        self.is_authed
    }
    pub fn get_status(&self) -> u8 {
        self.status as u8
    }
    pub fn is_pvp(&self) -> bool {
        self.is_pvp
    }
    pub fn get_server_type(&self) -> Option<ServerType> {
        ServerType::try_from(self.server_type).ok()
    }
    pub fn get_age_limit(&self) -> u8 {
        self.age_limit
    }
    pub fn show_brackets(&self) -> bool {
        self.show_brackets
    }
    pub fn get_max_players(&self) -> i32 {
        self.max_players as i32
    }
    pub fn set_max_players(&mut self, max_players: u32) {
        self.max_players = max_players;
    }
    pub fn set_age_limit(&mut self, age_limit: u8) {
        self.age_limit = age_limit;
    }
    pub fn use_square_brackets(&mut self, is_square_brackets: bool) {
        self.show_brackets = is_square_brackets;
    }
    pub fn set_server_type(&mut self, server_type: i32) {
        self.server_type = server_type;
    }
    pub fn set_server_status(&mut self, status: i32) {
        self.status = status;
    }
    pub fn get_hex_id(&self) -> Vec<u8> {
        self.hex_id.clone()
    }
    pub fn get_hosts(&self) -> Vec<ServerHost> {
        self.hosts.clone()
    }
    ///
    /// This function takes as an input an iterable of strings and try to compile a list of pairs:
    /// Subnet / Ip
    ///
    /// ```
    /// let hosts = vec!["127.0.0.5/32", "127.0.0.5"];
    /// let validated = GSInfo::validate_server_hosts(hosts);
    /// assert!(validated.is_ok());
    /// ```
    ///
    fn validate_server_hosts<I, T>(hosts: I) -> anyhow::Result<Vec<ServerHost>>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut validated_hosts = Vec::new();
        let mut iter = hosts.into_iter();

        loop {
            // Attempt to get the subnet and IP as a pair
            let Some(subnet_str) = iter.next() else { break };
            let Some(ip_str) = iter.next() else {
                bail!(
                    "Incorrect host data: Subnet provided but IP is missing {:?}",
                    subnet_str.as_ref()
                )
            };
            let subnet = Ipv4Network::from_str(subnet_str.as_ref())?;
            let ip = Ipv4Addr::from_str(ip_str.as_ref())?;
            if !subnet.contains(ip) {
                bail!("Subnet \"{subnet}\" doesn't contain IP \"{ip}\". Check game server configuration.")
            }
            validated_hosts.push(ServerHost { ip, subnet });
        }
        Ok(validated_hosts)
    }
}

#[cfg(test)]
mod test {
    use crate::dto::game_server::GSInfo;
    #[test]
    fn test_validated_hosts_error() {
        let hosts = vec!["127.0.0.1"];
        let validated = GSInfo::validate_server_hosts(hosts);
        assert!(validated.is_err());
    }
    #[test]
    fn test_validated_hosts_error_invalid_subnet() {
        let hosts = vec!["127.0.0.1", "127.0.0.2"];
        let validated = GSInfo::validate_server_hosts(hosts);
        assert!(validated.is_err());
    }
    #[test]
    fn test_validated_hosts_err_wrong_subnet_range() {
        let hosts = vec!["127.0.0.1/32", "127.0.0.5"];
        let validated = GSInfo::validate_server_hosts(hosts);
        assert!(validated.is_err());
    }
    #[test]
    fn test_validated_hosts_ok() {
        let hosts = vec!["127.0.0.5/32", "127.0.0.5"];
        let validated = GSInfo::validate_server_hosts(hosts);
        assert!(validated.is_ok());
    }
}
