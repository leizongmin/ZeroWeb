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
/// ct||tag，返 plaintext）；key 16/24/32 字节；iv 12/32 字节（nonce 为 aes-gcm crate 编译期
/// 泛型，按字节长二分派）；tag_len_bits ∈ {32,64,96,104,112,120,128}（合法集由 shim 校验，
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
    if iv.len() != 12 && iv.len() != 32 {
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
            _ => String::new(),
        };
    }
    let result: Option<Vec<u8>> = if iv.len() == 12 { by_tag!(U12) } else { by_tag!(U32) };
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
/// `cbc`/`ctr` crate（选型见 docs/goal/webcrypto-compat/evidence/2026-10-17-m2b-deps.md）。
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
