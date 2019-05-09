// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include <fuchsia/media/cpp/fidl.h>
#include <fuchsia/virtualaudio/cpp/fidl.h>
#include <lib/fzl/vmo-mapper.h>
#include <lib/gtest/real_loop_fixture.h>

#include "lib/component/cpp/environment_services_helper.h"
#include "src/lib/fxl/logging.h"
#include "src/media/audio/audio_core/test/audio_tests_shared.h"

namespace media::audio::test {

// The AudioLoopbackEnvironment class allows us to make configuration changes
// before any test case begins, and after all test cases complete.
class AudioLoopbackEnvironment : public ::testing::Environment {
 public:
  // Before any test cases in this program, synchronously connect to the service
  // to ensure that the audio and audio_core components are present and loaded,
  // and that at least one (virtual) audio output device is present.
  //
  // On assert-false during this SetUp method, no test cases run, and they may
  // display as passed. However, the overall binary returns non-zero (fail).
  void SetUp() override {
    // This is an unchanging input for the entire component; get it once here.
    auto environment_services = component::GetEnvironmentServices();

    // We need at least one active audio output, for loopback capture to work.
    // So use this Control to enable virtualaudio and add a virtual audio output
    // that will exist for the entirety of this binary's test cases. We will
    // remove it and disable virtual_audio immediately afterward.
    environment_services->ConnectToService(
        virtual_audio_control_sync_.NewRequest());
    zx_status_t status = virtual_audio_control_sync_->Enable();
    ASSERT_EQ(status, ZX_OK) << "Failed to enable virtualaudio";

    // Create an output device using default settings, save it while tests run.
    environment_services->ConnectToService(
        virtual_audio_output_sync_.NewRequest());
    status = virtual_audio_output_sync_->Add();
    ASSERT_EQ(status, ZX_OK) << "Failed to add virtual audio output";

    // Ensure that the output is active before we proceed to running tests.
    uint32_t num_inputs, num_outputs = 0, num_tries = 0;
    do {
      status =
          virtual_audio_control_sync_->GetNumDevices(&num_inputs, &num_outputs);
      ASSERT_EQ(status, ZX_OK) << "GetNumDevices failed";

      ++num_tries;
    } while (num_outputs == 0 && num_tries < 100);
    ASSERT_GT(num_outputs, 0u)
        << "Timed out trying to add virtual audio output";

    // Synchronously calling a FIDL method with a callback guarantees that the
    // service is loaded and running before the sync method itself returns.
    //
    // This is not the case for sync calls _without_ callback, nor async calls,
    // because of pipelining inherent in FIDL's design.
    environment_services->ConnectToService(audio_dev_enum_sync_.NewRequest());
    // Ensure that the audio_core binary is resident.
    uint64_t default_output;
    bool connected_to_svc = (audio_dev_enum_sync_->GetDefaultOutputDevice(
                                 &default_output) == ZX_OK);

    ASSERT_TRUE(connected_to_svc) << "Failed in GetDefaultOutputDevice";
  }

  // Ensure that our virtual device is gone when our test bin exits.
  void TearDown() override {
    // Remove our virtual audio output device
    zx_status_t status = virtual_audio_output_sync_->Remove();
    ASSERT_EQ(status, ZX_OK) << "Failed to add virtual audio output";

    // And ensure that virtualaudio is disabled, by the time we leave.
    status = virtual_audio_control_sync_->Disable();
    ASSERT_EQ(status, ZX_OK) << "Failed to disable virtualaudio";

    // Wait for GetNumDevices(output) to equal zero before proceeding.
    uint32_t num_inputs = 1, num_outputs = 1, num_tries = 0;
    do {
      status =
          virtual_audio_control_sync_->GetNumDevices(&num_inputs, &num_outputs);
      ASSERT_EQ(status, ZX_OK) << "GetNumDevices failed";

      ++num_tries;
    } while ((num_outputs != 0 || num_inputs != 0) && num_tries < 100);
    ASSERT_EQ(num_outputs, 0u) << "Timed out while disabling virtualaudio";
    ASSERT_EQ(num_inputs, 0u) << "Timed out while disabling virtualaudio";

    virtual_audio_output_sync_.Unbind();
    virtual_audio_control_sync_.Unbind();
  }

  // If needed, this (overriding) function would also need to be public.
  // ~AudioLoopbackEnvironment() override {}

