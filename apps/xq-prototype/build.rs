use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/bubbles_xq.proto"], &["proto/"])?;
    Ok(())
}
