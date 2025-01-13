fn main() {
    std::env::set_var("OUT_DIR", "src/");
    if let Err(error) = prost_build::compile_protos(&["../proto/packet.proto"], &["../proto/"]) {
        eprintln!("compile_protos error: {:?}", error);
    }
}
