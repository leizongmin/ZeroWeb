//! WebCrypto host 实现——非对称家族（EC + OKP）。从 js_dom_bridge/crypto.rs 拆出
//!（webcrypto-compat-v3 D1 域切片同轮行数治理：主文件超 2000 行红线，按家族
//! 切分；除下方 CSV byte wire helper 外与对称/digest 段零交叉依赖，crate 全限
//! 定路径调用）。

use super::crypto::{bytes_from_csv, bytes_to_csv};

// ---------------------------------------------------------------------------
// EC（ECDSA/ECDH，webcrypto M4b）：namedCurve P-256/P-384/P-521 密钥生成/导入
// （spki/pkcs8/raw+压缩点/jwk 分量）、DER 导出（私派生公）、ECDSA 签名/验签
//（签名 = IEEE P1363 r||s 定宽拼接，RFC 6979 确定性 nonce）、ECDH 共享秘密
//（共享点 x 坐标）。委托 p256/p384/p521 0.13（RustCrypto，选型见
// docs/goal/archive/webcrypto-compat/evidence/2026-10-17-m4b-deps.md）。**密钥材料零落
// 日志**：标量/坐标仅经 CSV 桥进出，无 tracing 输出。所有失败返空串（shim 按操
// 作映射 NotSupportedError/InvalidAccessError/DataError/OperationError）。
// https://w3c.github.io/webcrypto/#ecdsa  https://w3c.github.io/webcrypto/#ecdh
// https://www.secg.org/sec1-v2.pdf  https://datatracker.ietf.org/doc/html/rfc5915
// ---------------------------------------------------------------------------

/// namedCurve 名 → 分发臂（corpus 三曲线；其他值 shim 已按 NotSupportedError 拒绝，
/// 不达此处）。
enum EcCurve {
    P256,
    P384,
    P521,
}

fn parse_ec_curve(name: &str) -> Option<EcCurve> {
    match name {
        "P-256" => Some(EcCurve::P256),
        "P-384" => Some(EcCurve::P384),
        "P-521" => Some(EcCurve::P521),
        _ => None,
    }
}

/// EC 生成：namedCurve → pkcs8 DER csv（私钥；公钥 spki 经
/// `crypto_subtle_ec_export_der` 派生）。生成/编码失败 → 空串。
pub fn crypto_subtle_ec_generate(curve: &str) -> String {
    use p256::elliptic_curve::pkcs8::EncodePrivateKey;
    use p256::elliptic_curve::{SecretKey, rand_core::OsRng};
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    macro_rules! gen_key {
        ($c:ty) => {
            SecretKey::<$c>::random(&mut OsRng)
                .to_pkcs8_der()
                .map(|doc| bytes_to_csv(doc.as_bytes()))
                .unwrap_or_default()
        };
    }
    match c {
        EcCurve::P256 => gen_key!(p256::NistP256),
        EcCurve::P384 => gen_key!(p384::NistP384),
        EcCurve::P521 => gen_key!(p521::NistP521),
    }
}

