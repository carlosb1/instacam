use std::ptr::null_mut;
use std::slice;

use ffmpeg4_ffi::extra::defs::{averror, averror_eof, eagain};
use ffmpeg4_ffi::sys;

use super::utils;

use crate::app::settings::Settings;

pub struct DecoderCtx {
    pub av: *mut sys::AVFormatContext,
    pub video_stream_index: usize,
    pub codec: *mut sys::AVCodec,
    pub codec_ctx: *mut sys::AVCodecContext,
    pub stream: *mut sys::AVStream,
    pub packet: *mut sys::AVPacket,
}

impl DecoderCtx {
    pub fn open(settings: &Settings) -> DecoderCtx {
        unsafe {
            let path = settings.input.clone();

            let mut av = sys::avformat_alloc_context();

            let mut options: *mut sys::AVDictionary = null_mut();

            let framerate_key = utils::str_to_c_str("framerate");
            sys::av_dict_set_int(&mut options, framerate_key.as_ptr(), settings.fps, 0);

            sys::av_dict_set(
                &mut options,
                utils::str_to_c_str("video_size").as_ptr(),
                utils::string_to_c_str(format!("{}x{}", settings.width, settings.height)).as_ptr(),
                0,
            );

            sys::av_dict_set(
                &mut options,
                utils::str_to_c_str("input_format").as_ptr(),
                utils::str_to_c_str("mjpeg").as_ptr(),
                0,
            );

            let response = sys::avformat_open_input(
                &mut av,
                utils::str_to_c_str(path.as_str()).as_ptr(),
                null_mut(),
                &mut options,
            );

            println!("{:?}", response);
            if utils::check_error(response) {
                panic!("could not open {}", path.as_str());
            }

            let mut decoder = DecoderCtx {
                av,
                video_stream_index: 0,
                stream: null_mut(),
                codec: null_mut(),
                codec_ctx: null_mut(),
                packet: sys::av_packet_alloc(),
            };

            decoder.open_video_stream();

            decoder
        }
    }

    fn open_video_stream(&mut self) {
        unsafe {
            // load stream info
            sys::avformat_find_stream_info(self.av, null_mut());

            let index = sys::av_find_best_stream(
                self.av,
                sys::AVMediaType_AVMEDIA_TYPE_VIDEO,
                -1,
                -1,
                null_mut(),
                0,
            );

            if utils::check_error(index) {
                panic!("Could not find video stream");
            }

            self.video_stream_index = index as usize;
            self.stream = self.get_stream(self.video_stream_index);

            self.codec = sys::avcodec_find_decoder((*(*self.stream).codecpar).codec_id);
            self.codec_ctx = sys::avcodec_alloc_context3(self.codec);

            let mut options: *mut sys::AVDictionary = null_mut();
            sys::av_dict_set_int(
                &mut options,
                utils::str_to_c_str("refcounted_frames").as_ptr(),
                1,
                0,
            );

            sys::avcodec_parameters_to_context(self.codec_ctx, (*self.stream).codecpar);
            sys::avcodec_open2(self.codec_ctx, self.codec, &mut options);
        }
    }

    fn get_streams<'b>(&self) -> &'b [*mut sys::AVStream] {
        unsafe {
            let ptr = (*self.av).streams;
            let count = (*self.av).nb_streams as usize;

            slice::from_raw_parts(ptr, count)
        }
    }

    pub fn get_stream(&self, i: usize) -> *mut sys::AVStream {
        self.get_streams()[i]
    }

    pub fn decode_frame(&mut self, frame: &*mut sys::AVFrame) {
        unsafe {
            while sys::av_read_frame(self.av, self.packet) >= 0 {
                if (*self.packet).stream_index as usize == self.video_stream_index {
                    let response = self.decode_packet(frame);

                    if response < 0 {
                        break;
                    }
                }
            }
        }
    }

    fn decode_packet(&mut self, frame: &*mut sys::AVFrame) -> i32 {
        unsafe {
            let mut response;

            // decode packet
            response = sys::avcodec_send_packet(self.codec_ctx, self.packet);

            if utils::check_error(response) {
                panic!("failed to send packet");
            }

            while response >= 0 {
                response = sys::avcodec_receive_frame(self.codec_ctx, *frame);

                // eagain -> need to try again
                // eof -> input is over, not an actual error here
                if response == averror(eagain()) || response == averror(averror_eof()) {
                    break;
                } else if utils::check_error(response) {
                    return response;
                }

                if response >= 0 {
                    return -1;
                }
            }
        }

        0
    }
}
