// Copyright 2018 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Data for testing parsing and serialization.
//!
//! This data was obtained by capturing live network traffic.

use crate::device::ethernet::Mac;

/// IPv4 headers.
pub(crate) const IPV4_HEADERS: &[&[u8]] = &[
    &[
        0x45, 0x00, 0x00, 0x34, 0x00, 0x00, 0x40, 0x00, 0x40, 0x06, 0xae, 0xea, 0xc0, 0xa8, 0x01,
        0x0f, 0xc0, 0xb8, 0x09, 0x6a,
    ],
    &[
        0x45, 0x20, 0x00, 0x74, 0x5b, 0x6e, 0x40, 0x00, 0x37, 0x06, 0x5c, 0x1c, 0xc0, 0xb8, 0x09,
        0x6a, 0xc0, 0xa8, 0x01, 0x0f,
    ],
    &[
        0x45, 0x20, 0x02, 0x8f, 0x00, 0x00, 0x40, 0x00, 0x3b, 0x11, 0xc9, 0x3f, 0xac, 0xd9, 0x05,
        0x6e, 0xc0, 0xa8, 0x01, 0x0f,
    ],
];

/// An ARP Broadcast request in an Ethernet frame.
pub(crate) const ARP_REQUEST: &[u8] = &[
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x14, 0xab, 0xc5, 0x74, 0x20, 0x34, 0x08, 0x06, 0x00, 0x01,
    0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x14, 0xab, 0xc5, 0x74, 0x20, 0x34, 0xac, 0x10, 0x02, 0x03,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xac, 0x10, 0x02, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// These are for dns_request_(v4|v6) and tls_client_hello_(v4|v6). They need to be pub so
// they can be re-exported by those modules.
#[allow(missing_docs)]
#[doc(hidden)]
pub(crate) const ETHERNET_SRC_MAC: Mac = Mac::new([0x6b, 0xc3, 0x31, 0x1c, 0x79, 0x02]);
#[allow(missing_docs)]
#[doc(hidden)]
pub(crate) const ETHERNET_DST_MAC: Mac = Mac::new([0x0f, 0x41, 0xf1, 0xda, 0xaf, 0x6c]);

/// A DNS request.
///
/// An Ethernet frame containing an IPv4 packet containing a UDP packet
/// containing a DNS request for 0.docs.google.com.
pub(crate) mod dns_request_v4 {
    #![allow(missing_docs)]

    use std::ops::Range;

    use crate::ip::Ipv4Addr;

    pub(crate) use super::ETHERNET_DST_MAC;
    pub(crate) use super::ETHERNET_SRC_MAC;
    pub(crate) const ETHERNET_BODY_RANGE: Range<usize> = 14..77;

    pub(crate) const IP_DSCP: u8 = 0;
    pub(crate) const IP_ECN: u8 = 0;
    pub(crate) const IP_DONT_FRAGMENT: bool = false;
    pub(crate) const IP_MORE_FRAGMENTS: bool = false;
    pub(crate) const IP_FRAGMENT_OFFSET: u16 = 0;
    pub(crate) const IP_ID: u16 = 0x17f1;
    pub(crate) const IP_TTL: u8 = 64;
    pub(crate) const IP_SRC_IP: Ipv4Addr = Ipv4Addr::new([192, 168, 1, 15]);
    pub(crate) const IP_DST_IP: Ipv4Addr = Ipv4Addr::new([192, 168, 1, 1]);
    pub(crate) const IP_BODY_RANGE: Range<usize> = 20..63;

    pub(crate) const UDP_SRC_PORT: u16 = 35658;
    pub(crate) const UDP_DST_PORT: u16 = 53;
    pub(crate) const UDP_BODY_RANGE: Range<usize> = 8..43;
    pub(crate) const UDP_BODY: &[u8] = &[
        0x47, 0xb5, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x30, 0x04,
        0x64, 0x6f, 0x63, 0x73, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d,
        0x00, 0x00, 0x01, 0x00, 0x01,
    ];

    pub(crate) const ETHERNET_FRAME_BYTES: &[u8] = &[
        0x0f, 0x41, 0xf1, 0xda, 0xaf, 0x6c, 0x6b, 0xc3, 0x31, 0x1c, 0x79, 0x02, 0x08, 0x00, 0x45,
        0x00, 0x00, 0x3f, 0x17, 0xf1, 0x00, 0x00, 0x40, 0x11, 0xdf, 0x5c, 0xc0, 0xa8, 0x01, 0x0f,
        0xc0, 0xa8, 0x01, 0x01, 0x8b, 0x4a, 0x00, 0x35, 0x00, 0x2b, 0x07, 0xf7, 0x47, 0xb5, 0x01,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x30, 0x04, 0x64, 0x6f, 0x63,
        0x73, 0x06, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
        0x00, 0x01,
    ];
}

/// A TLS Client Hello message.
///
/// An Ethernet frame containing an IPv4 packet containing a TCP segment
/// containing a TLSv1.2 Client Hello message.
pub(crate) mod tls_client_hello_v4 {
    #![allow(missing_docs)]

    use std::ops::Range;

    use crate::ip::Ipv4Addr;
    use crate::transport::tcp::TcpOption;

    pub(crate) use super::ETHERNET_DST_MAC;
    pub(crate) use super::ETHERNET_SRC_MAC;
    pub(crate) const ETHERNET_BODY_RANGE: Range<usize> = 14..591;

    pub(crate) const IP_DSCP: u8 = 0;
    pub(crate) const IP_ECN: u8 = 0;
    pub(crate) const IP_DONT_FRAGMENT: bool = true;
    pub(crate) const IP_MORE_FRAGMENTS: bool = false;
    pub(crate) const IP_FRAGMENT_OFFSET: u16 = 0;
    pub(crate) const IP_ID: u16 = 0;
    pub(crate) const IP_TTL: u8 = 64;
    pub(crate) const IP_SRC_IP: Ipv4Addr = Ipv4Addr::new([192, 168, 1, 15]);
    pub(crate) const IP_DST_IP: Ipv4Addr = Ipv4Addr::new([104, 237, 191, 1]);
    pub(crate) const IP_BODY_RANGE: Range<usize> = 20..577;

    pub(crate) const TCP_SRC_PORT: u16 = 50772;
    pub(crate) const TCP_DST_PORT: u16 = 443;
    pub(crate) const TCP_ACK_FLAG: bool = true;
    #[allow(unused)]
    pub(crate) const TCP_PSH_FLAG: bool = true;
    pub(crate) const TCP_FIN_FLAG: bool = false;
    pub(crate) const TCP_SYN_FLAG: bool = false;
    pub(crate) const TCP_OPTIONS: &[TcpOption] =
        &[TcpOption::Timestamp { ts_val: 644_726_309, ts_echo_reply: 2_960_127_454 }];
    pub(crate) const TCP_BODY_RANGE: Range<usize> = 32..557;
    pub(crate) const TCP_BODY: &[u8] = &[
        0x16, 0x03, 0x01, 0x02, 0x08, 0x01, 0x00, 0x02, 0x04, 0x03, 0x03, 0x0f, 0x88, 0xf6, 0x67,
        0x08, 0x84, 0x41, 0xa3, 0xbf, 0x0b, 0x7c, 0x61, 0xd7, 0x57, 0x58, 0xa9, 0x5e, 0x77, 0xee,
        0x0c, 0x5f, 0x4a, 0x45, 0xc7, 0xa1, 0xf6, 0x15, 0xb6, 0x56, 0x07, 0xb8, 0x52, 0x20, 0x97,
        0x9d, 0xfe, 0x16, 0x1d, 0x32, 0x59, 0x74, 0xbd, 0x5a, 0x68, 0xcf, 0xe7, 0x44, 0x48, 0x67,
        0x87, 0x33, 0xc7, 0x84, 0x1b, 0xf1, 0xee, 0xdd, 0x62, 0x7e, 0x88, 0xd3, 0x80, 0x05, 0x36,
        0xcd, 0x00, 0x22, 0xba, 0xba, 0x13, 0x01, 0x13, 0x02, 0x13, 0x03, 0xc0, 0x2b, 0xc0, 0x2f,
        0xc0, 0x2c, 0xc0, 0x30, 0xcc, 0xa9, 0xcc, 0xa8, 0xc0, 0x13, 0xc0, 0x14, 0x00, 0x9c, 0x00,
        0x9d, 0x00, 0x2f, 0x00, 0x35, 0x00, 0x0a, 0x01, 0x00, 0x01, 0x99, 0x2a, 0x2a, 0x00, 0x00,
        0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x16, 0x00, 0x14, 0x00, 0x00, 0x11, 0x62,
        0x65, 0x61, 0x63, 0x6f, 0x6e, 0x73, 0x35, 0x2e, 0x67, 0x76, 0x74, 0x33, 0x2e, 0x63, 0x6f,
        0x6d, 0x00, 0x17, 0x00, 0x00, 0x00, 0x23, 0x00, 0xda, 0x00, 0x9c, 0x63, 0xc8, 0x43, 0x0e,
        0x35, 0x05, 0x52, 0x2f, 0x0e, 0x93, 0xcb, 0x36, 0x0d, 0x72, 0x93, 0xab, 0x88, 0xe6, 0xc3,
        0x74, 0x7e, 0x9f, 0xd0, 0x48, 0xf8, 0x11, 0x0f, 0xab, 0x40, 0x2e, 0x64, 0x43, 0x96, 0xf4,
        0xea, 0x37, 0xcb, 0x1b, 0x12, 0xf5, 0xa4, 0xbb, 0x41, 0xad, 0x1d, 0x1c, 0x16, 0xb5, 0xea,
        0xae, 0x77, 0x4f, 0x83, 0x8f, 0xd7, 0x39, 0x02, 0x20, 0xdf, 0xfc, 0x0f, 0xc4, 0x27, 0x75,
        0xc6, 0x83, 0xf4, 0x9e, 0xcd, 0xea, 0xc2, 0x0e, 0x31, 0x25, 0xdb, 0xdf, 0x43, 0x4b, 0x0a,
        0x4b, 0x47, 0xdc, 0xd5, 0x6c, 0x60, 0x38, 0xe4, 0x41, 0x25, 0xfa, 0xd9, 0xb0, 0xa3, 0x6f,
        0xa1, 0x48, 0xca, 0x0d, 0xb0, 0xfb, 0x4e, 0xee, 0xdd, 0xce, 0xd1, 0xc4, 0x0f, 0xf3, 0x20,
        0xa1, 0x4f, 0x99, 0xd5, 0x56, 0x11, 0xe6, 0x5d, 0x3d, 0x03, 0xc1, 0x8b, 0x64, 0x64, 0xf9,
        0x69, 0xca, 0xe7, 0x68, 0x81, 0x18, 0xd2, 0xb1, 0x93, 0x0f, 0x06, 0x99, 0xb4, 0xc1, 0x76,
        0xd8, 0x0d, 0x2f, 0x57, 0x7f, 0xdf, 0x24, 0x60, 0xbd, 0x1d, 0x84, 0x96, 0xa5, 0xc3, 0x6b,
        0x0c, 0xc2, 0x6f, 0x56, 0x08, 0x4a, 0x6f, 0xc4, 0x5d, 0x62, 0x49, 0xd9, 0xfe, 0xee, 0x40,
        0x9e, 0xbc, 0x89, 0x3e, 0xb2, 0x6a, 0xe3, 0x41, 0x15, 0xa5, 0x85, 0x02, 0xb6, 0x5d, 0xa2,
        0xd8, 0x92, 0x41, 0x8a, 0x0c, 0xbd, 0xea, 0x8f, 0x12, 0xf1, 0x43, 0x99, 0x9e, 0x17, 0x90,
        0xc1, 0xe0, 0xde, 0xb2, 0xc2, 0x40, 0xa3, 0x5f, 0x31, 0xab, 0x41, 0x88, 0xb5, 0xe5, 0xd5,
        0x54, 0xe5, 0x00, 0x0d, 0x00, 0x14, 0x00, 0x12, 0x04, 0x03, 0x08, 0x04, 0x04, 0x01, 0x05,
        0x03, 0x08, 0x05, 0x05, 0x01, 0x08, 0x06, 0x06, 0x01, 0x02, 0x01, 0x00, 0x05, 0x00, 0x05,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c,
        0x02, 0x68, 0x32, 0x08, 0x68, 0x74, 0x74, 0x70, 0x2f, 0x31, 0x2e, 0x31, 0x00, 0x0b, 0x00,
        0x02, 0x01, 0x00, 0x00, 0x33, 0x00, 0x2b, 0x00, 0x29, 0x9a, 0x9a, 0x00, 0x01, 0x00, 0x00,
        0x1d, 0x00, 0x20, 0xb8, 0x47, 0xb2, 0xfa, 0xcc, 0xff, 0x2d, 0xf3, 0xf8, 0x12, 0xc1, 0xd9,
        0xe7, 0x68, 0x72, 0xa1, 0xb2, 0xcd, 0x31, 0x15, 0x8c, 0xad, 0x11, 0xf7, 0x29, 0x8e, 0x3d,
        0xa8, 0x14, 0x86, 0x37, 0x56, 0x00, 0x2d, 0x00, 0x02, 0x01, 0x01, 0x00, 0x2b, 0x00, 0x0b,
        0x0a, 0x1a, 0x1a, 0x7f, 0x17, 0x03, 0x03, 0x03, 0x02, 0x03, 0x01, 0x00, 0x0a, 0x00, 0x0a,
        0x00, 0x08, 0x9a, 0x9a, 0x00, 0x1d, 0x00, 0x17, 0x00, 0x18, 0x3a, 0x3a, 0x00, 0x01, 0x00,
    ];

    pub(crate) const ETHERNET_FRAME_BYTES: &[u8] = &[
        0x0f, 0x41, 0xf1, 0xda, 0xaf, 0x6c, 0x6b, 0xc3, 0x31, 0x1c, 0x79, 0x02, 0x08, 0x00, 0x45,
        0x00, 0x02, 0x41, 0x00, 0x00, 0x40, 0x00, 0x40, 0x06, 0x4f, 0x11, 0xc0, 0xa8, 0x01, 0x0f,
        0x68, 0xed, 0xbf, 0x01, 0xc6, 0x54, 0x01, 0xbb, 0x3a, 0x0b, 0xbd, 0x40, 0x10, 0x4f, 0xb6,
        0xd2, 0x80, 0x18, 0x10, 0x0a, 0x92, 0x13, 0x00, 0x00, 0x01, 0x01, 0x08, 0x0a, 0x26, 0x6d,
        0xbe, 0x25, 0xb0, 0x6f, 0xf5, 0xde, 0x16, 0x03, 0x01, 0x02, 0x08, 0x01, 0x00, 0x02, 0x04,
        0x03, 0x03, 0x0f, 0x88, 0xf6, 0x67, 0x08, 0x84, 0x41, 0xa3, 0xbf, 0x0b, 0x7c, 0x61, 0xd7,
        0x57, 0x58, 0xa9, 0x5e, 0x77, 0xee, 0x0c, 0x5f, 0x4a, 0x45, 0xc7, 0xa1, 0xf6, 0x15, 0xb6,
        0x56, 0x07, 0xb8, 0x52, 0x20, 0x97, 0x9d, 0xfe, 0x16, 0x1d, 0x32, 0x59, 0x74, 0xbd, 0x5a,
        0x68, 0xcf, 0xe7, 0x44, 0x48, 0x67, 0x87, 0x33, 0xc7, 0x84, 0x1b, 0xf1, 0xee, 0xdd, 0x62,
        0x7e, 0x88, 0xd3, 0x80, 0x05, 0x36, 0xcd, 0x00, 0x22, 0xba, 0xba, 0x13, 0x01, 0x13, 0x02,
        0x13, 0x03, 0xc0, 0x2b, 0xc0, 0x2f, 0xc0, 0x2c, 0xc0, 0x30, 0xcc, 0xa9, 0xcc, 0xa8, 0xc0,
        0x13, 0xc0, 0x14, 0x00, 0x9c, 0x00, 0x9d, 0x00, 0x2f, 0x00, 0x35, 0x00, 0x0a, 0x01, 0x00,
        0x01, 0x99, 0x2a, 0x2a, 0x00, 0x00, 0xff, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x16,
        0x00, 0x14, 0x00, 0x00, 0x11, 0x62, 0x65, 0x61, 0x63, 0x6f, 0x6e, 0x73, 0x35, 0x2e, 0x67,
        0x76, 0x74, 0x33, 0x2e, 0x63, 0x6f, 0x6d, 0x00, 0x17, 0x00, 0x00, 0x00, 0x23, 0x00, 0xda,
        0x00, 0x9c, 0x63, 0xc8, 0x43, 0x0e, 0x35, 0x05, 0x52, 0x2f, 0x0e, 0x93, 0xcb, 0x36, 0x0d,
        0x72, 0x93, 0xab, 0x88, 0xe6, 0xc3, 0x74, 0x7e, 0x9f, 0xd0, 0x48, 0xf8, 0x11, 0x0f, 0xab,
        0x40, 0x2e, 0x64, 0x43, 0x96, 0xf4, 0xea, 0x37, 0xcb, 0x1b, 0x12, 0xf5, 0xa4, 0xbb, 0x41,
        0xad, 0x1d, 0x1c, 0x16, 0xb5, 0xea, 0xae, 0x77, 0x4f, 0x83, 0x8f, 0xd7, 0x39, 0x02, 0x20,
        0xdf, 0xfc, 0x0f, 0xc4, 0x27, 0x75, 0xc6, 0x83, 0xf4, 0x9e, 0xcd, 0xea, 0xc2, 0x0e, 0x31,
        0x25, 0xdb, 0xdf, 0x43, 0x4b, 0x0a, 0x4b, 0x47, 0xdc, 0xd5, 0x6c, 0x60, 0x38, 0xe4, 0x41,
        0x25, 0xfa, 0xd9, 0xb0, 0xa3, 0x6f, 0xa1, 0x48, 0xca, 0x0d, 0xb0, 0xfb, 0x4e, 0xee, 0xdd,
        0xce, 0xd1, 0xc4, 0x0f, 0xf3, 0x20, 0xa1, 0x4f, 0x99, 0xd5, 0x56, 0x11, 0xe6, 0x5d, 0x3d,
        0x03, 0xc1, 0x8b, 0x64, 0x64, 0xf9, 0x69, 0xca, 0xe7, 0x68, 0x81, 0x18, 0xd2, 0xb1, 0x93,
        0x0f, 0x06, 0x99, 0xb4, 0xc1, 0x76, 0xd8, 0x0d, 0x2f, 0x57, 0x7f, 0xdf, 0x24, 0x60, 0xbd,
        0x1d, 0x84, 0x96, 0xa5, 0xc3, 0x6b, 0x0c, 0xc2, 0x6f, 0x56, 0x08, 0x4a, 0x6f, 0xc4, 0x5d,
        0x62, 0x49, 0xd9, 0xfe, 0xee, 0x40, 0x9e, 0xbc, 0x89, 0x3e, 0xb2, 0x6a, 0xe3, 0x41, 0x15,
        0xa5, 0x85, 0x02, 0xb6, 0x5d, 0xa2, 0xd8, 0x92, 0x41, 0x8a, 0x0c, 0xbd, 0xea, 0x8f, 0x12,
        0xf1, 0x43, 0x99, 0x9e, 0x17, 0x90, 0xc1, 0xe0, 0xde, 0xb2, 0xc2, 0x40, 0xa3, 0x5f, 0x31,
        0xab, 0x41, 0x88, 0xb5, 0xe5, 0xd5, 0x54, 0xe5, 0x00, 0x0d, 0x00, 0x14, 0x00, 0x12, 0x04,
        0x03, 0x08, 0x04, 0x04, 0x01, 0x05, 0x03, 0x08, 0x05, 0x05, 0x01, 0x08, 0x06, 0x06, 0x01,
        0x02, 0x01, 0x00, 0x05, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00,
        0x00, 0x10, 0x00, 0x0e, 0x00, 0x0c, 0x02, 0x68, 0x32, 0x08, 0x68, 0x74, 0x74, 0x70, 0x2f,
        0x31, 0x2e, 0x31, 0x00, 0x0b, 0x00, 0x02, 0x01, 0x00, 0x00, 0x33, 0x00, 0x2b, 0x00, 0x29,
        0x9a, 0x9a, 0x00, 0x01, 0x00, 0x00, 0x1d, 0x00, 0x20, 0xb8, 0x47, 0xb2, 0xfa, 0xcc, 0xff,
        0x2d, 0xf3, 0xf8, 0x12, 0xc1, 0xd9, 0xe7, 0x68, 0x72, 0xa1, 0xb2, 0xcd, 0x31, 0x15, 0x8c,
        0xad, 0x11, 0xf7, 0x29, 0x8e, 0x3d, 0xa8, 0x14, 0x86, 0x37, 0x56, 0x00, 0x2d, 0x00, 0x02,
        0x01, 0x01, 0x00, 0x2b, 0x00, 0x0b, 0x0a, 0x1a, 0x1a, 0x7f, 0x17, 0x03, 0x03, 0x03, 0x02,
        0x03, 0x01, 0x00, 0x0a, 0x00, 0x0a, 0x00, 0x08, 0x9a, 0x9a, 0x00, 0x1d, 0x00, 0x17, 0x00,
        0x18, 0x3a, 0x3a, 0x00, 0x01, 0x00,
    ];
}

/// IPv6 DNS Request Packet.
///
/// An Ethernet frame containing an IPv6 packet containing a UDP datagram
/// containing a DNS request for google.com.
pub(crate) mod dns_request_v6 {
    #![allow(missing_docs)]

    use std::ops::Range;

    use crate::ip::Ipv6Addr;

    pub(crate) use super::ETHERNET_DST_MAC;
    pub(crate) use super::ETHERNET_SRC_MAC;
    pub(crate) const ETHERNET_BODY_RANGE: Range<usize> = 14..113;

    pub(crate) const IPV6_DS: u8 = 0;
    pub(crate) const IPV6_ECN: u8 = 0;
    pub(crate) const IPV6_FLOWLABEL: u32 = 0xea528;
    pub(crate) const IPV6_HOP_LIMIT: u8 = 64;
    pub(crate) const IPV6_SRC_IP: Ipv6Addr = Ipv6Addr::new([
        0x26, 0x20, 0x00, 0x00, 0x10, 0x00, 0x50, 0x10, 0x69, 0xb4, 0xab, 0xc9, 0x7b, 0x21, 0x92,
        0x15,
    ]);
    pub(crate) const IPV6_DST_IP: Ipv6Addr = Ipv6Addr::new([
        0x20, 0x01, 0x48, 0x60, 0x48, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88,
        0x88,
    ]);
    pub(crate) const IPV6_BODY_RANGE: Range<usize> = 54..113;

    pub(crate) const UDP_SRC_PORT: u16 = 55713;
    pub(crate) const UDP_DST_PORT: u16 = 53;
    pub(crate) const UDP_BODY_RANGE: Range<usize> = 62..113;
    pub(crate) const UDP_BODY: &[u8] = &[
        0xa6, 0xd9, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x67, 0x6f,
        0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
        0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x0a, 0x00, 0x08, 0x9f, 0x30,
        0xaa, 0x16, 0x63, 0xd2, 0x09, 0x25,
    ];

    pub(crate) const ETHERNET_FRAME_BYTES: &[u8] = &[
        0x0f, 0x41, 0xf1, 0xda, 0xaf, 0x6c, 0x6b, 0xc3, 0x31, 0x1c, 0x79, 0x02, 0x86, 0xdd, 0x60,
        0x0e, 0xa5, 0x28, 0x00, 0x3b, 0x11, 0x40, 0x26, 0x20, 0x00, 0x00, 0x10, 0x00, 0x50, 0x10,
        0x69, 0xb4, 0xab, 0xc9, 0x7b, 0x21, 0x92, 0x15, 0x20, 0x01, 0x48, 0x60, 0x48, 0x60, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0x88, 0xd9, 0xa1, 0x00, 0x35, 0x00, 0x3b,
        0x02, 0xf8, 0xa6, 0xd9, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06,
        0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x00, 0x00, 0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x0a, 0x00, 0x08,
        0x9f, 0x30, 0xaa, 0x16, 0x63, 0xd2, 0x09, 0x25,
    ];
}

/// IPv6 TCP SYN Packet.
///
/// An Ethernet frame containing an IPv6 packet containing a TCP SYN segment
/// with an empty body.
pub(crate) mod syn_v6 {
    #![allow(missing_docs)]

    use std::ops::Range;

    use crate::ip::Ipv6Addr;
    use crate::transport::tcp::TcpOption;

    pub(crate) use super::ETHERNET_DST_MAC;
    pub(crate) use super::ETHERNET_SRC_MAC;
    pub(crate) const ETHERNET_BODY_RANGE: Range<usize> = 14..94;

    pub(crate) const IPV6_DS: u8 = 0;
    pub(crate) const IPV6_ECN: u8 = 0;
    pub(crate) const IPV6_FLOWLABEL: u32 = 0x8c55d;
    pub(crate) const IPV6_HOP_LIMIT: u8 = 64;
    pub(crate) const IPV6_SRC_IP: Ipv6Addr = Ipv6Addr::new([
        0x26, 0x20, 0x00, 0x00, 0x10, 0x00, 0x50, 0x10, 0x69, 0xb4, 0xab, 0xc9, 0x7b, 0x21, 0x92,
        0x15,
    ]);
    pub(crate) const IPV6_DST_IP: Ipv6Addr = Ipv6Addr::new([
        0x26, 0x07, 0xf8, 0xb0, 0x40, 0x05, 0x08, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20,
        0x12,
    ]);
    pub(crate) const IPV6_BODY_RANGE: Range<usize> = 54..94;

    pub(crate) const TCP_SRC_PORT: u16 = 42924;
    pub(crate) const TCP_DST_PORT: u16 = 443;
    pub(crate) const TCP_ACK_FLAG: bool = false;
    #[allow(unused)]
    pub(crate) const TCP_PSH_FLAG: bool = false;
    pub(crate) const TCP_FIN_FLAG: bool = false;
    pub(crate) const TCP_SYN_FLAG: bool = true;
    pub(crate) const TCP_OPTIONS: &[TcpOption] = &[
        TcpOption::Mss(1440),
        TcpOption::SackPermitted,
        TcpOption::Timestamp { ts_val: 3_880_784_215, ts_echo_reply: 0 },
        TcpOption::WindowScale(7),
    ];
    pub(crate) const TCP_BODY_RANGE: Range<usize> = 94..94;
    pub(crate) const TCP_BODY: &[u8] = &[];

    pub(crate) const ETHERNET_FRAME_BYTES: &[u8] = &[
        0x0f, 0x41, 0xf1, 0xda, 0xaf, 0x6c, 0x6b, 0xc3, 0x31, 0x1c, 0x79, 0x02, 0x86, 0xdd, 0x60,
        0x08, 0xc5, 0x5d, 0x00, 0x28, 0x06, 0x40, 0x26, 0x20, 0x00, 0x00, 0x10, 0x00, 0x50, 0x10,
        0x69, 0xb4, 0xab, 0xc9, 0x7b, 0x21, 0x92, 0x15, 0x26, 0x07, 0xf8, 0xb0, 0x40, 0x05, 0x08,
        0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x12, 0xa7, 0xac, 0x01, 0xbb, 0x1c, 0x68,
        0x76, 0x8e, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x02, 0x70, 0x80, 0x72, 0xcf, 0x00, 0x00, 0x02,
        0x04, 0x05, 0xa0, 0x04, 0x02, 0x08, 0x0a, 0xe7, 0x50, 0x11, 0x57, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x03, 0x03, 0x07,
    ];
}

/// An ICMP echo reply in an IP packet in an Ethernet frame.
pub(crate) mod icmp_echo_ethernet {
    pub(crate) const REPLY_ETHERNET_FRAME_BYTES: &[u8] = &[
        0x8c, 0x85, 0x90, 0xc9, 0xc9, 0x00, 0x50, 0xc7, 0xbf, 0x1d, 0xf4, 0xd2, 0x08, 0x00, 0x45,
        0x00, 0x00, 0x54, 0x00, 0x00, 0x00, 0x00, 0x35, 0x01, 0x11, 0x91, 0xac, 0xd9, 0x06, 0x2e,
        0xc0, 0xa8, 0x00, 0x69, 0x00, 0x00, 0x01, 0x33, 0xb2, 0xa8, 0x00, 0x01, 0x5c, 0x68, 0x5c,
        0x32, 0x00, 0x03, 0xa8, 0x82, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11,
        0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    ];
}

/// An ICMP echo request/response pair
///
/// An IPv4 pair of packets containing ICMPv4 packets
pub(crate) mod icmp_echo {
    pub(crate) const REQUEST_IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x00, 0x00, 0x54, 0x10, 0x80, 0x40, 0x00, 0x40, 0x01, 0x92, 0x00, 0x64, 0x4c, 0x3a,
        0xd0, 0xd8, 0xef, 0x20, 0x1d, 0x08, 0x00, 0xee, 0x2b, 0x25, 0x8e, 0x00, 0x01, 0x21, 0x50,
        0x24, 0x5b, 0x00, 0x00, 0x00, 0x00, 0xd3, 0xc6, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e,
        0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    ];
    pub(crate) const RESPONSE_IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x80, 0x00, 0x54, 0x00, 0x00, 0x00, 0x00, 0x39, 0x01, 0xe9, 0x00, 0xd8, 0xef, 0x20,
        0x1d, 0x64, 0x4c, 0x3a, 0xd0, 0x00, 0x00, 0xf6, 0x2b, 0x25, 0x8e, 0x00, 0x01, 0x21, 0x50,
        0x24, 0x5b, 0x00, 0x00, 0x00, 0x00, 0xd3, 0xc6, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
        0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e,
        0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    ];
    pub(crate) const ECHO_DATA: &[u8] = &[
        0x21, 0x50, 0x24, 0x5b, 0x00, 0x00, 0x00, 0x00, 0xd3, 0xc6, 0x0c, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
        0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    ];
    pub(crate) const IDENTIFIER: u16 = 9614;
    pub(crate) const SEQUENCE_NUM: u16 = 1;
}