/// EC DER/点导入（spki/raw 公钥 / pkcs8 私钥）：spki 走 crate 解码，内建
/// AlgorithmIdentifier OID 校验（跨曲线密钥 → Err；spec importKey EC 曲线不匹配
/// → DataError），SEC1 点（压缩/未压缩）均接受（arithmetic feature 解压）；raw =
/// SEC1 点（首字节 0x02/0x03/0x04）。解析后规范化再编码（round-trip 逐字节稳定）。
/// fmt "spki"|"raw"|"pkcs8"。
pub fn crypto_subtle_ec_import(curve: &str, fmt: &str, der_csv: &str) -> String {
    use p256::elliptic_curve::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
    use p256::elliptic_curve::sec1::FromEncodedPoint;
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let der = bytes_from_csv(der_csv);
    macro_rules! imp {
        ($c:ty) => {
            match fmt {
                "spki" => p256::elliptic_curve::PublicKey::<$c>::from_public_key_der(&der)
                    .ok()
                    .and_then(|k| k.to_public_key_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default(),
                "raw" => p256::elliptic_curve::sec1::EncodedPoint::<$c>::from_bytes(&der)
                    .ok()
                    .and_then(|p| p256::elliptic_curve::PublicKey::<$c>::from_encoded_point(&p).into_option())
                    .and_then(|k| k.to_public_key_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default(),
                "pkcs8" => p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&der)
                    .ok()
                    .and_then(|k| k.to_pkcs8_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default(),
                _ => String::new(),
            }
        };
    }
    match c {
        EcCurve::P256 => imp!(p256::NistP256),
        EcCurve::P384 => imp!(p384::NistP384),
        EcCurve::P521 => imp!(p521::NistP521),
    }
}

/// EC jwk 分量导入：x/y 必填（坐标点，曲线外点 → 空串即 DataError）；d 非空 →
/// 私钥（标量非法——零或 ≥ 群阶——→ 空串）。输出规范化 spki（公）/pkcs8（私）
/// DER csv。坐标/标量长度须恰为域元素宽度（spec jwk 步骤按曲线长度校验）。
/// https://w3c.github.io/webcrypto/#importJwkEC
pub fn crypto_subtle_ec_jwk_import(curve: &str, x_csv: &str, y_csv: &str, d_csv: &str) -> String {
    use p256::elliptic_curve::pkcs8::{EncodePrivateKey, EncodePublicKey};
    use p256::elliptic_curve::sec1::FromEncodedPoint;
    use p256::elliptic_curve::{FieldBytes, SecretKey, sec1};
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let x = bytes_from_csv(x_csv);
    let y = bytes_from_csv(y_csv);
    let d = bytes_from_csv(d_csv);
    macro_rules! jwk {
        ($c:ty) => {{
            // FieldBytes::from_slice 对错误长度 panic——先按域宽度显式校验。
            let flen = FieldBytes::<$c>::default().len();
            if x.len() != flen || y.len() != flen {
                return String::new();
            }
            if d.is_empty() {
                let x_ga = FieldBytes::<$c>::from_slice(&x);
                let y_ga = FieldBytes::<$c>::from_slice(&y);
                let point = sec1::EncodedPoint::<$c>::from_affine_coordinates(x_ga, y_ga, false);
                p256::elliptic_curve::PublicKey::<$c>::from_encoded_point(&point)
                    .into_option()
                    .and_then(|k| k.to_public_key_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default()
            } else {
                if d.len() != flen {
                    return String::new();
                }
                let d_ga = FieldBytes::<$c>::from_slice(&d);
                SecretKey::<$c>::from_bytes(d_ga)
                    .ok()
                    .and_then(|k| k.to_pkcs8_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default()
            }
        }};
    }
    match c {
        EcCurve::P256 => jwk!(p256::NistP256),
        EcCurve::P384 => jwk!(p384::NistP384),
        EcCurve::P521 => jwk!(p521::NistP521),
    }
}

/// EC DER 导出。fmt "spki"：私钥派生公钥，公钥透传（均规范化编码）；"pkcs8"：
/// 仅私钥。错误/格式不符 → 空串。
pub fn crypto_subtle_ec_export_der(curve: &str, der_csv: &str, fmt: &str) -> String {
    use p256::elliptic_curve::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
    use p256::elliptic_curve::sec1::ToEncodedPoint;
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let der = bytes_from_csv(der_csv);
    macro_rules! exp {
        ($c:ty) => {
            if fmt == "spki" {
                // 私派生公（generateKey 后 publicKey 无独立存储），失败回落公钥透传。
                p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&der)
                    .ok()
                    .map(|k| k.public_key())
                    .or_else(|| p256::elliptic_curve::PublicKey::<$c>::from_public_key_der(&der).ok())
                    .and_then(|k| k.to_public_key_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default()
            } else if fmt == "raw" {
                // SEC1 未压缩点（spec exportKey raw：publicKey 步骤输出 0x04||X||Y）。
                p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&der)
                    .ok()
                    .map(|k| k.public_key())
                    .or_else(|| p256::elliptic_curve::PublicKey::<$c>::from_public_key_der(&der).ok())
                    .map(|k| bytes_to_csv(k.to_encoded_point(false).as_bytes()))
                    .unwrap_or_default()
            } else if fmt == "pkcs8" {
                p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&der)
                    .ok()
                    .and_then(|k| k.to_pkcs8_der().ok())
                    .map(|doc| bytes_to_csv(doc.as_bytes()))
                    .unwrap_or_default()
            } else {
                String::new()
            }
        };
    }
    match c {
        EcCurve::P256 => exp!(p256::NistP256),
        EcCurve::P384 => exp!(p384::NistP384),
        EcCurve::P521 => exp!(p521::NistP521),
    }
}

/// EC jwk 导出：`;` 分隔 x;y（公）/x;y;d（私）；坐标与标量均为定宽（域元素字节
/// 数）大端 csv——spec JWK 要求零填充定长（WPT generateKey 按 x/y/d 长度断言
/// P-521 缩位风险）。d = 标量定宽字节。错误 → 空串。
pub fn crypto_subtle_ec_export_jwk(curve: &str, der_csv: &str, want_private: bool) -> String {
    use p256::elliptic_curve::pkcs8::{DecodePrivateKey, DecodePublicKey};
    use p256::elliptic_curve::sec1;
    use p256::elliptic_curve::sec1::ToEncodedPoint;
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let der = bytes_from_csv(der_csv);
    macro_rules! jwk {
        ($c:ty) => {{
            let pub_part = |point: &sec1::EncodedPoint<$c>| -> String {
                let x = point.x();
                let y = point.y();
                match (x, y) {
                    (Some(x), Some(y)) => {
                        format!("{};{}", bytes_to_csv(x), bytes_to_csv(y))
                    }
                    _ => String::new(),
                }
            };
            if want_private {
                match p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&der) {
                    // 标量定宽（FieldBytes 零填充）——jwk d 长度 = 域元素宽度。
                    Ok(k) => {
                        let p = pub_part(&k.public_key().to_encoded_point(false));
                        if p.is_empty() {
                            String::new()
                        } else {
                            format!("{};{}", p, bytes_to_csv(&k.to_bytes()))
                        }
                    }
                    Err(_) => String::new(),
                }
            } else {
                match p256::elliptic_curve::PublicKey::<$c>::from_public_key_der(&der) {
                    Ok(k) => pub_part(&k.to_encoded_point(false)),
                    Err(_) => String::new(),
                }
            }
        }};
    }
    match c {
        EcCurve::P256 => jwk!(p256::NistP256),
        EcCurve::P384 => jwk!(p384::NistP384),
        EcCurve::P521 => jwk!(p521::NistP521),
    }
}

/// ECDSA 签名：arg curve = namedCurve，key_csv = pkcs8 DER csv，prehash_csv =
/// 已摘要数据（shim 经 digest host 预算，hash ∈ 注册集）。RFC 6979 确定性
/// nonce；签名 = r||s 定宽拼接（IEEE P1363）。错误 → 空串。
/// https://datatracker.ietf.org/doc/html/rfc6979
pub fn crypto_subtle_ecdsa_sign(curve: &str, key_csv: &str, prehash_csv: &str) -> String {
    use p256::ecdsa::signature::hazmat::PrehashSigner;
    use p256::elliptic_curve::pkcs8::DecodePrivateKey;
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let key = bytes_from_csv(key_csv);
    let ph = bytes_from_csv(prehash_csv);
    // 各 crate 的 ecdsa 模块别名已绑定本曲线（非泛型），故按模块路径传入
    macro_rules! sign {
        ($m:ident, $c:ty) => {{
            // spec bits2int：短摘要左侧零填充至域宽，长摘要取左起域宽字节
            //（RFC 6979 §2.3）。RustCrypto bits2field 额外要求 ≥ 域宽/2（会拒
            // SHA-1×P-384/P-521 合法组合），故自行归一后传入。
            use p256::elliptic_curve::FieldBytes;
            let flen = FieldBytes::<$c>::default().len();
            let phn = if ph.len() >= flen {
                ph[..flen].to_vec()
            } else {
                let mut v = vec![0u8; flen - ph.len()];
                v.extend_from_slice(&ph);
                v
            };
            p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&key)
                .ok()
                .and_then(|k| $m::ecdsa::SigningKey::from_bytes(&k.to_bytes()).ok())
                .and_then(|sk| {
                    // der feature 下 SigningKey 另有 PrehashSigner<der::Signature> impl，
                    // turbofish 选 raw r||s 签名
                    PrehashSigner::<$m::ecdsa::Signature>::sign_prehash(&sk, &phn).ok()
                })
                .map(|sig| bytes_to_csv(&sig.to_bytes()))
                .unwrap_or_default()
        }};
    }
    match c {
        EcCurve::P256 => sign!(p256, p256::NistP256),
        EcCurve::P384 => sign!(p384, p384::NistP384),
        EcCurve::P521 => sign!(p521, p521::NistP521),
    }
}

