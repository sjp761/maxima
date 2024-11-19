use maxima_resources::maxima_windows_rc;

fn main() -> std::io::Result<()> {
    prost_build::compile_protos(&["src/rtm/proto/rtm.proto"], &["src/rtm/proto/"])?;
    maxima_windows_rc("maximaserver", "Maxima Server")
}
