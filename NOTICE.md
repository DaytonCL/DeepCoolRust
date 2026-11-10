# Attribution and third-party dependencies

## Deepcool-Digital-Windows — HID protocol

The AK Series HID packet format and USB identifiers were researched using:

- Source: https://github.com/Shark1nat0r/deepcool-digital-windows
- Author: Sharkinator, copyright (c) 2025
- License: MIT

## SidebarMonitor — AMD SDK integration reference

The general technique of translating the AMD Ryzen Master Monitoring SDK C++ interface into plain C ABI functions was researched using:

- Source: https://github.com/WRCX/SidebarMonitor/blob/main/native/RyzenShim/RyzenShim.cpp
- Author: WRCX, copyright (c) 2026
- License: MIT

These projects are used as protocol/API references. The Rust application and native adapter are newly implemented for this repository.

## AMD Ryzen Master Monitoring SDK

- Vendor: Advanced Micro Devices, Inc.
- SDK: https://www.amd.com/en/developer/ryzen-master-monitoring-sdk.html
- SDK headers, Platform.dll, Device.dll, driver, and accompanying licensed files are **not** shipped with this repository.
- Users must install and use the AMD SDK under AMD's applicable license terms.

## MIT license notice for original reference works

MIT License

Copyright (c) 2025 Sharkinator
Copyright (c) 2026 WRCX

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Open-source dependencies

- Rust hidapi package: see its upstream license.
- Rust cc build dependency: see its upstream license.