/// ECDSA 验签：key_csv = spki DER csv，prehash_csv = 已摘要数据，sig_csv =
/// r||s（长度错/坏签名 → "0"，不报错——spec verify 无对应 reject 路径）。
/// 返 "1"/"0"。
pub fn crypto_subtle_ecdsa_verify(curve: &str, key_csv: &str, prehash_csv: &str, sig_csv: &str) -> String {
    use p256::ecdsa::signature::hazmat::PrehashVerifier;
    use p256::elliptic_curve::pkcs8::DecodePublicKey;
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let key = bytes_from_csv(key_csv);
    let ph = bytes_from_csv(prehash_csv);
    let sig = bytes_from_csv(sig_csv);
    macro_rules! verify {
        ($m:ident, $c:ty) => {{
            // 与 sign 同径 bits2int 归一（RustCrypto bits2field ≥ 域宽/2 下限不适用 spec）。
            use p256::elliptic_curve::FieldBytes;
            let flen = FieldBytes::<$c>::default().len();
            let phn = if ph.len() >= flen {
                ph[..flen].to_vec()
            } else {
                let mut v = vec![0u8; flen - ph.len()];
                v.extend_from_slice(&ph);
                v
            };
            p256::elliptic_curve::PublicKey::<$c>::from_public_key_der(&key)
                .ok()
                .and_then(|pk| {
                    // P-521 的 VerifyingKey 是 newtype（无 From<&PublicKey>），统一走 from_affine
                    $m::ecdsa::VerifyingKey::from_affine(*pk.as_affine())
                        .ok()
                        .and_then(|vk| {
                            $m::ecdsa::Signature::from_slice(&sig)
                                .ok()
                                .map(|s| vk.verify_prehash(&phn, &s).is_ok())
                        })
                })
                .unwrap_or(false)
        }};
    }
    let ok = match c {
        EcCurve::P256 => verify!(p256, p256::NistP256),
        EcCurve::P384 => verify!(p384, p384::NistP384),
        EcCurve::P521 => verify!(p521, p521::NistP521),
    };
    if ok { "1".into() } else { "0".into() }
}

