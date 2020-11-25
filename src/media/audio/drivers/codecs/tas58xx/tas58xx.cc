// Copyright 2019 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "tas58xx.h"

#include <lib/device-protocol/i2c.h>
#include <lib/simple-codec/simple-codec-helper.h>

#include <algorithm>
#include <memory>

#include <ddk/metadata.h>
#include <ddk/platform-defs.h>
#include <ddk/protocol/i2c.h>
#include <ddktl/protocol/composite.h>
#include <fbl/algorithm.h>
#include <fbl/alloc_checker.h>

#include "src/media/audio/drivers/codecs/tas58xx/ti_tas58xx-bind.h"

namespace {
// clang-format off
constexpr uint8_t kRegSelectPage  = 0x00;
constexpr uint8_t kRegReset       = 0x01;
constexpr uint8_t kRegDeviceCtrl1 = 0x02;
constexpr uint8_t kRegDeviceCtrl2 = 0x03;
constexpr uint8_t kRegSapCtrl1    = 0x33;
constexpr uint8_t kRegSapCtrl2    = 0x34;
constexpr uint8_t kRegDigitalVol  = 0x4c;
constexpr uint8_t kRegClearFault  = 0x78;
constexpr uint8_t kRegSelectbook  = 0x7f;

constexpr uint8_t kRegResetRegsAndModulesCtrl  = 0x11;
constexpr uint8_t kRegDeviceCtrl1BitsPbtlMode  = 0x04;
constexpr uint8_t kRegDeviceCtrl1Bits1SpwMode  = 0x01;
constexpr uint8_t kRegSapCtrl1Bits16bits       = 0x00;
constexpr uint8_t kRegSapCtrl1Bits32bits       = 0x03;
constexpr uint8_t kRegSapCtrl1BitsTdmSmallFs   = 0x14;
constexpr uint8_t kRegDeviceCtrl2BitsHiZ       = 0x02;
constexpr uint8_t kRegDeviceCtrl2BitsPlay      = 0x03;
constexpr uint8_t kRegDieId                    = 0x67;
constexpr uint8_t kRegClearFaultBitsAnalog     = 0x80;
// clang-format on

}  // namespace

