// Hybrid PQ/T KEM（webcrypto-compat-v3 d1s5 域切片）：MLKEM768-P256 /
// MLKEM768-X25519 / MLKEM1024-P384，依 draft-irtf-cfrg-concrete-hybrid-kems-03
// §4（CG 框架，见 draft-irtf-cfrg-hybrid-kems-09）：
//   ExpandSeed(seed)  = SHAKE256(seed, Nseed_T + 64) = seed_T || ML-KEM seed(d||z)
//   KeyGen(seed)      = (ek_PQ || pk_T, seed)；PQ/T 侧标量均自 seed_T 确定性展开
//   RandomScalar      = P-256/P-384 依次以 Nscalar 字节大端整数落入 [1, n-1] 拒
//                        绝采样；X25519 恒等（RFC 7748 钳位在运算内建）
//   Exp(x, y)         = X(y·(x·G))，P 系输出 X 坐标（Nss 字节）、X25519 恒等
//   ct                = ct_PQ || ct_T（ct_T = SEC1 未压缩 ephemeral pk / X25519 32B）
//   Combiner          = SHA3-256(ss_PQ || ss_T || ct_T || ek_T || label)（CG
//                        C2PRI，含 ek_T 不含 ct_PQ——与 X-Wing 式不同），label
//                        = "\.//^\"（X25519，同 X-Wing label）或算法名（P-256/P-384）
// 请求名字经 crypto_asym.rs 的 ML-KEM 回调透传分流至此；存储契约与 ML-KEM 同构：
// private _raw = 32B seed、public _raw = ek 拼接（csv 桥，密钥材料零落日志）。
// spki/pkcs8 的算法 OID 用 experimental 弧 1.3.6.1.3.<tail>：draft 尚无 IANA
// X.509 OID 注册（FIXME: 上游注册后切换正式 OID；本弧仅限本实现内部往返，
// 语料 fixtures 不含 hybrid spki/pkcs8 字节，无互操作承诺）。
// https://w3c.github.io/webcrypto/#ml-kem（usages/格式面同 ML-KEM 域）
// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-concrete-hybrid-kems-03

use super::crypto::{bytes_from_csv, bytes_to_csv};
use super::crypto_asym::{ml_pkcs8_seed_encode, ml_pkcs8_seed_parse, ml_spki_encode, ml_spki_parse};
use sha3::{Digest, Sha3_256};
use shake::Shake256;
use shake::digest::{ExtendableOutput, Update, XofReader};

/// 三组合（名字即规范拼写，shim 大小写归一后落此处）。
#[derive(Clone, Copy, PartialEq)]
enum HybridVariant {
    M768P256,
    M768X25519,
    M1024P384,
}

fn parse_hybrid_variant(name: &str) -> Option<HybridVariant> {
    match name {
        "MLKEM768-P256" => Some(HybridVariant::M768P256),
        "MLKEM768-X25519" => Some(HybridVariant::M768X25519),
        "MLKEM1024-P384" => Some(HybridVariant::M1024P384),
        _ => None,
    }
}

/// shim 家族判定探测入口（crypto_asym 六回调首行分流用）。
pub(crate) fn is_hybrid(name: &str) -> bool {
    parse_hybrid_variant(name).is_some()
}

/// 组合常量（draft §4.3/§4.4/§4.5；ct_T 长度 = 曲线 pk 编码长 = ek_T）。
struct HybridParams {
    ml1024: bool,
    /// ML-KEM 侧 ek / ct 长度。
    ek_pq: usize,
    ct_pq: usize,
    /// 传统侧 pk/ss 字节数（X25519 32；P-256 32；P-384 48）。
    ss_t: usize,
    /// 传统侧 pk 编码长（SEC1 未压缩 = 1 + 2·坐标长）。
    ek_t: usize,
    /// ExpandSeed 传统侧 seed 字节数（RandomScalar 采样池）。
    nseed_t: usize,
    /// 标量字节长（拒绝采样块长）。
    nscalar: usize,
    label: &'static [u8],
    /// experimental 弧 AlgId：SEQ { OID 1.3.6.1.3.<tail> }。
    alg_id: [u8; 9],
}

fn hybrid_params(v: HybridVariant) -> HybridParams {
    match v {
        HybridVariant::M768P256 => HybridParams {
            ml1024: false,
            ek_pq: 1184,
            ct_pq: 1088,
            ss_t: 32,
            ek_t: 65,
            nseed_t: 128,
            nscalar: 32,
            label: b"MLKEM768-P256",
            alg_id: [0x30, 0x07, 0x06, 0x03, 0x2B, 0x06, 0x01, 0x03, 0x79],
        },
        HybridVariant::M768X25519 => HybridParams {
            ml1024: false,
            ek_pq: 1184,
            ct_pq: 1088,
            ss_t: 32,
            ek_t: 32,
            nseed_t: 32,
            nscalar: 32,
            // X-Wing label（RFC 7748 风格 6 字节，draft §4.4）。
            label: b"\\.//^\\",
            alg_id: [0x30, 0x07, 0x06, 0x03, 0x2B, 0x06, 0x01, 0x03, 0x7A],
        },
        HybridVariant::M1024P384 => HybridParams {
            ml1024: true,
            ek_pq: 1568,
            ct_pq: 1568,
            ss_t: 48,
            ek_t: 97,
            nseed_t: 48,
            nscalar: 48,
            label: b"MLKEM1024-P384",
            alg_id: [0x30, 0x07, 0x06, 0x03, 0x2B, 0x06, 0x01, 0x03, 0x7B],
        },
    }
}

/// 传统侧标量：P 系存 SecretKey（from_bytes 已拒绝 0 与 ≥n）；X25519 存原
/// 32B（钳位在 dalek 运算内建，导入侧不做——OKP 域同先例）。
enum CurveSk {
    X25519([u8; 32]),
    P256(p256::SecretKey),
    P384(p384::SecretKey),
}

/// Combiner（CG 框架 C2PRI）：SHA3-256(ss_PQ || ss_T || ct_T || ek_T || label)。
/// 注意与 X-Wing 不同：含接收方公钥 ek_T、不含 ct_PQ
/// （draft-irtf-cfrg-hybrid-kems-09 §5.1.3/§5.5，向量已逐字节核验）。
fn combiner(ss_pq: &[u8], ss_t: &[u8], ct_t: &[u8], ek_t: &[u8], label: &[u8]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    Digest::update(&mut h, ss_pq);
    Digest::update(&mut h, ss_t);
    Digest::update(&mut h, ct_t);
    Digest::update(&mut h, ek_t);
    Digest::update(&mut h, label);
    h.finalize().into()
}

/// ExpandSeed：SHAKE256(seed, Nseed_T + 64)（draft §4.1 PRG）。
fn expand_seed(seed: &[u8; 32], nseed_t: usize) -> Vec<u8> {
    let mut h = Shake256::default();
    h.update(seed);
    let mut r = h.finalize_xof();
    let mut out = vec![0u8; nseed_t + 64];
    r.read(&mut out);
    out
}