/// ECDH 共享秘密：priv_csv = pkcs8 DER csv（base key），pub_csv = spki DER csv
///（peer public）。返回共享点 x 坐标定宽 csv（spec deriveBits 结果源；截断由
/// shim 按 length 位处理）。解析失败/共享点为无穷远 → 空串。
pub fn crypto_subtle_ecdh(curve: &str, priv_csv: &str, pub_csv: &str) -> String {
    use p256::elliptic_curve::pkcs8::{DecodePrivateKey, DecodePublicKey};
    let c = match parse_ec_curve(curve) {
        Some(c) => c,
        None => return String::new(),
    };
    let priv_der = bytes_from_csv(priv_csv);
    let pub_der = bytes_from_csv(pub_csv);
    macro_rules! derive {
        ($c:ty) => {
            p256::elliptic_curve::SecretKey::<$c>::from_pkcs8_der(&priv_der)
                .ok()
                .zip(p256::elliptic_curve::PublicKey::<$c>::from_public_key_der(&pub_der).ok())
                .map(|(sk, pk)| p256::elliptic_curve::ecdh::diffie_hellman(&sk.to_nonzero_scalar(), pk.as_affine()))
                .map(|ss| bytes_to_csv(ss.raw_secret_bytes()))
                .unwrap_or_default()
        };
    }
    match c {
        EcCurve::P256 => derive!(p256::NistP256),
        EcCurve::P384 => derive!(p384::NistP384),
        EcCurve::P521 => derive!(p521::NistP521),
    }
}

