use crate::proto;

pub fn hello(connection_id: String) -> proto::Packet {
    proto::Packet {
        data: Some(proto::packet::Data::Hello(proto::Hello { connection_id })),
    }
}
