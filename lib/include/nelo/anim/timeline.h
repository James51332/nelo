#pragma once

#include <cstdint>
#include <memory>
#include <stdexcept>
#include <type_traits>
#include <unordered_set>
#include <vector>

#include "anim/keyframe.h"
#include "anim/traits.h"

namespace nelo
{

// Timeline is an implicitly ref counted object which can be evaluated at any given time.
template <typename T>
class timeline;

namespace detail
{

// Concept which enables constructor for lambda timeline.
template <typename F, typename T>
concept is_generator = requires(F f, double t) {
  { f(t) } -> std::same_as<T>;
};

// Forward declaring timeline data which is stored on the heap always.
template <typename T>
struct timeline_data;

} // namespace detail

// Timeline represents a value that changes over time. It supports keyframes, relative animations,
// and blending with easing. Timelines can also be procedural, meaning they are defined a by a
// lambda. At any time t, the timeline can be sampled to produce the current value. Timelines can be
// composed or sequenced to form complex animations.
template <typename T>
class timeline
{
public:
  using value_type = T;
  using lambda = std::function<T(double)>;

public:
  // We need a default ctor because things get messy without one.
  timeline() = default;

  // The most basic type of timeline is constant timeline. However, all non-lambda timelines are
  // defined relative to anchor which captures state at t = 0.
  template <typename U>
    requires std::convertible_to<U, T> && (!detail::is_generator<U, T>)
  timeline(U&& anchor);

  // Constructs a procedural timeline. These cannot have keyframes.
  template <typename F>
    requires detail::is_generator<F, T>
  timeline(F&& generator);

  // Evaluate the timeline at any given time.
  T sample(double time) const;

  // Adds a keyframe to the timeline. Recomputes the default length. We can safely return a
  // timeline<T>&.
  timeline<T>& add_keyframe(double at, T value, easing_func easing);

  // Adds an animation to the timeline. Recomputes the default length. TODO negative lengths may be
  // useful for sequencing, or define separate sequencing API. TODO consider flattening (absorb
  // keyframes and animations).
  timeline<T>& add_timeline(double at, const timeline<T>& value)
    requires addable<T>;

  // Layer timelines via multiplication. Applies the passed timeline to this on LHS. Recomputes
  // the default length.
  timeline<T>& multiply_timeline(double at, const timeline<T>& value)
    requires multipliable<T>;

  // Manage the length of the animation. Set zero or negative value to use the default.
  void set_length(double length = 0.0);
  double default_length() const;
  double length() const;

private:
  // We want timelines to be light-ish objects. We are therefore going to store them on the heap
  // secretly, and wrap their data with shared_ptr's.
  std::shared_ptr<detail::timeline_data<T>> data;
};

namespace detail
{

template <typename T>
struct timeline_data
{
  // When we are adding timelines as animations, we want to make sure that we don't add ourself as a
  // relative animation. The easiest way to do this is to use a unique ID for each timeline, then
  // define the dependencies of timelines in a set. Then, we easily verify that we are not dependent
  // on ourself.
  std::uint64_t id;
  std::unordered_set<std::uint64_t> dependencies;

  // The anchor is the initial value of a timeline. Returned for time at or before zero.
  T anchor;

  // Store data about procedural timelines.
  bool is_procedural = false;
  timeline<T>::lambda generator;

  // Timelines need to have a length to be sequenced. The default length is one, or the the time
  // that the last animation ends. This is updated whenever a keyframe or animation is added, unless
  // it has been set by the user (tracked in user_length).
  double cached_length = 1.0; // Default timeline is one second. This is the length of paths.
  bool user_length = false;

  // These are the meat and cheese of our timeline. This is the system for defining timelines. Note
  // that keyframes are stored in chronological order.
  std::vector<keyframe<T>> keyframes;

