use prost::Message;

pub mod proto;

fn main() {
    let packet = proto::Packet {
        connection_id: "123".to_owned(),
        data: Some(proto::packet::Data::Hello(proto::Hello {
            connection_id: "123".to_owned(),
        })),
    };
    println!("{:?}", packet);
    let v = packet.encode_to_vec();
    println!("{:?}", v);
}
