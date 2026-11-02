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
// OKP（Ed25519/X25519，webcrypto M5；X448，webcrypto-compat-v3 D1 拍板
// 2026-11-02）：RFC 8032 纯 EdDSA 签名/验签 + RFC 7748 X25519/X448 共享秘密
//（全零结果拒绝）。委托 ed25519-dalek/x25519-dalek 3.0（dalek 纯 Rust 审计系，
// 选型见 docs/goal/archive/webcrypto-compat/evidence/2026-10-17-m5-okp-deps.md）
// 与 x448 0.6.0（host 两条义务：公钥导入不做贡献性检查；derive 全零后拒绝——
// 预检 evidence 2026-11-02-d2-pqc-kmac-x448-corpus-face-precheck.md）。SPKI/
// PKCS#8 按 RFC 8410 定长结构手写编解码（前缀常量 + 密钥长度按 kind 取
// okp_spec 表，无需 pkcs8 feature）。**密钥材料零落日志**：seed/标量仅经 CSV
// 桥进出，无 tracing 输出。所有失败返空串（shim 按操作映射
// NotSupportedError/InvalidAccessError/DataError/OperationError）。
// https://w3c.github.io/webcrypto/#ed25519  https://w3c.github.io/webcrypto/#x25519
// https://w3c.github.io/webcrypto/#x448
// https://datatracker.ietf.org/doc/html/rfc8032  https://datatracker.ietf.org/doc/html/rfc7748
// ---------------------------------------------------------------------------

/// OKP 种类（spec 密钥参数无成员，kind 即算法名）。
#[derive(Clone, Copy)]
enum OkpKind {
    Ed25519,
    X25519,
    X448,
}

fn parse_okp_kind(name: &str) -> Option<OkpKind> {
    match name {
        "Ed25519" => Some(OkpKind::Ed25519),
        "X25519" => Some(OkpKind::X25519),
        "X448" => Some(OkpKind::X448),
        _ => None,
    }
}

// RFC 8410 定长 DER 前缀：SPKI 12 字节（BIT STRING 头 0x03 len 0x00 尾）+ 密钥；
// PKCS#8 16 字节（0x04 len 0x04 len OCTET STRING 头）+ 私钥。X448 OID 1.3.101.111
//（0x2b 0x65 0x6f），密钥/标量 56 字节。
const OKP_SPKI_ED: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];
const OKP_SPKI_X: [u8; 12] = [0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x6e, 0x03, 0x21, 0x00];
const OKP_SPKI_X448: [u8; 12] = [0x30, 0x42, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x6f, 0x03, 0x39, 0x00];
const OKP_PKCS8_ED: [u8; 16] = [
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20,
];
const OKP_PKCS8_X: [u8; 16] = [
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x6e, 0x04, 0x22, 0x04, 0x20,
];
const OKP_PKCS8_X448: [u8; 16] = [
    0x30, 0x46, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x6f, 0x04, 0x3a, 0x04, 0x38,
];

/// OKP 定长结构表（kind → SPKI 前缀、PKCS#8 前缀、密钥字节数；RFC 8410 各 curve
/// 长度不同——Ed25519/X25519 32、X448 56；SPKI 总长 = 密钥 + 12，PKCS#8 = +16）。
fn okp_spec(kind: OkpKind) -> (&'static [u8], &'static [u8], usize) {
    match kind {
        OkpKind::Ed25519 => (&OKP_SPKI_ED, &OKP_PKCS8_ED, 32),
        OkpKind::X25519 => (&OKP_SPKI_X, &OKP_PKCS8_X, 32),
        OkpKind::X448 => (&OKP_SPKI_X448, &OKP_PKCS8_X448, 56),
    }
}

/// OS 随机 32 字节（Ed25519 seed / X25519 标量种子）。失败 → None。
fn okp_random_seed() -> Option<[u8; 32]> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).ok()?;
    Some(seed)
}

