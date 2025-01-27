#[cfg(test)]
pub mod l2_core {
    pub mod shared_packets {
        pub mod common {
            use crate::l2_core::shared_packets::write::SendablePacketBuffer;

            pub trait SendablePacket {
                fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer;
                fn get_buffer(&self) -> &SendablePacketBuffer;
            }
        }
        pub mod write {
            #[derive(Debug)]
            pub struct SendablePacketBuffer;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_core::shared_packets::common::SendablePacket;
    use crate::l2_core::shared_packets::write::SendablePacketBuffer;
    use macro_common::SendablePacketImpl;
    use std::ptr;

    #[derive(SendablePacketImpl, Debug)]
    struct PacketA {
        buffer: SendablePacketBuffer,
    }
    #[derive(SendablePacketImpl, Debug)]
    struct PacketB {
        buffer: SendablePacketBuffer,
    }
    #[test]
    fn packet_a() {
        let a = PacketA {
            buffer: SendablePacketBuffer {},
        };
        assert!(ptr::eq(&a.buffer, a.get_buffer()));
    }
    #[test]
    fn packet_b() {
        let mut b = PacketB {
            buffer: SendablePacketBuffer {},
        };
        assert!(ptr::eq(&b.buffer, b.get_buffer_mut()));
    }
}