// ---------------------------------------------------------------------------
// OKP（Ed25519/X25519，webcrypto M5）：RFC 8032 纯 EdDSA 签名/验签 + RFC 7748
// X25519 共享秘密（全零结果拒绝）。委托 ed25519-dalek/x25519-dalek 3.0（dalek
// 纯 Rust 审计系，选型见 docs/goal/archive/webcrypto-compat/evidence/2026-10-17-m5-okp-
// deps.md）。SPKI/PKCS#8 按 RFC 8410 定长结构手写编解码（前缀常量，无需 pkcs8
// feature）。**密钥材料零落日志**：seed/标量仅经 CSV 桥进出，无 tracing 输出。
// 所有失败返空串（shim 按操作映射 NotSupportedError/InvalidAccessError/
// DataError/OperationError）。
// https://w3c.github.io/webcrypto/#ed25519  https://w3c.github.io/webcrypto/#x25519
// https://datatracker.ietf.org/doc/html/rfc8032  https://datatracker.ietf.org/doc/html/rfc7748
// ---------------------------------------------------------------------------

/// OKP 种类（spec 密钥参数无成员，kind 即算法名）。
#[derive(Clone, Copy)]
enum OkpKind {
    Ed25519,
    X25519,
}

fn parse_okp_kind(name: &str) -> Option<OkpKind> {
    match name {
        "Ed25519" => Some(OkpKind::Ed25519),
        "X25519" => Some(OkpKind::X25519),
        _ => None,
    }
}

// RFC 8410 定长 DER 前缀：SPKI 12 字节（0x03 0x21 0x00 BIT STRING 头）+ 32 字节公钥；
// PKCS#8 16 字节（0x04 0x22 0x04 0x20 OCTET STRING 头）+ 32 字节私钥。
const OKP_SPKI_ED: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];
const OKP_SPKI_X: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x6e, 0x03, 0x21, 0x00];
const OKP_PKCS8_ED: [u8; 16] = [
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
];
const OKP_PKCS8_X: [u8; 16] = [
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x6e, 0x04, 0x22, 0x04, 0x20,
];

/// OS 随机 32 字节（Ed25519 seed / X25519 标量种子）。失败 → None。
fn okp_random_seed() -> Option<[u8; 32]> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).ok()?;
    Some(seed)
}

/// Ed25519：seed → 规范 pkcs8 DER（16 字节前缀 || seed）。
fn okp_ed_pkcs8_from_seed(seed: &[u8; 32]) -> Vec<u8> {
    let mut der = OKP_PKCS8_ED.to_vec();
    der.extend_from_slice(seed);
    der
}

/// X25519：标量 → 规范 pkcs8 DER。标量不在此钳位（导入密钥原样存储，RFC 7748
/// 钳位在运算时施加——dalek StaticSecret::from 内建）。
fn okp_x_pkcs8_from_scalar(scalar: &[u8; 32]) -> Vec<u8> {
    let mut der = OKP_PKCS8_X.to_vec();
    der.extend_from_slice(scalar);
    der
}

