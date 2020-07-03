extern crate sdl2;

use std::path::PathBuf;

use ffmpeg4_ffi::sys;

use crate::av::decoder_ctx::DecoderCtx;
use crate::av::encoder_ctx::EncoderCtx;
use crate::opts;

pub fn run(args: opts::Forward) {
    unsafe {
        sys::avdevice_register_all();

        let input_path = args.input.as_str();
        let output_path = args.output.as_str();

        assert!(
            PathBuf::from(input_path).exists(),
            "file {} does not exist",
            input_path
        );

        let mut ctx = DecoderCtx::new(input_path);
        ctx.open_video_stream();

        let mut out_ctx = EncoderCtx::new_with_format(output_path, "v4l2");
        out_ctx.load_stream(&ctx, sys::AVCodecID_AV_CODEC_ID_RAWVIDEO);
        out_ctx.build_frame_context(&ctx);
        out_ctx.open_file(output_path);

        loop {
            ctx.read_frame();
            out_ctx.convert_frame(&ctx);
            out_ctx.encode(&ctx);
        }

        out_ctx.close_file();
    }
}
