#[cfg(test)]
pub mod l2_core {
    pub mod shared_packets {
        pub mod common {
            pub trait SendablePacket {
                fn get_bytes(&mut self, with_padding: bool) -> &mut [u8];
            }
        }
        pub mod write {
            use bytes::BytesMut;

            #[derive(Debug, Default)]
            pub struct SendablePacketBuffer {
                buffer: BytesMut,
            }
            impl SendablePacketBuffer {
                #[must_use]
                pub fn new() -> Self {
                    Self {
                        buffer: BytesMut::new(),
                    }
                }
                pub fn get_data_mut(&mut self, _: bool) -> &mut [u8] {
                    self.buffer.as_mut()
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_core::shared_packets::common::SendablePacket;
    use crate::l2_core::shared_packets::write::SendablePacketBuffer;
    use macro_common::SendablePacketImpl;

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
        let mut a = PacketA {
            buffer: SendablePacketBuffer::new(),
        };
        let p = a.get_bytes(false);
        assert_eq!(p, &[0u8; 0]);
    }
    #[test]
    fn packet_b() {
        let mut b = PacketB {
            buffer: SendablePacketBuffer::new(),
        };
        let p = b.get_bytes(false);
        assert_eq!(p, &[0u8; 0]);
    }
}