  // An animation can be layered on top of another timeline at a given moment. We opt to
  // use a timeline object since it makes API much cleaner at cost of startup performance.
  // Animations are stored in the order that they they are added, since applying animations is
  // rarely commutative.
  struct animation
  {
    double start;
    timeline<T> effect;
    bool multiply;
  };
  std::vector<animation> animations;
};

} // namespace detail

// CTAD allow us to declare a timeline and implicitly infer the data type.
template <typename T>
  requires(!std::invocable<T, double>)
timeline(T) -> timeline<T>;

// Necessary to deduce type of timeline from a lambda.
template <typename T>
  requires(std::invocable<T, double>)
timeline(T) -> timeline<std::invoke_result_t<T, double>>;

// We'll specialize a few different cases here since double is our default.
timeline(int) -> timeline<double>;
timeline(float) -> timeline<double>;

// Since templated types in C++ each store their own static types, we store the id counter for
// timelines in a separate helper class.
namespace detail
{

struct timeline_id
{
  inline static std::uint64_t current = 0;
};

} // namespace detail

template <typename T>
template <typename U>
  requires std::convertible_to<U, T> && (!detail::is_generator<U, T>)
timeline<T>::timeline(U&& anchor)
  : data(std::make_shared<detail::timeline_data<T>>())
{
  data->anchor = std::forward<U>(anchor);
  data->id = detail::timeline_id::current++;
}

template <typename T>
template <typename F>
  requires detail::is_generator<F, T>
timeline<T>::timeline(F&& generator)
  : data(std::make_shared<detail::timeline_data<T>>())
{
  data->is_procedural = true;
  data->generator = generator;
  data->id = detail::timeline_id::current++;
}

template <typename T>
T timeline<T>::sample(double time) const
{
  // Track our state as we apply.
  T state;

  // If we are procedural, use the lambda. Otherwise, use keyframes.
  if (data->is_procedural)
    state = data->generator(time);
  else
  {
    // Negative times always return the anchor for non-procedural.
    if (time <= 0.0)
      return data->anchor;

    // Set our state as the anchor initially.
    state = data->anchor;

    // Next, handle the keyframes. For now, we can just loop to find the correct one.
    double lastTime = 0.0; // The time of the last keyframe (initial anchor).
    for (auto& keyframe : data->keyframes)
    {
      // We've found the keyframe to blend with!
      if (keyframe.at >= time)
      {
        // If we are lerpable, we use the easing function. Otherwise, we stick with last state.
        if constexpr (lerpable<T>)
        {
          double progress = (time - lastTime) / (keyframe.at - lastTime);
          state = timeline_traits<T>::lerp(state, keyframe.value, keyframe.easing(progress));
        }
      }
      else
      {
        // If we haven't found next yet, we could be blending with this one.
        lastTime = keyframe.at;
        state = keyframe.value;
      }
    }
  }

  // Now, we need to apply our animations. These are layered on top of the underlying keyframes in
  // order that they are added to the timeline.
  for (auto& anim : data->animations)
  {
    if (time < anim.start)
      continue;

    // Apply the animation either additively or multiplicatively.
    double relTime = time - anim.start;
    T delta = anim.effect.sample(relTime);

    // If we are addable, we can execute this code safely.
    if constexpr (addable<T>)
    {
      if (!anim.multiply)
        state = timeline_traits<T>::add(delta, state);
    }

    // If we are multipliable, we can execute the following.
    if constexpr (multipliable<T>)
    {
      if (anim.multiply)
        state = timeline_traits<T>::multiply(delta, state);
    }
  }

  // Finally, we can return our computed value.
  return state;
}

template <typename T>
timeline<T>& timeline<T>::add_keyframe(double at, T value, easing_func easing)
{
  // We can't add keyframes to lambda timelines.
  if (data->is_procedural)
    throw std::runtime_error("Unable to add keyframes to lambda timeline!");

  keyframe state{at, value, easing};

  // If we don't have any keyframes, add and return.
  if (data->keyframes.empty())
  {
    data->keyframes.push_back(state);
    return (*this);
  }

  // Otherwise, find where to insert.
  bool inserted = false;
  for (auto it = data->keyframes.begin(); it != data->keyframes.end(); ++it)
  {
    // If we are before the next, we'll insert.
    if (at < it->at)
    {
      data->keyframes.insert(it, state);
      inserted = true;
      break;
    }
  }

  // Check if we are the last to be inserted.
  if (!inserted)
    data->keyframes.push_back(state);

  // If the user has the length set to the default, we can recompute.
  if (!data->user_length)
    set_length();

  // Return this to allow factory API.
  return (*this);
}

template <typename T>
timeline<T>& timeline<T>::add_timeline(double start, const timeline<T>& value)
  requires addable<T>
{
  // Make sure that we can safely add this animation by checking if we are in its dependencies.
  if (value.data->dependencies.contains(data->id))
    throw std::runtime_error("Unable to add animation that depends on self!");

  // Add the dependency (and its dependencies)
  data->dependencies.insert(value.data->id);
  data->dependencies.insert(value.data->dependencies.begin(), value.data->dependencies.end());

  // Add our animation to the list of those to apply.
  data->animations.emplace_back(start, value, false);

  // If the user has the length set to the default, we can recompute.
  if (!data->user_length)
    set_length();

  // Return a reference to ourself to support factory style API.
  return (*this);
}

template <typename T>
timeline<T>& timeline<T>::multiply_timeline(double start, const timeline<T>& value)
  requires multipliable<T>
{
  // Make sure that we can safely add this animation by checking if we are in its dependencies
  if (value.data->dependencies.contains(data->id))
    throw std::runtime_error("Unable to add animation that depends on self!");

  // Add the dependency (and its dependencies)
  data->dependencies.insert(value.data->id);
  data->dependencies.insert(value.data->dependencies.begin(), value.data->dependencies.end());

  // Add our animation to the list of those to apply.
  data->animations.emplace_back(start, value, true);

  // If the user has the length set to the default, we can recompute.
  if (!data->user_length)
    set_length();

  return (*this);
}

template <typename T>
void timeline<T>::set_length(double length)
{
  // If we have a non-positive length, use the default length.
  if (length <= 0.0)
  {
    data->user_length = false;
    data->cached_length = default_length();
  }
  else
  {
    data->user_length = true;
    data->cached_length = length;
  }
}

template <typename T>
double timeline<T>::default_length() const
{
  // If we have no keyframes or animations, default length is one second.
  if (data->keyframes.empty() && data->animations.empty())
    return 1.0;

  // Otherwise, start with the last keyframe as the length.
  double len = 0.0;
  if (!data->keyframes.empty())
    len = data->keyframes.back().at;

  // Now, handle the animations.
  for (auto& anim : data->animations)
  {
    double end = anim.start + anim.effect.length();
    if (end > len)
      len = end;
  }

  // Finally, return the length.
  return len;
}

template <typename T>
double timeline<T>::length() const
{
  return data->cached_length;
}

} // namespace nelo
