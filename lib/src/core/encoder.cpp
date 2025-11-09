#include "core/encoder.h"

#include <glad/glad.h>
#include <stdexcept>

extern "C"
{
#include <libavutil/imgutils.h>
#include <libavutil/log.h>
#include <libavutil/opt.h>
}

#include "core/log.h"

namespace nelo
{

encoder::encoder(std::uint32_t width, std::uint32_t height, std::uint32_t fps,
                 const std::filesystem::path& output, bool verbose)
  : width(width), height(height), fps(fps)
{
  // Initializer avformat lib
  if (!created_encoder)
    init_ffmpeg();
  created_encoder = true;

  // Create our format context.
  avformat_alloc_output_context2(&fmt_ctx, nullptr, nullptr, output.string().c_str());
  if (!fmt_ctx)
    throw std::runtime_error("Unable to create output context for file: " + output.string());

  // Next we create the codec to reduce the video size from RAW frames.
  codec = avcodec_find_encoder(AV_CODEC_ID_H264);
  if (!codec)
    throw std::runtime_error("Unable to find H264 codec!");

  // Create our video stream.
  stream = avformat_new_stream(fmt_ctx, codec);
  if (!stream)
    throw std::runtime_error("Unable to create video output stream");

  // Create our specific codec context.
  codec_ctx = avcodec_alloc_context3(codec);
  codec_ctx->width = width;
  codec_ctx->height = height;
  codec_ctx->time_base = AVRational(1, static_cast<int>(fps));
  codec_ctx->pkt_timebase = AVRational(1, static_cast<int>(fps));
  codec_ctx->framerate = AVRational(static_cast<int>(fps), 1);
  codec_ctx->pix_fmt = AV_PIX_FMT_YUV420P;
  codec_ctx->gop_size = 12;
  codec_ctx->max_b_frames = 2;
  av_opt_set(codec_ctx->priv_data, "preset", "ultrafast", 0);

  // Now we open the codec.
  if (avcodec_open2(codec_ctx, codec, nullptr) < 0)
    throw std::runtime_error("Unable to open codec context!");

  // Pass the parameters from the stream to the codec.
  avcodec_parameters_from_context(stream->codecpar, codec_ctx);

  // Open output file
  if (!(fmt_ctx->oformat->flags & AVFMT_NOFILE))
    if (avio_open(&fmt_ctx->pb, output.string().c_str(), AVIO_FLAG_WRITE) < 0)
      throw std::runtime_error("Unable to write to output file!");

  // Write file header containing data about the video.
  if (avformat_write_header(fmt_ctx, nullptr) < 0)
    throw std::runtime_error("Unable to write file header to output file!");

  // Allocate our packet
  pkt = av_packet_alloc();
  pkt->time_base = AVRational(1, static_cast<int>(fps));
  pkt->duration = 1;

  // Allocate frames and buffers.
  yuv_frame = av_frame_alloc();
  yuv_frame->format = codec_ctx->pix_fmt;
  yuv_frame->width = codec_ctx->width;
  yuv_frame->height = codec_ctx->height;
  av_frame_get_buffer(yuv_frame, 32);

  // Allocate raw RGB data buffer.
  rgb_data = reinterpret_cast<uint8_t*>(av_malloc(3 * width * height));
  rgb_frame = av_frame_alloc();
  rgb_frame->format = AV_PIX_FMT_RGB24;
  rgb_frame->width = width;
  rgb_frame->height = height;
  av_image_fill_arrays(rgb_frame->data, rgb_frame->linesize, rgb_data, AV_PIX_FMT_RGB24, width,
                       height, 1);

  // Create scaler context to convert RGB to YUV
  sws_ctx = sws_getContext(width, height, AV_PIX_FMT_RGB24, width, height, AV_PIX_FMT_YUV420P,
                           SWS_BILINEAR, nullptr, nullptr, nullptr);

  // Mark ourselves as active.
  is_active = true;
}

void encoder::submit()
{
  if (!is_active)
    throw std::runtime_error("Unable to submit data because encoder has not begun encoding!");

  // Read the data into our rgb_data.
  glReadPixels(0, 0, width, height, GL_RGB, GL_UNSIGNED_BYTE, rgb_data);

  // We need to flip this before rendering into ffmpeg. It might be worth to flip on OpenGL side in
  // the future.
  int row_size = width * 3;
  uint8_t* temp_row = new uint8_t[row_size];
  for (int y = 0; y < height / 2; ++y)
  {
    uint8_t* row_top = rgb_data + y * row_size;
    uint8_t* row_bottom = rgb_data + (height - 1 - y) * row_size;

    // Swap the rows
    memcpy(temp_row, row_top, row_size);
    memcpy(row_top, row_bottom, row_size);
    memcpy(row_bottom, temp_row, row_size);
  }
  delete[] temp_row;

  // Convert RGB into YUV format used by codec.
  sws_scale(sws_ctx, (const uint8_t* const*)rgb_frame->data, rgb_frame->linesize, 0, height,
            yuv_frame->data, yuv_frame->linesize);

  // Record the frame number which is used for scrubbing.
  yuv_frame->pts = num_frames++;

  if (avcodec_send_frame(codec_ctx, yuv_frame))
    throw std::runtime_error("Unable to send frame to codec!");

  while (avcodec_receive_packet(codec_ctx, pkt) == 0)
  {
    av_packet_rescale_ts(pkt, codec_ctx->time_base, stream->time_base);
    av_interleaved_write_frame(fmt_ctx, pkt);
    av_packet_unref(pkt);
  }
}

void encoder::end()
{
  if (!is_active)
    throw std::runtime_error("Unable to end encoding because encoding has not begun!");
  is_active = false;

  // Flush encoder.
  avcodec_send_frame(codec_ctx, nullptr);
  while (avcodec_receive_packet(codec_ctx, pkt) == 0)
  {
    av_packet_rescale_ts(pkt, codec_ctx->time_base, stream->time_base);
    av_interleaved_write_frame(fmt_ctx, pkt);
    av_packet_unref(pkt);
  }

  // Write trailer and clean up.
  av_write_trailer(fmt_ctx);
  avcodec_free_context(&codec_ctx);
  av_packet_free(&pkt);
  av_frame_free(&yuv_frame);
  av_frame_free(&rgb_frame);
  av_free(rgb_data);
  sws_freeContext(sws_ctx);
  avio_closep(&fmt_ctx->pb);
  avformat_free_context(fmt_ctx);
}

static void ffmpeg_logger(void* ptr, int level, const char* fmt, va_list vargs)
{
  // We don't need to be overly verbose. TODO In the future, adjust using nelo verbosity.
  if (level > AV_LOG_INFO)
    return;

  // Format the ffmpeg message.
  char msg[1024];
  vsnprintf(msg, sizeof(msg), fmt, vargs);

  // Trim off trailing newlines since nelo adds them.
  std::string_view clean_msg(msg);
  if (!clean_msg.empty() && clean_msg.back() == '\n')
    clean_msg.remove_suffix(1);

  // Map FFmpeg log level to our own
  auto tag = [](int level) -> std::string_view
  {
    switch (level)
    {
      case AV_LOG_PANIC: return "PANIC";
      case AV_LOG_FATAL: return "FATAL";
      case AV_LOG_ERROR: return "ERROR";
      case AV_LOG_WARNING: return "WARN";
      case AV_LOG_INFO: return "INFO";
      case AV_LOG_VERBOSE: return "VERBOSE";
      case AV_LOG_DEBUG: return "DEBUG";
      case AV_LOG_TRACE: return "TRACE";
      default: return "LOG";
    }
  };

  // Now redirect to a buffer, file, or suppress
  log::out("[FFmpeg|{}] {}", tag(level), clean_msg);
}

void encoder::init_ffmpeg()
{
  avformat_network_init();

  // Be quiet unless nelo says otherwise.
  av_log_set_callback(ffmpeg_logger);
}

} // namespace nelo