/// Ed25519：pkcs8 seed → 公钥 32 字节（RFC 8032 A = [s]B 压缩编码）。
fn okp_ed_public_from_pkcs8(der: &[u8]) -> Option<[u8; 32]> {
    if der.len() != 48 || der[..16] != OKP_PKCS8_ED {
        return None;
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&der[16..48]);
    let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
    Some(*sk.verifying_key().as_bytes())
}

/// X25519：pkcs8 标量 → 公钥 32 字节（钳位后标量 × basepoint u 坐标）。
fn okp_x_public_from_pkcs8(der: &[u8]) -> Option<[u8; 32]> {
    if der.len() != 48 || der[..16] != OKP_PKCS8_X {
        return None;
    }
    let mut scalar = [0u8; 32];
    scalar.copy_from_slice(&der[16..48]);
    let ss = x25519_dalek::StaticSecret::from(scalar);
    Some(*x25519_dalek::PublicKey::from(&ss).as_bytes())
}

/// OKP 生成：kind → "raw 公钥 csv;pkcs8 DER csv"。生成/随机源失败 → 空串。
pub fn crypto_subtle_okp_generate(kind: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let seed = match okp_random_seed() {
        Some(s) => s,
        None => return String::new(),
    };
    match k {
        OkpKind::Ed25519 => {
            let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
            let mut out = bytes_to_csv(sk.verifying_key().as_bytes());
            out.push(';');
            out.push_str(&bytes_to_csv(&okp_ed_pkcs8_from_seed(&seed)));
            out
        }
        OkpKind::X25519 => {
            let ss = x25519_dalek::StaticSecret::from(seed);
            let mut out = bytes_to_csv(x25519_dalek::PublicKey::from(&ss).as_bytes());
            out.push(';');
            // 生成密钥导出钳位后标量（与 dalek 运算语义一致，round-trip 稳定）。
            out.push_str(&bytes_to_csv(&okp_x_pkcs8_from_scalar(&ss.to_bytes())));
            out
        }
    }
}

