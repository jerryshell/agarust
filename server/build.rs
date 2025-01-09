fn main() -> std::io::Result<()> {
    std::env::set_var("OUT_DIR", "src/");
    prost_build::compile_protos(&["../proto/packet.proto"], &["../proto/"])?;
    Ok(())
}
