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
