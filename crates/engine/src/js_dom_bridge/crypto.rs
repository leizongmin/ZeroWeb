//! WebCrypto host 实现（SubtleCrypto + CSPRNG）。从 js_dom_bridge.rs 拆出（R2973，文件大小治理）。
//! 纯字节级 crypto（无 DOM/CSS 依赖）：getrandom CSPRNG + SHA-1/256/384/512 digest/HMAC +
//! PBKDF2/HKDF 派生 + AES-GCM。pub 函数经 `pub use crypto::*` 重导出，register_dom_callbacks
//! 调用点零改动。

/// `crypto.getRandomValues` 的 OS 随机源（R2960）——`getrandom` crate（CSPRNG，Linux getrandom(2) /
/// macOS SecRandomCopyBytes / Windows BcryptGenRandom）。n 字节随机数 → 逗号分隔十进制串。
/// **getrandom 失败返空串**（shim 回退 Math.random——engine polyfill 路径亦走该回退）。
/// 供 `__zw_crypto_get_random_values` 回调 → shim `crypto.getRandomValues` / `randomUUID`。
pub fn crypto_random_bytes(n: usize) -> String {
    let mut buf = vec![0u8; n];
    if getrandom::fill(&mut buf).is_err() {
        return String::new();
    }
    buf.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// `crypto.subtle.digest(algo, data)`（R2793）——SHA-1/256/384/512 哈希（SRI / JWT / 内容哈希高频）。
/// 委托 RustCrypto `sha1`/`sha2`（digest 0.10 生态）。**字节传递**：JS 侧把 BufferSource 转
/// `number[]` → 逗号分隔十进制串（"72,73,..."）避免 UTF-8 编码歧义；本函数 split + parse 回 `Vec<u8>`，
/// 算哈希，返**逗号分隔十进制串**（shim 转 `Uint8Array`）。algo 归一大小写 + 接受 `SHA-256`/`SHA256`
/// 两种写法；unsupported algo 返**空串**（shim reject `NotSupportedError`）。
/// 供 `__zw_crypto_subtle_digest` 回调 → shim `crypto.subtle.digest`。
pub fn crypto_subtle_digest(algo: &str, bytes_csv: &str) -> String {
    use sha2::{Digest, Sha256, Sha384, Sha512};
    let data: Vec<u8> = bytes_csv
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.trim().parse::<u8>().ok())
        .collect();
    let hash: Vec<u8> = match algo.to_ascii_uppercase().as_str() {
        "SHA-1" | "SHA1" => sha1::Sha1::digest(&data).to_vec(),
        "SHA-256" | "SHA256" => Sha256::digest(&data).to_vec(),
        "SHA-384" | "SHA384" => Sha384::digest(&data).to_vec(),
        "SHA-512" | "SHA512" => Sha512::digest(&data).to_vec(),
        _ => return String::new(),
    };
    hash.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// 逗号分隔十进制字节串（"72,73,..."）→ `Vec<u8>`。空段跳过，非数字静默丢弃（与 digest 一致）。
/// `pub(super)` 供 compress 模块（CompressionStream/DecompressionStream，R2986）共用 byte wire。
pub(super) fn bytes_from_csv(csv: &str) -> Vec<u8> {
    csv.split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.trim().parse::<u8>().ok())
        .collect()
}

