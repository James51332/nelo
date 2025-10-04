# FFmpeg has weird path requirements for.
cmake_minimum_required(VERSION 3.24)

# Allow user to override FFMPEG path manually
if (DEFINED ENV{FFMPEG_PATH})
    set(FFMPEG_HINTS $ENV{FFMPEG_PATH})
endif()

# Common Windows package manager install locations
if (WIN32)
    list(APPEND FFMPEG_HINTS
        "$ENV{USERPROFILE}\\scoop\\apps\\ffmpeg\\current"
        "C:/ProgramData/chocolatey/lib/ffmpeg/tools/ffmpeg"
        "C:/Program Files/ffmpeg"
        "C:/ffmpeg"
    )
endif()

# System package manager defaults (Linux/macOS)
list(APPEND FFMPEG_HINTS
    "/usr"
    "/usr/local"
    "/opt/homebrew"
)

# ----------- Find FFmpeg libraries -----------
find_path(AVCODEC_INCLUDE_DIR libavcodec/avcodec.h HINTS ${FFMPEG_HINTS} PATH_SUFFIXES include REQUIRED)
find_library(AVCODEC_LIBRARY avcodec HINTS ${FFMPEG_HINTS} PATH_SUFFIXES lib REQUIRED)

find_path(AVUTIL_INCLUDE_DIR libavutil/avutil.h HINTS ${FFMPEG_HINTS} PATH_SUFFIXES include REQUIRED)
find_library(AVUTIL_LIBRARY avutil HINTS ${FFMPEG_HINTS} PATH_SUFFIXES lib REQUIRED)

find_path(AVFORMAT_INCLUDE_DIR libavformat/avformat.h HINTS ${FFMPEG_HINTS} PATH_SUFFIXES include REQUIRED)
find_library(AVFORMAT_LIBRARY avformat HINTS ${FFMPEG_HINTS} PATH_SUFFIXES lib REQUIRED)

find_path(SWSCALE_INCLUDE_DIR libswscale/swscale.h HINTS ${FFMPEG_HINTS} PATH_SUFFIXES include REQUIRED)
find_library(SWSCALE_LIBRARY swscale HINTS ${FFMPEG_HINTS} PATH_SUFFIXES lib REQUIRED)

# ----------- Group into imported target -----------
add_library(ffmpeg INTERFACE)
target_include_directories(ffmpeg INTERFACE
    ${AVCODEC_INCLUDE_DIR}
    ${AVUTIL_INCLUDE_DIR}
    ${AVFORMAT_INCLUDE_DIR}
    ${SWSCALE_INCLUDE_DIR}
)
target_link_libraries(ffmpeg INTERFACE
    ${AVCODEC_LIBRARY}
    ${AVUTIL_LIBRARY}
    ${AVFORMAT_LIBRARY}
    ${SWSCALE_LIBRARY}
)
