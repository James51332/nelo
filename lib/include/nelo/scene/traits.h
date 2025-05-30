#pragma once

#include <concepts>
#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>

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

// We can also go ahead an define the specialization for the most common data types here.
template <>
struct timeline_traits<double>
{
  static double lerp(double a, double b, double t) { return a + (b - a) * t; }
  static double add(double a, double b) { return a + b; }
  static double multiply(double a, double b) { return a * b; }
};

template <>
struct timeline_traits<glm::vec2>
{
  static glm::vec2 lerp(glm::vec2 a, glm::vec2 b, double t) { return glm::mix(a, b, t); }
  static glm::vec2 add(glm::vec2 a, glm::vec2 b) { return a + b; }
  static glm::vec2 multiply(glm::vec2 a, glm::vec2 b) { return a * b; }
};

template <>
struct timeline_traits<glm::vec3>
{
  static glm::vec3 lerp(glm::vec3 a, glm::vec3 b, double t) { return glm::mix(a, b, t); }
  static glm::vec3 add(glm::vec3 a, glm::vec3 b) { return a + b; }
  static glm::vec3 multiply(glm::vec3 a, glm::vec3 b) { return a * b; }
};

template <>
struct timeline_traits<glm::vec4>
{
  static glm::vec4 lerp(glm::vec4 a, glm::vec4 b, double t) { return glm::mix(a, b, t); }
  static glm::vec4 add(glm::vec4 a, glm::vec4 b) { return a + b; }
  static glm::vec4 multiply(glm::vec4 a, glm::vec4 b) { return a * b; }
};

template <>
struct timeline_traits<glm::quat>
{
  static glm::quat lerp(glm::quat a, glm::quat b, double t)
  {
    return glm::slerp(a, b, static_cast<float>(t));
  }
  static glm::quat add(glm::quat a, glm::quat b) = delete;
  static glm::quat multiply(glm::quat a, glm::quat b) { return a * b; }
};

} // namespace nelo