/// ML-KEM ek：seed(d||z) → from_seed → encapsulation_key bytes。失败 → None。
fn mlkem_ek_bytes(seed_pq: &[u8; 64], ml1024: bool) -> Option<Vec<u8>> {
    use ml_kem::kem::KeyExport;
    let arr = ml_kem::Seed::try_from(&seed_pq[..]).ok()?;
    let ek: Vec<u8> = match ml1024 {
        false => ml_kem::DecapsulationKey::<ml_kem::MlKem768>::from_seed(arr)
            .encapsulation_key()
            .to_bytes()
            .as_slice()
            .to_vec(),
        true => ml_kem::DecapsulationKey::<ml_kem::MlKem1024>::from_seed(arr)
            .encapsulation_key()
            .to_bytes()
            .as_slice()
            .to_vec(),
    };
    Some(ek)
}

/// RandomScalar(seed_T)（draft §4.2）：P 系按 nscalar 字节大端块依次拒绝采样
///（ScalarPrimitive::from_bytes 内建 [1, n-1] 判据）；X25519 恒等取首块。
fn curve_scalar_from_seed(seed_t: &[u8], p: &HybridParams) -> Option<CurveSk> {
    if seed_t.len() != p.nseed_t {
        return None;
    }
    match (p.nseed_t, p.ss_t) {
        // X25519：恒等（nseed_t = 32）。
        (32, 32) => Some(CurveSk::X25519(seed_t.try_into().ok()?)),
        // P-256：128B 池 = 4 个 32B 块。
        (128, 32) => {
            for i in 0..4 {
                let mut fb = p256::elliptic_curve::FieldBytes::<p256::NistP256>::default();
                fb.clone_from_slice(&seed_t[i * 32..(i + 1) * 32]);
                let cp = p256::elliptic_curve::ScalarPrimitive::<p256::NistP256>::from_bytes(&fb);
                if bool::from(cp.is_some()) {
                    return Some(CurveSk::P256(p256::SecretKey::new(cp.unwrap())));
                }
            }
            None
        }
        // P-384：48B 池 = 1 个 48B 块。
        (48, 48) => {
            let mut fb = p384::elliptic_curve::FieldBytes::<p384::NistP384>::default();
            fb.clone_from_slice(seed_t);
            let cp = p384::elliptic_curve::ScalarPrimitive::<p384::NistP384>::from_bytes(&fb);
            if bool::from(cp.is_some()) {
                Some(CurveSk::P384(p384::SecretKey::new(cp.unwrap())))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 曲线 pk 编码（SEC1 未压缩 / X25519 u 坐标 32B）。
fn curve_pk_bytes(sk: &CurveSk) -> Vec<u8> {
    // p256/p384 均 re-export elliptic_curve::sec1，同一 ToEncodedPoint trait。
    use p256::elliptic_curve::sec1::ToEncodedPoint;
    match sk {
        CurveSk::X25519(s) => {
            let ss = x25519_dalek::StaticSecret::from(*s);
            x25519_dalek::PublicKey::from(&ss).as_bytes().to_vec()
        }
        CurveSk::P256(k) => k.public_key().to_encoded_point(false).as_bytes().to_vec(),
        CurveSk::P384(k) => k.public_key().to_encoded_point(false).as_bytes().to_vec(),
    }
}

/// 曲线侧对端 pk 解码（encap/decap 共用；SEC1 非法点在此拒绝，X25519 任意
/// 32B 可用——RFC 7748 无点概念，小阶点不拒即 OKP 域同先例）。
fn curve_pk_decode(bytes: &[u8], p: &HybridParams) -> Option<()> {
    if bytes.len() != p.ek_t {
        return None;
    }
    match p.ss_t {
        48 => p384::PublicKey::from_sec1_bytes(bytes).ok().map(|_| ()),
        32 if p.ek_t == 65 => p256::PublicKey::from_sec1_bytes(bytes).ok().map(|_| ()),
        32 => Some(()),
        _ => None,
    }
}

/// Exp(sk, peer_pk) = X(peer_pk·sk·G)：P 系 X 坐标 / X25519 raw 输出。
fn curve_exp(sk: &CurveSk, peer: &[u8]) -> Option<Vec<u8>> {
    match sk {
        CurveSk::X25519(s) => {
            if peer.len() != 32 {
                return None;
            }
            let peer_arr: [u8; 32] = peer.try_into().ok()?;
            let ss = x25519_dalek::StaticSecret::from(*s).diffie_hellman(&x25519_dalek::PublicKey::from(peer_arr));
            Some(ss.as_bytes().to_vec())
        }
        CurveSk::P256(k) => {
            let pk = p256::PublicKey::from_sec1_bytes(peer).ok()?;
            Some(
                p256::elliptic_curve::ecdh::diffie_hellman(&k.to_nonzero_scalar(), pk.as_affine())
                    .raw_secret_bytes()
                    .to_vec(),
            )
        }
        CurveSk::P384(k) => {
            let pk = p384::PublicKey::from_sec1_bytes(peer).ok()?;
            Some(
                p384::elliptic_curve::ecdh::diffie_hellman(&k.to_nonzero_scalar(), pk.as_affine())
                    .raw_secret_bytes()
                    .to_vec(),
            )
        }
    }
}

/// OS 随机 ephemeral 标量（encap 传统侧）：getrandom 后同拒绝采样判据。
fn curve_ephemeral(p: &HybridParams) -> Option<CurveSk> {
    let mut buf = vec![0u8; p.nscalar];
    for _ in 0..255 {
        getrandom::fill(&mut buf).ok()?;
        match p.ss_t {
            48 => {
                let mut fb = p384::elliptic_curve::FieldBytes::<p384::NistP384>::default();
                fb.clone_from_slice(&buf);
                let cp = p384::elliptic_curve::ScalarPrimitive::<p384::NistP384>::from_bytes(&fb);
                if bool::from(cp.is_some()) {
                    return Some(CurveSk::P384(p384::SecretKey::new(cp.unwrap())));
                }
            }
            32 if p.ek_t == 65 => {
                let mut fb = p256::elliptic_curve::FieldBytes::<p256::NistP256>::default();
                fb.clone_from_slice(&buf);
                let cp = p256::elliptic_curve::ScalarPrimitive::<p256::NistP256>::from_bytes(&fb);
                if bool::from(cp.is_some()) {
                    return Some(CurveSk::P256(p256::SecretKey::new(cp.unwrap())));
                }
            }
            32 => return Some(CurveSk::X25519(buf.clone().try_into().ok()?)),
            _ => return None,
        }
    }
    None
}

/// seed → (完整 ek 拼接, 曲线标量, ML-KEM seed)——keygen/公私派生/decap 共用。
/// ExpandSeed 拆分序 = seed_PQ ‖ seed_T（draft §4.1：ML-KEM seed 在前）。
fn hybrid_from_seed(seed: &[u8; 32], p: &HybridParams) -> Option<(Vec<u8>, CurveSk, [u8; 64])> {
    let out = expand_seed(seed, p.nseed_t);
    let mut seed_pq = [0u8; 64];
    seed_pq.copy_from_slice(&out[..64]);
    let sk = curve_scalar_from_seed(&out[64..], p)?;
    let ek_pq = mlkem_ek_bytes(&seed_pq, p.ml1024)?;
    let mut ek = ek_pq;
    ek.extend_from_slice(&curve_pk_bytes(&sk));
    Some((ek, sk, seed_pq))
}

/// 生成：32B 随机 seed（draft KeyGen）→ "ek csv;seed csv"。失败 → 空串。
pub(crate) fn hybrid_generate(name: &str) -> String {
    let v = match parse_hybrid_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let p = hybrid_params(v);
    let mut seed = [0u8; 32];
    if getrandom::fill(&mut seed).is_err() {
        return String::new();
    }
    match hybrid_from_seed(&seed, &p) {
        Some((ek, _, _)) => format!("{};{}", bytes_to_csv(&ek), bytes_to_csv(&seed)),
        None => String::new(),
    }
}

/// 私派生公（getPublicKey / jwk 配对 / 公钥侧导出）：seed csv → ek csv。
pub(crate) fn hybrid_public_from_seed(name: &str, seed_csv: &str) -> String {
    let v = match parse_hybrid_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let p = hybrid_params(v);
    let seed = bytes_from_csv(seed_csv);
    if seed.len() != 32 {
        return String::new();
    }
    let seed: [u8; 32] = match seed.try_into() {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    match hybrid_from_seed(&seed, &p) {
        Some((ek, _, _)) => bytes_to_csv(&ek),
        None => String::new(),
    }
}

/// 导入（spki/raw-public 公 → ek csv；pkcs8/raw-seed 私 → 32B seed csv）。
/// 解析/长度/非法曲线点 → 空串即 DataError。
pub(crate) fn hybrid_import(name: &str, fmt: &str, der_csv: &str) -> String {
    let v = match parse_hybrid_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let p = hybrid_params(v);
    let ek_total = p.ek_pq + p.ek_t;
    let der = bytes_from_csv(der_csv);
    match fmt {
        "spki" => match ml_spki_parse(&der, &p.alg_id, ek_total) {
            Some(k) => bytes_to_csv(&k),
            None => String::new(),
        },
        "raw-public" => {
            if der.len() != ek_total || curve_pk_decode(&der[p.ek_pq..], &p).is_none() {
                String::new()
            } else {
                bytes_to_csv(&der)
            }
        }
        "pkcs8" => match ml_pkcs8_seed_parse(&der, &p.alg_id, 32) {
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

/// 导出：ek csv（公）或 seed csv（私，按长度判别）。spki/raw-public（公透传
/// 或私派生公）、pkcs8/raw-seed（仅私）。错误 → 空串即 InvalidAccessError。
pub(crate) fn hybrid_export_der(name: &str, raw_csv: &str, fmt: &str) -> String {
    let v = match parse_hybrid_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let p = hybrid_params(v);
    let ek_total = p.ek_pq + p.ek_t;
    let stored = bytes_from_csv(raw_csv);
    let (ek, seed) = if stored.len() == 32 {
        let seed_arr = match <[u8; 32]>::try_from(&stored[..]) {
            Ok(a) => a,
            Err(_) => return String::new(),
        };
        match hybrid_from_seed(&seed_arr, &p) {
            Some((ek, _, _)) => (ek, Some(stored)),
            None => return String::new(),
        }
    } else if stored.len() == ek_total {
        (stored, None)
    } else {
        return String::new();
    };
    match fmt {
        "spki" => bytes_to_csv(&ml_spki_encode(&p.alg_id, &ek)),
        "raw-public" => bytes_to_csv(&ek),
        "pkcs8" | "raw-seed" => match seed {
            Some(s) if fmt == "pkcs8" => bytes_to_csv(&ml_pkcs8_seed_encode(&p.alg_id, &s)),
            Some(s) => bytes_to_csv(&s),
            None => String::new(),
        },
        _ => String::new(),
    }
}

/// 封装：ek csv → "ct csv;ss csv"（ct = ct_PQ || ct_T）。ML-KEM 侧随机性由
/// ml-kem crate 内建 OS 源、传统侧 ephemeral 标量走 getrandom。ek 长度错/
/// 非法曲线点/随机源失败 → 空串即 OperationError。
pub(crate) fn hybrid_encapsulate(name: &str, ek_csv: &str) -> String {
    use ml_kem::kem::Encapsulate;
    let v = match parse_hybrid_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let p = hybrid_params(v);
    let ek = bytes_from_csv(ek_csv);
    if ek.len() != p.ek_pq + p.ek_t || curve_pk_decode(&ek[p.ek_pq..], &p).is_none() {
        return String::new();
    }
    let sk_e = match curve_ephemeral(&p) {
        Some(s) => s,
        None => return String::new(),
    };
    let ss_t = match curve_exp(&sk_e, &ek[p.ek_pq..]) {
        Some(s) => s,
        None => return String::new(),
    };
    let ct_t = curve_pk_bytes(&sk_e);
    macro_rules! enc {
        ($ty:ty) => {{
            let arr = match ml_kem::array::Array::try_from(&ek[..p.ek_pq]) {
                Ok(a) => a,
                Err(_) => return String::new(),
            };
            let kem = match ml_kem::EncapsulationKey::<$ty>::new(&arr) {
                Ok(k) => k,
                Err(_) => return String::new(),
            };
            let (ct, ss) = kem.encapsulate();
            (ct.as_slice().to_vec(), ss.as_slice().to_vec())
        }};
    }
    let (ct_pq, ss_pq) = if p.ml1024 {
        enc!(ml_kem::MlKem1024)
    } else {
        enc!(ml_kem::MlKem768)
    };
    let ss = combiner(&ss_pq, &ss_t, &ct_t, &ek[p.ek_pq..], p.label);
    let mut ct = ct_pq;
    ct.extend_from_slice(&ct_t);
    format!("{};{}", bytes_to_csv(&ct), bytes_to_csv(&ss))
}

/// 解封装：seed csv + ct csv → ss csv。ct 长度错/曲线解码失败 → 空串即
/// OperationError；ct_PQ 篡改走 ML-KEM 隐式拒绝（FIPS 203 §7.3），ss_T 为可
/// 预测值但 combiner 仍输出伪随机 ss（CG 框架安全语义）。
pub(crate) fn hybrid_decapsulate(name: &str, seed_csv: &str, ct_csv: &str) -> String {
    use ml_kem::kem::Decapsulate;
    let v = match parse_hybrid_variant(name) {
        Some(v) => v,
        None => return String::new(),
    };
    let p = hybrid_params(v);
    let seed = bytes_from_csv(seed_csv);
    let ct = bytes_from_csv(ct_csv);
    if seed.len() != 32 || ct.len() != p.ct_pq + p.ek_t {
        return String::new();
    }
    let seed: [u8; 32] = match seed.try_into() {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let (_, sk, seed_pq) = match hybrid_from_seed(&seed, &p) {
        Some(x) => x,
        None => return String::new(),
    };
    let ss_t = match curve_exp(&sk, &ct[p.ct_pq..]) {
        Some(s) => s,
        None => return String::new(),
    };
    macro_rules! dec {
        ($ty:ty) => {{
            let arr = match ml_kem::Seed::try_from(&seed_pq[..]) {
                Ok(a) => a,
                Err(_) => return String::new(),
            };
            let dk = ml_kem::DecapsulationKey::<$ty>::from_seed(arr);
            match dk.decapsulate_slice(&ct[..p.ct_pq]) {
                Ok(ss) => ss.as_slice().to_vec(),
                Err(_) => return String::new(),
            }
        }};
    }
    let ss_pq = if p.ml1024 {
        dec!(ml_kem::MlKem1024)
    } else {
        dec!(ml_kem::MlKem768)
    };
    let ek_t = curve_pk_bytes(&sk);
    let ss = combiner(&ss_pq, &ss_t, &ct[p.ct_pq..], &ek_t, p.label);
    bytes_to_csv(&ss)
}

#[cfg(test)]
mod hybrid_kem_tests {
    use super::*;

    fn csv_bytes(csv: &str) -> Vec<u8> {
        bytes_from_csv(csv)
    }

    // draft-irtf-cfrg-concrete-hybrid-kems-03 附录 A（语料 hybrid_kem_vectors.js
    // 同源）：seed = 32B 全零；hex 运行时解析（const fn 无法逐字节解码）。
    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    const PUBLIC_KEY_P256: &str = "3d209f716752f6408e7f89bceef97ac388530045377927644ef046c0a7cae978c8841a0133aac4f1e1a7027277f671219cf58b85d29c8fec08edd432e787a3cf9936fe0026a113cb9efb1d7214049527bfe2141ea170b0294a59403ab0ce16760a8baa95b823cbb8aacdcc17ef32775223c791e3740163941f9bb3f63346bef1c050c31f932c62719429aff14c2bd438ab135bed692d56c77c04cbbffd6335b578318b513771e84b14ea821262141ca006ccb8bf2500aa1008970f216fe7f1ae34125aa290492c069a189222adc322f97649c762c7d3128ad3bb2667971d0744014bc3b67445cbcd0b3e7ea69fb1cb9f9c331f97487920187292926d04a25a2650abbd44982bb0c3c6301fe6a61330d24d8a3c7021dc3e3392c79a139b37613bba67a2984298507b84a4d61eef18acfb979af2d39caa4c0db4513815359d76fc378c63a7f4f3053b17168d0221cf0c2eec5514ba235f81d04d67c3b5c518094917671c26a7c046457533cc32844581277a03eb065c4529a779a9a5878f2aac3f81db9ed3d8c9345697058cbb99d379bca16d8fdb61d129960390524791b9d3e501b900bd1e5002e095be06c23f1fb212f5801f24b6b28c0c5493d246d02aa29fa3acfbe15ac4e212eb0b6f69ebbea259a2703aa4c308224bdb741c65c7a5d4bff788279507bbfe513d7aa5694e7b3cdf62ab36432742d4a0ca9b3570ba742fa803b46989c8526ea586cc4fc32866143b79601725fa545fd280b404530318bbc3371194710b6d74beaa629eb18a36a953b75915ae96999ba5c88cdc56a46861c50032c9b630bcc1445a30878979bc55a2c0955bf399b231203b90c651b6afe0e242b5a543250b142f7291ed753d816098f7913302a8ce91641716623d4fc2ac6772aa5f3674042b7c4a18a2186289a4ac4e200774596ca03e6798c7506b984999db6ac142586bae0799f1e776f9f5247dc574d8556ddf9bbbc4ca3643263457f74248010d62d4311268360aecb4902b450bf2050ecb8ba7a92820d233f5a14ed31225a1d17ca6f19e825894cfb1807d922cbd60761134be419144bcf72006366a4460137ad9136c113f05eb54c409520edc72e4150cc3a24b0f819eec11bbd19ca9645b0810a60b4a8a9e9c3955396a1653955b047bcf4f98433c27236c570d75f809e44aaf2dc33665826351872c293350ab324518c8c0c80b521c80c81a56bdc968a5650315a830c8bb17532c62ccc23b1d46412c256b224fd4674491803501d0143125c7577239689965b6989ca561793c0f85c62a9e13487da17662a7188c70b1040a67ed4c3f85e74e3691822fb96314d6134fe6a626b3cbe1461d62a7b573b2cc75579ffa22967e36ceb2a1aa0b71875a22751d706b72ca9ecd0c8100ad0aa58009a5c83fffe91759e6baa0a9345af99fe3b69509dbc84032868844ab3f65bb1df8beadf36442e48e339c967023a525411544c789a2f04dacd06ffef78302210450b931f6b4c32aab34a3f5260b810f4c9a946fc22d3baabaa80ba8d9955d6dc35e8609b4256b482cdc9d8977c1a47a354e7c527fdb1672e166917b95cd6351820261daab361f8a2dcbb240c55abd6a8105e5291b427b566d731e6b7047189cff20d8b120e0b3e72472d1b0086812200fd3698e23f06e4f4e08bbb54cc2049c039c845be659999c8fa48d7f62327c146cf1bc0b0bb1b91b30174b7bc220d422023bff6b0dee263532c503f3982e4d3e27071b855578a9a9aa63b8a8c339bf";
    const SAMPLE_CT_P256: &str = "d81018a94f8078e02105beaa814e003390befa4589bb614f77397af42d8e8150796f2c88a4efca81b8cf93c0ae3716c54ec1b045e3875f38c2dd12d7f717bd7fb701a9fecda5ed8b764c9a35d4a5c1d8930f6071f653eebb2d1afa77debb8302d16f17e0f5f3920a71a4d49beafa0e1c7e443f8abca64a65a9e81a97e7357bf902573363c0e1a12e5228036828e3f759121fada92441fe334e85d79347e470d2fed945541d832c54baaa3cb7526c3853954db4f73547cc7c27fd38398bfa7704952cb841e38b270e4db7435f0ee22f57d7ad3270bd0c88e71b4b864cf2277c65daa10a6dad4c7abecd95cc4ebec39c08404b522e4ecc1545713f76bebd3b5a0f2feb3461936065dbd13f6a1f61e1b142a2af2e5a482ba2c50cf0317049c0b3bfd6d5e9240eba9111d2030fdea17e33b6524020d30b0c4f8069285f3a6ca267d287d01e827d8422bf5426e11688bfc73756af1841b1c87e126cb50c914b5b2b8673488ad3b074cad77a3840eb12dd688f313ee1e9ff8c479a678f276356fc9d65e1d5b4c1e9855b4175db144f7767c12061769190fe6b5e51563b91f94d131a2b796bd2980ed0dab4ae7a7110e920007a757158a5eb8662cbf89ddffe9d8196821313cdc00108853fc4746b111d5b56da638d8ed2973918960f5dfe93ead3ae521e957cec3c8d843e8fce234c70ad055177f235439d6098bdd771b1cfcfadaab4f50a7378185c62409f383c8ff658c2a2af66498cfd81e962766ac6b774e88424fb4f331837d0a28502708477caf8780a156d723f68fca791e1cd2397bfc2b24c77c765d9b2af36f732d52107517efd8157b283b440a613f756c364ca108971a8878199a93f260baec3e850033cc032c2e53f823576affb4d3b116e2d16049152c35aaa263ab376f0ad5ede6a749607a283e3016e62191c0e8fde33e718cd989591c9a205d608d99fcb8a7471603d716cb01b56328d7d880aec2851f4e6d8b5016c25647e9026ebb441543e8012dbfcf078d4012b8c39184dd64f3821b4774ae4e36365f8baf2bd1f6667c017a1e65ff8a1554458fb3f367c02721752bfa56fc7fd566ae95ffb208f919ef12f4cf8a2fdd141a8df559bddb7b8d1f04ee6d4cf7805d142989caf216dfae985faaab9974f6d9f8aa1129084db8db912b1655f595ffbaa66491ab4655fd734cfd4bb0c0289d4bcc8fc5e9943b351cb147c8db059a24004d1c3e3bb4c14a881e5101acb736c65c5d579acb67ee85a560277b43338fe79d34b772c5da001da3b5a3383dd81319a0b4542e6d7e46eed5314cc70eb231de27b6e760db598ba19995cf69be0e4458e35f3f274aca2455d43fe3344e183c6dc47c857dbe9907b41e41006d91b25adcafc098fe66f7554be8dad493c4f4b1dbf7a51464139db474afab5572f92a2232b59be56a72c0505149dae5cde1e602877037de7802b5f6fa47a4c9a3e52d6ca15339920254e9ffb53c7b834cc0288ed9905a1841e9390ea94a8898bd4c6b6d6027e4d43c7867242515bbeefe12340fc04428a824ea7cf56ad2a64ed368b71315d80cee846007cff1d2eea2c3f0f921537304ae598f98dd10d1f102811a4e2d161c3fd8bbb193d4b25bee950ac839c0f9d";
    const EXPECTED_SS_P256: &str = "9bd018e869bb01b63fb8f5da374a73d347ea14cb2bc570b13d0908e2288ec456";
    const PUBLIC_KEY_X25519: &str = "3d209f716752f6408e7f89bceef97ac388530045377927644ef046c0a7cae978c8841a0133aac4f1e1a7027277f671219cf58b85d29c8fec08edd432e787a3cf9936fe0026a113cb9efb1d7214049527bfe2141ea170b0294a59403ab0ce16760a8baa95b823cbb8aacdcc17ef32775223c791e3740163941f9bb3f63346bef1c050c31f932c62719429aff14c2bd438ab135bed692d56c77c04cbbffd6335b578318b513771e84b14ea821262141ca006ccb8bf2500aa1008970f216fe7f1ae34125aa290492c069a189222adc322f97649c762c7d3128ad3bb2667971d0744014bc3b67445cbcd0b3e7ea69fb1cb9f9c331f97487920187292926d04a25a2650abbd44982bb0c3c6301fe6a61330d24d8a3c7021dc3e3392c79a139b37613bba67a2984298507b84a4d61eef18acfb979af2d39caa4c0db4513815359d76fc378c63a7f4f3053b17168d0221cf0c2eec5514ba235f81d04d67c3b5c518094917671c26a7c046457533cc32844581277a03eb065c4529a779a9a5878f2aac3f81db9ed3d8c9345697058cbb99d379bca16d8fdb61d129960390524791b9d3e501b900bd1e5002e095be06c23f1fb212f5801f24b6b28c0c5493d246d02aa29fa3acfbe15ac4e212eb0b6f69ebbea259a2703aa4c308224bdb741c65c7a5d4bff788279507bbfe513d7aa5694e7b3cdf62ab36432742d4a0ca9b3570ba742fa803b46989c8526ea586cc4fc32866143b79601725fa545fd280b404530318bbc3371194710b6d74beaa629eb18a36a953b75915ae96999ba5c88cdc56a46861c50032c9b630bcc1445a30878979bc55a2c0955bf399b231203b90c651b6afe0e242b5a543250b142f7291ed753d816098f7913302a8ce91641716623d4fc2ac6772aa5f3674042b7c4a18a2186289a4ac4e200774596ca03e6798c7506b984999db6ac142586bae0799f1e776f9f5247dc574d8556ddf9bbbc4ca3643263457f74248010d62d4311268360aecb4902b450bf2050ecb8ba7a92820d233f5a14ed31225a1d17ca6f19e825894cfb1807d922cbd60761134be419144bcf72006366a4460137ad9136c113f05eb54c409520edc72e4150cc3a24b0f819eec11bbd19ca9645b0810a60b4a8a9e9c3955396a1653955b047bcf4f98433c27236c570d75f809e44aaf2dc33665826351872c293350ab324518c8c0c80b521c80c81a56bdc968a5650315a830c8bb17532c62ccc23b1d46412c256b224fd4674491803501d0143125c7577239689965b6989ca561793c0f85c62a9e13487da17662a7188c70b1040a67ed4c3f85e74e3691822fb96314d6134fe6a626b3cbe1461d62a7b573b2cc75579ffa22967e36ceb2a1aa0b71875a22751d706b72ca9ecd0c8100ad0aa58009a5c83fffe91759e6baa0a9345af99fe3b69509dbc84032868844ab3f65bb1df8beadf36442e48e339c967023a525411544c789a2f04dacd06ffef78302210450b931f6b4c32aab34a3f5260b810f4c9a946fc22d3baabaa80ba8d9955d6dc35e8609b4256b482cdc9d8977c1a47a354e7c527fdb1672e166917b95cd6351820261daab361f8a2dcbb240c55abd6a8105e5291b427b566d731e6b7047189cff20d8b120e0b3e72472d1b0086812200fd3698e23f06e4f4e08bbb54cc2f63601b7f85accfeea2d17964c66b5194b0f08e18519faaee194e3c102823062";
    const SAMPLE_CT_X25519: &str = "d81018a94f8078e02105beaa814e003390befa4589bb614f77397af42d8e8150796f2c88a4efca81b8cf93c0ae3716c54ec1b045e3875f38c2dd12d7f717bd7fb701a9fecda5ed8b764c9a35d4a5c1d8930f6071f653eebb2d1afa77debb8302d16f17e0f5f3920a71a4d49beafa0e1c7e443f8abca64a65a9e81a97e7357bf902573363c0e1a12e5228036828e3f759121fada92441fe334e85d79347e470d2fed945541d832c54baaa3cb7526c3853954db4f73547cc7c27fd38398bfa7704952cb841e38b270e4db7435f0ee22f57d7ad3270bd0c88e71b4b864cf2277c65daa10a6dad4c7abecd95cc4ebec39c08404b522e4ecc1545713f76bebd3b5a0f2feb3461936065dbd13f6a1f61e1b142a2af2e5a482ba2c50cf0317049c0b3bfd6d5e9240eba9111d2030fdea17e33b6524020d30b0c4f8069285f3a6ca267d287d01e827d8422bf5426e11688bfc73756af1841b1c87e126cb50c914b5b2b8673488ad3b074cad77a3840eb12dd688f313ee1e9ff8c479a678f276356fc9d65e1d5b4c1e9855b4175db144f7767c12061769190fe6b5e51563b91f94d131a2b796bd2980ed0dab4ae7a7110e920007a757158a5eb8662cbf89ddffe9d8196821313cdc00108853fc4746b111d5b56da638d8ed2973918960f5dfe93ead3ae521e957cec3c8d843e8fce234c70ad055177f235439d6098bdd771b1cfcfadaab4f50a7378185c62409f383c8ff658c2a2af66498cfd81e962766ac6b774e88424fb4f331837d0a28502708477caf8780a156d723f68fca791e1cd2397bfc2b24c77c765d9b2af36f732d52107517efd8157b283b440a613f756c364ca108971a8878199a93f260baec3e850033cc032c2e53f823576affb4d3b116e2d16049152c35aaa263ab376f0ad5ede6a749607a283e3016e62191c0e8fde33e718cd989591c9a205d608d99fcb8a7471603d716cb01b56328d7d880aec2851f4e6d8b5016c25647e9026ebb441543e8012dbfcf078d4012b8c39184dd64f3821b4774ae4e36365f8baf2bd1f6667c017a1e65ff8a1554458fb3f367c02721752bfa56fc7fd566ae95ffb208f919ef12f4cf8a2fdd141a8df559bddb7b8d1f04ee6d4cf7805d142989caf216dfae985faaab9974f6d9f8aa1129084db8db912b1655f595ffbaa66491ab4655fd734cfd4bb0c0289d4bcc8fc5e9943b351cb147c8db059a24004d1c3e3bb4c14a881e5101acb736c65c5d579acb67ee85a560277b43338fe79d34b772c5da001da3b5a3383dd81319a0b4542e6d7e46eed5314cc70eb231de27b6e760db598ba19995cf69be0e4458e35f3f274aca2455d43fe3344e183c6dc47c857dbe9907b41e41006d91b25adcafc098fe66f7554be8dad493c4f4b1dbf7a51464139db474afab5572f92a2232b59be56a72c0505149dae5cde1e602877037de7802b5f6fa47a4c9a3e52d6ca15339920254e9ffb53c7b834cc0288ed9905a1841e9390ea94a8898bd4c6b6d6027e4d43c7867242515bbeefe12340fc6b3d57762f8badb69433f9c6d060f85f5e5c6b6803a816d141c075f63541ad10";
    const EXPECTED_SS_X25519: &str = "e5ba94031ea6efd69c09c254f6d9783136ba6037e2d4c43bcccf19d6f3f4343a";
    const PUBLIC_KEY_P384: &str = "a10bc8b554cd51980cdbbccc3041420fd320fe8b74c7a84278c63c17070dc231b61ab269b9d677d920261186654b4571f51797d5c342b8070bc6c92bca16adecc631e4e94c7508b111730c749c73e2d6a6f97155cb269ccc06a71a21bef3d269463c935048a7f4636c7b320073709023f7b04d0530571a9a6f718280870bb63875d3f599bc229b95869cd5bb5d26640856d40b828198fdf2c099998ffdf772e462336c521cd326b5e4997bd95c135c57bd02c7afa80a2923d510951778ee5125b2aa18f90445453b85789224725b259279698ac9426c882baabc38d4fb3a3f6831180918b9825e0e418154d78aebab5e7e7066e69b2567476bf1177fe079a38298be6f01b098c33851ab25312b52e32a5750c2b73d293c0b810473b310aaf062f19914c7377b2e90388f575bf5e6853453b95a74aa18d62d4ae37e6996a48ab5217488a92d7b01e315c50b68204143792afc4f8367c0ce065ab32014bdb5515fe0594608aad1218994724afaaaa2df0355f46666b6e02a387b6d3da4713edb610bb048c3a2078b800e9ea483f2009c96d24c71b2cbc8e1200c0277383c5c27895e298c3607701ce58702a91903274a041408234cb0021ef2b1c5131419b444dc84b89d147d1fe43c43f676d906735d9ca2a59c2232d97fd4aa1ae2bb3d1b170ca553cb2574954fdc6689fac623cbaa31982d82424d5a564fef7a8ba51b44df15053b2b45bec4aa1ed49929123daf754175c5938258c608b24d062042ab4bbee5e553a5ea627521738ae5ab2e06bd98b020787b2f5fa51eb4c46c2bf90e55a49560340667f88ac41432b7f551dfd98c037c79f79b41b985a8b1f51345550cd816714362040778c43e378a288394bd028c8c31b5a904bc4a5648a596035cb38f0e276e12c9a96f8425056b05a136642dd2cb75463036485ba1a50539e420e1e31dfac529cad6c68ec06746749473e050a4ac92b7199beceb239b6c12c8e716b66607aeca64a5850b01f99d0b176a7759781ed77cb1ba40d17ac5c6cb06c942c002c2cf6efcb121f10ad2a45ff781426e7104cbdca73b81865ab22b00ba834355ae485a262f354248932c2be178369a3dd7e2428fdc379346ab2b754c43db657460cb09c5c48b5810cb7a5c6156cf87440c9e36a4869a8ac458b382fc178915a9ce1bcdda7c48807c207e656ffb80bf33e32bc8c7b20ef60572612ceac99ad1c56ce5a764b29b74c17a5b510b1afcb18a1afc35c12ac213725325f9b7a2eb338fe4c0080c31a58a995db7027d900e78544887f90ada467d0e383c119c5399310bc6735874e8804ff6c2bae57f2c3357cb627033c12a5924b20ce5abf113172bd2b77086cac543811793bba71734c9f005ac2656460bc30a442b388725758a623e37ba6e293abfb84f344229f373c214ca776a7c05adc465fed93b9cf77f0022ab71f1adde369dd8f420a58c057c14cc18dc47da7c12b086473eab419652967001c4e42a381c8ba539a875d21a9945133bab9bc1e53a600de77cbfb2aeab6b19ced4c6eaa8998ee6a1577255f7132d80a32d6c0c6ec44c9c4b28699a645bb0bc958e00275077925309519b0824c7000dfa61912ec049063a067d00b059053e508a5bfee63473869c8a8510af898cd7572854f5c38af96f5f97a7372632ea7bb4b6fb831c612af71191ff9806b379bcd43c6059b7b1f953741444af713c155d962722b947aa23a32a89b356a6a7508aad63968c1dea78ff18aac27a89aa7b42b0d7481dd3cc649421e51397782218ac5441760ba51a0328d66b436fec32d7aa4d68e0cad1bc14f7241c903480f809983fc2c30d93138cf63b59bc737ac08192893d039187a811bef3d3209eb7b8d1e05b5b251cef760a210b2732867ab32049ba3c354e3858aee7b71df792924730d8e842e484122b50677b0a306e61cf21b62091da18b937192936a09e5a418cf78b666157dd477af1c36a12320129522840e370941157808782a5335b0ac10d70e1beafd401074b84b9826cc58aad217bae0f419b2da896133272d8f22c6f420fcc738fccc1082fc93c7df0994c6bcf2cc8a29037b6bb2b4bcef4b0ee8caf8506bc5ecba082a56806c1cede0b944338a69a668254c1150ae05030e256b2b67661ba027d97576da613ac8c7c29051f1240b96b0c127e264d5e1dbbfe9561a567d5c9103673b446b3ccea6c5f7f34f09348a5d4a58b0498871dc940ee97b50c0336f9a60c3299f99560ac70657a27befa702265ce590583e04a28326092d3dea2118dd1df5e81d7d3014ec4b5ce67dcb45ef001769dd5d5ada76934d38d740924712bfae672169d8f8744c151346d285fbb653f83aa0f";
    const SAMPLE_CT_P384: &str = "dc63d18bb9715fb6e3ba71cb439fcd3377a75305cc9b144e6758bf5794a272e6b4a0da33234c0ac1bb5b4e60e4c82eb1fb780d59e4e4616641a0595ba031e3ae69d971dcd5fff14e21731a8e1a221f46c7820d214630b707fa1b0de3a484698f3d49e0a75f1212b8c42d330dd909f15eac0402f19ee77fba9447e1c44304b0d8c371c17c5549fdbdec1e0a2e7be9f577d7a4b5b2618d9ba67ab95a0297cd5c5a13c89cc5a57cbd9a8ae38d66455c9a3d2bc55b498775fee2f6dc224d376d5f526a8354c8ed724f60337e900b85627972383e1fd987d407a8834005814a4fdc94c947e5f3471459288cfb127952b3208f10c914200bbaac5fcebd2bc9e2848492bab17b9288ca8b81d1c2ac9522dcc0b6d5f51e10f3afbb5d65fbf919edef6323c4e92c6b0690c10db25a9182de9e919ea1b3e65ae6150635d5180ebd7d23a2264828bc3ee1fd34dba1924ad0db30c747e05baa9148f1a032769c685e04665fd802a79c4624f69a9198a426eac1b217d903cdacf8844e73365f3a219a700dda27edf6bea33602617c5fd105b301b884bfaaa1163b791ec09f82523fef65c87b75ed063ceb127729b82c8712e1f41b547d095f55ee71f3f8b47a306cb5d9bdd817854c74a42eebf934a1136dea3fbc546ad8ce51b3171913722f08b0261d197590342bfe4108dcb08c62a98610cbfb8d3b2831f56dcac2220e29a5811f38f0824f21a6cbebc64fd89a09b110dffbe03799ffc74fe565c80dbf6a66acd7bfd14cb90acba03405a7982d4c1c68caa75f8b72e4dd6401d7dce4db4f6b820a7886a604b66b4e5b9eea5e5eddc2bca458a25977bd1f02874c5d9daf2baf56b3040f24ce7fe14cc14d61c7960db4decb37d9779c8e36d69a7763066d8c1149312d26887a693dc222daa892dd00cd8f3a558cf605e4c65c011c2e9f0d671ba10af2bb90ee0351ae5078eb7878399ec9eb4ace87a68269618bda12a7aed6fda0385496c5d10ac36b35255f4a31edfa8a2c516b65c63431013ed4909ec7a787a5efb9d3c3887b80ac18a44934b6559bd8a84b18e86fa1b0b9e1d9f92ba495ba5595d82e5095612b79e805154bf428a7071662c7cefb6450165c6f8f6954c37219bff4a49894a8aa37f940a40f4ec942c281e6c47ea408199927a724ff1c7460fc8fd47a98d0c9d4d1f07994d8084f6e084935ad7c2985282fabd5ca13b942e10d35278f4ff4cb1cb96f3c862410e79144a46b4db1a3c3d4d63018ec5c01ca48cb67081482e7d434b4abe5fa3071f2fbb533f745602b0da6183b28e6c5dfa42dab7ae0bbbf7638e106be1bd7312cba399e08c96dbd69a128a2face2d4a02951533a25e82fe63d0aaaa2e8c75150215c93ab06c22f9cab8d1cae7424f8baa09b3260ecfa3c7c8d55a276b4b317f72ec86b1b145a63aca83ef8c1204d8ab0c96ea3f742de39db47020616e139285814f188029ace4587f14cf12b5ed81086d8213cf8cb578341e04e16f519b77ff4c2644a5732639d658d0c4eaf992bd7dbd5011b700a5fa63dc1b24a84a3c80656bab5705dc3a74312c80e8bdb24a7ac6e27bcb8c07ece62c6e5777dd3dc0657181f440c7524d907dd27950bcb252aef7f8cbf453cee3fe3143a665072c787cea76de323aa41537df2f3a40a518a694b918953bde8d57084e32d3b1fdcf9d153e73f02624beaf6ebe23e6828a6a489583494f3cd790fc96bb6f5d8b198402965e2e668e6581e7cf1c8a47a92198388f2b4cd38df660f0ddd48ad126819c4435af3a12c89113d778ac544fd8079cb8aaa97d2ff1b608da574c4dcd87f4979390de3be405f0e47788dd0b01662805079fd73c64e9278c036544add3694c838bfcfb08c8a5efb09549442123eaa59fa30fbb9198105f6be00163bac076193f6721c539714108bbfae167f5db8085c5838618f32a968bbb25c40645a17c17b9bec64aea45832eec5adc25b53e677f67566fbf5ce2d9193a06bd9b477e601d589b25f422defc49105252cd9ca6adcbb36be8a01a8472b4d463f655be14ccff9b0571a2048e31c14b9b23e2d43fafa3f85ece6fd41896cc5c68993dbaa926f285ec94c72887de9564881d735c05f83aa474b3d4cd133a630ac63850771cb5270f6cb7a391170d66af3e4901b6eb0253f3f34ef57d6babd97aa99ce718c3bcb53ff13d4028a0c943bb9681106ce176242cccb75df1d3f8d3706e5b068b042c3154d5e6292581b36499e6b069b9a490aa67f0675390539da8555e6a4e8a35a86fdfea83e1387bf4acc650ec1edae7c99aa3a48306ee1d1a5e513c0c6901f64d0a3ee285de3c11d49f90cd4323dafda14832f0d8b760c0e5a48633c967cfaf";
    const EXPECTED_SS_P384: &str = "8c028c6ea72a1c59408e2b15dd8fed8008517e861cd2329b159bda1919ea656c";

    const ZERO_SEED: [u8; 32] = [0u8; 32];

    /// ExpandSeed + 两层 keygen 对 draft 向量：X25519 seed_T 段 = draft
    /// decapsulation_key_t（恒等 RandomScalar 直接可见，位于 PRG 输出尾部），
    /// ML-KEM seed 段 = draft decapsulation_key_pq（输出头部），ek 逐字节 =
    /// 向量 publicKey（768 两组合共享 ML-KEM 段）。
    #[test]
    fn keygen_matches_draft_vectors() {
        // 拆分序 seed_PQ ‖ seed_T：头部 32B = decapsulation_key_pq（= ML-KEM d）。
        let out = expand_seed(&ZERO_SEED, 32);
        assert_eq!(
            &out[..32],
            &unhex("f5977c8283546a63723bc31d2619124f11db4658643336741df81757d5ad3062")[..],
            "ML-KEM seed must occupy the PRG output head"
        );
        assert_eq!(
            &out[64..96],
            &unhex("e00b3f9d338de90488973787b0916a4a9ae8bebf4e2bc07a7bc18f1a62215182")[..],
            "X25519 seed_T must equal draft decapsulation_key_t"
        );
        for (name, pk) in [
            ("MLKEM768-P256", PUBLIC_KEY_P256),
            ("MLKEM768-X25519", PUBLIC_KEY_X25519),
            ("MLKEM1024-P384", PUBLIC_KEY_P384),
        ] {
            let pk = &unhex(pk);
            let v = parse_hybrid_variant(name).unwrap();
            let p = hybrid_params(v);
            let (ek, _, _) = hybrid_from_seed(&ZERO_SEED, &p).expect("keygen from draft seed");
            assert_eq!(&ek[..], pk, "{name} full ek must match draft publicKey");
        }
    }

    /// decap KAT：seed 0^32 + sampleCiphertext → expectedSharedSecret 逐字节
    ///（钉死 ExpandSeed 顺序、随机标量、Exp、ct 切分与 combiner 全链）。
    #[test]
    fn decap_vectors_kat() {
        for (name, ct_hex, ss_hex) in [
            ("MLKEM768-P256", SAMPLE_CT_P256, EXPECTED_SS_P256),
            ("MLKEM768-X25519", SAMPLE_CT_X25519, EXPECTED_SS_X25519),
            ("MLKEM1024-P384", SAMPLE_CT_P384, EXPECTED_SS_P384),
        ] {
            let ct = unhex(ct_hex);
            let expected_ss = unhex(ss_hex);
            let seed_csv = bytes_to_csv(&ZERO_SEED);
            let out = hybrid_decapsulate(name, &seed_csv, &bytes_to_csv(&ct));
            assert_eq!(csv_bytes(&out), expected_ss, "{name} decapsulation KAT mismatch");
        }
    }

    /// 封装-解封装往返：encap(向量 pk) 的 ct 长度 == 向量 sampleCiphertext 长
    ///，decap 还原同一 ss；encapsulateBits 面 sharedKey = 32B。
    #[test]
    fn encap_decap_roundtrip() {
        for (name, pk_hex, ct_hex) in [
            ("MLKEM768-P256", PUBLIC_KEY_P256, SAMPLE_CT_P256),
            ("MLKEM768-X25519", PUBLIC_KEY_X25519, SAMPLE_CT_X25519),
            ("MLKEM1024-P384", PUBLIC_KEY_P384, SAMPLE_CT_P384),
        ] {
            let sample_ct = ct_hex.len() / 2;
            let out = hybrid_encapsulate(name, &bytes_to_csv(&unhex(pk_hex)));
            let parts: Vec<&str> = out.split(';').collect();
            assert_eq!(parts.len(), 2, "{name} encapsulate output shape");
            let ct = csv_bytes(parts[0]);
            let ss_enc = csv_bytes(parts[1]);
            assert_eq!(ct.len(), sample_ct, "{name} ciphertext length");
            assert_eq!(ss_enc.len(), 32, "{name} shared secret length");
            let ss_dec = hybrid_decapsulate(name, &bytes_to_csv(&ZERO_SEED), &bytes_to_csv(&ct));
            assert_eq!(csv_bytes(&ss_dec), ss_enc, "{name} encap/decap roundtrip");
        }
    }

    /// 格式往返：raw-public/raw-seed 原样；spki/pkcs8 编解码还原存储面；私派
    /// 生公与生成对一致；垃圾/长度错拒绝。
    #[test]
    fn import_export_formats() {
        for name in ["MLKEM768-P256", "MLKEM768-X25519", "MLKEM1024-P384"] {
            let pair = hybrid_generate(name);
            let (ek_csv, seed_csv) = pair.split_once(';').expect("generate shape");
            // raw-public（公透传）。
            assert_eq!(hybrid_import(name, "raw-public", ek_csv), ek_csv);
            assert_eq!(hybrid_export_der(name, ek_csv, "raw-public"), ek_csv);
            // raw-seed（私透传）+ 私派生公一致。
            assert_eq!(hybrid_import(name, "raw-seed", seed_csv), seed_csv);
            assert_eq!(hybrid_export_der(name, seed_csv, "raw-public"), ek_csv);
            assert_eq!(hybrid_public_from_seed(name, seed_csv), ek_csv);
            // spki/pkcs8 往返。
            let spki = hybrid_export_der(name, ek_csv, "spki");
            assert_eq!(hybrid_import(name, "spki", &spki), ek_csv);
            let pkcs8 = hybrid_export_der(name, seed_csv, "pkcs8");
            assert_eq!(hybrid_import(name, "pkcs8", &pkcs8), seed_csv);
            // 公钥不可导私钥格式（InvalidAccessError 面）。
            assert_eq!(hybrid_export_der(name, ek_csv, "pkcs8"), "");
            // 长度错拒绝。
            let short = &ek_csv[..ek_csv.len() - 4];
            assert_eq!(hybrid_import(name, "raw-public", short), "");
            assert_eq!(hybrid_import(name, "raw-seed", "1,2,3"), "");
        }
    }

    /// P 系 SEC1 非法点拒绝（raw-public 导入面 DataError）+ X25519 任意 32B
    /// 可导入（RFC 7748 无点语义）。
    #[test]
    fn import_validates_curve_point() {
        let ek_p256 = csv_bytes(&hybrid_public_from_seed("MLKEM768-P256", &bytes_to_csv(&[7u8; 32])));
        let mut bad = ek_p256.clone();
        bad[1184] ^= 0xff; // 破坏 SEC1 首字节 → 非法点。
        assert_eq!(hybrid_import("MLKEM768-P256", "raw-public", &bytes_to_csv(&bad)), "");
        assert_eq!(
            hybrid_import("MLKEM768-P256", "raw-public", &bytes_to_csv(&ek_p256)),
            bytes_to_csv(&ek_p256)
        );
        // X25519：任意 32B 传统段导入成功（无点校验义务）。
        let mut junk = vec![0u8; 1184];
        junk.extend_from_slice(&[9u8; 32]);
        assert_eq!(
            hybrid_import("MLKEM768-X25519", "raw-public", &bytes_to_csv(&junk)),
            bytes_to_csv(&junk)
        );
    }
}