/// OKP 导入（spki/raw 公钥 / pkcs8 私钥）：SPKI/PKCS#8 按 RFC 8410 定长前缀 +
/// 总长校验（跨族/截断/坏 OID → 空串即 DataError）；raw = 32 字节公钥。输出规范化
/// csv：public = raw 公钥、private = 规范 pkcs8。fmt "spki"|"raw"|"pkcs8"。
pub fn crypto_subtle_okp_import(kind: &str, fmt: &str, der_csv: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let der = bytes_from_csv(der_csv);
    match (k, fmt) {
        (OkpKind::Ed25519, "spki") | (OkpKind::X25519, "spki") => {
            let prefix = if matches!(k, OkpKind::Ed25519) {
                &OKP_SPKI_ED
            } else {
                &OKP_SPKI_X
            };
            if der.len() != 44 || der[..12] != *prefix {
                return String::new();
            }
            let mut raw = [0u8; 32];
            raw.copy_from_slice(&der[12..44]);
            bytes_to_csv(&raw)
        }
        (_, "raw") => {
            if der.len() != 32 {
                return String::new();
            }
            bytes_to_csv(&der)
        }
        (OkpKind::Ed25519, "pkcs8") => {
            if der.len() != 48 || der[..16] != OKP_PKCS8_ED {
                return String::new();
            }
            let seed: [u8; 32] = match der[16..48].try_into() {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            bytes_to_csv(&okp_ed_pkcs8_from_seed(&seed))
        }
        (OkpKind::X25519, "pkcs8") => {
            if der.len() != 48 || der[..16] != OKP_PKCS8_X {
                return String::new();
            }
            let scalar: [u8; 32] = match der[16..48].try_into() {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            bytes_to_csv(&okp_x_pkcs8_from_scalar(&scalar))
        }
        _ => String::new(),
    }
}

/// OKP jwk 分量导入：x 必填 32 字节公钥；d 非空 → 私钥，且须与 x 配对（spec
/// importJwk OKP 步骤 5：派生公钥 ≠ x → DataError——WPT "Invalid key pair" 组）。
/// 输出 public = raw csv / private = pkcs8 csv。x/d 非 32 字节 → 空串。
/// https://w3c.github.io/webcrypto/#dfn-Validate-OtpJwk
pub fn crypto_subtle_okp_jwk_import(kind: &str, x_csv: &str, d_csv: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let x = bytes_from_csv(x_csv);
    let d = bytes_from_csv(d_csv);
    if x.len() != 32 {
        return String::new();
    }
    if d.is_empty() {
        return bytes_to_csv(&x);
    }
    if d.len() != 32 {
        return String::new();
    }
    let (derived, pkcs8) = match k {
        OkpKind::Ed25519 => {
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&d);
            (
                *ed25519_dalek::SigningKey::from_bytes(&seed).verifying_key().as_bytes(),
                okp_ed_pkcs8_from_seed(&seed),
            )
        }
        OkpKind::X25519 => {
            let mut scalar = [0u8; 32];
            scalar.copy_from_slice(&d);
            let ss = x25519_dalek::StaticSecret::from(scalar);
            (
                *x25519_dalek::PublicKey::from(&ss).as_bytes(),
                okp_x_pkcs8_from_scalar(&ss.to_bytes()),
            )
        }
    };
    // jwk d 与 x 配对校验（失配 → 空串即 DataError）。
    if derived != x[..] {
        return String::new();
    }
    bytes_to_csv(&pkcs8)
}

/// OKP DER/raw 导出。fmt "spki"：公钥 raw → SPKI、私钥派生公；"raw"：同 spki 的
/// 公钥字节；"pkcs8"：仅私钥透传（导入/生成已规范化）。错误/格式不符 → 空串。
pub fn crypto_subtle_okp_export_der(kind: &str, raw_csv: &str, fmt: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let stored = bytes_from_csv(raw_csv);
    // 私钥（存储 = pkcs8）按需派生公钥字节。
    let public_raw: Option<Vec<u8>> = match stored.len() {
        32 => Some(stored.clone()),
        48 => match k {
            OkpKind::Ed25519 => okp_ed_public_from_pkcs8(&stored).map(|r| r.to_vec()),
            OkpKind::X25519 => okp_x_public_from_pkcs8(&stored).map(|r| r.to_vec()),
        },
        _ => None,
    };
    match fmt {
        "spki" => {
            let raw = match public_raw {
                Some(r) => r,
                None => return String::new(),
            };
            let mut der = (if matches!(k, OkpKind::Ed25519) {
                OKP_SPKI_ED
            } else {
                OKP_SPKI_X
            })
            .to_vec();
            der.extend_from_slice(&raw);
            bytes_to_csv(&der)
        }
        "raw" => match public_raw {
            Some(r) => bytes_to_csv(&r),
            None => String::new(),
        },
        "pkcs8" => {
            if stored.len() != 48 {
                return String::new();
            }
            bytes_to_csv(&stored)
        }
        _ => String::new(),
    }
}

/// OKP jwk 导出：`;` 分隔 x（公）/x;d（私），均 32 字节定宽 csv。公钥 raw 派生
/// 自 pkcs8（私钥导出时）。错误 → 空串。
pub fn crypto_subtle_okp_export_jwk(kind: &str, raw_csv: &str, want_private: bool) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let stored = bytes_from_csv(raw_csv);
    if want_private {
        if stored.len() != 48 {
            return String::new();
        }
        let x = match k {
            OkpKind::Ed25519 => okp_ed_public_from_pkcs8(&stored),
            OkpKind::X25519 => okp_x_public_from_pkcs8(&stored),
        };
        let x = match x {
            Some(x) => x,
            None => return String::new(),
        };
        let mut out = bytes_to_csv(&x);
        out.push(';');
        out.push_str(&bytes_to_csv(&stored[16..48]));
        out
    } else {
        if stored.len() != 32 {
            return String::new();
        }
        bytes_to_csv(&stored)
    }
}

