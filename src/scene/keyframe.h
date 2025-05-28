#pragma once

#include "easing.h"

namespace nelo
{

// Keyframes are critical states in a timeline. We define exactly what the state should be at a
// certain time, and the timeline blends between them in intermediate times. If it is not possible
// to blend between states in a well-defined manner (e.g. text), then keyframes use abrubt jumps as
// soon as they arrive. There can be multiple keyframes at the same time. The keyframes retain the
// order they are inserted with. This may be useful if the user wants to blend to one state then
// abrubtly jump to another.
template <typename T>
struct keyframe
{
  double at;
  T value;
  easing_func easing;
};

// CTAD allows compiler to guess type of keyframe from given argument.
template <typename T>
keyframe(T) -> keyframe<T>;

// Use doubles whenever possible
keyframe(int) -> keyframe<double>;
keyframe(float) -> keyframe<double>;

} // namespace nelo
