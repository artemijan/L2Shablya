use anyhow::anyhow;
use l2_core::game_objects::player::{Player, PlayerMacro};
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroUpdateType {
    Add,
    List,
    Modify,
    Delete,
}

impl MacroUpdateType {
    pub fn id(self) -> u8 {
        match self {
            MacroUpdateType::List | MacroUpdateType::Add => 1,
            MacroUpdateType::Modify => 2,
            MacroUpdateType::Delete => 0,
        }
    }
}

#[derive(Debug, Clone, SendablePacket)]
pub struct MacroList {
    pub(crate) buffer: SendablePacketBuffer,
    count: u8,
    update_type: MacroUpdateType,
}

impl MacroList {
    pub const PACKET_ID: u8 = 0xE8;

    pub fn list_macros(p: &Player) -> anyhow::Result<Vec<Self>> {
        let macros = p.get_macros();
        if macros.is_empty() {
            Ok(vec![Self::new(0, None, MacroUpdateType::List)?])
        } else {
            let macros_len = macros.len();
            let mut result = vec![];
            for m in macros {
                result.push(MacroList::new(
                    u8::try_from(macros_len)?,
                    Some(m),
                    MacroUpdateType::List,
                )?);
            }
            Ok(result)
        }
    }
    pub fn new(
        count: u8,
        m: Option<&PlayerMacro>,
        update: MacroUpdateType,
    ) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            count,
            update_type: update,
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write(update.id())?;
        let macro_id = if update == MacroUpdateType::List {
            0
        } else {
            m.ok_or(anyhow!("No macro provided"))?.id
        };
        inst.buffer.write_i32(macro_id)?;
        inst.buffer.write(count)?;
        inst.buffer.write_bool(m.is_some())?;
        if let Some(m) = m && update != MacroUpdateType::Delete{
                inst.buffer.write_i32(m.id)?;
                inst.buffer.write_c_utf16le_string(Some(&m.name))?;
                inst.buffer.write_c_utf16le_string(Some(&m.description))?;
                inst.buffer.write_c_utf16le_string(Some(&m.acronym))?;
                inst.buffer.write_i32(m.icon)?;
                inst.buffer.write(u8::try_from(m.commands.len())?)?;
                for (i, cmd) in m.commands.iter().enumerate() {
                    inst.buffer.write(u8::try_from(i + 1)?)?;
                    inst.buffer.write(cmd.get_type())?;
                    inst.buffer.write_i32(cmd.get_d1())?;
                    inst.buffer.write_i32(cmd.get_d2())?;
                    inst.buffer
                        .write_c_utf16le_string(Some(cmd.get_cmd_name()))?;
                }
        }
        Ok(inst)
    }
}
#[cfg(test)]
mod test {
    use crate::controller::GameController;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::data::classes::mapping::Class;
    use l2_core::game_objects::player::Player;
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use crate::packets::to_client::MacroList;

    #[tokio::test]
    async fn test_write_macro_list() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let char = char_factory(&db_pool, |mut m| {
            m.name = "Adelante".to_string();
            m.user_id = user.id;
            m
        })
            .await;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = GameController::from_config(cfg).await;
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let mut p = MacroList::list_macros(&player).unwrap();
        assert_eq!(p.len(), 1);
        let pack = p.pop().unwrap();
        assert_eq!(
            [232, 1, 0, 0, 0, 0, 0, 0],
            pack.get_buffer().get_data_mut(false)[2..]
        );
    }
}
