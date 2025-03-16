use vvcore::*;
use mp3lame_encoder::{Bitrate, Builder, FlushNoGap, MonoPcm, Quality};
use rodio::Decoder;

use std::io::Write;

enum Speakers{
    MetanAma = 0,
    ZundAma = 1,
    Chibi =42,
}

fn main(){
    let dict_dir = std::ffi::CString::new("open_jtalk_dic_utf_8-1.11").unwrap();
    let speaker = Speakers::ZundAma as u32; //複数回使用するためプリミティブにキャスト
    let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, false, dict_dir.as_c_str()).unwrap();
    vvc.load_model(speaker).unwrap();
    
    for i in 0..=12 {
        let sound = vvc.tts_simple(&format!("{}時", i), speaker).unwrap();
        let mut encoder = Builder::new().expect("Create LAME builder");
        encoder.set_num_channels(1).expect("set channels");
        encoder.set_sample_rate(44_100).expect("set sample rate");
        encoder.set_brate(Bitrate::Kbps128).expect("set brate");
        encoder.set_quality(Quality::Best).expect("set quality");

        let mut encoder = encoder.build().expect("To innitialize LAME enocder");

        let u16_sound = sound.as_slice().iter().map(|&x| x as u16).collect::<Vec<u16>>();
        let pcm = MonoPcm(u16_sound.as_slice());

        let mut out_buffer = Vec::new();
        out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(pcm.0.len()));
        let encoded_size = encoder.encode(pcm, out_buffer.spare_capacity_mut()).expect("To encode");

        unsafe {
            out_buffer.set_len(out_buffer.len().wrapping_add(encoded_size));
        }

        let encoded_size = encoder.flush::<FlushNoGap>(out_buffer.spare_capacity_mut()).expect("to flush");

        unsafe {
            out_buffer.set_len(out_buffer.len().wrapping_add(encoded_size));
        }

        // 音声の確認
        // let cursor = std::io::Cursor::new(sound.as_slice().to_vec());
        // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        // let sink = rodio::Sink::try_new(&handle).unwrap();
        // let source = Decoder::new(cursor).unwrap();
        // sink.append(source);
        // sink.sleep_until_end();

        let mut file = std::fs::File::create(format!("{}.mp3", i)).unwrap();
        file.write_all(&out_buffer).expect("To write mp3 file");
    }


}