/// An ICMP echo request/response pair
///
/// An IPv6 pair of packets containing ICMPv6 packets
pub(crate) mod icmp_echo_v6 {
    pub(crate) const REQUEST_IP_PACKET_BYTES: &[u8] = &[
        0x60, 0x00, 0x00, 0x00, 0x00, 0x3c, 0x3a, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xfe, 0xc0, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0xe3, 0x13, 0x11,
        0x0d, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a,
        0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
        0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
    ];
    pub(crate) const RESPONSE_IP_PACKET_BYTES: &[u8] = &[];
    pub(crate) const ECHO_DATA: &[u8] = &[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c,
        0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
    ];
    pub(crate) const IDENTIFIER: u16 = 0x110d;
    pub(crate) const SEQUENCE_NUM: u16 = 0;
}

pub(crate) mod icmp_timestamp {
    pub(crate) const REQUEST_IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x00, 0x00, 0x28, 0x8d, 0xff, 0x00, 0x00, 0x80, 0x01, 0x2a, 0xb8, 0xc0, 0xa8, 0x00,
        0x66, 0xc0, 0xa8, 0x00, 0x67, 0x0d, 0x00, 0x94, 0xe3, 0x39, 0x30, 0x00, 0x00, 0x00, 0xf6,
        0x23, 0xf6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    pub(crate) const RESPONSE_IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x00, 0x00, 0x28, 0x0f, 0x3a, 0x00, 0x00, 0x80, 0x01, 0xa9, 0x7d, 0xc0, 0xa8, 0x00,
        0x67, 0xc0, 0xa8, 0x00, 0x66, 0x0e, 0x00, 0xab, 0x87, 0x39, 0x30, 0x00, 0x00, 0x00, 0xf6,
        0x23, 0xf6, 0xfe, 0x2c, 0xf6, 0x00, 0xfe, 0x2c, 0xf6, 0x00,
    ];
    pub(crate) const IDENTIFIER: u16 = 0x3930;
    pub(crate) const SEQUENCE_NUM: u16 = 0;
    pub(crate) const ORIGIN_TIMESTAMP: u32 = 16_131_062;
    pub(crate) const RX_TX_TIMESTAMP: u32 = 0;
}

