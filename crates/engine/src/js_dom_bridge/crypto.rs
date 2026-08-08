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

/// `crypto.subtle.encrypt/decrypt` 的 AES-GCM 实现（R2957）——AES-128/256-GCM 认证对称加密。
/// **PBKDF2 派生密钥的典型消费者**，端到端「用密码加密」流程（TLS 级对称加密 + 完整性 + AAD）。
/// mode "encrypt"（返 ct||tag）/ "decrypt"（输入 ct||tag，返 plaintext）；key 16/32 字节（128/256 位，
/// AES-192 因 aes-gcm crate 默认未导出 `Aes192Gcm` 暂不支持——罕见）；iv 12 字节（GCM 标准 nonce）；
/// aad 附加认证数据；tag 固定 128 位（spec 默认，最常见）。返 csv，error（bad key/iv 长度、tag 校验失败）→ 空串。
/// https://datatracker.ietf.org/doc/html/rfc5116  https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf
pub fn crypto_subtle_aes_gcm(mode: &str, key_csv: &str, iv_csv: &str, data_csv: &str, aad_csv: &str) -> String {
    use aes_gcm::aead::{KeyInit, Payload, generic_array::GenericArray};
    use aes_gcm::{Aes128Gcm, Aes256Gcm};
    let key = bytes_from_csv(key_csv);
    let iv = bytes_from_csv(iv_csv);
    let data = bytes_from_csv(data_csv);
    let aad = bytes_from_csv(aad_csv);
    if iv.len() != 12 {
        return String::new();
    }
    let nonce = GenericArray::from_slice(&iv);
    let payload = Payload { msg: &data, aad: &aad };
    let result: Option<Vec<u8>> = match key.len() {
        16 => aes_gcm_run(&Aes128Gcm::new(GenericArray::from_slice(&key)), mode, nonce, payload),
        32 => aes_gcm_run(&Aes256Gcm::new(GenericArray::from_slice(&key)), mode, nonce, payload),
        _ => None,
    };
    match result {
        Some(v) => v.iter().map(u8::to_string).collect::<Vec<_>>().join(","),
        None => String::new(),
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
