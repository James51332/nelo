#pragma once

#include <filesystem>

extern "C"
{
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/avutil.h>
#include <libswscale/swscale.h>
}

namespace nelo
{

// An encoder is reponsible for outputting a video to a file. We'll use ffmpeg's internal libraries
// to accomplish this. The encoder selects the codec and outputs the file according to the format
// given in the output input.
class encoder
{
public:
  // The output format of the video is determined by the file extension.
  encoder(std::uint32_t width, std::uint32_t height, std::uint32_t fps,
          const std::filesystem::path& output, bool verbose = false);

  // Reads the data from the currently bound framebuffer into a video stream.
  void submit();

  // Ends the video and outputs a file. After this is called, we cannot add more frames.
  void end();

private:
  void init_ffmpeg();

private:
  // We only initalize the libraries once.
  inline static bool created_encoder = false;

private:
  // We can't end or submit if we're inactive.
  bool is_active = false;
  std::uint32_t width, height, fps;
  std::int32_t num_frames = 0;

  // This is the format of our output. It represents the output container.
  AVFormatContext* fmt_ctx = nullptr;

  // The encoder used to reduce the RAW video in size.
  const AVCodec* codec;

  // Our instance of the codec. The codec var stores the info about our codec type.
  AVCodecContext* codec_ctx = nullptr;

  // Represents one form of input to the output ctx. We will only have video.
  AVStream* stream = nullptr;

  // Our packet is used for communication with the codec.
  AVPacket* pkt = nullptr;

  // We have one frame in OpenGL RGB format, and another in YUV format, used by codecs.
  AVFrame* rgb_frame = nullptr;
  std::uint8_t* rgb_data = nullptr;
  AVFrame* yuv_frame = nullptr;

  // Context that encodes our output from RGB to YUV format.
  struct SwsContext* sws_ctx = nullptr;
};

} // namespace nelo
