#[cfg(test)]
mod tests {
    use macro_common::SendablePacketImpl;
    use std::ptr;
    #[derive(Debug)]
    struct SendablePacketBuffer;
    trait SendablePacket {
        fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer;
        fn get_buffer(&self) -> &SendablePacketBuffer;
    }
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