/// OS 随机 56 字节（X448 标量种子）。失败 → None。
fn okp_random_seed448() -> Option<[u8; 56]> {
    let mut seed = [0u8; 56];
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

/// X448：标量 → 规范 pkcs8 DER。标量不在此钳位（同 X25519——x448 Secret::from
/// 运算时钳位）。
fn okp_x448_pkcs8_from_scalar(scalar: &[u8; 56]) -> Vec<u8> {
    let mut der = OKP_PKCS8_X448.to_vec();
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

/// X448：pkcs8 标量 → 公钥 56 字节（钳位后标量 × basepoint u 坐标）。
fn okp_x448_public_from_pkcs8(der: &[u8]) -> Option<[u8; 56]> {
    if der.len() != 72 || der[..16] != OKP_PKCS8_X448 {
        return None;
    }
    let secret = x448::Secret::from_bytes(&der[16..72])?;
    Some(*x448::PublicKey::from(&secret).as_bytes())
}

/// OKP 生成：kind → "raw 公钥 csv;pkcs8 DER csv"。生成/随机源失败 → 空串。
pub fn crypto_subtle_okp_generate(kind: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    match k {
        OkpKind::Ed25519 => {
            let seed = match okp_random_seed() {
                Some(s) => s,
                None => return String::new(),
            };
            let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
            let mut out = bytes_to_csv(sk.verifying_key().as_bytes());
            out.push(';');
            out.push_str(&bytes_to_csv(&okp_ed_pkcs8_from_seed(&seed)));
            out
        }
        OkpKind::X25519 => {
            let seed = match okp_random_seed() {
                Some(s) => s,
                None => return String::new(),
            };
            let ss = x25519_dalek::StaticSecret::from(seed);
            let mut out = bytes_to_csv(x25519_dalek::PublicKey::from(&ss).as_bytes());
            out.push(';');
            // 生成密钥导出钳位后标量（与 dalek 运算语义一致，round-trip 稳定）。
            out.push_str(&bytes_to_csv(&okp_x_pkcs8_from_scalar(&ss.to_bytes())));
            out
        }
        OkpKind::X448 => {
            let seed = match okp_random_seed448() {
                Some(s) => s,
                None => return String::new(),
            };
            let secret = x448::Secret::from(seed);
            let mut out = bytes_to_csv(x448::PublicKey::from(&secret).as_bytes());
            out.push(';');
            // 生成密钥导出钳位后标量（x448 Secret::from 内建 RFC 7748 钳位，
            // round-trip 稳定）。
            out.push_str(&bytes_to_csv(&okp_x448_pkcs8_from_scalar(secret.as_bytes())));
            out
        }
    }
}

/// OKP 导入（spki/raw 公钥 / pkcs8 私钥）：SPKI/PKCS#8 按 RFC 8410 定长前缀 +
/// 总长校验（跨族/截断/坏 OID → 空串即 DataError）；raw = 密钥定宽公钥。输出
/// 规范化 csv：public = raw 公钥、private = 规范 pkcs8。公钥导入不做贡献性检查
///（语料小阶点 importKey 必须成功，OperationError 归 derive——x448 host 义务①）。
/// fmt "spki"|"raw"|"pkcs8"。
pub fn crypto_subtle_okp_import(kind: &str, fmt: &str, der_csv: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let (spki_prefix, pkcs8_prefix, klen) = okp_spec(k);
    let der = bytes_from_csv(der_csv);
    match (k, fmt) {
        (_, "spki") => {
            if der.len() != klen + 12 || der[..12] != *spki_prefix {
                return String::new();
            }
            bytes_to_csv(&der[12..])
        }
        (_, "raw") => {
            if der.len() != klen {
                return String::new();
            }
            bytes_to_csv(&der)
        }
        (OkpKind::Ed25519, "pkcs8") => {
            if der.len() != klen + 16 || der[..16] != *pkcs8_prefix {
                return String::new();
            }
            let seed: [u8; 32] = match der[16..].try_into() {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            bytes_to_csv(&okp_ed_pkcs8_from_seed(&seed))
        }
        (OkpKind::X25519, "pkcs8") => {
            if der.len() != klen + 16 || der[..16] != *pkcs8_prefix {
                return String::new();
            }
            let scalar: [u8; 32] = match der[16..].try_into() {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            bytes_to_csv(&okp_x_pkcs8_from_scalar(&scalar))
        }
        (OkpKind::X448, "pkcs8") => {
            if der.len() != klen + 16 || der[..16] != *pkcs8_prefix {
                return String::new();
            }
            let scalar: [u8; 56] = match der[16..].try_into() {
                Ok(s) => s,
                Err(_) => return String::new(),
            };
            bytes_to_csv(&okp_x448_pkcs8_from_scalar(&scalar))
        }
        _ => String::new(),
    }
}

/// OKP jwk 分量导入：x 必填密钥定宽公钥；d 非空 → 私钥，且须与 x 配对（spec
/// importJwk OKP 步骤 5：派生公钥 ≠ x → DataError——WPT "Invalid key pair" 组）。
/// 输出 public = raw csv / private = pkcs8 csv。x/d 非定宽 → 空串。
/// https://w3c.github.io/webcrypto/#dfn-Validate-OtpJwk
pub fn crypto_subtle_okp_jwk_import(kind: &str, x_csv: &str, d_csv: &str) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let x = bytes_from_csv(x_csv);
    let d = bytes_from_csv(d_csv);
    let klen = okp_spec(k).2;
    if x.len() != klen {
        return String::new();
    }
    if d.is_empty() {
        return bytes_to_csv(&x);
    }
    if d.len() != klen {
        return String::new();
    }
    // 三臂统一 (Vec<u8>, Vec<u8>)（X448 公钥 56 字节与 32 字节族不同定宽）。
    let (derived, pkcs8) = match k {
        OkpKind::Ed25519 => {
            let Ok(seed) = d.try_into() else {
                return String::new();
            };
            (
                ed25519_dalek::SigningKey::from_bytes(&seed)
                    .verifying_key()
                    .as_bytes()
                    .to_vec(),
                okp_ed_pkcs8_from_seed(&seed),
            )
        }
        OkpKind::X25519 => {
            let Ok(scalar) = <[u8; 32]>::try_from(&d[..]) else {
                return String::new();
            };
            let ss = x25519_dalek::StaticSecret::from(scalar);
            (
                x25519_dalek::PublicKey::from(&ss).as_bytes().to_vec(),
                okp_x_pkcs8_from_scalar(&ss.to_bytes()),
            )
        }
        OkpKind::X448 => {
            let Ok(scalar) = <[u8; 56]>::try_from(&d[..]) else {
                return String::new();
            };
            let secret = x448::Secret::from(scalar);
            (
                x448::PublicKey::from(&secret).as_bytes().to_vec(),
                okp_x448_pkcs8_from_scalar(secret.as_bytes()),
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
    let (spki_prefix, _, klen) = okp_spec(k);
    let stored = bytes_from_csv(raw_csv);
    // 私钥（存储 = pkcs8）按需派生公钥字节。
    let public_raw: Option<Vec<u8>> = match stored.len() {
        n if n == klen => Some(stored.clone()),
        n if n == klen + 16 => match k {
            OkpKind::Ed25519 => okp_ed_public_from_pkcs8(&stored).map(|r| r.to_vec()),
            OkpKind::X25519 => okp_x_public_from_pkcs8(&stored).map(|r| r.to_vec()),
            OkpKind::X448 => okp_x448_public_from_pkcs8(&stored).map(|r| r.to_vec()),
        },
        _ => None,
    };
    match fmt {
        "spki" => {
            let raw = match public_raw {
                Some(r) => r,
                None => return String::new(),
            };
            let mut der = spki_prefix.to_vec();
            der.extend_from_slice(&raw);
            bytes_to_csv(&der)
        }
        "raw" => match public_raw {
            Some(r) => bytes_to_csv(&r),
            None => String::new(),
        },
        "pkcs8" => {
            if stored.len() != klen + 16 {
                return String::new();
            }
            bytes_to_csv(&stored)
        }
        _ => String::new(),
    }
}

/// OKP jwk 导出：`;` 分隔 x（公）/x;d（私），均密钥定宽 csv。公钥 raw 派生
/// 自 pkcs8（私钥导出时）。错误 → 空串。
pub fn crypto_subtle_okp_export_jwk(kind: &str, raw_csv: &str, want_private: bool) -> String {
    let k = match parse_okp_kind(kind) {
        Some(k) => k,
        None => return String::new(),
    };
    let stored = bytes_from_csv(raw_csv);
    let klen = okp_spec(k).2;
    if want_private {
        if stored.len() != klen + 16 {
            return String::new();
        }
        let x = match k {
            OkpKind::Ed25519 => okp_ed_public_from_pkcs8(&stored).map(|r| r.to_vec()),
            OkpKind::X25519 => okp_x_public_from_pkcs8(&stored).map(|r| r.to_vec()),
            OkpKind::X448 => okp_x448_public_from_pkcs8(&stored).map(|r| r.to_vec()),
        };
        let x = match x {
            Some(x) => x,
            None => return String::new(),
        };
        let mut out = bytes_to_csv(&x);
        out.push(';');
        out.push_str(&bytes_to_csv(&stored[16..]));
        out
    } else {
        if stored.len() != klen {
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

/// X448 共享秘密（RFC 7748 §5）：priv_csv = pkcs8 DER csv（base key），pub_csv =
/// raw 公钥 csv。公钥按 `from_bytes_unchecked` 仅查长度（host 义务①——小阶点
/// 导入必须成功，贡献性检查由 crate `as_diffie_hellman` 的 `is_low_order` 在
/// derive 期承担）。返回共享 u 坐标 56 字节定宽 csv（spec deriveBits 结果源；
/// 截断由 shim 按 length 位处理）。**全零结果 → 空串**（host 义务②——order
/// p/p+1 两点 crate 返 Some 全零，RFC 7748 §6 all-zero 判据；小阶点 →
/// OperationError，WPT cfrg "all-zero value result"）。
pub fn crypto_subtle_x448(priv_csv: &str, pub_csv: &str) -> String {
    let der = bytes_from_csv(priv_csv);
    if der.len() != 72 || der[..16] != OKP_PKCS8_X448 {
        return String::new();
    }
    let pub_raw = bytes_from_csv(pub_csv);
    if pub_raw.len() != 56 {
        return String::new();
    }
    let secret = match x448::Secret::from_bytes(&der[16..72]) {
        Some(s) => s,
        None => return String::new(),
    };
    let peer = match x448::PublicKey::from_bytes_unchecked(&pub_raw) {
        Some(p) => p,
        None => return String::new(),
    };
    // 小阶点（order 0/1/2）crate 内拒绝 → None；order p/p+1 → Some 全零。
    let shared = match secret.as_diffie_hellman(&peer) {
        Some(s) => s,
        None => return String::new(),
    };
    if shared.as_bytes().iter().all(|&b| b == 0) {
        return String::new();
    }
    bytes_to_csv(shared.as_bytes())
}

// ---------------------------------------------------------------------------
// ML（PQC，webcrypto-compat-v3 D1 域切片 2026-11-02）：ML-KEM（FIPS 203）
// encapsulate/decapsulate + ML-DSA（FIPS 204）sign/verify。委托 ml-kem 0.3.3 /
// ml-dsa 0.1.1（RustCrypto 官方最终版线；选型、FIPS 203/204 KAT 逐字节复现与
// 钉版拍板见 docs/goal/webcrypto-compat-v3/evidence/
// 2026-11-02-d2-pqc-kmac-x448-corpus-face-precheck.md）。密钥线格式（WebCrypto
// draft）：raw-seed = 种子原样（KEM 64B / DSA 32B）、raw-public = 原始公钥
//（ek/pk）、spki = RFC 5280（AlgId + BIT STRING(0x00||key)）、pkcs8 = RFC 5958
// seed-only（OCTET STRING 包 [0] 上下文标签种子，与 WPT fixtures 形态逐字节
// 一致）。存储契
// 约：私钥 = 种子 csv、公钥 = raw 公钥 csv（DER/jwk 导出按需编码，ML-DSA jwk
// 在 shim 构造——AKP 成员 pub/priv 即两份 csv 的 b64u）。**密钥材料零落日志**：
// 种子/密钥/签名仅经 CSV 桥进出，无 tracing 输出。所有失败返空串（shim 按操
// 作映射 NotSupportedError/InvalidAccessError/DataError/OperationError）；
// verify 失败/坏签名返 "0"。ML-DSA 签名走 FIPS 204 确定性变体
// sign_deterministic（Algorithm 6，rnd 取零路径——spec ML-DSA.sign 允许的两变
// 体之一；hedged 仅随机化 rnd，签名可验性等价，语料 round trip 只断言自验）。
// https://w3c.github.io/webcrypto/#ml-kem  https://w3c.github.io/webcrypto/#ml-dsa
// https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.203.pdf
// https://nvlpubs.nist.gov/nistpubs/fips/nist.fips.204.pdf
// ---------------------------------------------------------------------------

/// DER TLV 编码：tag || 定长内容（长度按 DER 规则短/长形编码，覆盖本域最大
/// 2.6 kB 内容，两字节长形已足）。
fn der_tlv_encode(tag: u8, contents: &[u8]) -> Vec<u8> {
    let mut v = vec![tag];
    let n = contents.len();
    if n < 0x80 {
        v.push(n as u8);
    } else {
        v.push(0x82);
        v.push((n >> 8) as u8);
        v.push((n & 0xff) as u8);
    }
    v.extend_from_slice(contents);
    v
}

/// DER TLV 解析：tag 须匹配 want，长度须完整且恰尽声明字节。返 (内容, 余部)。
///（信任边界：密钥 DER 来自页面输入，解析失败一律拒绝，不做宽松容错。）
fn der_tlv_parse(buf: &[u8], want: u8) -> Option<(&[u8], &[u8])> {
    if buf.len() < 2 || buf[0] != want {
        return None;
    }
    let (len, hdr): (usize, usize) = match buf[1] {
        n if n < 0x80 => (n as usize, 2),
        0x81 if buf.len() >= 3 => (buf[2] as usize, 3),
        0x82 if buf.len() >= 4 => (((buf[2] as usize) << 8) | buf[3] as usize, 4),
        _ => return None,
    };
    let end: usize = hdr.checked_add(len)?;
    if end > buf.len() {
        return None;
    }
    Some((&buf[hdr..end], &buf[end..]))
}

/// NIST PQC OID 2.16.840.1.101.3.4.fam.var 的 AlgorithmIdentifier：
/// SEQUENCE(13B) { OID TLV(11B) }（fam 0x04 = KEM / 0x03 = DSA 签名）。
fn ml_alg_id_seq(fam: u8, var: u8) -> [u8; 13] {
    let mut v = [0u8; 13];
    v[0] = 0x30;
    v[1] = 0x0b;
    v[2] = 0x06;
    v[3] = 0x09;
    v[4..].copy_from_slice(&[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, fam, var]);
    v
}

/// SubjectPublicKeyInfo 编码：SEQ { AlgId, BIT STRING(0x00 || key) }。
fn ml_spki_encode(alg_id: &[u8; 13], key: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(key.len() + 1);
    bits.push(0x00);
    bits.extend_from_slice(key);
    let mut body = alg_id.to_vec();
    body.extend_from_slice(&der_tlv_encode(0x03, &bits));
    der_tlv_encode(0x30, &body)
}

/// spki 解析：外层 SEQ 恰尽、AlgId 逐字节匹配、BIT STRING 未用位为 0、密钥恰
/// key_len 字节。返公钥字节（跨变体/截断/尾随垃圾 → None 即 DataError）。
fn ml_spki_parse(der: &[u8], alg_id: &[u8; 13], key_len: usize) -> Option<Vec<u8>> {
    let (body, rest) = der_tlv_parse(der, 0x30)?;
    if !rest.is_empty() {
        return None;
    }
    let (alg, r1) = der_tlv_parse(body, 0x30)?;
    // der_tlv_parse 返回 SEQ 内容（OID TLV 11B），比对注册 AlgId 的 OID 段。
    if alg != &alg_id[2..] {
        return None;
    }
    let (bits, r2) = der_tlv_parse(r1, 0x03)?;
    if !r2.is_empty() || bits.first() != Some(&0) || bits.len() != key_len + 1 {
        return None;
    }
    Some(bits[1..].to_vec())
}

/// PKCS#8（seed-only，RFC 5958 oneAsymmetricKey，与 WPT fixtures 形态一致）编
/// 码：SEQ { INT 0, AlgId, OCTET STRING { [0] 上下文原语标签(0x80) || seed } }。
fn ml_pkcs8_seed_encode(alg_id: &[u8; 13], seed: &[u8]) -> Vec<u8> {
    let mut body = vec![0x02, 0x01, 0x00];
    body.extend_from_slice(alg_id);
    body.extend_from_slice(&der_tlv_encode(0x04, &der_tlv_encode(0x80, seed)));
    der_tlv_encode(0x30, &body)
}

/// pkcs8 seed-only 解析：结构同上逐层校验。返种子字节（失配 → None 即
/// DataError；expanded 私钥形态本域不支持——WebCrypto draft 只定义 seed 派生）。
fn ml_pkcs8_seed_parse(der: &[u8], alg_id: &[u8; 13], seed_len: usize) -> Option<Vec<u8>> {
    let (body, rest) = der_tlv_parse(der, 0x30)?;
    if !rest.is_empty() {
        return None;
    }
    let (ver, r1) = der_tlv_parse(body, 0x02)?;
    if *ver != [0x00] {
        return None;
    }
    let (alg, r2) = der_tlv_parse(r1, 0x30)?;
    // der_tlv_parse 返回 SEQ 内容（OID TLV 11B），比对注册 AlgId 的 OID 段。
    if alg != &alg_id[2..] {
        return None;
    }
    let (oct, r3) = der_tlv_parse(r2, 0x04)?;
    if !r3.is_empty() {
        return None;
    }
    // OCTET 内容 = [0] 上下文原语标签 TLV（0x80 <len> <seed>，与 WPT fixtures
    // 形态逐字节一致；expanded 私钥形态本域不支持——WebCrypto draft 只定义
    // seed 派生）。
    let (seed, r4) = der_tlv_parse(oct, 0x80)?;
    if !r4.is_empty() || seed.len() != seed_len {
        return None;
    }
    Some(seed.to_vec())
}

// ------------------------------ ML-KEM（FIPS 203）--------------------------

/// ML-KEM 变体（spec 密钥参数无成员，name 即算法名）。
#[derive(Clone, Copy)]
enum MlKemVariant {
    V512,
    V768,
    V1024,
}

fn parse_mlkem_variant(name: &str) -> Option<MlKemVariant> {
    match name {
        "ML-KEM-512" => Some(MlKemVariant::V512),
        "ML-KEM-768" => Some(MlKemVariant::V768),
        "ML-KEM-1024" => Some(MlKemVariant::V1024),
        _ => None,
    }
}

/// 变体参数（FIPS 203 §7）：(OID 尾, ek 字节, ct 字节)；种子恒 64B、共享秘密
/// 恒 32B。
fn mlkem_params(v: MlKemVariant) -> (u8, usize, usize) {
    match v {
        MlKemVariant::V512 => (0x01, 800, 768),
        MlKemVariant::V768 => (0x02, 1184, 1088),
        MlKemVariant::V1024 => (0x03, 1568, 1568),
    }
}

/// ML-KEM 生成：64 字节随机种子（FIPS 203 KeyGen：d||z）→ "ek csv;seed csv"
///（公 = ek / 私 = 种子）。随机源失败 → 空串。
pub fn crypto_subtle_mlkem_generate(name: &str) -> String {
    use ml_kem::kem::KeyExport;
    let v = match parse_mlkem_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let mut seed = [0u8; 64];
    if getrandom::fill(&mut seed).is_err() {
        return String::new();
    }
    let seed_arr = match ml_kem::Seed::try_from(&seed[..]) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    macro_rules! keygen {
        ($p:ty) => {
            bytes_to_csv(
                ml_kem::DecapsulationKey::<$p>::from_seed(seed_arr)
                    .encapsulation_key()
                    .to_bytes()
                    .as_slice(),
            )
        };
    }
    let ek = match v {
        MlKemVariant::V512 => keygen!(ml_kem::MlKem512),
        MlKemVariant::V768 => keygen!(ml_kem::MlKem768),
        MlKemVariant::V1024 => keygen!(ml_kem::MlKem1024),
    };
    format!("{};{}", ek, bytes_to_csv(&seed))
}

/// ML-KEM 导入（spki/raw-public 公钥 → ek csv；pkcs8/raw-seed 私钥 → 种子
/// csv）。解析/长度/跨变体失配 → 空串即 DataError。
pub fn crypto_subtle_mlkem_import(name: &str, fmt: &str, der_csv: &str) -> String {
    let v = match parse_mlkem_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (var, ek_len, _ct_len) = mlkem_params(v);
    let alg_id = ml_alg_id_seq(0x04, var);
    let der = bytes_from_csv(der_csv);
    match fmt {
        "spki" => match ml_spki_parse(&der, &alg_id, ek_len) {
            Some(k) => bytes_to_csv(&k),
            None => String::new(),
        },
        "raw-public" => {
            if der.len() != ek_len {
                String::new()
            } else {
                bytes_to_csv(&der)
            }
        }
        "pkcs8" => match ml_pkcs8_seed_parse(&der, &alg_id, 64) {
            Some(s) => bytes_to_csv(&s),
            None => String::new(),
        },
        "raw-seed" => {
            if der.len() != 64 {
                String::new()
            } else {
                bytes_to_csv(&der)
            }
        }
        _ => String::new(),
    }
}

/// ML-KEM 导出：输入 ek csv（公）或种子 csv（私，按长度判别）。fmt "spki"/
/// "raw-public"（公透传或私派生公）、"pkcs8"/"raw-seed"（仅私）。错误 → 空串。
pub fn crypto_subtle_mlkem_export_der(name: &str, raw_csv: &str, fmt: &str) -> String {
    use ml_kem::kem::KeyExport;
    let v = match parse_mlkem_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (var, ek_len, _ct_len) = mlkem_params(v);
    let alg_id = ml_alg_id_seq(0x04, var);
    let stored = bytes_from_csv(raw_csv);
    let (ek, seed) = if stored.len() == 64 {
        // 私派生公（getPublicKey / 公钥侧 jwk 导出路径）。
        let seed_arr = match ml_kem::Seed::try_from(&stored[..]) {
            Ok(a) => a,
            Err(_) => return String::new(),
        };
        macro_rules! derive {
            ($p:ty) => {
                ml_kem::DecapsulationKey::<$p>::from_seed(seed_arr)
                    .encapsulation_key()
                    .to_bytes()
                    .as_slice()
                    .to_vec()
            };
        }
        let ek = match v {
            MlKemVariant::V512 => derive!(ml_kem::MlKem512),
            MlKemVariant::V768 => derive!(ml_kem::MlKem768),
            MlKemVariant::V1024 => derive!(ml_kem::MlKem1024),
        };
        (ek, Some(stored))
    } else if stored.len() == ek_len {
        (stored, None)
    } else {
        return String::new();
    };
    match fmt {
        "spki" => bytes_to_csv(&ml_spki_encode(&alg_id, &ek)),
        "raw-public" => bytes_to_csv(&ek),
        "pkcs8" | "raw-seed" => match seed {
            Some(s) if fmt == "pkcs8" => bytes_to_csv(&ml_pkcs8_seed_encode(&alg_id, &s)),
            Some(s) => bytes_to_csv(&s),
            None => String::new(),
        },
        _ => String::new(),
    }
}

/// ML-KEM 封装（FIPS 203 Encapsulate）：ek csv → "ct csv;ss csv"（crate
/// `encapsulate` 内置 OS 随机源取 m；宿主随机面见 crate getrandom feature）。
/// ek 长度错/解析失败 → 空串即 OperationError。
pub fn crypto_subtle_mlkem_encapsulate(name: &str, ek_csv: &str) -> String {
    use ml_kem::kem::Encapsulate;
    let v = match parse_mlkem_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (_var, ek_len, _ct_len) = mlkem_params(v);
    let ek_bytes = bytes_from_csv(ek_csv);
    if ek_bytes.len() != ek_len {
        return String::new();
    }
    macro_rules! enc {
        ($p:ty) => {{
            let arr = match ml_kem::array::Array::try_from(&ek_bytes[..]) {
                Ok(a) => a,
                Err(_) => return String::new(),
            };
            let ek = match ml_kem::EncapsulationKey::<$p>::new(&arr) {
                Ok(k) => k,
                Err(_) => return String::new(),
            };
            let (ct, ss) = ek.encapsulate();
            format!("{};{}", bytes_to_csv(ct.as_slice()), bytes_to_csv(ss.as_slice()))
        }};
    }
    match v {
        MlKemVariant::V512 => enc!(ml_kem::MlKem512),
        MlKemVariant::V768 => enc!(ml_kem::MlKem768),
        MlKemVariant::V1024 => enc!(ml_kem::MlKem1024),
    }
}

/// ML-KEM 解封装（FIPS 203 Decapsulate）：种子 csv + ct csv → ss csv。ct 长度
/// 错/种子失配 → 空串即 OperationError；损坏 ct 走隐式拒绝路径（FIPS 203 §7.3
///——确定性伪共享秘密，语料 KAT 按此断言）。
pub fn crypto_subtle_mlkem_decapsulate(name: &str, seed_csv: &str, ct_csv: &str) -> String {
    use ml_kem::kem::Decapsulate;
    let v = match parse_mlkem_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (_var, _ek_len, ct_len) = mlkem_params(v);
    let seed = bytes_from_csv(seed_csv);
    let ct = bytes_from_csv(ct_csv);
    if seed.len() != 64 || ct.len() != ct_len {
        return String::new();
    }
    let seed_arr = match ml_kem::Seed::try_from(&seed[..]) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    macro_rules! dec {
        ($p:ty) => {{
            let dk = ml_kem::DecapsulationKey::<$p>::from_seed(seed_arr);
            match dk.decapsulate_slice(&ct) {
                Ok(ss) => bytes_to_csv(ss.as_slice()),
                Err(_) => String::new(),
            }
        }};
    }
    match v {
        MlKemVariant::V512 => dec!(ml_kem::MlKem512),
        MlKemVariant::V768 => dec!(ml_kem::MlKem768),
        MlKemVariant::V1024 => dec!(ml_kem::MlKem1024),
    }
}

/// ML-KEM 私派生公（getPublicKey / jwk 配对校验用）：种子 csv → ek csv。
pub fn crypto_subtle_mlkem_public_from_seed(name: &str, seed_csv: &str) -> String {
    use ml_kem::kem::KeyExport;
    let v = match parse_mlkem_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let seed = bytes_from_csv(seed_csv);
    if seed.len() != 64 {
        return String::new();
    }
    let seed_arr = match ml_kem::Seed::try_from(&seed[..]) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    macro_rules! derive {
        ($p:ty) => {
            bytes_to_csv(
                ml_kem::DecapsulationKey::<$p>::from_seed(seed_arr)
                    .encapsulation_key()
                    .to_bytes()
                    .as_slice(),
            )
        };
    }
    match v {
        MlKemVariant::V512 => derive!(ml_kem::MlKem512),
        MlKemVariant::V768 => derive!(ml_kem::MlKem768),
        MlKemVariant::V1024 => derive!(ml_kem::MlKem1024),
    }
}

// ------------------------------ ML-DSA（FIPS 204）---------------------------

/// ML-DSA 变体（spec 密钥参数无成员，name 即算法名）。
#[derive(Clone, Copy)]
enum MlDsaVariant {
    V44,
    V65,
    V87,
}

fn parse_mldsa_variant(name: &str) -> Option<MlDsaVariant> {
    match name {
        "ML-DSA-44" => Some(MlDsaVariant::V44),
        "ML-DSA-65" => Some(MlDsaVariant::V65),
        "ML-DSA-87" => Some(MlDsaVariant::V87),
        _ => None,
    }
}

/// 变体参数（FIPS 204 §5）：(OID 尾, pk 字节)；种子恒 32B。
fn mldsa_params(v: MlDsaVariant) -> (u8, usize) {
    match v {
        MlDsaVariant::V44 => (0x11, 1312),
        MlDsaVariant::V65 => (0x12, 1952),
        MlDsaVariant::V87 => (0x13, 2592),
    }
}

/// ML-DSA 生成：32 字节随机种子（FIPS 204 KeyGen ξ）→ "pk csv;seed csv"
///（公 = pk / 私 = 种子）。随机源失败 → 空串。
pub fn crypto_subtle_mldsa_generate(name: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let mut seed = [0u8; 32];
    if getrandom::fill(&mut seed).is_err() {
        return String::new();
    }
    let seed_arr = match ml_dsa::Seed::try_from(&seed[..]) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    macro_rules! keygen {
        ($p:ty) => {{
            let sk = ml_dsa::SigningKey::<$p>::from_seed(&seed_arr);
            bytes_to_csv(sk.expanded_key().verifying_key().encode().as_slice())
        }};
    }
    let pk = match v {
        MlDsaVariant::V44 => keygen!(ml_dsa::MlDsa44),
        MlDsaVariant::V65 => keygen!(ml_dsa::MlDsa65),
        MlDsaVariant::V87 => keygen!(ml_dsa::MlDsa87),
    };
    format!("{};{}", pk, bytes_to_csv(&seed))
}

/// ML-DSA 导入（spki/raw-public 公钥 → pk csv；pkcs8/raw-seed 私钥 → 种子
/// csv）。解析/长度/跨变体失配 → 空串即 DataError。
pub fn crypto_subtle_mldsa_import(name: &str, fmt: &str, der_csv: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (var, pk_len) = mldsa_params(v);
    let alg_id = ml_alg_id_seq(0x03, var);
    let der = bytes_from_csv(der_csv);
    match fmt {
        "spki" => match ml_spki_parse(&der, &alg_id, pk_len) {
            Some(k) => bytes_to_csv(&k),
            None => String::new(),
        },
        "raw-public" => {
            if der.len() != pk_len {
                String::new()
            } else {
                bytes_to_csv(&der)
            }
        }
        "pkcs8" => match ml_pkcs8_seed_parse(&der, &alg_id, 32) {
            Some(s) => bytes_to_csv(&s),
            None => String::new(),
        },
        "raw-seed" => {
            if der.len() != 32 {
                String::new()
            } else {
                bytes_to_csv(&der)
            }
        }
        _ => String::new(),
    }
}

/// ML-DSA 导出：输入 pk csv（公）或种子 csv（私，按长度判别）。fmt "spki"/
/// "raw-public"（公透传或私派生公）、"pkcs8"/"raw-seed"（仅私）。错误 → 空串。
pub fn crypto_subtle_mldsa_export_der(name: &str, raw_csv: &str, fmt: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (var, pk_len) = mldsa_params(v);
    let alg_id = ml_alg_id_seq(0x03, var);
    let stored = bytes_from_csv(raw_csv);
    let (pk, seed) = if stored.len() == 32 {
        let derived = crypto_subtle_mldsa_public_from_seed(name, raw_csv);
        if derived.is_empty() {
            return String::new();
        }
        (bytes_from_csv(&derived), Some(stored))
    } else if stored.len() == pk_len {
        (stored, None)
    } else {
        return String::new();
    };
    match fmt {
        "spki" => bytes_to_csv(&ml_spki_encode(&alg_id, &pk)),
        "raw-public" => bytes_to_csv(&pk),
        "pkcs8" | "raw-seed" => match seed {
            Some(s) if fmt == "pkcs8" => bytes_to_csv(&ml_pkcs8_seed_encode(&alg_id, &s)),
            Some(s) => bytes_to_csv(&s),
            None => String::new(),
        },
        _ => String::new(),
    }
}

/// ML-DSA jwk 分量导入：pub 必填 raw pk csv；priv 非空 → 私钥且须与 pub 配对
///（种子派生公钥 ≠ pub → 空串即 DataError，spec Validate MpJwk 配对检查）。
/// 输出公 = pk csv / 私 = 种子 csv（jwk 对象在 shim 构造）。
/// https://w3c.github.io/webcrypto/#dfn-Validate-MpJwk
pub fn crypto_subtle_mldsa_jwk_import(name: &str, pub_csv: &str, priv_csv: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let (_var, pk_len) = mldsa_params(v);
    let pk = bytes_from_csv(pub_csv);
    if pk.len() != pk_len {
        return String::new();
    }
    let priv_seed = bytes_from_csv(priv_csv);
    if priv_seed.is_empty() {
        return bytes_to_csv(&pk);
    }
    if priv_seed.len() != 32 {
        return String::new();
    }
    let derived = crypto_subtle_mldsa_public_from_seed(name, priv_csv);
    // csv 编码确定，字符串相等即字节相等。
    if derived.is_empty() || derived != bytes_to_csv(&pk) {
        return String::new();
    }
    bytes_to_csv(&priv_seed)
}

/// ML-DSA 私派生公（getPublicKey / jwk 配对校验用）：种子 csv → pk csv。
pub fn crypto_subtle_mldsa_public_from_seed(name: &str, seed_csv: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let seed = bytes_from_csv(seed_csv);
    if seed.len() != 32 {
        return String::new();
    }
    let seed_arr = match ml_dsa::Seed::try_from(&seed[..]) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    macro_rules! derive {
        ($p:ty) => {{
            let sk = ml_dsa::SigningKey::<$p>::from_seed(&seed_arr);
            bytes_to_csv(sk.expanded_key().verifying_key().encode().as_slice())
        }};
    }
    match v {
        MlDsaVariant::V44 => derive!(ml_dsa::MlDsa44),
        MlDsaVariant::V65 => derive!(ml_dsa::MlDsa65),
        MlDsaVariant::V87 => derive!(ml_dsa::MlDsa87),
    }
}

/// ML-DSA 签名：种子 csv + msg csv + ctx csv（上下文 ≤255 字节由 shim 校验，
/// 超限 crate 侧亦拒）→ 签名 csv（FIPS 204 sigEncode 定宽）。失败 → 空串。
pub fn crypto_subtle_mldsa_sign(name: &str, seed_csv: &str, msg_csv: &str, ctx_csv: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let seed = bytes_from_csv(seed_csv);
    let msg = bytes_from_csv(msg_csv);
    let ctx = bytes_from_csv(ctx_csv);
    if seed.len() != 32 {
        return String::new();
    }
    let seed_arr = match ml_dsa::Seed::try_from(&seed[..]) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };
    macro_rules! sign {
        ($p:ty) => {{
            let sk = ml_dsa::SigningKey::<$p>::from_seed(&seed_arr);
            match sk.expanded_key().sign_deterministic(&msg, &ctx) {
                Ok(sig) => bytes_to_csv(sig.encode().as_slice()),
                Err(_) => String::new(),
            }
        }};
    }
    match v {
        MlDsaVariant::V44 => sign!(ml_dsa::MlDsa44),
        MlDsaVariant::V65 => sign!(ml_dsa::MlDsa65),
        MlDsaVariant::V87 => sign!(ml_dsa::MlDsa87),
    }
}

/// ML-DSA 验签：pk csv + msg csv + sig csv + ctx csv。pk/sig 长度错、sigDecode
/// 范数拒绝（spec 无 reject 路径）、验签失败 → "0"。返 "1"/"0"。
pub fn crypto_subtle_mldsa_verify(name: &str, pk_csv: &str, msg_csv: &str, sig_csv: &str, ctx_csv: &str) -> String {
    let v = match parse_mldsa_variant(name) {
        Some(v) => v,
        None => return "0".into(),
    };
    let (_var, pk_len) = mldsa_params(v);
    let pk = bytes_from_csv(pk_csv);
    let msg = bytes_from_csv(msg_csv);
    let sig = bytes_from_csv(sig_csv);
    let ctx = bytes_from_csv(ctx_csv);
    if pk.len() != pk_len {
        return "0".into();
    }
    macro_rules! verify {
        ($p:ty) => {{
            let pk_arr = match ml_dsa::EncodedVerifyingKey::<$p>::try_from(&pk[..]) {
                Ok(a) => a,
                Err(_) => return "0".into(),
            };
            let vk = ml_dsa::VerifyingKey::<$p>::decode(&pk_arr);
            // Signature::try_from 含长度与 sigDecode（z 无穷范数/hint 解包）双检
            //——5 字节短签名等畸形输入在此拒绝，verify 返 false 不抛。
            let s = match ml_dsa::Signature::<$p>::try_from(&sig[..]) {
                Ok(s) => s,
                Err(_) => return "0".into(),
            };
            if vk.verify_with_context(&msg, &ctx, &s) {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }};
    }
    match v {
        MlDsaVariant::V44 => verify!(ml_dsa::MlDsa44),
        MlDsaVariant::V65 => verify!(ml_dsa::MlDsa65),
        MlDsaVariant::V87 => verify!(ml_dsa::MlDsa87),
    }
}

#[cfg(test)]
mod okp_x448_tests {
    use super::*;

    /// WPT cfrg_curves_bits_fixtures.js X448 字节原样落仓（语料同源）。
    const FIXTURE_PKCS8: [u8; 72] = [
        48, 70, 2, 1, 0, 48, 5, 6, 3, 43, 101, 111, 4, 58, 4, 56, 88, 199, 210, 154, 62, 181, 25, 178, 157, 0, 207,
        177, 145, 187, 100, 252, 109, 138, 66, 216, 241, 113, 118, 39, 43, 137, 242, 39, 45, 24, 25, 41, 92, 101, 37,
        192, 130, 150, 113, 176, 82, 239, 7, 39, 83, 15, 24, 142, 49, 208, 204, 83, 191, 38, 146, 158,
    ];
    const FIXTURE_SPKI: [u8; 68] = [
        48, 66, 48, 5, 6, 3, 43, 101, 111, 3, 57, 0, 182, 4, 161, 209, 165, 205, 29, 148, 38, 213, 97, 239, 99, 10,
        158, 177, 108, 190, 105, 213, 185, 202, 97, 94, 220, 83, 99, 62, 251, 82, 234, 49, 230, 230, 160, 161, 219,
        172, 198, 231, 108, 188, 230, 72, 45, 126, 75, 163, 213, 93, 158, 128, 39, 101, 206, 111,
    ];
    const FIXTURE_DERIVATION: [u8; 56] = [
        240, 246, 197, 241, 127, 148, 244, 41, 30, 171, 113, 120, 134, 109, 55, 236, 137, 6, 221, 108, 81, 65, 67, 220,
        133, 190, 124, 242, 141, 239, 243, 155, 114, 110, 15, 109, 207, 129, 14, 181, 148, 220, 169, 123, 72, 130, 189,
        68, 196, 62, 167, 220, 103, 244, 154, 78,
    ];
    /// 5 小阶点 spki（order 0/1/p-1/p/p+1；cfrg_curves_bits_fixtures.js kSmallOrderPoint）。
    const SMALL_ORDER_SPKIS: [[u8; 68]; 5] = [
        // order 0
        [
            48, 66, 48, 5, 6, 3, 43, 101, 111, 3, 57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // order 1
        [
            48, 66, 48, 5, 6, 3, 43, 101, 111, 3, 57, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        // p-1 (order 2)
        [
            48, 66, 48, 5, 6, 3, 43, 101, 111, 3, 57, 0, 254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255,
        ],
        // p (=0, order 4)
        [
            48, 66, 48, 5, 6, 3, 43, 101, 111, 3, 57, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255,
        ],
        // p+1 (=1, order 1)
        [
            48, 66, 48, 5, 6, 3, 43, 101, 111, 3, 57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ],
    ];

    fn csv(bytes: &[u8]) -> String {
        bytes_to_csv(bytes)
    }

    #[test]
    fn fixture_derivation_kat() {
        // 语料 "good" 面：pkcs8 私钥 × spki 公钥 = 56B derivation fixture 逐字节。
        let priv_csv = csv(&FIXTURE_PKCS8);
        let pub_csv = csv(&FIXTURE_SPKI[12..]);
        assert_eq!(crypto_subtle_x448(&priv_csv, &pub_csv), csv(&FIXTURE_DERIVATION));
    }

    #[test]
    fn small_order_import_succeeds_derive_rejected() {
        // 义务①：小阶点 importKey（spki）必须成功——host 不做贡献性检查。
        for v in &SMALL_ORDER_SPKIS {
            let imported = crypto_subtle_okp_import("X448", "spki", &csv(v));
            assert_eq!(imported, csv(&v[12..]), "small-order point import must succeed");
        }
        // 义务②：5 小阶点 deriveBits → 全部拒绝（order 0/1/2 → None；p/p+1 →
        // Some 全零 → all-zero 判据），输出空串 ≡ OperationError。
        let priv_csv = csv(&FIXTURE_PKCS8);
        for v in &SMALL_ORDER_SPKIS {
            assert_eq!(
                crypto_subtle_x448(&priv_csv, &csv(&v[12..])),
                "",
                "small-order point derive must be rejected"
            );
        }
    }

    #[test]
    fn pkcs8_import_preserves_unclamped_scalar() {
        // 导入存原标量不钳位（义务①的存储面）：fixture 标量首字节 0x58（钳位形）
        // 改 0xD8（高两位 11，非钳位形）→ 导入后 pkcs8 原样保留，不回写钳位。
        let mut unclamped = FIXTURE_PKCS8;
        unclamped[16] = 0xd8;
        let imported = crypto_subtle_okp_import("X448", "pkcs8", &csv(&unclamped));
        assert_eq!(imported, csv(&unclamped));
    }

    #[test]
    fn generate_produces_wellformed_pair() {
        let out = crypto_subtle_okp_generate("X448");
        let (pub_csv, pkcs8_csv) = out.split_once(';').expect("generate output shape");
        let pub_raw: Vec<u8> = pub_csv.split(',').map(|s| s.parse().unwrap()).collect();
        let pkcs8: Vec<u8> = pkcs8_csv.split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(pub_raw.len(), 56);
        assert_eq!(pkcs8.len(), 72);
        assert_eq!(&pkcs8[..16], &OKP_PKCS8_X448);
        // 生成标量为钳位形（Secret::from 内建 RFC 7748 §5 钳位，小端约定）：
        // 首字节低 2 位清零（&= 252）、末字节最高位置位（|= 128）。
        assert_eq!(pkcs8[16] & 0b0000_0011, 0);
        assert_eq!(pkcs8[71] & 0b1000_0000, 0b1000_0000);
        // 生成对自派生非零可用。
        assert!(!crypto_subtle_x448(pkcs8_csv, pub_csv).is_empty());
    }
}
