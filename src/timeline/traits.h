#pragma once

#include <concepts>

namespace nelo
{

// In order for a type to be used a the data type for a timeline, it must
// specialize the timeline_traits. This allows each type to choose to define (or
// not define) the method for lerping, adding, or multiplying.
template <typename T>
struct timeline_traits
{
  // A protocol for linearly interpolating between two types (used for
  // keyframes).
  static T lerp(T a, T b, double t);

  // A protocol for adding two types (additive animations).
  static T add(T a, T b);

  // A protocol for composing two types (multiplicative animations).
  static T multiply(T a, T b);
};

// Implement C++20 concepts to easily check if a type defines these methods.
template <typename T>
concept lerpable = requires(T a, T b) {
  { timeline_traits<T>::lerp(a, b, 0.5) } -> std::same_as<T>;
};

template <typename T>
concept addable = requires(T a, T b) {
  { timeline_traits<T>::add(a, b) } -> std::same_as<T>;
};

template <typename T>
concept multipliable = requires(T a, T b) {
  { timeline_traits<T>::multiply(a, b) } -> std::same_as<T>;
};

} // namespace nelo