/// `Vec<u8>` / `&[u8]` → 逗号分隔十进制字节串（"72,73,..."）。`pub(super)` 供 compress 模块共用。
pub(super) fn bytes_to_csv(bytes: &[u8]) -> String {
    bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// 通用 HMAC（RFC 2104）：`MAC = H((K⊕opad) || H((K⊕ipad) || data))`。`block_size` 为 hash 块大小
///（SHA-1/256 = 64；SHA-384/512 = 128）。手写以复用既有 `sha1`/`sha2` 原语，避免引入 `hmac` 依赖。
/// https://datatracker.ietf.org/doc/html/rfc2104
fn compute_hmac<D: sha2::Digest>(key: &[u8], data: &[u8], block_size: usize) -> Vec<u8> {
    // 1. 密钥归一：len > B → K = H(K)（输出恒 ≤ B）；pad 到 B 字节。
    let mut k = if key.len() > block_size {
        D::digest(key).to_vec()
    } else {
        key.to_vec()
    };
    k.resize(block_size, 0u8);
    // 2. ipad (0x36) / opad (0x5c)。
    let mut ipad = vec![0x36u8; block_size];
    let mut opad = vec![0x5cu8; block_size];
    for i in 0..block_size {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    // 3. inner = H(ipad || data)。
    let mut inner = D::new();
    inner.update(&ipad);
    inner.update(data);
    let inner_hash = inner.finalize();
    // 4. MAC = H(opad || inner)。
    let mut outer = D::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize().to_vec()
}

/// `crypto.subtle.sign/verify` 的 HMAC 实现（R2955）——HMAC-SHA-1/256/384/512。**JWT HS256 / 请求签名 /
/// webhook 校验**（Stripe / AWS Sig v4 风格）高频。key_csv/data_csv = 逗号分隔十进制字节串；返 MAC
/// 逗号分隔十进制串（unsupported hash → 空串，shim reject `NotSupportedError`）。
/// 供 `__zw_crypto_subtle_hmac` 回调 → shim `crypto.subtle.sign("HMAC", ...)` / `verify`。
/// **scope**：仅 HMAC（RSASSA/ECDSA/AES/HKDF/PBKDF2 仍 defer——大表面，HMAC 为对称 MAC 最高频子集）。
pub fn crypto_subtle_hmac(hash_algo: &str, key_csv: &str, data_csv: &str) -> String {
    use sha2::{Sha256, Sha384, Sha512};
    let key = bytes_from_csv(key_csv);
    let data = bytes_from_csv(data_csv);
    let mac: Vec<u8> = match hash_algo.to_ascii_uppercase().as_str() {
        "SHA-1" | "SHA1" => compute_hmac::<sha1::Sha1>(&key, &data, 64),
        "SHA-256" | "SHA256" => compute_hmac::<Sha256>(&key, &data, 64),
        "SHA-384" | "SHA384" => compute_hmac::<Sha384>(&key, &data, 128),
        "SHA-512" | "SHA512" => compute_hmac::<Sha512>(&key, &data, 128),
        _ => return String::new(),
    };
    mac.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

// ---------------------------------------------------------------------------
// KMAC（crypto.subtle.sign/verify KMAC128/KMAC256，webcrypto-compat-v3 D1 拍板
// 2026-11-02）：SP 800-185 §4.3 手写框架叠官方 cshake 0.2.1（option b——sha3-kmac
// MIN_KEY_SIZE 会拒语料 16B/24B key，见预检 evidence 2026-11-02-dc2-msrv-compile-
// precheck.md）。NIST 向量单测随切片落仓（kmac_nist_tests，6/6 逐字节）。
// **密钥材料零落日志**：key 仅经 CSV 桥进出，无 tracing 输出。
// https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf
// ---------------------------------------------------------------------------

/// SP 800-185 `left_encode(n)`：n 大端字节（最少 1 字节）+ 前导长度字节。
fn enc_left_encode(n: usize) -> Vec<u8> {
    let be = n.to_be_bytes();
    let first = be.iter().position(|&b| b != 0).unwrap_or(be.len() - 1);
    let mut out = Vec::with_capacity(9);
    out.push((be.len() - first) as u8);
    out.extend_from_slice(&be[first..]);
    out
}

/// SP 800-185 `right_encode(n)`：n 大端字节（最少 1 字节）+ 尾随长度字节。
fn enc_right_encode(n: usize) -> Vec<u8> {
    let be = n.to_be_bytes();
    let first = be.iter().position(|&b| b != 0).unwrap_or(be.len() - 1);
    let mut out = Vec::with_capacity(9);
    out.extend_from_slice(&be[first..]);
    out.push((be.len() - first) as u8);
    out
}

/// KMAC（SP 800-185 §4.3）：`KMAC(K, X, L, S) = cSHAKE(bytepad(encode_string(K), rate)
/// || X || right_encode(L), "KMAC", S)`。encode_string 长度取 **bit**（K 位数）；
/// rate KMAC128 = 168 / KMAC256 = 136（cSHAKE128/256）；L = 输出位长。key_csv/data_csv/
/// custom_csv = 逗号分隔十进制字节串（custom 空 ≡ 缺席 customization）。返 MAC csv；
/// 未知算法名 → 空串。供 `__zw_crypto_subtle_kmac` 回调 → shim KMAC sign/verify 臂。
pub fn crypto_subtle_kmac(algo: &str, key_csv: &str, data_csv: &str, out_bits: u32, custom_csv: &str) -> String {
    let key = bytes_from_csv(key_csv);
    let data = bytes_from_csv(data_csv);
    let custom = bytes_from_csv(custom_csv);
    let rate = match algo.to_ascii_uppercase().as_str() {
        "KMAC128" => 168usize,
        "KMAC256" => 136,
        _ => return String::new(),
    };
    // 信任边界：回调在沙箱全局暴露，JS 可直调任意 out_bits——输出上限护栏（语料
    // 最大 512 bit，16 MiB 顶留大裕量防误用 OOM；与 crypto_subtle_xof_digest 同口径）。
    if out_bits as usize / 8 > 16 * 1024 * 1024 {
        return String::new();
    }
    // new_data = bytepad(encode_string(K), rate) || X || right_encode(L)（SP 800-185
    // §4.3 原式——仅 K 加 encode_string 位长前缀，X 是裸数据；S 不入 new_data，经
    // cSHAKE customization 参数传入）。bytepad(X, w) = left_encode(w) || X || 补零至
    // w 整数倍——K 段后必须对齐 rate 吸收边界（cSHAKE 按 rate 分块）。
    let mut input = Vec::with_capacity(2 * rate + 18 + key.len() + data.len());
    input.extend_from_slice(&enc_left_encode(rate));
    input.extend_from_slice(&enc_left_encode(key.len() * 8));
    input.extend_from_slice(&key);
    input.resize(input.len() + (rate - input.len() % rate) % rate, 0);
    input.extend_from_slice(&data);
    input.extend_from_slice(&enc_right_encode(out_bits as usize));
    let mac = match rate {
        168 => xof_squeeze(
            cshake::CShake128::new_with_function_name(b"KMAC", &custom),
            &input,
            (out_bits / 8) as usize,
        ),
        _ => xof_squeeze(
            cshake::CShake256::new_with_function_name(b"KMAC", &custom),
            &input,
            (out_bits / 8) as usize,
        ),
    };
    mac.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// `crypto.subtle.deriveBits` 的 PBKDF2 实现（R2956）——PBKDF2-HMAC-SHA-1/256/384/512。**密码派生密钥**
/// 高频（密码管理器 / 加密备份 / 「用密码加密」流程的密钥派生步骤）。PRF = HMAC-<hash>（复用
/// `compute_hmac`，block_size 内置于闭包）。password_csv/salt_csv = 逗号分隔十进制字节串；iterations 为
/// 循环次数；dklen 为派生字节长度。返派生密钥 csv（unsupported hash / iterations=0 / dklen=0 → 空，shim reject）。
/// 供 `__zw_crypto_subtle_pbkdf2` 回调 → shim `crypto.subtle.deriveBits("PBKDF2", ...)`。
/// https://datatracker.ietf.org/doc/html/rfc2898#section-5.2
pub fn crypto_subtle_pbkdf2(
    hash_algo: &str,
    password_csv: &str,
    salt_csv: &str,
    iterations: u32,
    dklen: usize,
) -> String {
    let password = bytes_from_csv(password_csv);
    let salt = bytes_from_csv(salt_csv);
    if iterations == 0 || dklen == 0 {
        return String::new();
    }
    // PRF = HMAC-<hash>；block_size 按 hash（SHA-1/256=64；SHA-384/512=128）内置于闭包。
    let hmac: Box<dyn Fn(&[u8], &[u8]) -> Vec<u8>> = match hash_algo.to_ascii_uppercase().as_str() {
        "SHA-1" | "SHA1" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha1::Sha1>(k, d, 64)),
        "SHA-256" | "SHA256" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha2::Sha256>(k, d, 64)),
        "SHA-384" | "SHA384" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha2::Sha384>(k, d, 128)),
        "SHA-512" | "SHA512" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha2::Sha512>(k, d, 128)),
        _ => return String::new(),
    };
    let mut out: Vec<u8> = Vec::with_capacity(dklen);
    let mut block_index: u32 = 1;
    while out.len() < dklen {
        // U_1 = PRF(password, salt || INT_32_BE(block_index))。
        let mut msg = salt.clone();
        msg.extend_from_slice(&block_index.to_be_bytes());
        let mut u = hmac(&password, &msg);
        let mut t = u.clone();
        // U_2..U_c = PRF(password, U_{j-1})；T_i = U_1 ⊕ ... ⊕ U_c。
        for _ in 1..iterations {
            u = hmac(&password, &u);
            for (tb, ub) in t.iter_mut().zip(u.iter()) {
                *tb ^= *ub;
            }
        }
        out.extend_from_slice(&t);
        block_index += 1;
    }
    out.truncate(dklen);
    out.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// AES-GCM 单次操作（encrypt→`ct||tag` / decrypt→plaintext），AAD 经 `Payload` 传入。
/// tag 校验失败 / 未知 mode → None（shim reject OperationError）。
fn aes_gcm_run<C: aes_gcm::aead::Aead>(
    cipher: &C,
    mode: &str,
    nonce: &aes_gcm::aead::generic_array::GenericArray<u8, C::NonceSize>,
    payload: aes_gcm::aead::Payload,
) -> Option<Vec<u8>> {
    match mode {
        "encrypt" => cipher.encrypt(nonce, payload).ok(),
        "decrypt" => cipher.decrypt(nonce, payload).ok(),
        _ => None,
    }
}

/// `crypto.subtle.encrypt/decrypt` 的 AES-GCM 实现（R2957；webcrypto M3a 矩阵扩展）——
/// AES-128/192/256-GCM 认证对称加密。**PBKDF2 派生密钥的典型消费者**，端到端「用密码加密」
/// 流程（TLS 级对称加密 + 完整性 + AAD）。mode "encrypt"（返 ct||tag）/ "decrypt"（输入
/// ct||tag，返 plaintext）；key 16/24/32 字节；iv 12/16/32 字节（nonce 为 aes-gcm crate 编译期
/// 泛型，按字节长分派；16 字节 M3b 起供 wrapKey AES-GCM 封装路径）；tag_len_bits ∈ {32,64,96,104,112,120,128}（合法集由 shim 校验，
/// 缺省 128）：96-128 位走 crate Aead（TagSize 密封于 U12..U16）；32/64 位 crate 不密封，
/// 按 NIST SP 800-38D §5.2.1.2 截断语义手做——encrypt 取 128 位 tag 前 tag_len_bits/8 字节，
/// decrypt 重算全 tag 常时比对前缀。aad 附加认证数据。返 csv，error（bad key/iv/tag 长度、
/// tag 校验失败）→ 空串。
/// https://datatracker.ietf.org/doc/html/rfc5116  https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf
pub fn crypto_subtle_aes_gcm(
    mode: &str,
    key_csv: &str,
    iv_csv: &str,
    data_csv: &str,
    aad_csv: &str,
    tag_len_bits: u32,
) -> String {
    use aes::cipher::consts::{U12, U13, U14, U15, U16, U32};
    use aes_gcm::AesGcm;
    use aes_gcm::aead::{KeyInit, Payload, generic_array::GenericArray};
    let key = bytes_from_csv(key_csv);
    let iv = bytes_from_csv(iv_csv);
    let data = bytes_from_csv(data_csv);
    let aad = bytes_from_csv(aad_csv);
    // 96/128/256 位 IV（M3a 收敛 12/32，M3b 增 16 供 wrapKey AES-GCM 封装路径——spec 允许
    // 任意 IV 长度，crate NonceSize 开放泛型按字节长分派）。
    if iv.len() != 12 && iv.len() != 16 && iv.len() != 32 {
        return String::new();
    }
    let tag_bytes = (tag_len_bits / 8) as usize;
    // 32/64 位 tag：全 128 位认证后截断（crate 只密封 U12..U16，见函数注释）。
    let truncated = matches!(tag_len_bits, 32 | 64);
    let effective_tag_bits = if truncated { 128 } else { tag_len_bits };
    let payload = Payload { msg: &data, aad: &aad };
    macro_rules! gcm {
        ($nonce:ty, $tag:ty, $aes:ty) => {
            aes_gcm_run(
                &AesGcm::<$aes, $nonce, $tag>::new(GenericArray::from_slice(&key)),
                mode,
                GenericArray::from_slice(&iv),
                payload,
            )
        };
    }
    macro_rules! by_aes {
        ($nonce:ty, $tag:ty) => {
            match key.len() {
                16 => gcm!($nonce, $tag, aes::Aes128),
                24 => gcm!($nonce, $tag, aes::Aes192),
                32 => gcm!($nonce, $tag, aes::Aes256),
                _ => None,
            }
        };
    }
    macro_rules! by_tag {
        ($nonce:ty) => {
            match effective_tag_bits {
                96 => by_aes!($nonce, U12),
                104 => by_aes!($nonce, U13),
                112 => by_aes!($nonce, U14),
                120 => by_aes!($nonce, U15),
                128 => by_aes!($nonce, U16),
                _ => None,
            }
        };
    }
    if truncated {
        // 截断路径：detached API 手做。encrypt：明文 in-place 加密 → ct || tag 前 tag_bytes；
        // decrypt：重算期望全 tag，常时比对前缀（SP 800-38D §5.2.1.2）。
        // nonce 泛型按 iv 实际字节长分派（from_slice 断言长度，须在分支内构造）。
        return match (iv.len(), key.len()) {
            (12, 16) => {
                truncated_gcm::<aes::Aes128, U12>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (12, 24) => {
                truncated_gcm::<aes::Aes192, U12>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (12, 32) => {
                truncated_gcm::<aes::Aes256, U12>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (32, 16) => {
                truncated_gcm::<aes::Aes128, U32>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (32, 24) => {
                truncated_gcm::<aes::Aes192, U32>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (32, 32) => {
                truncated_gcm::<aes::Aes256, U32>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (16, 16) => {
                truncated_gcm::<aes::Aes128, U16>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (16, 24) => {
                truncated_gcm::<aes::Aes192, U16>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            (16, 32) => {
                truncated_gcm::<aes::Aes256, U16>(mode, &key, GenericArray::from_slice(&iv), &data, &aad, tag_bytes)
            }
            _ => String::new(),
        };
    }
    let result: Option<Vec<u8>> = match iv.len() {
        12 => by_tag!(U12),
        // 96/128/256 位 IV——crate NonceSize 为开放泛型（仅 TagSize 密封），M3b 起
        // 16 字节 IV 供 wrapKey AES-GCM 封装路径（spec 允许任意 IV 长度）。
        16 => by_tag!(U16),
        32 => by_tag!(U32),
        _ => None,
    };
    match result {
        Some(v) => v.iter().map(u8::to_string).collect::<Vec<_>>().join(","),
        None => String::new(),
    }
}

/// GCM 截断 tag（32/64 位）encrypt/decrypt——crate `AeadInPlace` detached API 之上的
/// SP 800-38D §5.2.1.2 截断语义：tag 取全 128 位前缀；decrypt 重算全 tag 后**定长无早退**
/// 比对前缀（不引 `subtle` 依赖，与 shim HMAC 比对先例一致）。decrypt 输入为 ct||tag。
fn truncated_gcm<A, N>(
    mode: &str,
    key: &[u8],
    nonce: &aes_gcm::aead::generic_array::GenericArray<u8, N>,
    data: &[u8],
    aad: &[u8],
    tag_bytes: usize,
) -> String
where
    A: aes::cipher::BlockSizeUser<BlockSize = aes::cipher::consts::U16>
        + aes::cipher::BlockEncrypt
        + aes::cipher::KeyInit,
    N: aes_gcm::aead::generic_array::ArrayLength<u8>,
    aes_gcm::AesGcm<A, N, aes::cipher::consts::U16>:
        aes_gcm::aead::AeadCore<NonceSize = N> + aes_gcm::aead::AeadInPlace,
{
    use aes_gcm::aead::{AeadInPlace, KeyInit, generic_array::GenericArray as GA};
    let cipher = aes_gcm::AesGcm::<A, N, aes::cipher::consts::U16>::new(GA::from_slice(key));
    if mode == "encrypt" {
        let mut buf = data.to_vec();
        let full_tag = match cipher.encrypt_in_place_detached(nonce, aad, &mut buf) {
            Ok(t) => t,
            Err(_) => return String::new(),
        };
        buf.extend_from_slice(&full_tag[..tag_bytes]);
        return buf.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
    }
    if mode == "decrypt" {
        if data.len() < tag_bytes {
            return String::new();
        }
        let ct_len = data.len() - tag_bytes;
        let provided = &data[ct_len..];
        // crate 的 detached encrypt 先做 CTR 变换、后对**变换后**缓冲（=ct）算 GHASH tag
        //（见 aes-gcm 0.10.3 src/lib.rs encrypt_in_place_detached）。故：第一次调用输入 ct
        // 得明文（tag 对 pt，弃）；第二次调用输入 pt 变换回 ct，其 tag 恰为期望全 128 位
        // tag（同 mask 同 GHASH 输入，确定性）——以此做截断前缀比对，明文在 pt 中不受影响。
        // CTR 为对合变换（SP 800-38D §6.5），两次变换还原。
        let mut pt = data[..ct_len].to_vec();
        if cipher.encrypt_in_place_detached(nonce, aad, &mut pt).is_err() {
            return String::new();
        }
        let mut scratch = pt.clone();
        let expected = match cipher.encrypt_in_place_detached(nonce, aad, &mut scratch) {
            Ok(t) => t,
            Err(_) => return String::new(),
        };
        // 常时近似：全前缀逐字节异或累积，无早退。
        let mut diff = 0u8;
        for i in 0..tag_bytes {
            diff |= expected[i] ^ provided[i];
        }
        if diff == 0 {
            return pt.iter().map(u8::to_string).collect::<Vec<_>>().join(",");
        }
    }
    String::new()
}

/// `crypto.subtle.encrypt/decrypt` 的 AES-CBC/CTR 实现（webcrypto M2b）——AES-128/192/256
/// CBC（PKCS#7 填充）与 CTR（16 字节大端计数器，NIST SP 800-38A）。RustCrypto
/// `cbc`/`ctr` crate（选型见 docs/goal/archive/webcrypto-compat/evidence/2026-10-17-m2b-deps.md）。
/// kind "cbc"/"ctr"，mode "encrypt"/"decrypt"（CTR 加解密同路径）；key 16/24/32 字节；
/// iv/counter 16 字节（长度由 shim 校验）；CBC 解密 padding 非法（含密文非块对齐/空串）→
/// 空串（shim reject OperationError，填充校验在 crate 内不经手写 unpad）。CTR 无认证，
/// 空输入返空输出（合法）。返 csv。
/// https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
pub fn crypto_subtle_aes_block(kind: &str, mode: &str, key_csv: &str, iv_csv: &str, data_csv: &str) -> String {
    use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
    use ctr::cipher::StreamCipher;
    let key = bytes_from_csv(key_csv);
    let iv = bytes_from_csv(iv_csv);
    let mut data = bytes_from_csv(data_csv);
    if iv.len() != 16 {
        return String::new();
    }
    if kind == "ctr" {
        let applied = match key.len() {
            16 => ctr::Ctr128BE::<aes::Aes128>::new_from_slices(&key, &iv).map(|mut c| c.apply_keystream(&mut data)),
            24 => ctr::Ctr128BE::<aes::Aes192>::new_from_slices(&key, &iv).map(|mut c| c.apply_keystream(&mut data)),
            32 => ctr::Ctr128BE::<aes::Aes256>::new_from_slices(&key, &iv).map(|mut c| c.apply_keystream(&mut data)),
            _ => Err(cbc::cipher::InvalidLength),
        };
        if applied.is_err() {
            return String::new();
        }
        return bytes_to_csv(&data);
    }
    if mode == "encrypt" {
        let out = match key.len() {
            16 => cbc::Encryptor::<aes::Aes128>::new_from_slices(&key, &iv)
                .ok()
                .map(|e| e.encrypt_padded_vec_mut::<Pkcs7>(&data)),
            24 => cbc::Encryptor::<aes::Aes192>::new_from_slices(&key, &iv)
                .ok()
                .map(|e| e.encrypt_padded_vec_mut::<Pkcs7>(&data)),
            32 => cbc::Encryptor::<aes::Aes256>::new_from_slices(&key, &iv)
                .ok()
                .map(|e| e.encrypt_padded_vec_mut::<Pkcs7>(&data)),
            _ => None,
        };
        match out {
            Some(v) => bytes_to_csv(&v),
            None => String::new(),
        }
    } else if mode == "decrypt" {
        let out = match key.len() {
            16 => cbc::Decryptor::<aes::Aes128>::new_from_slices(&key, &iv)
                .ok()
                .and_then(|d| d.decrypt_padded_vec_mut::<Pkcs7>(&data).ok()),
            24 => cbc::Decryptor::<aes::Aes192>::new_from_slices(&key, &iv)
                .ok()
                .and_then(|d| d.decrypt_padded_vec_mut::<Pkcs7>(&data).ok()),
            32 => cbc::Decryptor::<aes::Aes256>::new_from_slices(&key, &iv)
                .ok()
                .and_then(|d| d.decrypt_padded_vec_mut::<Pkcs7>(&data).ok()),
            _ => None,
        };
        match out {
            Some(v) => bytes_to_csv(&v),
            None => String::new(),
        }
    } else {
        String::new()
    }
}

/// AES-KW（RFC 3394）wrap/unwrap——`crypto.subtle.wrapKey/unwrapKey` 的 AES-KW 封装路径
/// （webcrypto M3b）。mode "wrap"（8 字节对齐明文 → +8 字节封装输出）/ "unwrap"（校验
/// 回绕寄存器 A 恒等于初始 IV A6A6×8 后剥去 8 字节；不匹配 → 空串，shim reject
/// OperationError）。key 16/24/32 字节；wrap 输入 ≥16 字节且 8 字节对齐、unwrap 输入
/// ≥24 字节且 8 字节对齐。纯块密码运算（aes crate BlockEncrypt/BlockDecrypt），无认证
/// 依赖、无时间秘密分支（IV 比对为长度固定的公开常量比较）。
/// https://datatracker.ietf.org/doc/html/rfc3394#section-2.2.1
pub fn crypto_subtle_aes_kw(mode: &str, key_csv: &str, data_csv: &str) -> String {
    use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
    let key = bytes_from_csv(key_csv);
    let data = bytes_from_csv(data_csv);
    if !data.len().is_multiple_of(8) {
        return String::new();
    }
    macro_rules! kw {
        ($aes:ty) => {{
            let cipher = <$aes>::new(GenericArray::from_slice(&key));
            const IV: u64 = 0xA6A6A6A6A6A6A6A6;
            if mode == "wrap" {
                if data.len() < 16 {
                    return String::new();
                }
                let n = data.len() / 8;
                let mut a = IV;
                let mut blocks: Vec<[u8; 8]> = (0..n)
                    .map(|i| {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&data[i * 8..i * 8 + 8]);
                        b
                    })
                    .collect();
                let mut buf = [0u8; 16];
                for j in 0..=5u64 {
                    for i in 0..n {
                        buf[..8].copy_from_slice(&a.to_be_bytes());
                        buf[8..].copy_from_slice(&blocks[i]);
                        let mut ga = GenericArray::clone_from_slice(&buf);
                        cipher.encrypt_block(&mut ga);
                        let mut hi = [0u8; 8];
                        hi.copy_from_slice(&ga[..8]);
                        a = u64::from_be_bytes(hi) ^ (n as u64 * j + i as u64 + 1);
                        blocks[i].copy_from_slice(&ga[8..]);
                    }
                }
                let mut out = Vec::with_capacity(data.len() + 8);
                out.extend_from_slice(&a.to_be_bytes());
                for b in blocks {
                    out.extend_from_slice(&b);
                }
                return bytes_to_csv(&out);
            }
            if mode == "unwrap" {
                if data.len() < 24 {
                    return String::new();
                }
                let n = data.len() / 8 - 1;
                let mut a = u64::from_be_bytes(data[..8].try_into().unwrap_or([0; 8]));
                let mut blocks: Vec<[u8; 8]> = (0..n)
                    .map(|i| {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(&data[(i + 1) * 8..(i + 1) * 8 + 8]);
                        b
                    })
                    .collect();
                let mut buf = [0u8; 16];
                for j in (0..=5u64).rev() {
                    for i in (0..n).rev() {
                        let t = (n as u64 * j + i as u64 + 1).to_be_bytes();
                        buf[..8].copy_from_slice(&(a ^ u64::from_be_bytes(t)).to_be_bytes());
                        buf[8..].copy_from_slice(&blocks[i]);
                        let mut ga = GenericArray::clone_from_slice(&buf);
                        cipher.decrypt_block(&mut ga);
                        a = u64::from_be_bytes(ga[..8].try_into().unwrap_or([0; 8]));
                        blocks[i].copy_from_slice(&ga[8..]);
                    }
                }
                if a != IV {
                    return String::new();
                }
                let mut out = Vec::with_capacity(data.len() - 8);
                for b in blocks {
                    out.extend_from_slice(&b);
                }
                return bytes_to_csv(&out);
            }
            String::new()
        }};
    }
    match key.len() {
        16 => kw!(aes::Aes128),
        24 => kw!(aes::Aes192),
        32 => kw!(aes::Aes256),
        _ => String::new(),
    }
}

/// `crypto.subtle.encrypt/decrypt` 的 AES-OCB 实现（webcrypto-compat-v3 域切片；WICG
/// webcrypto-modern-algos §10）——AES-128/192/256-OCB（RFC 7253）。RustCrypto `ocb3`
/// crate（构建于 cipher 0.5/aead 0.6 新栈，D1 拍板选型见
/// docs/goal/webcrypto-compat-v3/master.md 待用户决策表；aes09 =0.9.2 钉版）。mode
/// "encrypt"（返 ct||tag）/ "decrypt"（输入 ct||tag，返 plaintext）；key 16/24/32 字节；
/// iv：spec 仅拦 >15 字节（RFC 7253 N ≤ 120 位），crate NonceSize 密封 6..=15 字节
///（KnoX 短 nonce 防护），0..=5 同 reject OperationError（语料 failing iv [0,16] 均
/// 落拦截区）；tag_len_bits ∈ {64,96,128}（spec 合法集，缺省 128，由 shim 校验）。nonce
/// 长度泛型宏分派 6..=15 十档 × tag 三档 × key 三档（NonceSize 编译期固定，RFC 7253
/// nonce 构造依赖其位长）。返 csv，error（tag 校验失败等）→ 空串。
/// https://wicg.github.io/webcrypto-modern-algos/#aes-ocb  https://datatracker.ietf.org/doc/html/rfc7253
pub fn crypto_subtle_aes_ocb(
    mode: &str,
    key_csv: &str,
    iv_csv: &str,
    data_csv: &str,
    aad_csv: &str,
    tag_len_bits: u32,
) -> String {
    use ocb3::aead::inout::InOutBuf;
    let key = bytes_from_csv(key_csv);
    let iv = bytes_from_csv(iv_csv);
    let data = bytes_from_csv(data_csv);
    let aad = bytes_from_csv(aad_csv);
    if iv.len() < 6 || iv.len() > 15 {
        return String::new();
    }
    if !matches!(tag_len_bits, 64 | 96 | 128) {
        return String::new();
    }
    let tag_bytes = (tag_len_bits / 8) as usize;
    macro_rules! ocb_call {
        ($aes:ty, $nonce:ty, $tag:ty) => {{
            use ocb3::aead::{AeadInOut, KeyInit};
            type Ocb = ocb3::Ocb3<$aes, $nonce, $tag>;
            let cipher = Ocb::new(key.as_slice().try_into().expect("key length validated"));
            if mode == "encrypt" {
                let mut buf = data.clone();
                match cipher.encrypt_inout_detached(
                    iv.as_slice().try_into().expect("nonce length dispatched"),
                    &aad,
                    InOutBuf::from(buf.as_mut_slice()),
                ) {
                    Ok(tag) => {
                        buf.extend_from_slice(&tag);
                        Some(buf)
                    }
                    Err(_) => None,
                }
            } else if data.len() >= tag_bytes {
                // spec：ciphertext < tagLength 位 → OperationError（shim 前置拦截，host 兜底）。
                let ct_len = data.len() - tag_bytes;
                let mut buf = data[..ct_len].to_vec();
                match cipher.decrypt_inout_detached(
                    iv.as_slice().try_into().expect("nonce length dispatched"),
                    &aad,
                    InOutBuf::from(buf.as_mut_slice()),
                    (&data[ct_len..]).try_into().expect("tag length dispatched"),
                ) {
                    Ok(()) => Some(buf),
                    Err(_) => None,
                }
            } else {
                None
            }
        }};
    }
    macro_rules! by_key {
        ($nonce:ty, $tag:ty) => {
            match key.len() {
                16 => ocb_call!(aes09::Aes128, $nonce, $tag),
                24 => ocb_call!(aes09::Aes192, $nonce, $tag),
                32 => ocb_call!(aes09::Aes256, $nonce, $tag),
                _ => None,
            }
        };
    }
    macro_rules! by_tag {
        ($nonce:ty) => {
            match tag_len_bits {
                64 => by_key!($nonce, aes09::cipher::consts::U8),
                96 => by_key!($nonce, aes09::cipher::consts::U12),
                128 => by_key!($nonce, aes09::cipher::consts::U16),
                _ => None,
            }
        };
    }
    // nonce 长度分派：ocb3 NonceSize 密封 6..=15 字节（<6 由前置校验拦）。
    let result: Option<Vec<u8>> = match iv.len() {
        6 => by_tag!(aes09::cipher::consts::U6),
        7 => by_tag!(aes09::cipher::consts::U7),
        8 => by_tag!(aes09::cipher::consts::U8),
        9 => by_tag!(aes09::cipher::consts::U9),
        10 => by_tag!(aes09::cipher::consts::U10),
        11 => by_tag!(aes09::cipher::consts::U11),
        12 => by_tag!(aes09::cipher::consts::U12),
        13 => by_tag!(aes09::cipher::consts::U13),
        14 => by_tag!(aes09::cipher::consts::U14),
        15 => by_tag!(aes09::cipher::consts::U15),
        _ => None,
    };
    match result {
        // 空明文（密文仅 tag）成功输出空字节串，与「失败 → 空串」的 wire 约定冲突——
        // 返单逗号哨兵（shim 特判为空 Uint8Array）。
        Some(v) if mode == "decrypt" && v.is_empty() => String::from(","),
        Some(v) => bytes_to_csv(&v),
        None => String::new(),
    }
}

/// `crypto.subtle.encrypt/decrypt` 的 ChaCha20-Poly1305 实现（webcrypto-compat-v3 域切片；
/// WICG webcrypto-modern-algos §11）——AEAD_CHACHA20_POLY1305（RFC 8439）。RustCrypto
/// `chacha20poly1305` crate（aead 0.6 新栈，D1 拍板选型）。mode "encrypt"（返 ct||tag）/
/// "decrypt"；key 恒 32 字节（KeySize = U32）；iv 恒 12 字节（spec ≠12 → OperationError）；
/// tag 恒 128 位（spec tagLength 成员若存在须 =128，由 shim 校验）。返 csv，error（tag
/// 校验失败等）→ 空串。
/// https://wicg.github.io/webcrypto-modern-algos/#chacha20-poly1305  https://datatracker.ietf.org/doc/html/rfc8439
pub fn crypto_subtle_chacha20_poly1305(
    mode: &str,
    key_csv: &str,
    iv_csv: &str,
    data_csv: &str,
    aad_csv: &str,
) -> String {
    use chacha20poly1305::aead::{AeadInOut, KeyInit, inout::InOutBuf};
    let key = bytes_from_csv(key_csv);
    let iv = bytes_from_csv(iv_csv);
    let data = bytes_from_csv(data_csv);
    let aad = bytes_from_csv(aad_csv);
    if key.len() != 32 || iv.len() != 12 {
        return String::new();
    }
    let cipher = chacha20poly1305::ChaCha20Poly1305::new(key.as_slice().try_into().expect("key length checked"));
    let nonce: chacha20poly1305::Nonce = iv.as_slice().try_into().expect("iv length checked");
    let result = if mode == "encrypt" {
        let mut buf = data.clone();
        match cipher.encrypt_inout_detached(&nonce, &aad, InOutBuf::from(buf.as_mut_slice())) {
            Ok(tag) => {
                buf.extend_from_slice(&tag);
                Some(buf)
            }
            Err(_) => None,
        }
    } else if data.len() >= 16 {
        // spec：ciphertext < 128 位 → OperationError（shim 前置拦截，host 兜底）。
        let ct_len = data.len() - 16;
        let mut buf = data[..ct_len].to_vec();
        let tag: chacha20poly1305::Tag = (&data[ct_len..]).try_into().expect("tag length checked");
        match cipher.decrypt_inout_detached(&nonce, &aad, InOutBuf::from(buf.as_mut_slice()), &tag) {
            Ok(()) => Some(buf),
            Err(_) => None,
        }
    } else {
        None
    };
    match result {
        // 空明文（密文仅 tag）成功输出空字节串，与「失败 → 空串」的 wire 约定冲突——
        // 返单逗号哨兵（shim 特判为空 Uint8Array）。
        Some(v) if mode == "decrypt" && v.is_empty() => String::from(","),
        Some(v) => bytes_to_csv(&v),
        None => String::new(),
    }
}

/// `crypto.subtle.deriveBits` 的 Argon2 实现（webcrypto-compat-v3 域切片；WICG
/// webcrypto-modern-algos §17）——Argon2d/i/id 密码基派生（RFC 9106）。RustCrypto
/// `argon2` crate（D1 拍板选型）。type_str ∈ ARGON2D/ARGON2I/ARGON2ID（shim 归一后形态）；
/// password = raw-secret 导入的密钥材料（spec [[handle]] 内槽即密码消息 P）；nonce（spec
/// Argon2Params required）；secret/ad 可选（空 csv ≡ 缺席）；parallelism/memory/passes 由
/// shim 按规范约束校验（version ≠19 同），host 经 ParamsBuilder::build 兜底（m ≥ 8p、
/// p ∈ [1,2^24-1]、t ≥ 1、m ≥ 16）；out_len = spec length/8（shim 校验 ≥32 位且 8 倍数）。
/// version 恒 0x13（spec 唯一定义版本）。返派生位 csv，error → 空串。
/// https://wicg.github.io/webcrypto-modern-algos/#argon2  https://datatracker.ietf.org/doc/html/rfc9106
pub fn crypto_subtle_argon2(
    type_str: &str,
    password_csv: &str,
    nonce_csv: &str,
    secret_csv: &str,
    ad_csv: &str,
    parallelism: u32,
    memory: u32,
    passes: u32,
    out_len: usize,
) -> String {
    use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
    let password = bytes_from_csv(password_csv);
    let nonce = bytes_from_csv(nonce_csv);
    let secret = bytes_from_csv(secret_csv);
    let ad = bytes_from_csv(ad_csv);
    let algorithm = match type_str {
        "ARGON2D" => Algorithm::Argon2d,
        "ARGON2I" => Algorithm::Argon2i,
        "ARGON2ID" => Algorithm::Argon2id,
        _ => return String::new(),
    };
    let mut builder = ParamsBuilder::new();
    builder.m_cost(memory).t_cost(passes).p_cost(parallelism);
    if !ad.is_empty() {
        match argon2::AssociatedData::new(&ad) {
            Ok(data) => {
                builder.data(data);
            }
            Err(_) => return String::new(),
        }
    }
    let params = match builder.build() {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    // secretValue 缺席 ≡ 空 secret（RFC 9106 H0 组装长度 0，两种传法等价）。
    let ctx = if secret.is_empty() {
        Argon2::new(algorithm, Version::V0x13, params)
    } else {
        match Argon2::new_with_secret(&secret, algorithm, Version::V0x13, params) {
            Ok(c) => c,
            Err(_) => return String::new(),
        }
    };
    let mut out = vec![0u8; out_len];
    if ctx.hash_password_into(&password, &nonce, &mut out).is_err() {
        return String::new();
    }
    out.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

/// `crypto.subtle.deriveBits` 的 HKDF 实现（R2958）——HKDF-SHA-1/256/384/512（RFC 5869）。**密钥协商派生**
/// 高频（TLS 1.3 / MLS / WebRTC DTLS-SRTP / E2EE 协议的输入密钥材料→会话密钥）。PRF = HMAC-<hash>（复用
/// `compute_hmac`）。Extract：`PRK = HMAC(salt, IKM)`（空 salt → HashLen 零）；Expand：`T(i)=HMAC(PRK, T(i-1)||info||i)`，
/// `OKM = T(1)||..||T(N)` 截断 dklen。ikm_csv/salt_csv/info_csv = 逗号分隔十进制字节串；返派生密钥 csv。
/// 供 `__zw_crypto_subtle_hkdf` 回调 → shim `crypto.subtle.deriveBits("HKDF", ...)`。
/// https://datatracker.ietf.org/doc/html/rfc5869
pub fn crypto_subtle_hkdf(hash_algo: &str, ikm_csv: &str, salt_csv: &str, info_csv: &str, dklen: usize) -> String {
    let ikm = bytes_from_csv(ikm_csv);
    let salt = bytes_from_csv(salt_csv);
    let info = bytes_from_csv(info_csv);
    if dklen == 0 {
        return String::new();
    }
    // PRF = HMAC-<hash>；block_size / hash_len 按 hash（SHA-1/256=64；SHA-384/512=128）。
    let (block_size, hash_len): (usize, usize) = match hash_algo.to_ascii_uppercase().as_str() {
        "SHA-1" | "SHA1" => (64, 20),
        "SHA-256" | "SHA256" => (64, 32),
        "SHA-384" | "SHA384" => (128, 48),
        "SHA-512" | "SHA512" => (128, 64),
        _ => return String::new(),
    };
    let hmac: Box<dyn Fn(&[u8], &[u8]) -> Vec<u8>> = match hash_algo.to_ascii_uppercase().as_str() {
        "SHA-1" | "SHA1" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha1::Sha1>(k, d, block_size)),
        "SHA-256" | "SHA256" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha2::Sha256>(k, d, block_size)),
        "SHA-384" | "SHA384" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha2::Sha384>(k, d, block_size)),
        "SHA-512" | "SHA512" => Box::new(|k: &[u8], d: &[u8]| compute_hmac::<sha2::Sha512>(k, d, block_size)),
        _ => unreachable!(),
    };
    // Extract：空 salt → HashLen 零（RFC 5869 §2.2）；PRK = HMAC(salt, IKM)。
    let salt_filled = if salt.is_empty() { vec![0u8; hash_len] } else { salt };
    let prk = hmac(&salt_filled, &ikm);
    // Expand：N = ceil(dklen / hash_len)（≤ 255，RFC 5869 §2.3）；T(i)=HMAC(PRK, T(i-1)||info||i)。
    let n = dklen.div_ceil(hash_len);
    if n > 255 {
        return String::new();
    }
    let mut t: Vec<u8> = Vec::new();
    let mut okm: Vec<u8> = Vec::with_capacity(dklen);
    for i in 1..=n {
        let mut msg = t.clone();
        msg.extend_from_slice(&info);
        msg.push(i as u8);
        t = hmac(&prk, &msg);
        okm.extend_from_slice(&t);
    }
    okm.truncate(dklen);
    okm.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

// ---------------------------------------------------------------------------
// XOF 摘要域（webcrypto-compat-v3 D1 域切片）——crypto.subtle.digest 扩展算法
// 注册面：SHA-3 定长家族（FIPS 202）+ SHAKE128/256 + cSHAKE128/256（NIST SP
// 800-185）+ KangarooTwelve KT128/256 + TurboSHAKE128/256（RFC 9861）。
// **hex wire**：XOF 语料最大输入 24,137,569 字节（RFC 9861 ptn(17^6) 向量），
// 既有 csv 十进制 wire 的 JS 侧 number[] 构成 + host 侧 split+parse 在 QuickJS
// 案预算（timeout=long 60s）内不保——hex 2 char/byte ASCII 直建 + 逐 nibble
// 解码快一个量级；输出亦 hex（最大 10,032 字节 KT128 大输出档）。失败 → 空串；
// **合法空输出 → 单逗号哨兵 ","**（空串已作失败语义占用，d1s1 空明文解密哨兵
// 同款）。五 crate 同构 digest 0.11.3（Update + ExtendableOutput 面）。
// https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf
// https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-185.pdf
// https://datatracker.ietf.org/doc/html/rfc9861

/// hex 串 → `Vec<u8>`（奇数长 / 非 hex 字符 → `None`）。XOF hex wire 伴生解码器。
fn bytes_from_hex(hex: &str) -> Option<Vec<u8>> {
    fn nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }
    let raw = hex.as_bytes();
    if !raw.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(raw.len() / 2);
    for pair in raw.as_chunks::<2>().0 {
        out.push(nibble(pair[0])? << 4 | nibble(pair[1])?);
    }
    Some(out)
}

/// `&[u8]` → hex 串（小写）。XOF hex wire 伴生编码器。
fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 15) as usize] as char);
    }
    s
}

/// XOF 统一吸收/挤出（digest 0.11 ExtendableOutput 面）：update → finalize_xof →
/// read 满长。语料实证输出长度不入态（KT128 空 256/512 bit 向量同前缀）。
fn xof_squeeze<D>(state: D, input: &[u8], out_bytes: usize) -> Vec<u8>
where
    D: shake::digest::Update + shake::digest::ExtendableOutput,
{
    use shake::digest::XofReader;
    let mut st = state;
    st.update(input);
    let mut reader = st.finalize_xof();
    let mut out = vec![0u8; out_bytes];
    reader.read(&mut out);
    out
}

/// TurboSHAKE DS 字节运行时分派——crate 为 const 泛型面（`TurboShake<RATE, DS>`），
/// 合法 DS 0x01..=0x7f 全集逐值实例化（keccak-f[1600] 置换共享，每实例仅吸收/挤出
/// 包装层）。语料 DS ∈ {0x01, 0x06, 0x07, 0x0b, 0x1f（缺省）, 0x7f}。
macro_rules! turbo_digest {
    ($rate:literal, $ds:expr, $input:expr, $out_bytes:expr; [$($d:literal),+ $(,)?]) => {
        match $ds {
            $($d => xof_squeeze(turboshake::TurboShake::<$rate, $d>::default(), $input, $out_bytes),)+
            _ => return String::new(),
        }
    };
}

/// `crypto.subtle.digest` XOF 扩展域——见上方域注释。arg[0]=归一算法名，arg[1]=输入
/// hex，arg[2]=输出 bit 数（**shim 已解析缺省**：128 系 256 / 256 系 512——cSHAKE
/// 显式 0 bit 合法空输出与缺席在 shim 分流，host 不二次定缺省），arg[3]=customization
/// 字节 hex（cSHAKE/KT，空 ≡ 缺席），arg[4]=domainSeparation 字节（TurboSHAKE，
/// 0 ≡ 缺席 → 0x1f）。SHA-3 定长家族忽略 arg[2]（出参定长）。返输出 hex（合法空
/// 输出 → "," 哨兵；失败 → 空串）。供 `__zw_crypto_subtle_xof_digest` 回调 →
/// shim `crypto.subtle.digest` XOF 分派臂。
pub fn crypto_subtle_xof_digest(
    algo: &str,
    input_hex: &str,
    output_bits: u32,
    param_hex: &str,
    param_num: u32,
) -> String {
    // 信任边界：回调在沙箱全局暴露，JS 可直调任意参数——输出上限护栏（语料最大
    // 10,032 字节，16 MiB 顶留千倍裕量防误用 OOM）。
    const MAX_OUT_BYTES: usize = 16 * 1024 * 1024;
    use sha3::Digest;
    // `CustomizedInit`：k12 CustomKt128/256::new_customized（digest 0.11 trait 面，
    // 五 crate 同源 re-export）。
    use k12::digest::CustomizedInit;
    let Some(input) = bytes_from_hex(input_hex) else {
        return String::new();
    };
    let customization = bytes_from_hex(param_hex).unwrap_or_default();
    let ds: u8 = match param_num {
        0 => turboshake::DEFAULT_DS,
        1..=127 => param_num as u8,
        _ => return String::new(),
    };
    let name = algo.to_ascii_uppercase();
    // SHA-3 定长家族（FIPS 202）先出——outputLength 成员不适用，不受输出上限约束。
    match name.as_str() {
        "SHA3-224" => return bytes_to_hex(&sha3::Sha3_224::digest(&input)),
        "SHA3-256" => return bytes_to_hex(&sha3::Sha3_256::digest(&input)),
        "SHA3-384" => return bytes_to_hex(&sha3::Sha3_384::digest(&input)),
        "SHA3-512" => return bytes_to_hex(&sha3::Sha3_512::digest(&input)),
        _ => {}
    }
    let out_bytes = output_bits as usize / 8;
    if out_bytes > MAX_OUT_BYTES {
        return String::new();
    }
    let out = match name.as_str() {
        "SHAKE128" => xof_squeeze(shake::Shake128::default(), &input, out_bytes),
        "SHAKE256" => xof_squeeze(shake::Shake256::default(), &input, out_bytes),
        // cSHAKE（SP 800-185）：WebCrypto 面无 function-name 成员 → N 恒空，S =
        // customization。
        "CSHAKE128" => xof_squeeze(
            cshake::CShake128::new_with_function_name(&[], &customization),
            &input,
            out_bytes,
        ),
        "CSHAKE256" => xof_squeeze(
            cshake::CShake256::new_with_function_name(&[], &customization),
            &input,
            out_bytes,
        ),
        // KangarooTwelve（RFC 9861）：customization 缺席 ≡ 空定制。
        "KT128" => match customization.is_empty() {
            true => xof_squeeze(k12::Kt128::default(), &input, out_bytes),
            false => xof_squeeze(k12::CustomKt128::new_customized(&customization), &input, out_bytes),
        },
        "KT256" => match customization.is_empty() {
            true => xof_squeeze(k12::Kt256::default(), &input, out_bytes),
            false => xof_squeeze(k12::CustomKt256::new_customized(&customization), &input, out_bytes),
        },
        "TURBOSHAKE128" => {
            turbo_digest!(168, ds, &input, out_bytes; [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127])
        }
        "TURBOSHAKE256" => {
            turbo_digest!(136, ds, &input, out_bytes; [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88,89,90,91,92,93,94,95,96,97,98,99,100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115,116,117,118,119,120,121,122,123,124,125,126,127])
        }
        _ => return String::new(),
    };
    if out.is_empty() {
        // 合法空输出（cSHAKE outputLength=0 档）——哨兵区分失败空串。
        return ",".to_string();
    }
    bytes_to_hex(&out)
}

// ---------------------------------------------------------------------------
// RSA 家族（webcrypto M4a）——RSA-OAEP encrypt/decrypt、RSASSA-PKCS1-v1_5 / RSA-PSS
// sign/verify、spki/pkcs8/jwk 密钥导入导出与 generateKey。委托 `rsa` 0.9（RustCrypto，
// 选型见 docs/goal/archive/webcrypto-compat/evidence/2026-10-17-m4a-deps.md）。**密钥材料零落
// 日志**：分量与 DER 仅经 CSV 桥进出，无 tracing 输出。所有失败返空串（shim 按操作
// 映射 NotSupportedError/InvalidAccessError/DataError/OperationError）。
// https://w3c.github.io/webcrypto/#rsa-oaep  https://w3c.github.io/webcrypto/#rsassa-pkcs1
// https://w3c.github.io/webcrypto/#rsa-pss  https://datatracker.ietf.org/doc/html/rfc8017
// ---------------------------------------------------------------------------

/// hash 名 → 分发臂（sign/verify/OAEP 共用；接受 `SHA-256`/`SHA256` 两种写法）。
enum RsaHash {
    S1,
    S256,
    S384,
    S512,
}

fn parse_rsa_hash(name: &str) -> Option<RsaHash> {
    match name.to_ascii_uppercase().as_str() {
        "SHA-1" | "SHA1" => Some(RsaHash::S1),
        "SHA-256" | "SHA256" => Some(RsaHash::S256),
        "SHA-384" | "SHA384" => Some(RsaHash::S384),
        "SHA-512" | "SHA512" => Some(RsaHash::S512),
        _ => None,
    }
}

/// 大端字节 csv → 大整数（jwk n/e/d 分量与 generateKey publicExponent 共用）。
fn rsa_uint(bytes_csv: &str) -> rsa::BigUint {
    rsa::BigUint::from_bytes_be(&bytes_from_csv(bytes_csv))
}

/// RSA 生成：`bits` 位模长 + `e_csv` 公钥指数（大端字节 csv，通常 1,0,1）→ pkcs8 DER
/// csv（私钥；公钥经 `crypto_subtle_rsa_export_der` 派生）。e 非法（1/偶数）或生成
/// 失败 → 空串（shim 映射 OperationError，WPT failures "Bad algorithm property"）。
/// https://datatracker.ietf.org/doc/html/rfc8017#section-3.1
pub fn crypto_subtle_rsa_generate(bits: u32, e_csv: &str) -> String {
    use rsa::pkcs8::EncodePrivateKey;
    use rsa::rand_core::OsRng;
    let e = rsa_uint(e_csv);
    // e 须 ≥3 且为奇数（RFC 8017 §3.1）；奇偶用最低位与 1 位与判定。
    let one = rsa::BigUint::from(1u8);
    if e < rsa::BigUint::from(3u8) || (&e & &one) != one {
        return String::new();
    }
    match rsa::RsaPrivateKey::new_with_exp(&mut OsRng, bits as usize, &e) {
        Ok(key) => match key.to_pkcs8_der() {
            Ok(doc) => bytes_to_csv(doc.as_bytes()),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    }
}

/// RSA DER 导入（spki 公钥 / pkcs8 私钥）→ 规范化再编码 DER csv（WPT exportKey
/// round-trip 逐字节比对，canonical DER 必须稳定）。格式与内容不符（公钥喂 pkcs8、
/// 私钥喂 spki、损坏 DER）→ 空串。fmt "spki"|"pkcs8"。
pub fn crypto_subtle_rsa_import(fmt: &str, der_csv: &str) -> String {
    use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
    let der = bytes_from_csv(der_csv);
    match fmt {
        "spki" => rsa::RsaPublicKey::from_public_key_der(&der)
            .ok()
            .and_then(|k| k.to_public_key_der().ok())
            .map(|doc| bytes_to_csv(doc.as_bytes()))
            .unwrap_or_default(),
        "pkcs8" => rsa::RsaPrivateKey::from_pkcs8_der(&der)
            .ok()
            .and_then(|k| k.to_pkcs8_der().ok())
            .map(|doc| bytes_to_csv(doc.as_bytes()))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// RSA jwk 分量导入：n/e 必填；d 非空 → 私钥（p/q 为 CRT 素数；二者同缺走 crate 的
/// d 反推素数恢复，SP 800-56B App C.2；dp/dq/qi 由 crate 校验重算，不信任入参）。
/// 一致性校验（p·q=n、d·e≡1 mod λ(n)）失败 → 空串。输出规范化 pkcs8（私）/
/// spki（公）DER csv。
/// https://w3c.github.io/webcrypto/#importJwkRSA
pub fn crypto_subtle_rsa_jwk_import(n_csv: &str, e_csv: &str, d_csv: &str, p_csv: &str, q_csv: &str) -> String {
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
    let n = rsa_uint(n_csv);
    let e = rsa_uint(e_csv);
    // 0 的位长为 0——以此判零（BigUint 的 is_zero 来自 num-traits，不引依赖）。
    if n.bits() == 0 || e.bits() == 0 {
        return String::new();
    }
    if bytes_from_csv(d_csv).is_empty() {
        return match rsa::RsaPublicKey::new(n, e) {
            Ok(k) => match k.to_public_key_der() {
                Ok(doc) => bytes_to_csv(doc.as_bytes()),
                Err(_) => String::new(),
            },
            Err(_) => String::new(),
        };
    }
    let d = rsa_uint(d_csv);
    let p_bytes = bytes_from_csv(p_csv);
    let q_bytes = bytes_from_csv(q_csv);
    // 素数要么全缺（d 反推），要么成对出现——单侧出现按 spec 数据不一致 → 空串。
    let primes: Vec<rsa::BigUint> = match (p_bytes.is_empty(), q_bytes.is_empty()) {
        (true, true) => Vec::new(),
        (false, false) => vec![rsa_uint(p_csv), rsa_uint(q_csv)],
        _ => return String::new(),
    };
    match rsa::RsaPrivateKey::from_components(n, e, d, primes) {
        Ok(key) => match key.to_pkcs8_der() {
            Ok(doc) => bytes_to_csv(doc.as_bytes()),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    }
}

/// RSA 密钥属性：DER（spki 公钥或 pkcs8 私钥，自动识别）→ `"<bits>,<e 字节 csv>"`。
/// 供 shim 构造 RsaHashedKeyAlgorithm（modulusLength = n 位长、publicExponent =
/// e 大端字节）。非法 DER → 空串。
pub fn crypto_subtle_rsa_key_attrs(der_csv: &str) -> String {
    use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
    use rsa::traits::PublicKeyParts;
    let der = bytes_from_csv(der_csv);
    let parse = |der: &[u8]| -> Option<(rsa::BigUint, rsa::BigUint)> {
        if let Ok(k) = rsa::RsaPrivateKey::from_pkcs8_der(der) {
            return Some((k.n().clone(), k.e().clone()));
        }
        if let Ok(k) = rsa::RsaPublicKey::from_public_key_der(der) {
            return Some((k.n().clone(), k.e().clone()));
        }
        None
    };
    match parse(&der) {
        Some((n, e)) => format!("{},{}", n.bits(), bytes_to_csv(&e.to_bytes_be())),
        None => String::new(),
    }
}

/// RSA DER 转换：pkcs8 私钥 → spki 公钥（fmt "spki"）；fmt "pkcs8" 透传（导入时已
/// 规范化）。私钥导 spki 失败 / 其他 fmt → 空串。
pub fn crypto_subtle_rsa_export_der(der_csv: &str, fmt: &str) -> String {
    use rsa::pkcs8::{DecodePrivateKey, EncodePublicKey};
    let der = bytes_from_csv(der_csv);
    match fmt {
        "spki" => rsa::RsaPrivateKey::from_pkcs8_der(&der)
            .ok()
            .and_then(|k| k.to_public_key().to_public_key_der().ok())
            .map(|doc| bytes_to_csv(doc.as_bytes()))
            .unwrap_or_default(),
        "pkcs8" => bytes_to_csv(&der),
        _ => String::new(),
    }
}

/// RSA jwk 导出：DER（spki 公钥 / pkcs8 私钥）→ `;` 分隔的大端分量字段（公钥 2 字段
/// `n;e`，私钥 8 字段 `n;e;d;p;q;dp;dq;qi`，各字段自身为字节 csv；缺失素数为空字段）。
/// dp = d mod (p−1)、dq = d mod (q−1)、qi = q^(p−2) mod p（素数 p，Fermat 小定理，
/// RFC 8017 §3.2）。want_private=true 但 DER 是公钥 → 空串。
pub fn crypto_subtle_rsa_export_jwk(der_csv: &str, want_private: bool) -> String {
    use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
    use rsa::traits::{PrivateKeyParts, PublicKeyParts};
    let der = bytes_from_csv(der_csv);
    if want_private {
        let key = match rsa::RsaPrivateKey::from_pkcs8_der(&der) {
            Ok(k) => k,
            Err(_) => return String::new(),
        };
        let primes = key.primes();
        let empty = rsa::BigUint::default();
        let p = primes.first().unwrap_or(&empty);
        let q = primes.get(1).unwrap_or(&empty);
        let one = rsa::BigUint::from(1u8);
        let two = rsa::BigUint::from(2u8);
        let dp = key.d() % (p - &one);
        let dq = key.d() % (q - &one);
        let qi = q.modpow(&(p - &two), p);
        return [
            key.n().to_bytes_be(),
            key.e().to_bytes_be(),
            key.d().to_bytes_be(),
            p.to_bytes_be(),
            q.to_bytes_be(),
            dp.to_bytes_be(),
            dq.to_bytes_be(),
            qi.to_bytes_be(),
        ]
        .iter()
        .map(|b| bytes_to_csv(b))
        .collect::<Vec<_>>()
        .join(";");
    }
    match rsa::RsaPublicKey::from_public_key_der(&der) {
        Ok(k) => [k.n().to_bytes_be(), k.e().to_bytes_be()]
            .iter()
            .map(|b| bytes_to_csv(b))
            .collect::<Vec<_>>()
            .join(";"),
        Err(_) => String::new(),
    }
}

/// RSA-OAEP encrypt/decrypt（RFC 8017 §7.1）。hash 为 normalize 后的摘要名；
/// mode "encrypt"（spki 公钥）/"decrypt"（pkcs8 私钥）；label_csv 可空（空 = RFC
/// 缺省空标签）。明文超长（k − 2·hLen − 2）、密文长度 ≠ k、解封校验失败 → 空串
///（shim 映射 OperationError）。
/// FIXME: `rsa` 0.9 的 Oaep.label 为 Option<String>（UTF-8），非 UTF-8 标签字节串
/// 不支持——corpus 标签均 ASCII；标准侧标签为任意 octet string。
pub fn crypto_subtle_rsa_oaep(hash: &str, mode: &str, key_csv: &str, label_csv: &str, data_csv: &str) -> String {
    use rsa::Oaep;
    use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
    use rsa::rand_core::OsRng;
    use sha2::Digest;
    let key = bytes_from_csv(key_csv);
    let data = bytes_from_csv(data_csv);
    let label_bytes = bytes_from_csv(label_csv);
    let label = if label_bytes.is_empty() {
        None
    } else {
        match String::from_utf8(label_bytes) {
            Ok(s) => Some(s),
            Err(_) => return String::new(),
        }
    };
    macro_rules! oaep {
        ($h:ty) => {
            Oaep {
                digest: Box::new(<$h>::new()),
                mgf_digest: Box::new(<$h>::new()),
                label: label.clone(),
            }
        };
    }
    let out = match (mode, parse_rsa_hash(hash)) {
        ("encrypt", Some(h)) => {
            let k = match rsa::RsaPublicKey::from_public_key_der(&key) {
                Ok(k) => k,
                Err(_) => return String::new(),
            };
            match h {
                RsaHash::S1 => k.encrypt(&mut OsRng, oaep!(sha1::Sha1), &data),
                RsaHash::S256 => k.encrypt(&mut OsRng, oaep!(sha2::Sha256), &data),
                RsaHash::S384 => k.encrypt(&mut OsRng, oaep!(sha2::Sha384), &data),
                RsaHash::S512 => k.encrypt(&mut OsRng, oaep!(sha2::Sha512), &data),
            }
        }
        ("decrypt", Some(h)) => {
            let k = match rsa::RsaPrivateKey::from_pkcs8_der(&key) {
                Ok(k) => k,
                Err(_) => return String::new(),
            };
            match h {
                RsaHash::S1 => k.decrypt(oaep!(sha1::Sha1), &data),
                RsaHash::S256 => k.decrypt(oaep!(sha2::Sha256), &data),
                RsaHash::S384 => k.decrypt(oaep!(sha2::Sha384), &data),
                RsaHash::S512 => k.decrypt(oaep!(sha2::Sha512), &data),
            }
        }
        _ => return String::new(),
    };
    out.map(|v| bytes_to_csv(&v)).unwrap_or_default()
}

/// RSASSA-PKCS1-v1_5 / RSA-PSS 签名（RFC 8017 §8.2/§8.1）。pad "pkcs1v15"|"pss"，
/// salt 仅 PSS 有效（spec：PSS saltLength 必填；0 = 确定性无盐）。pkcs8 私钥；先对
/// data 摘要（`sign` 的 digest_in 为 prehash，DigestInfo 前缀由 padding 添加），
/// 签名 → 字节 csv；key/参数非法 → 空串。
pub fn crypto_subtle_rsa_sign(hash: &str, pad: &str, salt: u32, key_csv: &str, data_csv: &str) -> String {
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::rand_core::OsRng;
    let key = match rsa::RsaPrivateKey::from_pkcs8_der(&bytes_from_csv(key_csv)) {
        Ok(k) => k,
        Err(_) => return String::new(),
    };
    let data = bytes_from_csv(data_csv);
    let h = match parse_rsa_hash(hash) {
        Some(h) => h,
        None => return String::new(),
    };
    // prehash：Pkcs1v15Sign / Pss 的 digest_in 均为消息摘要而非原文（RFC 8017 §8.1/§8.2 步骤 2）。
    macro_rules! prehash {
        ($h:ty) => {
            <$h as sha2::Digest>::digest(&data).to_vec()
        };
    }
    if pad == "pss" {
        let sig = match h {
            RsaHash::S1 => {
                let ph = prehash!(sha1::Sha1);
                key.sign_with_rng(
                    &mut OsRng,
                    rsa::pss::Pss::new_with_salt::<sha1::Sha1>(salt as usize),
                    &ph,
                )
            }
            RsaHash::S256 => {
                let ph = prehash!(sha2::Sha256);
                key.sign_with_rng(
                    &mut OsRng,
                    rsa::pss::Pss::new_with_salt::<sha2::Sha256>(salt as usize),
                    &ph,
                )
            }
            RsaHash::S384 => {
                let ph = prehash!(sha2::Sha384);
                key.sign_with_rng(
                    &mut OsRng,
                    rsa::pss::Pss::new_with_salt::<sha2::Sha384>(salt as usize),
                    &ph,
                )
            }
            RsaHash::S512 => {
                let ph = prehash!(sha2::Sha512);
                key.sign_with_rng(
                    &mut OsRng,
                    rsa::pss::Pss::new_with_salt::<sha2::Sha512>(salt as usize),
                    &ph,
                )
            }
        };
        return sig.map(|v| bytes_to_csv(&v)).unwrap_or_default();
    }
    if pad == "pkcs1v15" {
        let sig = match h {
            RsaHash::S1 => key.sign(rsa::Pkcs1v15Sign::new::<sha1::Sha1>(), &prehash!(sha1::Sha1)),
            RsaHash::S256 => key.sign(rsa::Pkcs1v15Sign::new::<sha2::Sha256>(), &prehash!(sha2::Sha256)),
            RsaHash::S384 => key.sign(rsa::Pkcs1v15Sign::new::<sha2::Sha384>(), &prehash!(sha2::Sha384)),
            RsaHash::S512 => key.sign(rsa::Pkcs1v15Sign::new::<sha2::Sha512>(), &prehash!(sha2::Sha512)),
        };
        return sig.map(|v| bytes_to_csv(&v)).unwrap_or_default();
    }
    String::new()
}

/// RSASSA-PKCS1-v1_5 / RSA-PSS 验签（spki 公钥）。签名非法或验证失败 → "0"
///（spec：verify 对非法签名 resolve false 而非 reject）；key/参数非法 → 空串（shim
/// reject）。
pub fn crypto_subtle_rsa_verify(
    hash: &str,
    pad: &str,
    salt: u32,
    key_csv: &str,
    sig_csv: &str,
    data_csv: &str,
) -> String {
    use rsa::pkcs8::DecodePublicKey;
    let key = match rsa::RsaPublicKey::from_public_key_der(&bytes_from_csv(key_csv)) {
        Ok(k) => k,
        Err(_) => return String::new(),
    };
    let sig = bytes_from_csv(sig_csv);
    let data = bytes_from_csv(data_csv);
    let h = match parse_rsa_hash(hash) {
        Some(h) => h,
        None => return String::new(),
    };
    macro_rules! prehash {
        ($h:ty) => {
            <$h as sha2::Digest>::digest(&data).to_vec()
        };
    }
    if pad == "pss" {
        let ok = match h {
            RsaHash::S1 => {
                let ph = prehash!(sha1::Sha1);
                key.verify(rsa::pss::Pss::new_with_salt::<sha1::Sha1>(salt as usize), &ph, &sig)
            }
            RsaHash::S256 => {
                let ph = prehash!(sha2::Sha256);
                key.verify(rsa::pss::Pss::new_with_salt::<sha2::Sha256>(salt as usize), &ph, &sig)
            }
            RsaHash::S384 => {
                let ph = prehash!(sha2::Sha384);
                key.verify(rsa::pss::Pss::new_with_salt::<sha2::Sha384>(salt as usize), &ph, &sig)
            }
            RsaHash::S512 => {
                let ph = prehash!(sha2::Sha512);
                key.verify(rsa::pss::Pss::new_with_salt::<sha2::Sha512>(salt as usize), &ph, &sig)
            }
        };
        return if ok.is_ok() { "1".into() } else { "0".into() };
    }
    if pad == "pkcs1v15" {
        let ok = match h {
            RsaHash::S1 => key.verify(rsa::Pkcs1v15Sign::new::<sha1::Sha1>(), &prehash!(sha1::Sha1), &sig),
            RsaHash::S256 => key.verify(rsa::Pkcs1v15Sign::new::<sha2::Sha256>(), &prehash!(sha2::Sha256), &sig),
            RsaHash::S384 => key.verify(rsa::Pkcs1v15Sign::new::<sha2::Sha384>(), &prehash!(sha2::Sha384), &sig),
            RsaHash::S512 => key.verify(rsa::Pkcs1v15Sign::new::<sha2::Sha512>(), &prehash!(sha2::Sha512), &sig),
        };
        return if ok.is_ok() { "1".into() } else { "0".into() };
    }
    String::new()
}

#[cfg(test)]
mod kmac_nist_tests {
    use super::*;

    /// NIST SP 800-185 KMAC samples 全 6 向量逐字节复现（KMAC128×3 + KMAC256×3；
    /// 源 = WPT sign_verify/kmac_vectors.js，PDF
    /// csrc.nist.gov/.../KMAC_samples.pdf）。
    const KEY: [u8; 32] = [
        0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51,
        0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    ];
    const MSG4: [u8; 4] = [0x00, 0x01, 0x02, 0x03];
    const MSG200: [u8; 200] = [0; 200];
    const CUSTOM: &[u8] = b"My Tagged Application";

    fn csv(bytes: &[u8]) -> String {
        bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
    }

    fn mac_hex(algo: &str, data: &[u8], custom: &[u8], bits: u32) -> String {
        let out = crypto_subtle_kmac(algo, &csv(&KEY), &csv(data), bits, &csv(custom));
        assert!(!out.is_empty(), "KMAC {algo} failed");
        let parts: Vec<u8> = out.split(',').map(|s| s.parse().unwrap()).collect();
        parts.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn nist_samples_all_six() {
        // Sample #1：KMAC128 无 customization，256 bit。
        assert_eq!(
            mac_hex("KMAC128", &MSG4, &[], 256),
            "e5780b0d3ea6f7d3a429c5706aa43a00fadbd7d49628839e3187243f456ee14e"
        );
        // Sample #2 KMAC128 customization，256 bit。
        assert_eq!(
            mac_hex("KMAC128", &MSG4, CUSTOM, 256),
            "3b1fba963cd8b0b59e8c1a6d71888b7143651af8ba0a7070c0979e2811324aa5"
        );
        // Sample #3 KMAC128 200B 数据 + customization，256 bit。
        let mut m3 = MSG200;
        for (i, b) in m3.iter_mut().enumerate() {
            *b = i as u8;
        }
        assert_eq!(
            mac_hex("KMAC128", &m3, CUSTOM, 256),
            "1f5b4e6cca02209e0dcb5ca635b89a15e271ecc760071dfd805faa38f9729230"
        );
        // Sample #4 KMAC256 customization，512 bit。
        assert_eq!(
            mac_hex("KMAC256", &MSG4, CUSTOM, 512),
            "20c570c31346f703c9ac36c61c03cb64c3970d0cfc787e9b79599d273a68d2f7f69d4cc3de9d104a351689f27cf6f5951f0103f33f4f24871024d9c27773a8dd"
        );
        // Sample #5 KMAC256 200B 数据无 customization，512 bit。
        let mut m5 = MSG200;
        for (i, b) in m5.iter_mut().enumerate() {
            *b = i as u8;
        }
        assert_eq!(
            mac_hex("KMAC256", &m5, &[], 512),
            "75358cf39e41494e949707927cee0af20a3ff553904c86b08f21cc414bcfd691589d27cf5e15369cbbff8b9a4c2eb17800855d0235ff635da82533ec6b759b69"
        );
        // Sample #6 KMAC256 200B 数据 + customization，512 bit。
        assert_eq!(
            mac_hex("KMAC256", &m5, CUSTOM, 512),
            "b58618f71f92e1d56c1b8c55ddd7cd188b97b4ca4d99831eb2699a837da2e4d970fbacfde50033aea585f1a2708510c32d07880801bd182898fe476876fc8965"
        );
    }
}