  fuchsia::virtualaudio::ControlSyncPtr virtual_audio_control_sync_;
  fuchsia::virtualaudio::OutputSyncPtr virtual_audio_output_sync_;
  fuchsia::media::AudioDeviceEnumeratorSyncPtr audio_dev_enum_sync_;
};

//
// AudioLoopbackTest
//
// Base Class for testing simple playback and capture with loopback.
class AudioLoopbackTest : public gtest::RealLoopFixture {
 public:
  static constexpr int32_t kSampleRate = 8000;
  static constexpr int kChannelCount = 1;
  static constexpr int kSampleSeconds = 1;
  static constexpr int16_t kPlaybackData1 = 0x1000;
  static constexpr int16_t kPlaybackData2 = 0xfff;
  static constexpr int16_t kCaptureData1 = 0x7fff;

 protected:
  void SetUp() override;
  void TearDown() override;

  std::shared_ptr<component::Services> environment_services_;
  fuchsia::media::AudioPtr audio_;

  void SetUpRenderer(unsigned int index, int16_t data);
  void CleanUpRenderer(unsigned int index);

  fuchsia::media::AudioRendererPtr audio_renderer_[2];
  fzl::VmoMapper payload_buffer_[2];
  size_t playback_size_[2];
  size_t playback_sample_size_[2];

  void SetUpCapturer(unsigned int index, int16_t data);
  void CleanUpCapturer(unsigned int index);

  fuchsia::media::AudioCapturerPtr audio_capturer_[1];
  fzl::VmoMapper capture_buffer_[1];
  size_t capture_size_[1];
  size_t capture_sample_size_[1];

  bool error_occurred_ = false;

  void ErrorHandler(zx_status_t error) {
    error_occurred_ = true;
    FXL_LOG(ERROR) << "Unexpected error: " << error;
  }
};

// AudioLoopbackTest implementation
//

// SetUpRenderer
//
// For loopback tests, setup the first audio_renderer interface.
void AudioLoopbackTest::SetUpRenderer(unsigned int index, int16_t data) {
  ASSERT_LT(index, countof(audio_renderer_));

  int16_t* buffer;
  zx::vmo payload_vmo;

  audio_->CreateAudioRenderer(audio_renderer_[index].NewRequest());
  ASSERT_TRUE(audio_renderer_[index].is_bound());

  audio_renderer_[index].set_error_handler(
      [this](zx_status_t error) { ErrorHandler(error); });

  fuchsia::media::AudioStreamType format;
  format.sample_format = fuchsia::media::AudioSampleFormat::SIGNED_16;
  format.channels = kChannelCount;
  format.frames_per_second = kSampleRate;

  playback_sample_size_[index] = sizeof(int16_t);

  playback_size_[index] = format.frames_per_second * format.channels *
                          playback_sample_size_[index] * kSampleSeconds;

  zx_status_t status = payload_buffer_[index].CreateAndMap(
      playback_size_[index], ZX_VM_PERM_READ | ZX_VM_PERM_WRITE, nullptr,
      &payload_vmo, ZX_RIGHT_READ | ZX_RIGHT_MAP | ZX_RIGHT_TRANSFER);
  ASSERT_EQ(status, ZX_OK) << "Renderer VmoMapper:::CreateAndMap(" << index
                           << ") failed - " << status;

  buffer = reinterpret_cast<int16_t*>(payload_buffer_[index].start());
  for (int32_t i = 0; i < kSampleRate * kSampleSeconds; i++)
    buffer[i] = data;

  audio_renderer_[index]->SetPcmStreamType(format);
  audio_renderer_[index]->AddPayloadBuffer(0, std::move(payload_vmo));
}

// CleanUpRenderer
//
// Flush the output and free the vmo that was used by Renderer1.
void AudioLoopbackTest::CleanUpRenderer(unsigned int index) {
  ASSERT_LT(index, countof(audio_renderer_));
  bool flushed = false;

  // Flush the audio
  audio_renderer_[index]->DiscardAllPackets(
      [this, &flushed]() { flushed = true; });
  EXPECT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &flushed]() { return error_occurred_ || flushed; },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;

  EXPECT_TRUE(flushed);
  payload_buffer_[index].Unmap();
}

