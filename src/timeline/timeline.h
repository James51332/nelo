#pragma once

#include <cstdint>
#include <memory>
#include <unordered_set>
#include <vector>

#include "traits.h"

namespace nelo
{

// Timeline represents a value that changes over time. It supports keyframes, relative animations,
// and blending with easing. At any time t, the timeline can be sampled to produce the current
// value. Timelines can be composed or sequenced to form complex animations.
template <typename T>
class timeline
{
public:
  using value_type = T;
  using easing_func = std::function<double(double)>;

  // Keyframes are like anchor points set for a given time.
  struct keyframe
  {
    double at;
    T value;
    easing_func easing;
  };

  struct animation
  {
    double start;
    std::shared_ptr<timeline<T>> timeline;
    bool multiply;
  };

public:
  // Every timeline is defined relative to anchor.
  timeline(T anchor);

  // Evaluate the timeline at any given time.
  T evaluate(double time) const;

  // Adds a keyframe to the timeline. Recomputes the default length.
  void add_keyframe(double at, T value, easing_func easing);

  // Adds an animation to the timeline. Recomputes the default length.
  void add_timeline(double at, std::shared_ptr<timeline<T>> value)
    requires addable<T>;

  // Layer timelines via multiplication. Applies the passed timeline to this on LHS. Recomputes
  // the default length.
  void multiply_timeline(double at, std::shared_ptr<timeline<T>> value)
    requires multipliable<T>;

  // Manage the length of the animation. Set zero or negative value to use the default.
  void set_length(double length = 0.0);
  double default_length() const;
  double length() const;

private:
  // When we are adding timelines as animations, we want to make sure that we don't add ourself as a
  // relative animation. The easiest way to do this is to use a unique ID for each timeline, then
  // define the dependencies of timelines in a set. Then, we easily verify that we are not dependent
  // on ourself.
  std::uint64_t id;
  std::unordered_set<std::uint64_t> dependencies;

  // The anchor is the initial value of a timeline. Returned for time at or before zero.
  T anchor;

  // Timelines need to have a length to be sequenced. The default length is one, or the the time
  // that the last animation ends. This is updated whenever a keyframe or animation is added, unless
  // it has been set by the user (tracked in user_length).
  double cached_length = 3.0; // Default timeline is three seconds.
  bool user_length = false;

  // These are the meat and cheese of our timeline. This is the system for defining timelines. Note
  // that keyframes are stored in chronological order, and animations are stored in application
  // order.
  std::vector<keyframe> keyframes;
  std::vector<animation> animations;
};

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
timeline<T>::timeline(T anchor)
  : anchor(anchor), id(detail::timeline_id::current++)
{
}

template <typename T>
T timeline<T>::evaluate(double time) const
{
  // First, return the anchor if we are at zero or negative time.
  if (time <= 0.0)
    return anchor;

  // Track our state in the following variable.
  T state = anchor;

  // Next, handle the keyframes. For now, we can just loop to find the correct one.
  double lastTime = 0.0; // The time of the last keyframe (initial anchor).
  for (auto& keyframe : keyframes)
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

  // Now, we need to apply our animations. These are layered on top of the underlying keyframes in
  // order that they are added to the timeline.
  for (auto& anim : animations)
  {
    if (time < anim.start)
      continue;

    // Apply the animation either additively or multiplicatively.
    double relTime = time - anim.start;
    T delta = anim.timeline.evaluate(relTime);

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
void timeline<T>::add_keyframe(double at, T value, easing_func easing)
{
  keyframe state{at, value, easing};

  // If we don't have any keyframes, add and return.
  if (keyframes.empty())
  {
    keyframes.push_back(state);
    return;
  }

  // Otherwise, find where to insert.
  bool inserted = false;
  for (auto it = keyframes.begin(); it != keyframes.end(); ++it)
  {
    // If we are before the next, we'll insert.
    if (at < it->at)
    {
      keyframes.insert(it, state);
      inserted = true;
      break;
    }
  }

  // Check if we are the last to be inserted.
  if (!inserted)
    keyframes.push_back(state);

  // If the user has the length set to the default, we can recompute.
  if (!user_length)
    set_length();
}

template <typename T>
void timeline<T>::add_timeline(double start, std::shared_ptr<timeline<T>> value)
  requires addable<T>
{
  // Make sure that we can safely add this animation by checking if we are in its dependencies.
  if (value.dependencies.contains(id))
    throw new std::runtime_error("Unable to add animation that depends on self!");
  dependencies.insert(value.dependencies.begin(), value.dependencies.end());

  // Add our animation to the list of those to apply.
  animations.emplace_back(start, value, false);

  // If the user has the length set to the default, we can recompute.
  if (!user_length)
    set_length();
}

template <typename T>
void timeline<T>::multiply_timeline(double start, std::shared_ptr<timeline<T>> value)
  requires multipliable<T>
{
  // Make sure that we can safely add this animation by checking if we are in its dependencies
  if (value.dependencies.contains(id))
    throw new std::runtime_error("Unable to add animation that depends on self!");

  // Absorb the dependencies into our set.
  for (auto dep : value.dependencies)
    dependencies.insert(dep);

  // Add our animation to the list of those to apply.
  animations.emplace_back(start, value, true);

  // If the user has the length set to the default, we can recompute.
  if (!user_length)
    set_length();
}

template <typename T>
void timeline<T>::set_length(double length)
{
  // If we have a non-positive length, use the default length.
  if (length <= 0.0)
  {
    user_length = false;
    cached_length = default_length();
  }
  else
  {
    user_length = true;
    cached_length = length;
  }
}

template <typename T>
double timeline<T>::default_length() const
{
  // If we have no keyframes or animations, default length is three seconds.
  if (keyframes.empty() && animations.empty())
    return 3.0;

  // Otherwise, start with the last keyframe as the length.
  double len = 0.0;
  if (!keyframes.empty())
    len = keyframes.back().at;

  // Now, handle the animations.
  for (auto& anim : animations)
  {
    double end = anim.start + anim.timeline.length();
    if (end > len)
      len = end;
  }

  // Finally, return the length.
  return len;
}

template <typename T>
double timeline<T>::length() const
{
  return cached_length;
}

} // namespace nelo