namespace audio {

// TODO(andresoportus): Add handling for the other formats supported by this codec.
static const std::vector<uint32_t> kSupportedDaiNumberOfChannels = {2, 4};
static const std::vector<SampleFormat> kSupportedDaiSampleFormats = {SampleFormat::PCM_SIGNED};
static const std::vector<FrameFormat> kSupportedDaiFrameFormats = {FrameFormat::I2S,
                                                                   FrameFormat::TDM1};
static const std::vector<uint32_t> kSupportedDaiRates = {48'000};
static const std::vector<uint8_t> kSupportedDaiBitsPerSlot = {16, 32};
static const std::vector<uint8_t> kSupportedDaiBitsPerSample = {16, 32};
static const audio::DaiSupportedFormats kSupportedDaiDaiFormats = {
    .number_of_channels = kSupportedDaiNumberOfChannels,
    .sample_formats = kSupportedDaiSampleFormats,
    .frame_formats = kSupportedDaiFrameFormats,
    .frame_rates = kSupportedDaiRates,
    .bits_per_slot = kSupportedDaiBitsPerSlot,
    .bits_per_sample = kSupportedDaiBitsPerSample,
};

Tas58xx::Tas58xx(zx_device_t* device, const ddk::I2cChannel& i2c)
    : SimpleCodecServer(device), i2c_(i2c) {
  size_t actual = 0;
  auto status = device_get_metadata(parent(), DEVICE_METADATA_PRIVATE, &metadata_,
                                    sizeof(metadata_), &actual);
  if (status != ZX_OK) {
    zxlogf(DEBUG, "%s device_get_metadata failed %d", __FILE__, status);
  }
}

zx_status_t Tas58xx::Stop() {
  fbl::AutoLock lock(&lock_);
  return UpdateReg(kRegDeviceCtrl2, 0x3, kRegDeviceCtrl2BitsHiZ);
}

zx_status_t Tas58xx::Start() {
  fbl::AutoLock lock(&lock_);
  return UpdateReg(kRegDeviceCtrl2, 0x3, kRegDeviceCtrl2BitsPlay);
}

zx_status_t Tas58xx::Reset() {
  {  // Limit scope of lock, SetGainState will grab it again below.
    fbl::AutoLock lock(&lock_);
    // From the reference manual:
    // "9.5.3.1 Startup Procedures
    // 1. Configure ADR/FAULT pin with proper settings for I2C device address.
    // 2. Bring up power supplies (it does not matter if PVDD or DVDD comes up first).
    // 3. Once power supplies are stable, bring up PDN to High and wait 5ms at least, then start
    // SCLK, LRCLK.
    // 4. Once I2S clocks are stable, set the device into HiZ state and enable DSP via the I2C
    // control port.
    // 5. Wait 5ms at least. Then initialize the DSP Coefficient, then set the device to Play state.
    // 6. The device is now in normal operation."
    // Steps 4+ are execute below.

    // Run the first init sequence from metadata if available otherwise kDefaultsStart.
    if (metadata_.number_of_writes1) {
      for (size_t i = 0; i < metadata_.number_of_writes1; ++i) {
        auto status =
            WriteReg(metadata_.init_sequence1[i].address, metadata_.init_sequence1[i].value);
        if (status != ZX_OK) {
          zxlogf(ERROR, "%s Failed to write I2C register 0x%02X for %s", __FILE__,
                 metadata_.init_sequence1[i].address, __func__);
          return status;
        }
      }
    } else {
      constexpr uint8_t kDefaultsStart[][2] = {
          {kRegSelectPage, 0x00},
          {kRegSelectbook, 0x00},
          {kRegDeviceCtrl2, kRegDeviceCtrl2BitsHiZ},  // Enables DSP.
          {kRegReset, kRegResetRegsAndModulesCtrl},
      };
      for (auto& i : kDefaultsStart) {
        auto status = WriteReg(i[0], i[1]);
        if (status != ZX_OK) {
          zxlogf(ERROR, "%s Failed to write I2C register 0x%02X for %s", __FILE__, i[0], __func__);
          return status;
        }
      }
    }

    zx_nanosleep(zx_deadline_after(ZX_MSEC(5)));

    // Run the second init sequence from metadata if available, and then kDefaultsEnd.
    for (size_t i = 0; i < metadata_.number_of_writes2; ++i) {
      auto status =
          WriteReg(metadata_.init_sequence2[i].address, metadata_.init_sequence2[i].value);
      if (status != ZX_OK) {
        zxlogf(ERROR, "%s Failed to write I2C register 0x%02X for %s", __FILE__,
               metadata_.init_sequence2[i].address, __func__);
        return status;
      }
    }

    const uint8_t kDefaultsEnd[][2] = {
        {kRegSelectPage, 0x00},
        {kRegSelectbook, 0x00},
        {kRegDeviceCtrl1,
         static_cast<uint8_t>((metadata_.bridged ? kRegDeviceCtrl1BitsPbtlMode : 0) |
                              kRegDeviceCtrl1Bits1SpwMode)},

        {kRegDeviceCtrl2, kRegDeviceCtrl2BitsPlay},
        {kRegSelectPage, 0x00},
        {kRegSelectbook, 0x00},
        {kRegClearFault, kRegClearFaultBitsAnalog}};
    for (auto& i : kDefaultsEnd) {
      auto status = WriteReg(i[0], i[1]);
      if (status != ZX_OK) {
        zxlogf(ERROR, "%s Failed to write I2C register 0x%02X for %s", __FILE__, i[0], __func__);
        return status;
      }
    }
  }
  constexpr float kDefaultGainDb = -30.f;
  SetGainState({.gain = kDefaultGainDb, .muted = true});
  initialized_ = true;
  return ZX_OK;
}

zx::status<DriverIds> Tas58xx::Initialize() {
  return zx::ok(DriverIds{
      .vendor_id = PDEV_VID_TI,
      .device_id = PDEV_DID_TI_TAS58xx,
      .instance_count = metadata_.instance_count,
  });
}

zx_status_t Tas58xx::Create(zx_device_t* parent) {
  ddk::CompositeProtocolClient composite(parent);
  if (!composite.is_valid()) {
    zxlogf(ERROR, "%s Could not get composite protocol", __FILE__);
    return ZX_ERR_NOT_SUPPORTED;
  }

  ddk::I2cChannel i2c(composite, "i2c");
  if (!i2c.is_valid()) {
    zxlogf(ERROR, "%s Could not get i2c protocol", __FILE__);
    return ZX_ERR_NO_RESOURCES;
  }

  auto dev = SimpleCodecServer::Create<Tas58xx>(parent, i2c);

  // devmgr is now in charge of the memory for dev.
  dev.release();
  return ZX_OK;
}

Info Tas58xx::GetInfo() {
  fbl::AutoLock lock(&lock_);
  uint8_t die_id = 0;
  zx_status_t status = ReadReg(kRegDieId, &die_id);
  if (status != ZX_OK) {
    zxlogf(ERROR, "%s Failed to read DIE ID %d", __FILE__, status);
  }
  const char* name = nullptr;
  if (die_id == 0x95) {
    printf("tas58xx: Found TAS5825m\n");
    name = "TAS5825m";
  } else if (die_id == 0x00) {
    printf("tas58xx: Found TAS5805m\n");
    name = "TAS5805m";
  }
  return {.unique_id = "", .manufacturer = "Texas Instruments", .product_name = name};
}

zx_status_t Tas58xx::Shutdown() { return ZX_OK; }

bool Tas58xx::IsBridgeable() { return false; }

void Tas58xx::SetBridgedMode(bool enable_bridged_mode) {
  // TODO(andresoportus): Add support and report true in CodecIsBridgeable.
}

std::vector<DaiSupportedFormats> Tas58xx::GetDaiFormats() {
  std::vector<DaiSupportedFormats> formats;
  formats.push_back(kSupportedDaiDaiFormats);
  return formats;
}

zx_status_t Tas58xx::SetDaiFormat(const DaiFormat& format) {
  if (!IsDaiFormatSupported(format, kSupportedDaiDaiFormats)) {
    zxlogf(ERROR, "%s unsupported format\n", __FILE__);
    return ZX_ERR_NOT_SUPPORTED;
  }
  if (format.number_of_channels == 2 && format.channels_to_use_bitmask != 3) {
    zxlogf(ERROR, "%s DAI format channels to use not supported %u 0x%lX", __FILE__,
           format.number_of_channels, format.channels_to_use_bitmask);
    return ZX_ERR_NOT_SUPPORTED;
  }
  if (format.number_of_channels == 4 && format.channels_to_use_bitmask != 3 &&
      format.channels_to_use_bitmask != 0xc) {
    zxlogf(ERROR, "%s DAI format channels to use not supported %u 0x%lX", __FILE__,
           format.number_of_channels, format.channels_to_use_bitmask);
    return ZX_ERR_NOT_SUPPORTED;
  }

  uint8_t reg_value =
      (format.bits_per_sample == 32 ? kRegSapCtrl1Bits32bits : kRegSapCtrl1Bits16bits) |
      (format.frame_format == FrameFormat::I2S ? 0x00 : kRegSapCtrl1BitsTdmSmallFs);

  fbl::AutoLock lock(&lock_);
  auto status = WriteReg(kRegSapCtrl1, reg_value);
  if (status != ZX_OK) {
    return status;
  }
  return WriteReg(kRegSapCtrl2,
                  (format.number_of_channels == 4 && format.channels_to_use_bitmask == 0xc)
                      ? 2 * format.bits_per_slot
                      : 0x00);
}

GainFormat Tas58xx::GetGainFormat() {
  return {
      .type = GainType::DECIBELS,
      .min_gain = kMinGain,
      .max_gain = kMaxGain,
      .gain_step = kGainStep,
      .can_mute = true,
      .can_agc = false,
  };
}

void Tas58xx::SetGainState(GainState gain_state) {
  fbl::AutoLock lock(&lock_);
  float gain = std::clamp(gain_state.gain, kMinGain, kMaxGain);
  uint8_t gain_reg = static_cast<uint8_t>(48 - gain * 2);
  zx_status_t status = WriteReg(kRegDigitalVol, gain_reg);
  if (status != ZX_OK) {
    return;
  }
  if (gain_state.agc_enable) {
    zxlogf(ERROR, "%s AGC enable not supported\n", __FILE__);
    gain_state.agc_enable = false;
  }
  gain_state_ = gain_state;
  static_cast<void>(UpdateReg(kRegDeviceCtrl2, 0x08, gain_state.muted ? 0x08 : 0x00));
}

GainState Tas58xx::GetGainState() { return gain_state_; }
PlugState Tas58xx::GetPlugState() { return {.hardwired = true, .plugged = true}; }

zx_status_t Tas58xx::WriteReg(uint8_t reg, uint8_t value) {
  uint8_t write_buf[2];
  write_buf[0] = reg;
  write_buf[1] = value;
//#define TRACE_I2C
#ifdef TRACE_I2C
  printf("Writing register 0x%02X to value 0x%02X\n", reg, value);
  auto status = i2c_.WriteSync(write_buf, 2);
  if (status != ZX_OK) {
    printf("Could not I2C write %d\n", status);
    return status;
  }
  return ZX_OK;
#else
  return i2c_.WriteSync(write_buf, 2);
#endif
}

zx_status_t Tas58xx::ReadReg(uint8_t reg, uint8_t* value) {
  auto status = i2c_.WriteReadSync(&reg, 1, value, 1);
  if (status != ZX_OK) {
    return status;
  }
#ifdef TRACE_I2C
  printf("Read register 0x%02X, value %02X\n", reg, *value);
#endif
  return status;
}

zx_status_t Tas58xx::UpdateReg(uint8_t reg, uint8_t mask, uint8_t value) {
  uint8_t old_value = 0;
  auto status = ReadReg(reg, &old_value);
  if (status != ZX_OK) {
    return status;
  }
  return WriteReg(reg, (old_value & ~mask) | (value & mask));
}

zx_status_t tas58xx_bind(void* ctx, zx_device_t* parent) { return Tas58xx::Create(parent); }

static constexpr zx_driver_ops_t driver_ops = []() {
  zx_driver_ops_t ops = {};
  ops.version = DRIVER_OPS_VERSION;
  ops.bind = tas58xx_bind;
  return ops;
}();

}  // namespace audio

ZIRCON_DRIVER(ti_tas58xx, audio::driver_ops, "zircon", "0.1")