pub(crate) mod icmp_dest_unreachable {
    pub(crate) const IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x00, 0x00, 0x38, 0x00, 0x03, 0x00, 0x00, 0xff, 0x01, 0xa3, 0xbd, 0x0a, 0x01, 0x02,
        0x01, 0x0a, 0x01, 0x02, 0x02, 0x03, 0x01, 0x2e, 0xfc, 0x00, 0x00, 0x00, 0x00, 0x45, 0x00,
        0x00, 0x64, 0x00, 0x05, 0x00, 0x00, 0xfe, 0x01, 0xae, 0x8f, 0x0a, 0x01, 0x02, 0x02, 0x01,
        0x01, 0x01, 0x01, 0x08, 0x00, 0xc6, 0x01, 0x00, 0x01, 0x00, 0x00,
    ];
    // First 8 bytes of original datagram.
    pub(crate) const ORIGIN_DATA: &[u8] = &[0x08, 0x00, 0xc6, 0x01, 0x00, 0x01, 0x00, 0x00];
}

pub(crate) mod icmp_redirect {
    use crate::ip::Ipv4Addr;

    pub(crate) const IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x00, 0x00, 0x38, 0x01, 0x38, 0x00, 0x00, 0xff, 0x01, 0xa5, 0x94, 0x0a, 0x7b, 0x00,
        0x02, 0x0a, 0x7b, 0x00, 0x01, 0x05, 0x01, 0x13, 0xbe, 0x0a, 0x7b, 0x00, 0x03, 0x45, 0x00,
        0x00, 0x64, 0x00, 0x05, 0x00, 0x00, 0xfe, 0x01, 0xaa, 0x10, 0x0a, 0x7b, 0x00, 0x01, 0x04,
        0x04, 0x04, 0x04, 0x08, 0x00, 0xd4, 0xc1, 0x00, 0x01, 0x00, 0x00,
    ];
    pub(crate) const GATEWAY_ADDR: Ipv4Addr = Ipv4Addr::new([10, 123, 0, 3]);
}

pub(crate) mod icmp_time_exceeded {
    pub(crate) const IP_PACKET_BYTES: &[u8] = &[
        0x45, 0x00, 0x00, 0x38, 0x07, 0xcf, 0x00, 0x00, 0x40, 0x01, 0xf1, 0x9a, 0xc0, 0xa8, 0x00,
        0x01, 0xc0, 0xa8, 0x00, 0x0a, 0x0b, 0x00, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x45, 0x00,
        0x00, 0x34, 0xde, 0x75, 0x00, 0x00, 0x01, 0x11, 0x5c, 0x74, 0xc0, 0xa8, 0x00, 0x0a, 0xac,
        0xd9, 0x11, 0x44, 0xde, 0x74, 0x82, 0x9b, 0x00, 0x20, 0x1f, 0xce,
    ];
    // First 8 bytes of original datagram.
    pub(crate) const ORIGIN_DATA: &[u8] = &[0xde, 0x74, 0x82, 0x9b, 0x00, 0x20, 0x1f, 0xce];
}