// SetUpCapturer
//
// For loopback tests, setup an audio_capturer interface
void AudioLoopbackTest::SetUpCapturer(unsigned int index, int16_t data) {
  ASSERT_LT(index, countof(audio_capturer_));

  int16_t* buffer;
  zx::vmo capture_vmo;

  audio_->CreateAudioCapturer(audio_capturer_[index].NewRequest(), true);
  ASSERT_TRUE(audio_capturer_[index].is_bound());

  audio_capturer_[index].set_error_handler(
      [this](zx_status_t error) { ErrorHandler(error); });

  fuchsia::media::AudioStreamType format;
  format.sample_format = fuchsia::media::AudioSampleFormat::SIGNED_16;
  format.channels = kChannelCount;
  format.frames_per_second = kSampleRate;

  capture_sample_size_[index] = sizeof(int16_t);

  capture_size_[index] = format.frames_per_second * format.channels *
                         capture_sample_size_[index] * kSampleSeconds;

  // ZX_VM_PERM_WRITE taken here as we pre-fill the buffer to catch any
  // cases where we get back a packet without anything having been done
  // with it.
  zx_status_t status = capture_buffer_[index].CreateAndMap(
      capture_size_[index], ZX_VM_PERM_READ | ZX_VM_PERM_WRITE, nullptr,
      &capture_vmo,
      ZX_RIGHT_READ | ZX_RIGHT_WRITE | ZX_RIGHT_MAP | ZX_RIGHT_TRANSFER);
  ASSERT_EQ(status, ZX_OK) << "Capturer VmoMapper:::CreateAndMap failed - "
                           << status;

  buffer = reinterpret_cast<int16_t*>(capture_buffer_[index].start());
  for (int32_t i = 0; i < kSampleRate * kSampleSeconds; i++)
    buffer[i] = data;

  audio_capturer_[index]->SetPcmStreamType(format);
  audio_capturer_[index]->AddPayloadBuffer(0, std::move(capture_vmo));
}

// CleanUpCapturer
//
void AudioLoopbackTest::CleanUpCapturer(unsigned int index) {
  ASSERT_LT(index, countof(audio_capturer_));
}

//
// AudioLoopbackTest implementation
//
void AudioLoopbackTest::SetUp() {
  ::gtest::RealLoopFixture::SetUp();

  environment_services_ = component::GetEnvironmentServices();
  environment_services_->ConnectToService(audio_.NewRequest());
  ASSERT_TRUE(audio_.is_bound());

  audio_.set_error_handler([this](zx_status_t error) { ErrorHandler(error); });
  audio_->SetSystemGain(0.0f);
  audio_->SetSystemMute(false);
}

void AudioLoopbackTest::TearDown() {
  EXPECT_FALSE(error_occurred_);
  EXPECT_TRUE(audio_.is_bound());

  ::gtest::RealLoopFixture::TearDown();
}

// SingleStream
//
// Creates a single output stream and a loopback capture and verifies it gets
// back what it puts in.
TEST_F(AudioLoopbackTest, SingleStream) {
  fuchsia::media::StreamPacket packet, captured;

  // SetUp playback stream
  SetUpRenderer(0, kPlaybackData1);
  SetUpCapturer(0, kCaptureData1);

  auto* capture = reinterpret_cast<int16_t*>(capture_buffer_[0].start());

  // Add a callback for when we get our captured packet.
  bool produced_packet = false;
  audio_capturer_[0].events().OnPacketProduced =
      [this, &captured, &produced_packet](fuchsia::media::StreamPacket packet) {
        // We only care about the first set of captured samples
        if (captured.payload_size == 0) {
          captured = packet;
          audio_capturer_[0]->StopAsyncCaptureNoReply();
          produced_packet = true;
        }
      };

  // Get the minimum duration after submitting a packet to when we can start
  // capturing what we sent on the loopback interface
  zx_duration_t sleep_duration = 0;
  audio_renderer_[0]->GetMinLeadTime([this, &sleep_duration](zx_duration_t t) {
    // Give a little wiggle room.
    sleep_duration = t + ZX_MSEC(5);
  });
  ASSERT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &sleep_duration]() {
        return error_occurred_ || (sleep_duration > 0);
      },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;
  ASSERT_FALSE(error_occurred_);

  packet.payload_offset = 0;
  packet.payload_size = playback_size_[0];

  audio_renderer_[0]->SendPacketNoReply(packet);

  int64_t ref_time_received = -1;
  int64_t media_time_received = -1;

  // Start playing right now, so that after we've delayed at least 1 leadtime,
  // we should have mixed audio available for capture.  Our playback is sized
  // to be much much larger than our capture to prevent test flakes.
  audio_renderer_[0]->Play(zx_clock_get_monotonic(), 0,
                           [this, &ref_time_received, &media_time_received](
                               int64_t ref_time, int64_t media_time) {
                             ref_time_received = ref_time;
                             media_time_received = media_time;
                           });
  ASSERT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &ref_time_received]() {
        return error_occurred_ || (ref_time_received > -1);
      },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;
  ASSERT_FALSE(error_occurred_);

  // We expect that media_time 0 played back at some point after the 'zero'
  // time on the system.
  EXPECT_EQ(media_time_received, 0);
  EXPECT_GE(ref_time_received, 0);

  // Give the playback some time to get mixed.
  zx_nanosleep(zx_clock_get_monotonic() + sleep_duration);

  // Capture 10 samples of audio.
  audio_capturer_[0]->StartAsyncCapture(10);
  EXPECT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &produced_packet]() { return error_occurred_ || produced_packet; },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;

  // Check that we got 10 samples as we expected.
  EXPECT_EQ(captured.payload_size / capture_sample_size_[0], 10U);

  // Check that all of the samples contain the expected data.
  for (size_t i = 0; i < (captured.payload_size / capture_sample_size_[0]);
       i++) {
    size_t index = (captured.payload_offset + i) % 8000;

    EXPECT_EQ(capture[index], kPlaybackData1);
  }

  CleanUpRenderer(0);
  CleanUpCapturer(0);
}

