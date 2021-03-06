#!/bin/sh

# play from webcam in b&w
# ffmpeg \
#   -loglevel debug \
#   -f v4l2 -framerate 25 -video_size 640x480 \
#   -i /dev/video0 \
#   -vcodec rawvideo -pix_fmt yuv420p \
#   -filter_complex "zmq, hue=s=0" \
#   -f v4l2 -f matroska - | ffplay -fflags nobuffer -

ffmpeg \
  -loglevel debug \
  -f v4l2 -framerate 25 -video_size 1280x720 \
  -input_format mjpeg \
  -i /dev/video0 \
  -vcodec rawvideo -pix_fmt yuv422p \
  -f v4l2 -f matroska - | ffplay -fflags nobuffer -
  # -filter_complex "zmq, hue=s=0" \
