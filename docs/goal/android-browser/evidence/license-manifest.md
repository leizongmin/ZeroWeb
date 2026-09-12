# Android APK 依赖与许可证清单

**生成日期**: 2026-09-12
**目标平台**: aarch64-linux-android（release APK ABI；`make android-renderer-apk` 产物）
**闭包根**: zero-android-browser（browser 进程 cdylib，全部 JNI/Rust 面）
**包总数**: 341（workspace 内 13）

## Rust 依赖闭包（cargo metadata --filter-platform，可重复）

| crate | version | license |
|-------|---------|---------|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| aho-corasick | 1.1.4 | Unlicense OR MIT |
| alloc-no-stdlib | 2.0.4 | BSD-3-Clause |
| alloc-stdlib | 0.2.4 | BSD-3-Clause |
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 |
| android-activity | 0.6.1 | MIT OR Apache-2.0 |
| android-properties | 0.2.2 | MIT |
| android_system_properties | 0.1.5 | MIT/Apache-2.0 |
| anes | 0.1.6 | MIT OR Apache-2.0 |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| arrayref | 0.3.9 | BSD-2-Clause |
| arrayvec | 0.7.7 | MIT OR Apache-2.0 |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 |
| async-compression | 0.4.42 | MIT OR Apache-2.0 |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| base64 | 0.22.1 | MIT OR Apache-2.0 |
| bincode | 1.3.3 | MIT |
| bit-set | 0.10.0 | Apache-2.0 OR MIT |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT |
| bitflags | 1.3.2 | MIT/Apache-2.0 |
| bitflags | 2.13.0 | MIT OR Apache-2.0 |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| brotli | 8.0.4 | BSD-3-Clause AND MIT |
| brotli-decompressor | 5.0.3 | BSD-3-Clause/MIT |
| bytemuck | 1.25.0 | Zlib OR Apache-2.0 OR MIT |
| bytemuck_derive | 1.10.2 | Zlib OR Apache-2.0 OR MIT |
| byteorder-lite | 0.1.0 | Unlicense OR MIT |
| bytes | 1.12.0 | MIT |
| cast | 0.3.0 | MIT OR Apache-2.0 |
| cc | 1.2.65 | MIT OR Apache-2.0 |
| cesu8 | 1.1.0 | Apache-2.0/MIT |
| cfg-if | 1.0.4 | MIT OR Apache-2.0 |
| cfg_aliases | 0.2.1 | MIT |
| chacha20 | 0.10.1 | MIT OR Apache-2.0 |
| ciborium | 0.2.2 | Apache-2.0 |
| ciborium-io | 0.2.2 | Apache-2.0 |
| ciborium-ll | 0.2.2 | Apache-2.0 |
| clap | 4.6.1 | MIT OR Apache-2.0 |
| clap_builder | 4.6.0 | MIT OR Apache-2.0 |
| clap_lex | 1.1.0 | MIT OR Apache-2.0 |
| codespan-reporting | 0.13.1 | Apache-2.0 |
| color_quant | 1.1.0 | MIT |
| combine | 4.6.7 | MIT |
| compression-codecs | 0.4.38 | MIT OR Apache-2.0 |
| compression-core | 0.4.32 | MIT OR Apache-2.0 |
| core_maths | 0.1.1 | MIT |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| crc32fast | 1.5.0 | MIT OR Apache-2.0 |
| criterion | 0.5.1 | Apache-2.0 OR MIT |
| criterion-plot | 0.5.0 | MIT/Apache-2.0 |
| crossbeam-deque | 0.8.6 | MIT OR Apache-2.0 |
| crossbeam-epoch | 0.9.18 | MIT OR Apache-2.0 |
| crossbeam-utils | 0.8.21 | MIT OR Apache-2.0 |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib |
| data-encoding | 2.11.0 | MIT |
| data-url | 0.3.2 | MIT OR Apache-2.0 |
| digest | 0.10.7 | MIT OR Apache-2.0 |
| dirs | 6.0.0 | MIT OR Apache-2.0 |
| dirs-sys | 0.5.0 | MIT OR Apache-2.0 |
| displaydoc | 0.2.6 | MIT OR Apache-2.0 |
| dlib | 0.5.3 | MIT |
| document-features | 0.2.12 | MIT OR Apache-2.0 |
| dpi | 0.1.2 | Apache-2.0 AND MIT |
| either | 1.16.0 | MIT OR Apache-2.0 |
| encoding_rs | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| equivalent | 1.0.2 | Apache-2.0 OR MIT |
| euclid | 0.22.14 | MIT OR Apache-2.0 |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| find-msvc-tools | 0.1.9 | MIT OR Apache-2.0 |
| flate2 | 1.1.9 | MIT OR Apache-2.0 |
| float-cmp | 0.9.0 | MIT |
| foldhash | 0.1.5 | Zlib |
| foldhash | 0.2.0 | Zlib |
| font-types | 0.11.3 | MIT OR Apache-2.0 |
| fontdb | 0.23.0 | MIT |
| fontdue | 0.9.3 | MIT OR Apache-2.0 OR Zlib |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 |
| freetype-rs | 0.38.0 | MIT |
| freetype-sys | 0.23.0 | MIT |
| futures-channel | 0.3.32 | MIT OR Apache-2.0 |
| futures-core | 0.3.32 | MIT OR Apache-2.0 |
| futures-executor | 0.3.32 | MIT OR Apache-2.0 |
| futures-io | 0.3.32 | MIT OR Apache-2.0 |
| futures-sink | 0.3.32 | MIT OR Apache-2.0 |
| futures-task | 0.3.32 | MIT OR Apache-2.0 |
| futures-util | 0.3.32 | MIT OR Apache-2.0 |
| generic-array | 0.14.7 | MIT |
| getrandom | 0.2.17 | MIT OR Apache-2.0 |
| getrandom | 0.3.4 | MIT OR Apache-2.0 |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| gif | 0.14.2 | MIT OR Apache-2.0 |
| glow | 0.17.0 | MIT OR Apache-2.0 OR Zlib |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 |
| half | 2.7.1 | MIT OR Apache-2.0 |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| http | 1.4.2 | MIT OR Apache-2.0 |
| http-body | 1.0.1 | MIT |
| http-body-util | 0.1.3 | MIT |
| httparse | 1.10.1 | MIT OR Apache-2.0 |
| hyper | 1.10.1 | MIT |
| hyper-rustls | 0.27.9 | Apache-2.0 OR ISC OR MIT |
| hyper-util | 0.1.20 | MIT |
| icu_collections | 2.2.0 | Unicode-3.0 |
| icu_locale_core | 2.2.0 | Unicode-3.0 |
| icu_normalizer | 2.2.0 | Unicode-3.0 |
| icu_normalizer_data | 2.2.0 | Unicode-3.0 |
| icu_properties | 2.2.0 | Unicode-3.0 |
| icu_properties_data | 2.2.0 | Unicode-3.0 |
| icu_provider | 2.2.0 | Unicode-3.0 |
| idna | 1.1.0 | MIT OR Apache-2.0 |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT |
| image-webp | 0.2.4 | MIT OR Apache-2.0 |
| imagesize | 0.14.0 | MIT |
| indexmap | 2.14.0 | Apache-2.0 OR MIT |
| ipnet | 2.12.0 | MIT OR Apache-2.0 |
| is-terminal | 0.4.17 | MIT |
| itertools | 0.10.5 | MIT/Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| jni | 0.21.1 | MIT/Apache-2.0 |
| jni | 0.22.4 | MIT OR Apache-2.0 |
| jni-macros | 0.22.4 | MIT OR Apache-2.0 |
| jni-sys | 0.3.1 | MIT OR Apache-2.0 |
| jni-sys | 0.4.1 | MIT OR Apache-2.0 |
| jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 |
| jobserver | 0.1.34 | MIT OR Apache-2.0 |
| jpeg-decoder | 0.3.2 | MIT OR Apache-2.0 |
| khronos-egl | 6.0.0 | MIT/Apache-2.0 |
| kurbo | 0.13.1 | Apache-2.0 OR MIT |
| lazy_static | 1.5.0 | MIT OR Apache-2.0 |
| libc | 0.2.186 | MIT OR Apache-2.0 |
| libloading | 0.8.9 | ISC |
| libm | 0.2.16 | MIT |
| libz-sys | 1.1.29 | MIT OR Apache-2.0 |
| litemap | 0.8.2 | Unicode-3.0 |
| litrs | 1.0.0 | MIT OR Apache-2.0 |
| lock_api | 0.4.14 | MIT OR Apache-2.0 |
| log | 0.4.33 | MIT OR Apache-2.0 |
| lru-slab | 0.1.2 | MIT OR Apache-2.0 OR Zlib |
| memchr | 2.8.2 | Unlicense OR MIT |
| memmap2 | 0.9.11 | MIT OR Apache-2.0 |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| mio | 1.2.1 | MIT |
| naga | 30.0.0 | MIT OR Apache-2.0 |
| naga-types | 30.0.0 | MIT OR Apache-2.0 |
| ndk | 0.9.0 | MIT OR Apache-2.0 |
| ndk-context | 0.1.1 | MIT OR Apache-2.0 |
| ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 |
| nu-ansi-term | 0.50.3 | MIT |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| oorandom | 11.1.5 | MIT |
| option-ext | 0.2.0 | MPL-2.0 |
| ordered-float | 4.6.0 | MIT |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 |
| pico-args | 0.5.0 | MIT |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| pkg-config | 0.3.33 | MIT OR Apache-2.0 |
| plotters | 0.3.7 | MIT |
| plotters-backend | 0.3.7 | MIT |
| plotters-svg | 0.3.7 | MIT |
| png | 0.17.16 | MIT OR Apache-2.0 |
| png | 0.18.1 | MIT OR Apache-2.0 |
| pollster | 0.4.0 | Apache-2.0/MIT |
| polycool | 0.4.0 | MIT OR Apache-2.0 |
| potential_utf | 0.1.5 | Unicode-3.0 |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 |
| presser | 0.3.1 | MIT OR Apache-2.0 |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.106 | MIT OR Apache-2.0 |
| profiling | 1.0.18 | MIT OR Apache-2.0 |
| quick-error | 2.0.1 | MIT/Apache-2.0 |
| quinn | 0.11.11 | MIT OR Apache-2.0 |
| quinn-proto | 0.11.16 | MIT OR Apache-2.0 |
| quinn-udp | 0.5.15 | MIT OR Apache-2.0 |
| quote | 1.0.46 | MIT OR Apache-2.0 |
| rand | 0.10.2 | MIT OR Apache-2.0 |
| rand | 0.9.4 | MIT OR Apache-2.0 |
| rand_chacha | 0.9.0 | MIT OR Apache-2.0 |
| rand_core | 0.10.1 | MIT OR Apache-2.0 |
| rand_core | 0.6.4 | MIT OR Apache-2.0 |
| rand_core | 0.9.5 | MIT OR Apache-2.0 |
| rand_pcg | 0.10.2 | MIT OR Apache-2.0 |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| rayon | 1.12.0 | MIT OR Apache-2.0 |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 |
| read-fonts | 0.39.2 | MIT OR Apache-2.0 |
| regex | 1.12.4 | MIT OR Apache-2.0 |
| regex-automata | 0.4.14 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 |
| reqwest | 0.12.28 | MIT OR Apache-2.0 |
| resvg | 0.47.0 | Apache-2.0 OR MIT |
| rgb | 0.8.53 | MIT |
| ring | 0.17.14 | Apache-2.0 AND ISC |
| roxmltree | 0.21.1 | MIT OR Apache-2.0 |
| rustc-hash | 1.1.0 | Apache-2.0/MIT |
| rustc-hash | 2.1.2 | Apache-2.0 OR MIT |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 |
| rustls | 0.23.41 | Apache-2.0 OR ISC OR MIT |
| rustls-pki-types | 1.15.0 | MIT OR Apache-2.0 |
| rustls-webpki | 0.103.13 | ISC |
| rustversion | 1.0.22 | MIT OR Apache-2.0 |
| rustybuzz | 0.20.1 | MIT |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| same-file | 1.0.6 | Unlicense/MIT |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde | 1.0.228 | MIT OR Apache-2.0 |
| serde_core | 1.0.228 | MIT OR Apache-2.0 |
| serde_derive | 1.0.228 | MIT OR Apache-2.0 |
| serde_json | 1.0.150 | MIT OR Apache-2.0 |
| serde_urlencoded | 0.7.1 | MIT/Apache-2.0 |
| serial_test | 3.5.0 | MIT |
| serial_test_derive | 3.5.0 | MIT |
| sha1 | 0.10.6 | MIT OR Apache-2.0 |
| sharded-slab | 0.1.7 | MIT |
| shlex | 2.0.1 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.9 | MIT |
| simd_cesu8 | 1.1.1 | Apache-2.0 OR MIT |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 |
| simplecss | 0.2.2 | Apache-2.0 OR MIT |
| siphasher | 1.0.3 | MIT/Apache-2.0 |
| skrifa | 0.42.1 | MIT OR Apache-2.0 |
| slab | 0.4.12 | MIT |
| slotmap | 1.1.1 | Zlib |
| smallvec | 1.15.2 | MIT OR Apache-2.0 |
| smol_str | 0.2.2 | MIT OR Apache-2.0 |
| socket2 | 0.6.4 | MIT OR Apache-2.0 |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 |
| strict-num | 0.1.1 | MIT |
| subtle | 2.6.1 | BSD-3-Clause |
| svgtypes | 0.16.1 | Apache-2.0 OR MIT |
| swash | 0.2.9 | Apache-2.0 OR MIT |
| syn | 2.0.118 | MIT OR Apache-2.0 |
| syn | 3.0.3 | MIT OR Apache-2.0 |
| sync_wrapper | 1.0.2 | Apache-2.0 |
| synstructure | 0.13.2 | MIT |
| termcolor | 1.4.1 | Unlicense OR MIT |
| thiserror | 1.0.69 | MIT OR Apache-2.0 |
| thiserror | 2.0.18 | MIT OR Apache-2.0 |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.18 | MIT OR Apache-2.0 |
| thread_local | 1.1.9 | MIT OR Apache-2.0 |
| tiny-skia | 0.11.4 | BSD-3-Clause |
| tiny-skia | 0.12.0 | BSD-3-Clause |
| tiny-skia-path | 0.11.4 | BSD-3-Clause |
| tiny-skia-path | 0.12.0 | BSD-3-Clause |
| tinystr | 0.8.3 | Unicode-3.0 |
| tinytemplate | 1.2.1 | Apache-2.0 OR MIT |
| tinyvec | 1.11.0 | Zlib OR Apache-2.0 OR MIT |
| tinyvec_macros | 0.1.1 | MIT OR Apache-2.0 OR Zlib |
| tokio | 1.52.3 | MIT |
| tokio-macros | 2.7.2 | MIT |
| tokio-rustls | 0.26.4 | MIT OR Apache-2.0 |
| tokio-util | 0.7.18 | MIT |
| toml_datetime | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_edit | 0.25.12+spec-1.1.0 | MIT OR Apache-2.0 |
| toml_parser | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 |
| tower | 0.5.3 | MIT |
| tower-http | 0.6.11 | MIT |
| tower-layer | 0.3.3 | MIT |
| tower-service | 0.3.3 | MIT |
| tracing | 0.1.44 | MIT |
| tracing-attributes | 0.1.31 | MIT |
| tracing-core | 0.1.36 | MIT |
| tracing-log | 0.2.0 | MIT |
| tracing-subscriber | 0.3.23 | MIT |
| try-lock | 0.2.5 | MIT |
| ttf-parser | 0.21.1 | MIT OR Apache-2.0 |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 |
| tungstenite | 0.29.0 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 |
| unicode-bidi-mirroring | 0.4.0 | MIT/Apache-2.0 |
| unicode-ccc | 0.4.0 | MIT/Apache-2.0 |
| unicode-ident | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| unicode-properties | 0.1.4 | MIT/Apache-2.0 |
| unicode-script | 0.5.8 | MIT OR Apache-2.0 |
| unicode-segmentation | 1.12.0 | MIT OR Apache-2.0 |
| unicode-vo | 0.1.0 | MIT/Apache-2.0 |
| unicode-width | 0.2.2 | MIT OR Apache-2.0 |
| untrusted | 0.9.0 | ISC |
| url | 2.5.8 | MIT OR Apache-2.0 |
| usvg | 0.47.0 | Apache-2.0 OR MIT |
| utf8_iter | 1.0.4 | Apache-2.0 OR MIT |
| vcpkg | 0.2.15 | MIT/Apache-2.0 |
| version_check | 0.9.5 | MIT/Apache-2.0 |
| walkdir | 2.5.0 | Unlicense/MIT |
| want | 0.3.1 | MIT |
| wayland-sys | 0.31.11 | MIT |
| webpki-roots | 1.0.8 | CDLA-Permissive-2.0 |
| weezl | 0.1.12 | MIT OR Apache-2.0 |
| wgpu | 30.0.0 | MIT OR Apache-2.0 |
| wgpu-core | 30.0.0 | MIT OR Apache-2.0 |
| wgpu-core-deps-windows-linux-android | 30.0.0 | MIT OR Apache-2.0 |
| wgpu-hal | 30.0.0 | MIT OR Apache-2.0 |
| wgpu-naga-bridge | 30.0.0 | MIT OR Apache-2.0 |
| wgpu-types | 30.0.0 | MIT OR Apache-2.0 |
| winit | 0.30.13 | Apache-2.0 |
| winnow | 1.0.3 | MIT |
| writeable | 0.6.3 | Unicode-3.0 |
| wuff | 0.2.8 | MIT |
| xmlwriter | 0.1.0 | MIT |
| yazi | 0.2.1 | Apache-2.0 OR MIT |
| yoke | 0.8.3 | Unicode-3.0 |
| yoke-derive | 0.8.2 | Unicode-3.0 |
| zeno | 0.3.3 | Apache-2.0 OR MIT |
| zero-android-browser | 0.1.0 | MIT |
| zero-browser-shell | 0.1.0 | MIT |
| zero-compositor | 0.1.0 | MIT |
| zero-host-runtime | 0.1.0 | MIT |
| zero-image-decoder | 0.1.0 | MIT |
| zero-net | 0.1.0 | MIT |
| zero-paint-convert | 0.1.0 | MIT |
| zero-product-version | 0.1.0 | MIT |
| zero-protocol | 0.1.0 | MIT |
| zero-psl | 0.1.0 | MIT |
| zero-render-foundation | 0.1.0 | MIT |
| zero-runtime-config | 0.1.0 | MIT |
| zero-security | 0.1.0 | MIT |
| zerocopy | 0.8.52 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerocopy-derive | 0.8.52 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerofrom | 0.1.8 | Unicode-3.0 |
| zerofrom-derive | 0.1.7 | Unicode-3.0 |
| zeroize | 1.9.0 | Apache-2.0 OR MIT |
| zerotrie | 0.2.4 | Unicode-3.0 |
| zerovec | 0.11.6 | Unicode-3.0 |
| zerovec-derive | 0.11.3 | Unicode-3.0 |
| zmij | 1.0.21 | MIT |
| zune-core | 0.5.1 | MIT OR Apache-2.0 OR Zlib |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib |

## APK 捆绑组件（非 cargo 闭包）

| 组件 | 许可证 | 说明 |
|------|--------|------|
| libc++_shared.so | Apache-2.0 WITH LLVM-exception | NDK C++ 运行时（renderer/完整版 APK 打包） |
| V8（renderer feature APK） | BSD-3-Clause（Chromium/V8） | rusty_v8 v150.2.0 源码交叉编译，静态链接进 libzero_android_browser.so |
| Kotlin/Compose 运行时（Jetpack Compose、Material3） | Apache-2.0 | gradle libs.versions.toml 版本锁（apps/android-browser/gradle/libs.versions.toml） |
| Android framework API | Android SDK 许可证 | minSdk 26 / targetSdk 36 |

## 许可证未知项跟进

（无）