// DualStream
//
// Creates a pair of output streams and a loopback capture and verifies it gets
// back what it puts in.
TEST_F(AudioLoopbackTest, DualStream) {
  fuchsia::media::StreamPacket packet[2], captured;

  // SetUp playback streams
  SetUpRenderer(0, kPlaybackData1);
  SetUpRenderer(1, kPlaybackData2);

  // SetUp loopback capture
  SetUpCapturer(0, kCaptureData1);

  auto* capture = reinterpret_cast<int16_t*>(capture_buffer_[0].start());

  // Add a callback for when we get our captured packet.
  bool produced_packet = false;
  audio_capturer_[0].events().OnPacketProduced =
      [this, &captured, &produced_packet](fuchsia::media::StreamPacket packet) {
        // We only care about the first set of captured samples
        if (captured.payload_size == 0) {
          captured = packet;
          audio_capturer_[0]->StopAsyncCaptureNoReply();
          produced_packet = true;
        }
      };

  // Get the minimum duration after submitting a packet to when we can start
  // capturing what we sent on the loopback interface.  This assumes that the
  // latency will be the same for both playback streams.  This happens to be
  // true for this test as we create the renderers with the same parameters, but
  // is not a safe assumption for the general users of this API to make.
  zx_duration_t sleep_duration = 0;
  audio_renderer_[0]->GetMinLeadTime([this, &sleep_duration](zx_duration_t t) {
    // Give a little wiggle room.
    sleep_duration = t + ZX_MSEC(5);
  });
  ASSERT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &sleep_duration]() {
        return error_occurred_ || (sleep_duration > 0);
      },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;
  ASSERT_FALSE(error_occurred_);

  for (int i = 0; i < 2; i++) {
    packet[i].payload_offset = 0;
    packet[i].payload_size = playback_size_[i];
    audio_renderer_[i]->SendPacketNoReply(packet[i]);
  }

  int64_t ref_time_received = -1;
  int64_t media_time_received = -1;

  // Start playing right now, so that after we've delayed at least 1 leadtime,
  // we should have mixed audio available for capture.  Our playback is sized
  // to be much much larger than our capture to prevent test flakes.
  auto playat = zx_clock_get_monotonic();
  audio_renderer_[0]->PlayNoReply(playat, 0);
  // Only get the callback for the second renderer.
  audio_renderer_[1]->Play(playat, 0,
                           [this, &ref_time_received, &media_time_received](
                               int64_t ref_time, int64_t media_time) {
                             ref_time_received = ref_time;
                             media_time_received = media_time;
                           });
  ASSERT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &ref_time_received]() {
        return error_occurred_ || (ref_time_received > -1);
      },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;
  ASSERT_FALSE(error_occurred_);

  // We expect that media_time 0 played back at some point after the 'zero'
  // time on the system.
  EXPECT_EQ(media_time_received, 0);
  EXPECT_GT(ref_time_received, 0);

  // Give the playback some time to get mixed.
  zx_nanosleep(zx_deadline_after(sleep_duration));

  // Capture 10 samples of audio.
  audio_capturer_[0]->StartAsyncCapture(10);
  EXPECT_TRUE(RunLoopWithTimeoutOrUntil(
      [this, &produced_packet]() { return error_occurred_ || produced_packet; },
      kDurationResponseExpected, kDurationGranularity))
      << kTimeoutErr;

  // Check that we got 10 samples as we expected.
  EXPECT_EQ(captured.payload_size / capture_sample_size_[0], 10U);

  // Check that all of the samples contain the expected data.
  for (size_t i = 0; i < (captured.payload_size / capture_sample_size_[0]);
       i++) {
    size_t index = (captured.payload_offset + i) % 8000;
    EXPECT_EQ(capture[index], kPlaybackData1 + kPlaybackData2);
  }

  CleanUpRenderer(1);
  CleanUpRenderer(0);
  CleanUpCapturer(0);
}

}  // namespace media::audio::test

int main(int argc, char** argv) {
  ::testing::InitGoogleTest(&argc, argv);

  // gtest takes ownership of registered environments: *do not delete them*
  ::testing::AddGlobalTestEnvironment(
      new ::media::audio::test::AudioLoopbackEnvironment);

  int result = RUN_ALL_TESTS();

  return result;
}