/// Ed25519 签名（RFC 8032 纯 EdDSA，消息直接签名不预摘要）：key_csv = pkcs8 DER
/// csv，msg_csv = 消息。签名 = R||S 64 字节定宽 csv。错误 → 空串。
pub fn crypto_subtle_ed25519_sign(key_csv: &str, msg_csv: &str) -> String {
    use ed25519_dalek::Signer;
    let der = bytes_from_csv(key_csv);
    if der.len() != 48 || der[..16] != OKP_PKCS8_ED {
        return String::new();
    }
    let msg = bytes_from_csv(msg_csv);
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&der[16..48]);
    let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
    bytes_to_csv(&sk.sign(&msg).to_bytes())
}

/// Ed25519 验签：key_csv = raw 公钥 csv，sig_csv = R||S。坏签名/错长度/小阶点/
/// 非规范编码 → "0"（spec verify 无对应 reject 路径）。返 "1"/"0"。
pub fn crypto_subtle_ed25519_verify(key_csv: &str, msg_csv: &str, sig_csv: &str) -> String {
    let key = bytes_from_csv(key_csv);
    let msg = bytes_from_csv(msg_csv);
    let sig = bytes_from_csv(sig_csv);
    if key.len() != 32 || sig.len() != 64 {
        return "0".into();
    }
    let pk: [u8; 32] = key.as_slice().try_into().unwrap_or([0; 32]);
    let vk = match ed25519_dalek::VerifyingKey::from_bytes(&pk) {
        Ok(vk) => vk,
        Err(_) => return "0".into(),
    };
    let s = match ed25519_dalek::Signature::from_slice(&sig) {
        Ok(s) => s,
        Err(_) => return "0".into(),
    };
    // 严格验证（RFC 8032 规范性）：拒小阶 R/A（eprint 2020/1244 小阶伪造面，WPT
    // eddsa_small_order_points 14 例全 expect false/唯一 mixed 例 true——非严格
    // verify 对小阶 A 误 true）+ 非 canonical S/R。
    if vk.verify_strict(&msg, &s).is_ok() {
        "1".into()
    } else {
        "0".into()
    }
}

/// X25519 共享秘密（RFC 7748 §5）：priv_csv = pkcs8 DER csv（base key），
/// pub_csv = raw 公钥 csv。返回共享 u 坐标 32 字节定宽 csv（spec deriveBits 结果
/// 源；截断由 shim 按 length 位处理）。**全零结果 → 空串**（spec 步骤：结果全零
/// → OperationError——小阶公钥/dom 钳位退化；WPT cfrg "all-zero value result"）。
pub fn crypto_subtle_x25519(priv_csv: &str, pub_csv: &str) -> String {
    let der = bytes_from_csv(priv_csv);
    if der.len() != 48 || der[..16] != OKP_PKCS8_X {
        return String::new();
    }
    let pub_raw = bytes_from_csv(pub_csv);
    if pub_raw.len() != 32 {
        return String::new();
    }
    let mut scalar = [0u8; 32];
    scalar.copy_from_slice(&der[16..48]);
    let mut peer = [0u8; 32];
    peer.copy_from_slice(&pub_raw);
    let ss = x25519_dalek::StaticSecret::from(scalar);
    let shared = ss.diffie_hellman(&x25519_dalek::PublicKey::from(peer));
    // RFC 7748 §6.1 + spec：全零共享秘密（小阶点输入）→ 拒绝。
    if shared.as_bytes().iter().all(|&b| b == 0) {
        return String::new();
    }
    bytes_to_csv(shared.as_bytes())
}
