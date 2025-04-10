use hound::WavReader;
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
        encoder.set_sample_rate(22_000).expect("set sample rate"); // tts_simpleの出力結果から決め打ち
        encoder.set_brate(Bitrate::Kbps128).expect("set brate");
        encoder.set_quality(Quality::Best).expect("set quality");

        let mut encoder = encoder.build().expect("To innitialize LAME enocder");

        // tts_simpleの出力結果からi16で決め打ち
        let mut sound_reader = WavReader::new(sound.as_slice()).unwrap();
        let samples = sound_reader.samples::<i16>()
                                .map(|x| x.unwrap())
                                .collect::<Vec<i16>>();

        let pcm = MonoPcm(samples.as_slice());

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