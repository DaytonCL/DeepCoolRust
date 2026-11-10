# Protocol attribution

The DeepCool AK Series USB HID constants, device IDs, and packet layouts were researched using the MIT-licensed project:

- Project: https://github.com/Shark1nat0r/deepcool-digital-windows
- Original author: Sharkinator, copyright (c) 2025
- License: MIT

The Rust source is independently written. It uses Rust USB HID access instead of HidSharp and Windows system time counters instead of LibreHardwareMonitor.

The reference project's MIT license text:

MIT License

Copyright (c) 2025 Sharkinator

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

The Rust hidapi dependency is licensed separately; consult its package metadata before binary redistribution.
